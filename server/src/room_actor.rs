use std::collections::HashMap;
use std::sync::Arc;

use mmo_common::object::ObjectId;
use mmo_common::player_command::RoomCommand;
use mmo_common::rle;
use mmo_common::room::{RoomId, RoomSync};
use tokio::sync::mpsc;
use tracing::instrument;

use crate::room_state::{Player, RoomMap, RoomState, UpstreamMessage};
use crate::room_writer::{RoomWriter, RoomWriterTarget};
use crate::server_context::ServerContext;
use crate::{mob_logic, room_logic, tick};

#[derive(Debug)]
pub enum Message {
    PlayerConnected {
        player: Player,
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
    server_context: Arc<ServerContext>,
    mut messages: mpsc::Receiver<Message>,
    mut tick_receiver: tick::Receiver,
    upstream_sender: mpsc::Sender<UpstreamMessage>,
) {
    tracing::debug!("Spawned");

    let first_tick = tick_receiver
        .recv()
        .await
        .expect("Failed to receive first tick");

    let now = first_tick.monotonic_time;
    let map = server_context.world.maps.get(&room_id).unwrap().clone();
    let room = make_room_sync(room_id, &map);
    let mobs = mob_logic::populate_mobs(&map, &server_context, now);
    let mut state = RoomState {
        server_context,
        map,
        room,
        last_tick: first_tick,
        players: HashMap::new(),
        mobs,
        mob_respawns: vec![],
    };
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
                        state.last_tick = tick;
                        room_logic::on_tick(&mut state, &mut writer);
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
        Message::PlayerConnected { player } => {
            room_logic::on_connect(player, state, writer);
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
    // TODO: serialize once per batch, not for each player
    while !writer.events.is_empty() {
        let len = writer.events.len();
        let target = writer.events[len - 1].target;
        let mut batch = writer
            .events
            .iter()
            .rev()
            .take_while(|event| event.target == target)
            .map(|event| event.event.clone())
            .collect::<Vec<_>>();
        batch.reverse();
        writer.events.truncate(len - batch.len());

        match target {
            RoomWriterTarget::Player(player_id) => {
                if let Some(player) = state.players.get(&player_id) {
                    player.connection.send(batch).await.unwrap(); // TODO: unwrap
                } else {
                    tracing::error!(player_id = player_id.0, "Player not found");
                }
            }

            RoomWriterTarget::All => {
                for player in state.players.values() {
                    player.connection.send(batch.clone()).await.unwrap(); // TODO: unwrap
                }
            }

            RoomWriterTarget::AllExcept(player_id) => {
                for player in state.players.values() {
                    if player.id != player_id {
                        player.connection.send(batch.clone()).await.unwrap(); // TODO: unwrap
                    }
                }
            }
        }
    }

    for message in writer.upstream_messages.drain(..) {
        upstream_sender.send(message).await.unwrap(); // TODO: unwrap
    }
}

fn make_room_sync(room_id: RoomId, map: &RoomMap) -> RoomSync {
    let bg_dense_layers = map
        .bg_dense_layers
        .iter()
        .map(|layer| rle::encode(layer))
        .collect();
    let bg_sparse_layer = map.bg_sparse_layer.clone();
    let fg_sparse_layer = map.fg_sparse_layer.clone();
    let collisions = rle::encode(&map.collisions);

    RoomSync {
        room_id,
        size: map.size,
        bg_dense_layers,
        bg_sparse_layer,
        fg_sparse_layer,
        collisions,
    }
}
