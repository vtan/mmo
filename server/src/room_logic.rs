use mmo_common::{object::ObjectId, player_command::RoomCommand, player_event::PlayerEvent};
use nalgebra::Vector2;
use tokio::time::Instant;
use tracing::instrument;

use crate::{
    player::PlayerConnection,
    room_state::{LocalMovement, Player, RemoteMovement, RoomState, RoomWriter, UpstreamMessage},
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
    let remote_movement = RemoteMovement { position, direction: None, received_at: now };
    let player = Player { id: player_id, connection, local_movement, remote_movement };
    player_entered(player, state, writer);
}

fn player_entered(player: Player, state: &mut RoomState, writer: &mut RoomWriter) {
    let player_id = player.id;
    let player_position = player.local_movement.position;
    let now = Instant::now();

    state.players.insert(player_id, player);

    writer.broadcast(
        state.players.keys().copied(),
        PlayerEvent::PlayerMovementChanged {
            object_id: player_id,
            position: player_position,
            direction: None,
        },
    );

    writer.tell(
        player_id,
        PlayerEvent::RoomEntered { room: state.room.clone() },
    );
    for player_in_room in state.players.values() {
        let position =
            interpolate_position(&state.server_context, player_in_room.remote_movement, now);
        writer.tell(
            player_id,
            PlayerEvent::PlayerMovementChanged {
                object_id: player_in_room.id,
                position: position.position,
                direction: None,
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
            PlayerEvent::PlayerDisappeared { object_id: player_id },
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
        RoomCommand::Move { position, direction } => {
            // TODO: at least a basic check whether the position is plausible
            let now = Instant::now();
            let remote_movement = RemoteMovement { position, direction, received_at: now };
            let local_movement = LocalMovement { position, updated_at: now };
            state.players.entry(player_id).and_modify(|p| {
                p.remote_movement = remote_movement;
                p.local_movement = local_movement;
            });

            writer.broadcast(
                state.players.keys().copied().filter(|pid| *pid != player_id),
                PlayerEvent::PlayerMovementChanged {
                    object_id: player_id,
                    position: local_movement.position,
                    direction: remote_movement.direction,
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
            player.local_movement = local_movement;

            let crossed_tile =
                last_position.map(|a| a as u32) != local_movement.position.map(|a| a as u32);

            if crossed_tile {
                writer.broadcast(
                    player_ids.iter().copied().filter(|pid| *pid != player_id),
                    PlayerEvent::PlayerMovementChanged {
                        object_id: player.id,
                        position: local_movement.position,
                        direction: player.remote_movement.direction,
                    },
                );
            }
        }
    }

    for player_id in players_left {
        player_left(player_id, state, writer);
    }
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
