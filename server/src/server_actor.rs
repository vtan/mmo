use std::collections::HashMap;
use std::sync::Arc;

use mmo_common::player_command::{GlobalCommand, PlayerCommand};
use mmo_common::player_event::PlayerEvent;
use nalgebra::Vector2;
use tokio::sync::mpsc;
use tracing::instrument;

use crate::player::{PlayerConnection, CLIENT_CONFIG};
use crate::{room_actor, room_state};

#[derive(Debug)]
pub enum Message {
    PlayerConnected { player_id: u64, connection: PlayerConnection },
    PlayerDisconnected { player_id: u64 },
    PlayerCommand { player_id: u64, command: PlayerCommand },
}

impl Message {
    pub fn player_id(&self) -> u64 {
        match self {
            Message::PlayerConnected { player_id, .. } => *player_id,
            Message::PlayerDisconnected { player_id } => *player_id,
            Message::PlayerCommand { player_id, .. } => *player_id,
        }
    }
}

struct State {
    players: HashMap<u64, Player>,
    rooms: HashMap<u64, Room>,
    room_actor_upstream_sender: mpsc::Sender<room_state::UpstreamMessage>,
}

struct Player {
    id: u64,
    room_id: u64,
    connection: mpsc::Sender<Vec<Arc<PlayerEvent>>>,
}

struct Room {
    sender: mpsc::Sender<room_actor::Message>,
}

#[instrument(skip_all)]
pub async fn run(mut messages: mpsc::Receiver<Message>) {
    let (room_actor_upstream_sender, mut room_actor_upstream_receiver) =
        mpsc::channel::<room_state::UpstreamMessage>(4096);

    let mut state = State {
        players: HashMap::new(),
        rooms: HashMap::new(),
        room_actor_upstream_sender,
    };

    loop {
        tokio::select! {
            message = messages.recv() => {
                if let Some(message) = message {
                    handle_message(&mut state, message).await;
                } else {
                    break;
                }
            }
            upstream_message = room_actor_upstream_receiver.recv() => {
                if let Some(upstream_message) = upstream_message {
                    handle_upstream_message(&mut state, upstream_message).await;
                }
            }
        }
    }
}

#[instrument(skip_all, fields(player_id = message.player_id()))]
async fn handle_message(state: &mut State, message: Message) {
    match message {
        Message::PlayerConnected { player_id, connection } => {
            let start_room_id = 0;

            let player = Player {
                id: player_id,
                room_id: start_room_id,
                connection: connection.clone(),
            };
            state.players.insert(player_id, player);

            connection
                .send(vec![Arc::new(PlayerEvent::Initial {
                    player_id,
                    client_config: CLIENT_CONFIG,
                })])
                .await
                .unwrap(); // TODO: unwrap

            let room = get_or_create_room(state, start_room_id);
            room.sender
                .send(room_actor::Message::PlayerConnected {
                    player_id,
                    connection,
                    position: Vector2::new(0, 0),
                })
                .await
                .unwrap(); // TODO: unwrap
        }
        Message::PlayerDisconnected { player_id } => {
            if let Some(player) = state.players.remove(&player_id) {
                let room_id = player.room_id;
                if let Some(room) = state.rooms.get_mut(&room_id) {
                    room.sender
                        .send(room_actor::Message::PlayerDisconnected { player_id })
                        .await
                        .unwrap(); // TODO: unwrap
                } else {
                    tracing::warn!(
                        "Player disconnected but room {room_id} not found",
                        room_id = player.room_id
                    );
                    remove_room_if_empty(state, room_id);
                }
            } else {
                tracing::warn!("Player disconnected but not found");
            }
        }
        Message::PlayerCommand { player_id, command: PlayerCommand::GlobalCommand { command } } => {
            handle_global_command(state, player_id, command).await;
        }
        Message::PlayerCommand {
            player_id,
            command: PlayerCommand::RoomCommand { room_id, command },
        } => {
            let player_room_id = state.players.get(&player_id).map(|p| p.room_id);
            match player_room_id {
                Some(player_room_id) if player_room_id == room_id => {
                    get_or_create_room(state, room_id)
                        .sender
                        .send(room_actor::Message::PlayerCommand { player_id, command })
                        .await
                        .unwrap()
                }
                Some(_) => {
                    tracing::warn!("Got command with wrong room id {room_id}")
                }
                None => tracing::error!("Player sent command but not found"),
            }
        }
    }
}

async fn handle_global_command(state: &mut State, player_id: u64, message: GlobalCommand) {
    match message {
        GlobalCommand::Pong { .. } => {
            tracing::error!("Received pong in server actor")
        }
    }
}

async fn handle_upstream_message(state: &mut State, message: room_state::UpstreamMessage) {
    match message {
        room_state::UpstreamMessage::PlayerLeftRoom {
            sender_room_id,
            player_id,
            target_room_id,
            target_position,
        } => {
            // TODO: propagate to other players
            if let Some(player) = state.players.get_mut(&player_id) {
                player.room_id = target_room_id;

                let connection = player.connection.clone();
                let target_room = get_or_create_room(state, target_room_id);
                target_room
                    .sender
                    .send(room_actor::Message::PlayerConnected {
                        player_id,
                        connection,
                        position: target_position,
                    })
                    .await
                    .unwrap(); // TODO: unwrap
            } else {
                tracing::error!("Player not found");
            }
            remove_room_if_empty(state, sender_room_id);
        }
    }
}

fn get_or_create_room(state: &mut State, room_id: u64) -> &mut Room {
    let State { rooms, room_actor_upstream_sender, .. } = state;
    rooms.entry(room_id).or_insert_with(|| {
        let upstream_sender = room_actor_upstream_sender.clone();
        let (room_actor_sender, room_actor_receiver) = mpsc::channel::<room_actor::Message>(4096);
        tokio::spawn(async move {
            room_actor::run(room_id, room_actor_receiver, upstream_sender).await
        });
        Room { sender: room_actor_sender }
    })
}

fn remove_room_if_empty(state: &mut State, room_id: u64) {
    if !state.players.values().any(|player| player.room_id == room_id) {
        state.rooms.remove(&room_id);
    }
}
