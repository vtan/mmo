use std::collections::HashMap;

use mmo_common::{
    animation::AnimationAction,
    object::{Direction, ObjectId, ObjectType},
    player_command::RoomCommand,
    player_event::PlayerEvent,
    room,
};
use tokio::time::Instant;
use tracing::instrument;

use crate::{
    combat_logic, mob_logic,
    room_state::{LocalMovement, Player, RemoteMovement, RoomState, RoomWriter, UpstreamMessage},
    server_context::ServerContext,
    tick::Tick,
};

#[instrument(skip_all, fields(player_id = player.id.0))]
pub fn on_connect(mut player: Player, state: &mut RoomState, writer: &mut RoomWriter) {
    let now = Instant::now();
    player.local_movement.updated_at = now;
    player.remote_movement.received_at = now;
    player.remote_movement.direction = None;
    player.remote_movement.look_direction = Direction::Down;
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
                object_type: ObjectType::Player,
                animation_id: state.server_context.player_animation,
                velocity: state.server_context.player_velocity,
                health: player.health,
                max_health: player.max_health,
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
                    object_type: ObjectType::Player,
                    animation_id: state.server_context.player_animation,
                    velocity: state.server_context.player_velocity,
                    health: player_in_room.health,
                    max_health: player_in_room.max_health,
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
    for mob in state.mobs.iter() {
        writer.tell(
            player_id,
            PlayerEvent::ObjectAppeared {
                object_id: mob.id,
                object_type: ObjectType::Mob,
                animation_id: mob.animation_id,
                velocity: mob.template.velocity,
                health: mob.health,
                max_health: mob.template.max_health,
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
    remove_player(player_id, &mut state.players, writer);
}

fn remove_player(
    player_id: ObjectId,
    players: &mut HashMap<ObjectId, Player>,
    writer: &mut RoomWriter,
) -> Option<Player> {
    // FIXME: this removes the player before flushing the writer
    if let Some(player) = players.remove(&player_id) {
        writer.broadcast(
            players.keys().copied().filter(|id| *id != player_id),
            PlayerEvent::ObjectDisappeared { object_id: player_id },
        );
        Some(player)
    } else {
        tracing::error!("Player not found");
        None
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
            combat_logic::player_attack(player_id, state, writer);
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

        let last_position = player.local_movement.position;
        let local_movement =
            interpolate_position(&state.server_context, player.remote_movement, now);
        let crossed_tile =
            last_position.map(|a| a as u32) != local_movement.position.map(|a| a as u32);

        let portal = state
            .map
            .portals
            .iter()
            .find(|portal| portal.position == local_movement.position.map(|a| a as u32));

        if room::collision_at(
            state.map.size,
            &state.map.collisions,
            local_movement.position,
        ) {
            prevent_collision(player, &player_ids, now, writer);
        } else if let Some(portal) = portal {
            if crossed_tile {
                players_left.push((player_id, portal));
            }
        } else {
            player.local_movement = local_movement;

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

    for (player_id, portal) in players_left {
        if let Some(player) = remove_player(player_id, &mut state.players, writer) {
            let target_room_id = portal.target_room_id;
            let target_position = portal.target_position.add_scalar(0.5);
            writer.upstream_messages.push(UpstreamMessage::PlayerLeftRoom {
                sender_room_id: state.room.room_id,
                player,
                target_room_id,
                target_position,
            });
        }
    }

    mob_logic::on_tick(tick, state, writer);
    handle_dead_players(state, writer);
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

pub fn handle_dead_players(state: &mut RoomState, writer: &mut RoomWriter) {
    let dead_player_ids = state
        .players
        .values()
        .filter_map(
            |player| {
                if player.health == 0 {
                    Some(player.id)
                } else {
                    None
                }
            },
        )
        .collect::<Vec<_>>();

    for dead_player_id in dead_player_ids {
        if let Some(mut player) = remove_player(dead_player_id, &mut state.players, writer) {
            player.health = player.max_health;
            writer.upstream_messages.push(UpstreamMessage::PlayerLeftRoom {
                sender_room_id: state.room.room_id,
                player,
                target_room_id: state.server_context.start_room,
                target_position: state.server_context.start_position,
            })
        }
    }
}
