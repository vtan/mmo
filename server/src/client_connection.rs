use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, SystemTime};

use bincode::config::{Limit, LittleEndian, Varint};
use futures_util::stream::SplitSink;
use futures_util::{SinkExt, StreamExt};
use mmo_common::player_command::PlayerCommand;
use mmo_common::player_event::PlayerEvent;
use tokio::sync::{broadcast, mpsc};
use warp::ws::{self, WebSocket};

use crate::server_actor;

static NEXT_PLAYER_ID: AtomicU64 = AtomicU64::new(0);

const BINCODE_CONFIG: bincode::config::Configuration<LittleEndian, Varint, Limit<32_768>> =
    bincode::config::standard().with_limit::<32_768>();

pub async fn handle(
    ws: WebSocket,
    server_actor_sender: mpsc::Sender<server_actor::Message>,
    mut tick_receiver: broadcast::Receiver<(SystemTime, Duration)>,
) {
    log::debug!("New connection");
    let player_id = NEXT_PLAYER_ID.fetch_add(1, Ordering::SeqCst);

    let next_ping_sequence_number = Arc::new(AtomicU32::new(0));

    let (mut ws_sink, mut ws_stream) = ws.split();

    let (event_sender, mut event_receiver) = mpsc::channel::<PlayerEvent>(64);
    let next_ping_sequence_number_for_sender = next_ping_sequence_number.clone();
    tokio::spawn(async move {
        let next_ping_sequence_number = next_ping_sequence_number_for_sender;
        let mut ticks_since_last_ping = 0;
        loop {
            tokio::select! {
                event = event_receiver.recv() => {
                    if let Some(event) = event {
                        send_player_event(event, &mut ws_sink).await;
                    } else {
                        break;
                    }
                }
                tick = tick_receiver.recv() => {
                    if let Ok((now, since_start)) = tick {
                        ticks_since_last_ping += 1;
                        if  ticks_since_last_ping >= 10 {
                            let sequence_number = next_ping_sequence_number.fetch_add(1, Ordering::SeqCst);
                            let event = PlayerEvent::Ping{
                                sequence_number,
                                sent_at: since_start.as_millis() as u64
                            };
                            send_player_event(event, &mut ws_sink).await;

                            ticks_since_last_ping = 0;
                        }
                    }
                }
            }
        }
        ws_sink.close().await.unwrap(); // TODO: unwrap
        log::debug!("Sender closed");
    });

    server_actor_sender
        .send(server_actor::Message::PlayerConnected { player_id, connection: event_sender })
        .await
        .unwrap();

    while let Some(Ok(message)) = ws_stream.next().await {
        if message.is_binary() {
            let bytes = message.as_bytes();
            let (command, _) = bincode::decode_from_slice(bytes, BINCODE_CONFIG).unwrap();
            log::debug!("{player_id} {command:?}");

            match command {
                PlayerCommand::Pong { sequence_number, ping_sent_at } => {
                    let expected = next_ping_sequence_number.load(Ordering::SeqCst) - 1;
                    if expected == sequence_number {
                    } else {
                        log::debug!("Overdue pong from {player_id}: got {sequence_number}, expected {expected}");
                    }
                }
                command => {
                    server_actor_sender
                        .send(server_actor::Message::PlayerCommand { player_id, command })
                        .await
                        .unwrap(); // TODO: unwrap
                }
            }
        } else {
            log::warn!("Unexpected websocket message type");
            break;
        }
    }
    server_actor_sender
        .send(server_actor::Message::PlayerDisconnected { player_id })
        .await
        .unwrap(); // TODO: unwrap
    log::debug!("Receiver closed");
}

async fn send_player_event(event: PlayerEvent, ws_sink: &mut SplitSink<WebSocket, ws::Message>) {
    let encoded = bincode::encode_to_vec(event, BINCODE_CONFIG)
        .map_err(|e| e.to_string())
        .unwrap();
    ws_sink.send(warp::ws::Message::binary(encoded)).await.unwrap(); // TODO: unwrap
}
