use std::collections::HashMap;

use mmo_common::{
    object::{Direction4, ObjectId, ObjectType},
    player_command::RoomCommand,
    player_event::PlayerEvent,
    room,
};
use nalgebra::Vector2;
use tokio::time::Instant;
use tracing::instrument;

use crate::{
    combat_logic, mob_logic,
    room_state::{
        LocalMovement, Player, Portal, RemoteMovement, RoomMap, RoomState, UpstreamMessage,
    },
    room_writer::{RoomWriter, RoomWriterTarget},
    server_context::ServerContext,
    tick::TickRate,
};

#[instrument(skip_all, fields(player_id = player.id.0))]
pub fn on_connect(mut player: Player, state: &mut RoomState, writer: &mut RoomWriter) {
    let now = Instant::now();
    player.local_movement.updated_at = now;
    player.remote_movement.received_at = now;
    player.remote_movement.direction = None;
    player.remote_movement.look_direction = Direction4::Down;
    player_entered(player, state, writer);
}

fn player_entered(player: Player, state: &mut RoomState, writer: &mut RoomWriter) {
    let player_id = player.id;
    let player_local_movement = player.local_movement;
    let player_remote_movement = player.remote_movement;
    let now = Instant::now();

    writer.tell_many(
        RoomWriterTarget::AllExcept(player_id),
        &[
            PlayerEvent::ObjectAppeared {
                object_id: player_id,
                object_type: ObjectType::Player,
                animation_id: state.server_context.player_animation,
                health: player.health,
                max_health: player.max_health,
            },
            PlayerEvent::ObjectMovementChanged {
                object_id: player_id,
                position: player_local_movement.position,
                velocity: state.server_context.player.velocity,
                direction: player_remote_movement.direction,
                look_direction: player_remote_movement.look_direction,
            },
        ],
    );

    state.players.insert(player_id, player);

    writer.tell(
        RoomWriterTarget::Player(player_id),
        PlayerEvent::RoomEntered {
            room: Box::new(state.room.clone()),
        },
    );
    for player_in_room in state.players.values() {
        let position =
            interpolate_position(&state.server_context, player_in_room.remote_movement, now);
        writer.tell_many(
            RoomWriterTarget::Player(player_id),
            &[
                PlayerEvent::ObjectAppeared {
                    object_id: player_in_room.id,
                    object_type: ObjectType::Player,
                    animation_id: state.server_context.player_animation,
                    health: player_in_room.health,
                    max_health: player_in_room.max_health,
                },
                PlayerEvent::ObjectMovementChanged {
                    object_id: player_in_room.id,
                    position: position.position,
                    velocity: state.server_context.player.velocity,
                    direction: player_in_room.remote_movement.direction,
                    look_direction: player_in_room.remote_movement.look_direction,
                },
            ],
        );
    }
    for mob in state.mobs.iter() {
        writer.tell_many(
            RoomWriterTarget::Player(player_id),
            &mob_logic::mob_appeared_events(mob),
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
        writer.tell(
            RoomWriterTarget::AllExcept(player_id),
            PlayerEvent::ObjectDisappeared {
                object_id: player_id,
            },
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
        RoomCommand::Move {
            position,
            direction,
            look_direction,
        } => {
            // TODO: at least a basic check whether the position is plausible
            let now = Instant::now();

            let player = if let Some(player) = state.players.get_mut(&player_id) {
                player
            } else {
                return;
            };

            player.remote_movement = RemoteMovement {
                position,
                direction,
                look_direction,
                received_at: now,
            };

            if room::collision_at(state.map.size, &state.map.collisions, position) {
                prevent_collision(player, now, &state.server_context, writer);
            } else if let Some(portal) =
                find_player_portal(&state.map, player.local_movement.position, position)
            {
                let portal = portal.clone();
                move_player_through_portal(player_id, &portal, state, writer);
            } else {
                player.local_movement = LocalMovement {
                    position,
                    updated_at: now,
                };

                writer.tell(
                    RoomWriterTarget::AllExcept(player_id),
                    PlayerEvent::ObjectMovementChanged {
                        object_id: player_id,
                        position: player.local_movement.position,
                        velocity: state.server_context.player.velocity,
                        direction: player.remote_movement.direction,
                        look_direction: player.remote_movement.look_direction,
                    },
                );
            }
        }
        RoomCommand::Attack => {
            combat_logic::player_attack(player_id, state, writer);
            writer.tell(
                RoomWriterTarget::AllExcept(player_id),
                PlayerEvent::ObjectAnimationAction {
                    object_id: player_id,
                    animation_index: state.server_context.player.attack_animation_index,
                },
            );
        }
    }
}

pub fn on_tick(state: &mut RoomState, writer: &mut RoomWriter) {
    if state.last_tick.tick.is_nth(TickRate(10)) {
        mob_logic::respawn_mobs(state, writer);
    }

    move_players(state, writer);
    combat_logic::heal_players(state, writer);
    mob_logic::on_tick(state, writer);
    handle_dead_players(state, writer);
}

fn move_players(state: &mut RoomState, writer: &mut RoomWriter) {
    let now = state.last_tick.monotonic_time;

    let mut players_left = vec![];

    for player in state.players.values_mut() {
        let last_position = player.local_movement.position;
        let local_movement =
            interpolate_position(&state.server_context, player.remote_movement, now);
        let crossed_tile =
            last_position.map(|a| a as u32) != local_movement.position.map(|a| a as u32);

        if room::collision_at(
            state.map.size,
            &state.map.collisions,
            local_movement.position,
        ) {
            prevent_collision(player, now, &state.server_context, writer);
        } else if let Some(portal) =
            find_player_portal(&state.map, last_position, local_movement.position)
        {
            players_left.push((player.id, portal.clone()));
        } else {
            player.local_movement = local_movement;

            if crossed_tile {
                writer.tell(
                    RoomWriterTarget::AllExcept(player.id),
                    PlayerEvent::ObjectMovementChanged {
                        object_id: player.id,
                        position: local_movement.position,
                        velocity: state.server_context.player.velocity,
                        direction: player.remote_movement.direction,
                        look_direction: player.remote_movement.look_direction,
                    },
                );
            }
        }
    }

    for (player_id, portal) in players_left {
        move_player_through_portal(player_id, &portal, state, writer);
    }
}

fn find_player_portal(
    map: &RoomMap,
    previous_position: Vector2<f32>,
    new_position: Vector2<f32>,
) -> Option<&Portal> {
    let crossed_tile = previous_position.map(|a| a as u32) != new_position.map(|a| a as u32);
    if crossed_tile {
        map.portals
            .iter()
            .find(|portal| portal.position == new_position.map(|a| a as u32))
    } else {
        None
    }
}

fn move_player_through_portal(
    player_id: ObjectId,
    portal: &Portal,
    state: &mut RoomState,
    writer: &mut RoomWriter,
) {
    if let Some(player) = remove_player(player_id, &mut state.players, writer) {
        let target_room_id = portal.target_room_id;
        let target_position = portal.target_position.add_scalar(0.5);
        writer
            .upstream_messages
            .push(UpstreamMessage::PlayerLeftRoom {
                sender_room_id: state.room.room_id,
                player,
                target_room_id,
                target_position,
            });
    }
}

fn prevent_collision(
    player: &mut Player,
    now: Instant,
    ctx: &ServerContext,
    writer: &mut RoomWriter,
) {
    player.remote_movement = RemoteMovement {
        position: player.local_movement.position,
        direction: None,
        look_direction: player.remote_movement.look_direction,
        received_at: now, // TODO: mark that this was a correction?
    };
    writer.tell(
        RoomWriterTarget::All,
        PlayerEvent::ObjectMovementChanged {
            object_id: player.id,
            position: player.remote_movement.position,
            velocity: ctx.player.velocity,
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
        let direction = direction.to_unit_vector();
        let delta = direction * ctx.player.velocity * elapsed.as_secs_f32();
        let position = remote_movement.position + delta;
        LocalMovement {
            position,
            updated_at: now,
        }
    } else {
        LocalMovement {
            position: remote_movement.position,
            updated_at: now,
        }
    }
}

pub fn handle_dead_players(state: &mut RoomState, writer: &mut RoomWriter) {
    let dead_player_ids = state
        .players
        .values()
        .filter_map(|player| {
            if player.health == 0 {
                Some(player.id)
            } else {
                None
            }
        })
        .collect::<Vec<_>>();

    for dead_player_id in dead_player_ids {
        if let Some(mut player) = remove_player(dead_player_id, &mut state.players, writer) {
            player.health = player.max_health;
            writer
                .upstream_messages
                .push(UpstreamMessage::PlayerLeftRoom {
                    sender_room_id: state.room.room_id,
                    player,
                    target_room_id: state.server_context.world.start_room_id,
                    target_position: state.server_context.world.start_position,
                })
        }
    }
}
