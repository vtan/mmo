use std::collections::HashMap;

use mmo_common::player_command::PlayerCommand;
use mmo_common::player_event::PlayerEvent;
use tokio::sync::mpsc;

use crate::room_actor;

#[derive(Debug)]
pub enum Message {
    PlayerConnected { player_id: u64, connection: mpsc::Sender<PlayerEvent> },
    PlayerDisconnected { player_id: u64 },
    PlayerCommand { player_id: u64, command: PlayerCommand },
}

struct State {
    players: HashMap<u64, Player>,
    rooms: HashMap<u64, Room>,
}

struct Player {
    id: u64,
    room_id: u64,
    connection: mpsc::Sender<PlayerEvent>,
}

struct Room {
    sender: mpsc::Sender<room_actor::Message>,
    player_count: i32,
}

pub async fn run(mut messages: mpsc::Receiver<Message>) {
    let mut state = State { players: HashMap::new(), rooms: HashMap::new() };

    while let Some(message) = messages.recv().await {
        match message {
            Message::PlayerConnected { player_id, connection } => {
                let start_room_id = 0;

                let player = Player {
                    id: player_id,
                    room_id: start_room_id,
                    connection: connection.clone(),
                };
                state.players.insert(player_id, player);

                let room = get_or_create_room(&mut state, start_room_id);
                room.player_count += 1;
                room.sender
                    .send(room_actor::Message::PlayerConnected { player_id, connection })
                    .await
                    .unwrap(); // TODO: unwrap
            }
            Message::PlayerDisconnected { player_id } => {
                if let Some(player) = state.players.remove(&player_id) {
                    if let Some(room) = state.rooms.get_mut(&player.room_id) {
                        room.sender
                            .send(room_actor::Message::PlayerDisconnected { player_id })
                            .await
                            .unwrap(); // TODO: unwrap

                        if room.player_count == 1 {
                            state.rooms.remove(&player.room_id);
                        } else {
                            room.player_count -= 1;
                        }
                    } else {
                        log::warn!(
                            "Player {player_id} disconnected but room {room_id} not found",
                            room_id = player.room_id
                        );
                    }
                } else {
                    log::warn!("Player {player_id} disconnected but not found");
                }
            }
            Message::PlayerCommand { player_id, command } => {
                let room_id = state.players.get(&player_id).map(|p| p.room_id);
                if let Some(room_id) = room_id {
                    get_or_create_room(&mut state, room_id)
                        .sender
                        .send(room_actor::Message::PlayerCommand { player_id, command })
                        .await
                        .unwrap(); // TODO: unwrap
                } else {
                    log::warn!("Player {player_id} sent command but not found");
                }
            }
        }
    }
}

fn get_or_create_room(state: &mut State, room_id: u64) -> &mut Room {
    state.rooms.entry(room_id).or_insert_with(|| {
        let (room_actor_sender, room_actor_receiver) = mpsc::channel::<room_actor::Message>(4096);
        tokio::spawn(async move { room_actor::run(room_id, room_actor_receiver).await });
        Room { sender: room_actor_sender, player_count: 0 }
    })
}
