use std::collections::HashMap;

use mmo_common::{PlayerCommand, PlayerEvent};
use tokio::sync::mpsc;

#[derive(Debug)]
pub enum Message {
    PlayerConnected { player_id: u64, connection: mpsc::Sender<PlayerEvent> },
    PlayerDisconnected { player_id: u64 },
    PlayerCommand { player_id: u64, command: PlayerCommand },
}

pub async fn run(mut messages: mpsc::Receiver<Message>) {
    let mut state = HashMap::new();

    while let Some(message) = messages.recv().await {
        match message {
            Message::PlayerConnected { player_id, connection } => {
                state.insert(player_id, connection);
            }
            Message::PlayerDisconnected { player_id } => {
                state.remove(&player_id);
            }
            Message::PlayerCommand { player_id, command } => {
                for (recipient_id, connection) in state.iter() {
                    if *recipient_id != player_id {
                        match command {
                            PlayerCommand::Move { position } => {
                                let event = PlayerEvent::PlayerMoved { player_id, position };
                                connection.send(event).await.unwrap();
                            }
                        }
                    }
                }
            }
        }
    }
}
