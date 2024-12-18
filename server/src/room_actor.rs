use std::collections::HashMap;

use mmo_common::client_config::ClientConfig;
use mmo_common::object::ObjectId;
use mmo_common::player_command::RoomCommand;
use mmo_common::rle;
use mmo_common::room::{RoomId, RoomSync, TileIndex};
use nalgebra::Vector2;
use tokio::sync::mpsc;
use tracing::instrument;

use crate::player::PlayerConnection;
use crate::room_state::{Portal, RoomState, RoomWriter, UpstreamMessage};
use crate::{room_logic, tick};

#[derive(Debug)]
pub enum Message {
    PlayerConnected {
        player_id: ObjectId,
        connection: PlayerConnection,
        position: Vector2<f32>,
    },
    PlayerDisconnected {
        player_id: ObjectId,
    },
    PlayerCommand {
        player_id: ObjectId,
        command: RoomCommand,
    },
}

#[instrument(skip_all, fields(room_id = room_id.0))]
pub async fn run(
    room_id: RoomId,
    client_config: ClientConfig,
    mut messages: mpsc::Receiver<Message>,
    mut tick_receiver: tick::Receiver,
    upstream_sender: mpsc::Sender<UpstreamMessage>,
) {
    tracing::debug!("Spawned");

    let mut state = make_room(room_id, client_config);
    let mut writer = RoomWriter::new();

    loop {
        tokio::select! {
            message = messages.recv() => {
                if let Some(message) = message {
                    handle_message(&mut state, &mut writer, &upstream_sender, message).await;
                } else {
                    break;
                }
            }
            tick = tick_receiver.recv() => {
                match tick {
                    Ok(tick) => {
                        room_logic::on_tick(tick, &mut state, &mut writer);
                        flush_writer(&mut writer, &state, &upstream_sender).await;
                    }
                    Err(err) => {
                        tracing::error!("Error receiving tick: {err}");
                    }
                }
            }
        }
    }

    if !state.players.is_empty() {
        tracing::warn!("Terminating but still have {} players", state.players.len());
    }

    tracing::debug!("Terminated");
}

async fn handle_message(
    state: &mut RoomState,
    writer: &mut RoomWriter,
    upstream_sender: &mpsc::Sender<UpstreamMessage>,
    message: Message,
) {
    match message {
        Message::PlayerConnected { player_id, connection, position } => {
            room_logic::on_connect(player_id, connection, position, state, writer);
            flush_writer(writer, state, upstream_sender).await;
        }

        Message::PlayerDisconnected { player_id } => {
            room_logic::on_disconnect(player_id, state, writer);
            flush_writer(writer, state, upstream_sender).await;
        }

        Message::PlayerCommand { player_id, command } => {
            if state.players.contains_key(&player_id) {
                room_logic::on_command(player_id, command, state, writer);
                flush_writer(writer, state, upstream_sender).await;
            } else {
                tracing::error!(player_id = player_id.0, "Player not found");
            }
        }
    }
}

// TODO: less awaits?
async fn flush_writer(
    writer: &mut RoomWriter,
    state: &RoomState,
    upstream_sender: &mpsc::Sender<UpstreamMessage>,
) {
    for (player_id, events) in writer.events.drain() {
        if let Some(player) = state.players.get(&player_id) {
            player.connection.send(events).await.unwrap(); // TODO: unwrap
        } else {
            tracing::error!(player_id = player_id.0, "Player not found");
        }
    }
    for message in writer.upstream_messages.drain(..) {
        upstream_sender.send(message).await.unwrap(); // TODO: unwrap
    }
}

fn make_room(room_id: RoomId, client_config: ClientConfig) -> RoomState {
    let room_sync = if room_id.0 == 0 {
        let tiles: Vec<TileIndex> = (0..8)
            .flat_map(move |y| {
                (0..8).map(move |x| {
                    if x >= 2 && x < 5 && y >= 2 && y < 5 {
                        TileIndex(21)
                    } else if y < 7 || x == 4 {
                        TileIndex(0)
                    } else {
                        TileIndex(21)
                    }
                })
            })
            .collect();
        let tiles = rle::encode(&tiles);
        RoomSync { room_id, size: Vector2::new(8, 8), tiles }
    } else {
        let tiles: Vec<TileIndex> = (0..8)
            .flat_map(move |y| {
                (0..8).map(move |x| {
                    if x >= 2 && x <= 5 && y >= 2 && y <= 5 && y != 4 {
                        TileIndex(21)
                    } else if y > 0 || x == 4 {
                        TileIndex(0)
                    } else {
                        TileIndex(21)
                    }
                })
            })
            .collect();
        let tiles = rle::encode(&tiles);
        RoomSync { room_id, size: Vector2::new(8, 8), tiles }
    };

    let portals = if room_id.0 == 0 {
        vec![Portal {
            position: Vector2::new(4, 7),
            target_room_id: RoomId(1),
            target_position: Vector2::new(4.5, 1.5),
        }]
    } else {
        vec![Portal {
            position: Vector2::new(4, 0),
            target_room_id: RoomId(0),
            target_position: Vector2::new(4.5, 6.5),
        }]
    };
    RoomState {
        room: room_sync,
        portals,
        client_config,
        players: HashMap::new(),
    }
}
