use std::collections::HashMap;
use std::sync::Arc;

use eyre::Result;
use mmo_common::object::ObjectId;
use mmo_common::player_command::{
    GlobalCommand, PlayerCommand, PlayerCommandEnvelope, RoomCommand,
};
use mmo_common::player_event::PlayerEvent;
use mmo_common::room::RoomId;
use tokio::sync::mpsc;
use tracing::instrument;

use crate::player::{self, PlayerConnection};
use crate::room_state::{LocalMovement, Player, RemoteMovement};
use crate::server_context::ServerContext;
use crate::tick::{self, Tick};
use crate::{room_actor, room_state};

#[derive(Debug)]
pub enum Message {
    PlayerConnected {
        player_id: ObjectId,
        connection: PlayerConnection,
    },
    PlayerDisconnected {
        player_id: ObjectId,
    },
    PlayerCommand {
        player_id: ObjectId,
        command: PlayerCommandEnvelope,
    },
}

impl Message {
    pub fn player_id(&self) -> ObjectId {
        match self {
            Message::PlayerConnected { player_id, .. } => *player_id,
            Message::PlayerDisconnected { player_id } => *player_id,
            Message::PlayerCommand { player_id, .. } => *player_id,
        }
    }
}

struct State {
    server_context: Arc<ServerContext>,
    players: HashMap<ObjectId, PlayerMeta>,
    rooms: HashMap<RoomId, Room>,
    tick_sender: tick::Sender,
    room_actor_upstream_sender: mpsc::Sender<room_state::UpstreamMessage>,
}

struct PlayerMeta {
    id: ObjectId,
    room_id: RoomId,
    connection: mpsc::Sender<Vec<Arc<PlayerEvent>>>,
}

struct Room {
    sender: mpsc::Sender<room_actor::Message>,
}

#[instrument(skip_all)]
pub async fn run(
    server_context: Arc<ServerContext>,
    mut messages: mpsc::Receiver<Message>,
    tick_sender: tick::Sender,
) {
    let (room_actor_upstream_sender, mut room_actor_upstream_receiver) =
        mpsc::channel::<room_state::UpstreamMessage>(4096);

    let mut state = State {
        server_context,
        players: HashMap::new(),
        rooms: HashMap::new(),
        tick_sender,
        room_actor_upstream_sender,
    };

    loop {
        tokio::select! {
            message = messages.recv() => {
                if let Some(message) = message {
                    if let Err(err) = handle_message(&mut state, message).await {
                        tracing::error!("Error handling message: {err}");
                    }
                } else {
                    break;
                }
            }
            upstream_message = room_actor_upstream_receiver.recv() => {
                if let Some(upstream_message) = upstream_message {
                    if let Err(err) = handle_upstream_message(&mut state, upstream_message).await {
                        tracing::error!("Error handling upstream message: {err}");
                    }
                }
            }
        }
    }
}

fn create_new_player(id: ObjectId, connection: PlayerConnection, ctx: &ServerContext) -> Player {
    let now = tokio::time::Instant::now();
    let max_health = ctx.player.max_health;
    Player {
        id,
        connection,
        remote_movement: RemoteMovement {
            position: ctx.world.start_position,
            direction: None,
            look_direction: mmo_common::object::Direction4::Down,
            received_at: now,
        },
        local_movement: LocalMovement {
            position: ctx.world.start_position,
            updated_at: now,
        },
        health: max_health,
        max_health,
        last_damaged_at: Tick(0),
    }
}

