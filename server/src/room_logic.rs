use std::{collections::HashMap, sync::Arc};

use mmo_common::{
    animation::AnimationAction,
    object::{Direction, ObjectId},
    player_command::RoomCommand,
    player_event::PlayerEvent,
    room,
};
use nalgebra::Vector2;
use tokio::time::Instant;
use tracing::instrument;

use crate::{
    mob::MobTemplate,
    object,
    player::PlayerConnection,
    room_state::{
        LocalMovement, Mob, Player, RemoteMovement, RoomMap, RoomState, RoomWriter, UpstreamMessage,
    },
    server_context::ServerContext,
    tick::Tick,
};

#[instrument(skip_all, fields(player_id = player_id.0))]
pub fn on_connect(
    player_id: ObjectId,
    connection: PlayerConnection,
    position: Vector2<f32>,
    state: &mut RoomState,
    writer: &mut RoomWriter,
) {
    let now = Instant::now();
    let local_movement = LocalMovement { position, updated_at: now };
    let remote_movement = RemoteMovement {
        position,
        direction: None,
        look_direction: Direction::Down,
        received_at: now,
    };
    let player = Player { id: player_id, connection, local_movement, remote_movement };
    player_entered(player, state, writer);
}

fn player_entered(player: Player, state: &mut RoomState, writer: &mut RoomWriter) {
    let player_id = player.id;
    let player_local_movement = player.local_movement;
    let player_remote_movement = player.remote_movement;
    let now = Instant::now();

    writer.broadcast_many(
        state.players.keys().copied(),
        &[
            PlayerEvent::ObjectAppeared {
                object_id: player_id,
                animation_id: state.server_context.player_animation,
                velocity: state.server_context.player_velocity,
            },
            PlayerEvent::ObjectMovementChanged {
                object_id: player_id,
                position: player_local_movement.position,
                direction: player_remote_movement.direction,
                look_direction: player_remote_movement.look_direction,
            },
        ],
    );

    state.players.insert(player_id, player);

    writer.tell(
        player_id,
        PlayerEvent::RoomEntered { room: Box::new(state.room.clone()) },
    );
    for player_in_room in state.players.values() {
        let position =
            interpolate_position(&state.server_context, player_in_room.remote_movement, now);
        writer.tell_many(
            player_id,
            &[
                PlayerEvent::ObjectAppeared {
                    object_id: player_in_room.id,
                    animation_id: state.server_context.player_animation,
                    velocity: state.server_context.player_velocity,
                },
                PlayerEvent::ObjectMovementChanged {
                    object_id: player_in_room.id,
                    position: position.position,
                    direction: player_in_room.remote_movement.direction,
                    look_direction: player_in_room.remote_movement.look_direction,
                },
            ],
        );
    }
    for mob in state.mobs.values() {
        writer.tell(
            player_id,
            PlayerEvent::ObjectAppeared {
                object_id: mob.id,
                animation_id: mob.animation_id,
                velocity: mob.template.velocity,
            },
        );
        writer.tell(
            player_id,
            PlayerEvent::ObjectMovementChanged {
                object_id: mob.id,
                position: mob.movement.position,
                direction: mob.movement.direction,
                look_direction: mob.movement.look_direction,
            },
        );
    }
}

#[instrument(skip_all, fields(player_id = player_id.0))]
pub fn on_disconnect(player_id: ObjectId, state: &mut RoomState, writer: &mut RoomWriter) {
    player_left(player_id, state, writer)
}

fn player_left(player_id: ObjectId, state: &mut RoomState, writer: &mut RoomWriter) {
    if state.players.remove(&player_id).is_some() {
        writer.broadcast(
            state.players.keys().copied(),
            PlayerEvent::ObjectDisappeared { object_id: player_id },
        );
    } else {
        tracing::error!("Player not found");
    }
}

pub fn on_command(
    player_id: ObjectId,
    command: RoomCommand,
    state: &mut RoomState,
    writer: &mut RoomWriter,
) {
    match command {
        RoomCommand::Move { position, direction, look_direction } => {
            // TODO: at least a basic check whether the position is plausible
            let now = Instant::now();

            let player_ids = state.players.keys().copied().collect::<Vec<_>>();
            let player = if let Some(player) = state.players.get_mut(&player_id) {
                player
            } else {
                return;
            };

            player.remote_movement =
                RemoteMovement { position, direction, look_direction, received_at: now };

            if room::collision_at(state.map.size, &state.map.collisions, position) {
                prevent_collision(player, &player_ids, now, writer);
            } else {
                player.local_movement = LocalMovement { position, updated_at: now };

                writer.broadcast(
                    player_ids.iter().copied().filter(|pid| *pid != player_id),
                    PlayerEvent::ObjectMovementChanged {
                        object_id: player_id,
                        position: player.local_movement.position,
                        direction: player.remote_movement.direction,
                        look_direction: player.remote_movement.look_direction,
                    },
                );
            }
        }
        RoomCommand::Attack => {
            writer.broadcast(
                state.players.keys().copied().filter(|pid| *pid != player_id),
                PlayerEvent::ObjectAnimationAction {
                    object_id: player_id,
                    action: AnimationAction::Attack,
                },
            );
        }
    }
}

