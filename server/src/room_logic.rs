use mmo_common::{player_command::RoomCommand, player_event::PlayerEvent};
use nalgebra::Vector2;
use tokio::sync::mpsc;
use tracing::instrument;

use crate::room_state::{Player, RoomState, RoomWriter, UpstreamMessage};

#[instrument(skip_all, fields(player_id = player_id))]
pub fn on_connect(
    player_id: u64,
    connection: mpsc::Sender<PlayerEvent>,
    position: Vector2<u32>,
    state: &mut RoomState,
    writer: &mut RoomWriter,
) {
    let player = Player {
        id: player_id,
        connection,
        position: position.map(|a| a as _),
    };
    player_entered(player, state, writer);
}

fn player_entered(player: Player, state: &mut RoomState, writer: &mut RoomWriter) {
    let player_id = player.id;
    writer.tell(
        player_id,
        PlayerEvent::SyncRoom {
            room: state.room.clone(),
            position: player.position,
            players: state.players.iter().map(|(k, v)| (*k, v.position)).collect(),
        },
    );
    writer.tell_many(
        state.players.keys().copied().filter(|pid| *pid != player_id),
        PlayerEvent::PlayerMoved { player_id, position: player.position },
    );

    state.players.insert(player_id, player);
}

#[instrument(skip_all, fields(player_id = player_id))]
pub fn on_disconnect(player_id: u64, state: &mut RoomState, writer: &mut RoomWriter) {
    player_left(player_id, state, writer)
}

fn player_left(player_id: u64, state: &mut RoomState, writer: &mut RoomWriter) {
    if state.players.remove(&player_id).is_some() {
        writer.tell_many(
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
        RoomCommand::Move { position } => {
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
                writer.tell_many(
                    state.players.keys().copied().filter(|pid| *pid != player_id),
                    PlayerEvent::PlayerMoved { player_id, position },
                );
            }
        }
    }
}
