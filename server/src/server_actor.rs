use std::collections::HashMap;

use mmo_common::{PlayerCommand, PlayerEvent};
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

pub async fn run(mut messages: mpsc::Receiver<Message>) {
    let mut players: HashMap<u64, Player> = HashMap::new();

    while let Some(message) = messages.recv().await {
        match message {
            Message::PlayerConnected { player_id, connection } => {
                let player_position = Vector2::new(0.0, 0.0);
                let player = Player {
                    id: player_id,
                    connection: connection.clone(),
                    position: player_position,
                };
                for player in players.values() {
                    connection
                        .send(PlayerEvent::PlayerMoved {
                            player_id: player.id,
                            position: player.position,
                        })
                        .await
                        .unwrap();
                }
                for observer in players.values() {
                    if observer.id != player_id {
                        observer
                            .connection
                            .send(PlayerEvent::PlayerMoved { player_id, position: player_position })
                            .await
                            .unwrap();
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
                        .unwrap();
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
}