pub fn on_tick(tick: Tick, state: &mut RoomState, writer: &mut RoomWriter) {
    let now = tick.monotonic_time;

    let player_ids = state.players.keys().copied().collect::<Vec<_>>();
    let mut players_left = vec![];

    for player_id in player_ids.iter().copied() {
        let player = state.players.get_mut(&player_id).expect("Player not found");

        let local_movement =
            interpolate_position(&state.server_context, player.remote_movement, now);

        let portal = state
            .map
            .portals
            .iter()
            .find(|portal| portal.position == local_movement.position.map(|a| a as u32));

        if let Some(portal) = portal {
            let target_room_id = portal.target_room_id;
            let target_position = portal.target_position;
            players_left.push(player_id);
            writer.upstream_messages.push(UpstreamMessage::PlayerLeftRoom {
                sender_room_id: state.room.room_id,
                player_id,
                target_room_id,
                target_position,
            });
        } else {
            let last_position = player.local_movement.position;

            if room::collision_at(
                state.map.size,
                &state.map.collisions,
                local_movement.position,
            ) {
                prevent_collision(player, &player_ids, now, writer);
            } else {
                player.local_movement = local_movement;

                let crossed_tile =
                    last_position.map(|a| a as u32) != local_movement.position.map(|a| a as u32);

                if crossed_tile {
                    writer.broadcast(
                        player_ids.iter().copied().filter(|pid| *pid != player_id),
                        PlayerEvent::ObjectMovementChanged {
                            object_id: player.id,
                            position: local_movement.position,
                            direction: player.remote_movement.direction,
                            look_direction: player.remote_movement.look_direction,
                        },
                    );
                }
            }
        }
    }

    for player_id in players_left {
        player_left(player_id, state, writer);
    }
}

fn prevent_collision(
    player: &mut Player,
    player_ids: &[ObjectId],
    now: Instant,
    writer: &mut RoomWriter,
) {
    player.remote_movement = RemoteMovement {
        position: player.local_movement.position,
        direction: None,
        look_direction: player.remote_movement.look_direction,
        received_at: now, // TODO: mark that this was a correction?
    };
    writer.broadcast(
        player_ids.iter().copied(),
        PlayerEvent::ObjectMovementChanged {
            object_id: player.id,
            position: player.remote_movement.position,
            direction: player.remote_movement.direction,
            look_direction: player.remote_movement.look_direction,
        },
    );
}

fn interpolate_position(
    ctx: &ServerContext,
    remote_movement: RemoteMovement,
    now: Instant,
) -> LocalMovement {
    if let Some(direction) = remote_movement.direction {
        let elapsed = now - remote_movement.received_at;
        let direction = direction.to_vector();
        let delta = direction * ctx.player_velocity * elapsed.as_secs_f32();
        let position = remote_movement.position + delta;
        LocalMovement { position, updated_at: now }
    } else {
        LocalMovement { position: remote_movement.position, updated_at: now }
    }
}

pub fn populate_mobs(map: &RoomMap, ctx: &ServerContext, now: Instant) -> HashMap<ObjectId, Mob> {
    map.mob_spawns
        .iter()
        .filter_map(|mob_spawn| {
            let resolve = || -> Option<(Arc<MobTemplate>, u32)> {
                let mob_template = ctx.mob_templates.get(&mob_spawn.mob_template)?;
                let animation_id = ctx.mob_animations.get(&mob_template.animation_id)?;
                Some((mob_template.clone(), *animation_id))
            };
            if let Some((mob_template, animation_id)) = resolve() {
                let mob = Mob {
                    id: object::next_object_id(),
                    animation_id,
                    template: mob_template,
                    movement: RemoteMovement {
                        position: mob_spawn.position.cast(),
                        direction: None,
                        look_direction: Direction::Down,
                        received_at: now,
                    },
                };
                Some((mob.id, mob))
            } else {
                None
            }
        })
        .collect()
}
