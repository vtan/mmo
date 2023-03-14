use std::collections::HashMap;

use mmo_common::player_command::PlayerCommand;
use mmo_common::player_event::PlayerEvent;
use nalgebra::Vector2;
use tokio::sync::mpsc;

#[derive(Debug)]
pub enum Message {
    PlayerConnected { player_id: u64, connection: mpsc::Sender<PlayerEvent> },
    PlayerDisconnected { player_id: u64 },
    PlayerCommand { player_id: u64, command: PlayerCommand },
}

struct Player {
    id: u64,
    connection: mpsc::Sender<PlayerEvent>,
    position: Vector2<f32>,
}

pub async fn run(room_id: u64, mut messages: mpsc::Receiver<Message>) {
    log::debug!("Spawned for room {room_id}");

    let mut players: HashMap<u64, Player> = HashMap::new();
    let tiles = {
        let mut v = vec![];
        for x in 0..16 {
            for y in 0..16 {
                v.push((x, y));
            }
        }
        v
    };

    while let Some(message) = messages.recv().await {
        match message {
            Message::PlayerConnected { player_id, connection } => {
                let player_position = Vector2::new(0.0, 0.0);
                let player = Player {
                    id: player_id,
                    connection: connection.clone(),
                    position: player_position,
                };
                player
                    .connection
                    .send(PlayerEvent::SyncRoom { room_id, tiles: tiles.clone() })
                    .await
                    .unwrap(); // TODO: unwrap

                for player in players.values() {
                    connection
                        .send(PlayerEvent::PlayerMoved {
                            player_id: player.id,
                            position: player.position,
                        })
                        .await
                        .unwrap(); // TODO: unwrap
                }
                for observer in players.values() {
                    if observer.id != player_id {
                        observer
                            .connection
                            .send(PlayerEvent::PlayerMoved { player_id, position: player_position })
                            .await
                            .unwrap(); // TODO: unwrap
                    }
                }
                players.insert(player_id, player);
            }
            Message::PlayerDisconnected { player_id } => {
                players.remove(&player_id);
                for observer in players.values() {
                    observer
                        .connection
                        .send(PlayerEvent::PlayerDisappeared { player_id })
                        .await
                        .unwrap(); // TODO: unwrap
                }
            }
            Message::PlayerCommand { player_id, command } => {
                for (recipient_id, player) in players.iter() {
                    if *recipient_id != player_id {
                        match command {
                            PlayerCommand::Move { position } => {
                                let event = PlayerEvent::PlayerMoved { player_id, position };
                                player.connection.send(event).await.unwrap();
                            }
                        }
                    }
                }
            }
        }
    }

    if !players.is_empty() {
        log::warn!(
            "Terminating room {room_id} but still has {len} players",
            len = players.len()
        );
    }

    log::debug!("Terminated for room {room_id}");
}
