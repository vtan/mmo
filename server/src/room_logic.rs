use mmo_common::{player_command::RoomCommand, player_event::PlayerEvent};
use nalgebra::Vector2;
use tracing::instrument;

use crate::{
    player::PlayerConnection,
    room_state::{Player, RoomState, RoomWriter, UpstreamMessage},
};

#[instrument(skip_all, fields(player_id = player_id))]
pub fn on_connect(
    player_id: u64,
    connection: PlayerConnection,
    position: Vector2<f32>,
    state: &mut RoomState,
    writer: &mut RoomWriter,
) {
    let player = Player { id: player_id, connection, position };
    player_entered(player, state, writer);
}

fn player_entered(player: Player, state: &mut RoomState, writer: &mut RoomWriter) {
    let player_id = player.id;
    let player_position = player.position;

    state.players.insert(player_id, player);

    writer.broadcast(
        state.players.keys().copied(),
        PlayerEvent::PlayerMovementChanged {
            player_id,
            position: player_position,
            direction: None,
        },
    );

    writer.tell(
        player_id,
        PlayerEvent::RoomEntered { room: state.room.clone() },
    );
    for player_in_room in state.players.values() {
        writer.tell(
            player_id,
            PlayerEvent::PlayerMovementChanged {
                player_id: player_in_room.id,
                position: player_in_room.position,
                direction: None,
            },
        );
    }
}

#[instrument(skip_all, fields(player_id = player_id))]
pub fn on_disconnect(player_id: u64, state: &mut RoomState, writer: &mut RoomWriter) {
    player_left(player_id, state, writer)
}

fn player_left(player_id: u64, state: &mut RoomState, writer: &mut RoomWriter) {
    if state.players.remove(&player_id).is_some() {
        writer.broadcast(
            state.players.keys().copied(),
            PlayerEvent::PlayerDisappeared { player_id },
        );
    } else {
        tracing::error!("Player not found");
    }
}

pub fn on_command(
    player_id: u64,
    command: RoomCommand,
    state: &mut RoomState,
    writer: &mut RoomWriter,
) {
    match command {
        RoomCommand::Move { position, direction } => {
            let portal = state
                .portals
                .iter()
                .find(|portal| portal.position == position.map(|a| a as u32));

            if let Some(portal) = portal {
                let target_room_id = portal.target_room_id;
                let target_position = portal.target_position;
                player_left(player_id, state, writer);
                writer.upstream_messages.push(UpstreamMessage::PlayerLeftRoom {
                    sender_room_id: state.room.room_id,
                    player_id,
                    target_room_id,
                    target_position,
                });
            } else {
                state.players.entry(player_id).and_modify(|p| p.position = position);
                writer.broadcast(
                    state.players.keys().copied().filter(|pid| *pid != player_id),
                    PlayerEvent::PlayerMovementChanged { player_id, position, direction },
                );
            }
        }
    }
}