#[instrument(skip_all, fields(player_id = message.player_id().0))]
async fn handle_message(state: &mut State, message: Message) -> Result<()> {
    match message {
        Message::PlayerConnected {
            player_id,
            connection,
        } => {
            let room_id = state.server_context.world.start_room_id;

            let player_meta = PlayerMeta {
                id: player_id,
                room_id,
                connection: connection.clone(),
            };
            state.players.insert(player_id, player_meta);

            connection
                .send(vec![Arc::new(PlayerEvent::Initial {
                    self_id: player_id,
                    client_config: Box::new(player::client_config(&state.server_context)),
                })])
                .await?;

            let player = create_new_player(player_id, connection, &state.server_context);

            let room = get_or_create_room(state, room_id);
            room.sender
                .send(room_actor::Message::PlayerConnected { player })
                .await?;
        }
        Message::PlayerDisconnected { player_id } => {
            if let Some(player) = state.players.remove(&player_id) {
                let room_id = player.room_id;
                if let Some(room) = state.rooms.get_mut(&room_id) {
                    room.sender
                        .send(room_actor::Message::PlayerDisconnected { player_id })
                        .await?;
                    remove_room_if_empty(state, room_id);
                } else {
                    tracing::warn!(
                        "Player disconnected but room {room_id:?} not found",
                        room_id = player.room_id
                    );
                }
            } else {
                tracing::warn!("Player disconnected but not found");
            }
        }
        Message::PlayerCommand { player_id, command } => {
            let room_id = command.room_id;
            for command in command.commands {
                match command {
                    PlayerCommand::GlobalCommand(command) => {
                        handle_global_command(state, player_id, command).await?
                    }
                    PlayerCommand::RoomCommand(command) => {
                        handle_room_command(state, player_id, room_id, command).await?
                    }
                }
            }
        }
    }
    Ok(())
}

async fn handle_room_command(
    state: &mut State,
    player_id: ObjectId,
    room_id: RoomId,
    command: RoomCommand,
) -> Result<()> {
    let player_room_id = state.players.get(&player_id).map(|p| p.room_id);
    match player_room_id {
        Some(player_room_id) if player_room_id == room_id => {
            get_or_create_room(state, room_id)
                .sender
                .send(room_actor::Message::PlayerCommand { player_id, command })
                .await?
        }
        Some(_) => {
            tracing::warn!("Got command with wrong room id {room_id:?}")
        }
        None => tracing::error!("Player sent command but not found"),
    }
    Ok(())
}

async fn handle_global_command(
    state: &mut State,
    player_id: ObjectId,
    message: GlobalCommand,
) -> Result<()> {
    match message {
        GlobalCommand::Ping { sequence_number } => {
            let pong = PlayerEvent::Pong { sequence_number };
            if let Some(player) = state.players.get(&player_id) {
                player.connection.send(vec![Arc::new(pong)]).await?
            }
        }
    }
    Ok(())
}

async fn handle_upstream_message(
    state: &mut State,
    message: room_state::UpstreamMessage,
) -> Result<()> {
    match message {
        room_state::UpstreamMessage::PlayerLeftRoom {
            sender_room_id,
            mut player,
            target_room_id,
            target_position,
        } => {
            if let Some(player_meta) = state.players.get_mut(&player.id) {
                player_meta.room_id = target_room_id;
                player.remote_movement.position = target_position;
                player.local_movement.position = target_position;

                let target_room = get_or_create_room(state, target_room_id);
                target_room
                    .sender
                    .send(room_actor::Message::PlayerConnected { player })
                    .await?;
            } else {
                tracing::error!("Player not found");
            }
            remove_room_if_empty(state, sender_room_id);
        }
    }
    Ok(())
}

fn get_or_create_room(state: &mut State, room_id: RoomId) -> &mut Room {
    let State {
        rooms,
        room_actor_upstream_sender,
        ..
    } = state;
    rooms.entry(room_id).or_insert_with(|| {
        let server_context = state.server_context.clone();
        let upstream_sender = room_actor_upstream_sender.clone();
        let (room_actor_sender, room_actor_receiver) = mpsc::channel::<room_actor::Message>(4096);
        let tick_receiver = state.tick_sender.subscribe();
        tokio::spawn(async move {
            room_actor::run(
                room_id,
                server_context,
                room_actor_receiver,
                tick_receiver,
                upstream_sender,
            )
            .await
        });
        Room {
            sender: room_actor_sender,
        }
    })
}

fn remove_room_if_empty(state: &mut State, room_id: RoomId) {
    if !state
        .players
        .values()
        .any(|player| player.room_id == room_id)
    {
        state.rooms.remove(&room_id);
    }
}
