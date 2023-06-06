use std::collections::HashMap;

use mmo_common::player_command::RoomCommand;
use mmo_common::player_event::PlayerEvent;
use mmo_common::room::{RoomSync, Tile, TileIndex};
use nalgebra::Vector2;
use tokio::sync::mpsc;

use crate::room_logic;
use crate::room_state::{Portal, RoomState, RoomWriter, UpstreamMessage};

#[derive(Debug)]
pub enum Message {
    PlayerConnected { player_id: u64, connection: mpsc::Sender<PlayerEvent> },
    PlayerDisconnected { player_id: u64 },
    PlayerCommand { player_id: u64, command: RoomCommand },
}

pub async fn run(
    room_id: u64,
    mut messages: mpsc::Receiver<Message>,
    upstream_sender: mpsc::Sender<UpstreamMessage>,
) {
    tracing::debug!("Spawned for room {room_id}");

    let mut state = {
        let room_sync = if room_id == 0 {
            RoomSync {
                room_id,
                size: Vector2::new(8, 8),
                tiles: (0..8)
                    .flat_map(move |x| {
                        (0..8).filter_map(move |y| {
                            if x >= 2 && x < 5 && y >= 2 && y < 5 {
                                Some(Tile {
                                    position: Vector2::new(x, y),
                                    tile_index: TileIndex(21),
                                })
                            } else if y < 7 || x == 4 {
                                Some(Tile {
                                    position: Vector2::new(x, y),
                                    tile_index: TileIndex(0),
                                })
                            } else {
                                None
                            }
                        })
                    })
                    .collect(),
            }
        } else {
            RoomSync {
                room_id,
                size: Vector2::new(8, 8),
                tiles: (0..8)
                    .flat_map(move |x| {
                        (0..8).filter_map(move |y| {
                            if x >= 2 && x <= 5 && y >= 2 && y <= 5 && y != 4 {
                                Some(Tile {
                                    position: Vector2::new(x, y),
                                    tile_index: TileIndex(21),
                                })
                            } else if y > 0 || x == 4 {
                                Some(Tile {
                                    position: Vector2::new(x, y),
                                    tile_index: TileIndex(0),
                                })
                            } else {
                                None
                            }
                        })
                    })
                    .collect(),
            }
        };

        let portals = if room_id == 0 {
            vec![Portal { position: Vector2::new(4, 7), target_room_id: 1 }]
        } else {
            vec![Portal { position: Vector2::new(4, 0), target_room_id: 0 }]
        };
        RoomState { room: room_sync, portals, players: HashMap::new() }
    };
    let mut writer = RoomWriter::new();

    while let Some(message) = messages.recv().await {
        match message {
            Message::PlayerConnected { player_id, connection } => {
                room_logic::on_connect(player_id, connection, &mut state, &mut writer);
                flush_writer(&mut writer, &state, &upstream_sender).await;
            }

            Message::PlayerDisconnected { player_id } => {
                room_logic::on_disconnect(player_id, &mut state, &mut writer);
                flush_writer(&mut writer, &state, &upstream_sender).await;
            }

            Message::PlayerCommand { player_id, command } => {
                if state.players.contains_key(&player_id) {
                    room_logic::on_command(player_id, command, &mut state, &mut writer);
                    flush_writer(&mut writer, &state, &upstream_sender).await;
                } else {
                    tracing::error!("Player not found: {player_id}");
                }
            }
        }
    }

    if !state.players.is_empty() {
        tracing::warn!(
            "Terminating room {room_id} but still has {len} players",
            len = state.players.len()
        );
    }

    tracing::debug!("Terminated for room {room_id}");
}

// TODO: less awaits?
async fn flush_writer(
    writer: &mut RoomWriter,
    state: &RoomState,
    upstream_sender: &mpsc::Sender<UpstreamMessage>,
) {
    for (player_id, events) in writer.events.drain() {
        if let Some(player) = state.players.get(&player_id) {
            for event in events {
                player.connection.send(event).await.unwrap(); // TODO: unwrap
            }
        } else {
            tracing::error!("Player not found: {player_id}");
        }
    }
    for message in writer.upstream_messages.drain(..) {
        upstream_sender.send(message).await.unwrap(); // TODO: unwrap
    }
}
