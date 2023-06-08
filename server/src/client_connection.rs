use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, SystemTime};

use axum::extract::ws;
use axum::extract::ws::WebSocket;
use futures_util::stream::SplitSink;
use futures_util::{SinkExt, StreamExt};
use mmo_common::player_command::{GlobalCommand, PlayerCommand};
use mmo_common::player_event::{PlayerEvent, PlayerEventEnvelope};
use tokio::sync::{broadcast, mpsc};
use tracing::instrument;

use crate::server_actor;

static NEXT_PLAYER_ID: AtomicU64 = AtomicU64::new(0);

#[instrument(skip_all)]
pub async fn handle(
    ws: WebSocket,
    server_actor_sender: mpsc::Sender<server_actor::Message>,
    mut tick_receiver: broadcast::Receiver<(SystemTime, Duration)>,
) {
    let player_id = NEXT_PLAYER_ID.fetch_add(1, Ordering::SeqCst);

    let next_ping_sequence_number = Arc::new(AtomicU32::new(0));

    let (mut ws_sink, mut ws_stream) = ws.split();

    let (event_sender, mut event_receiver) = mpsc::channel::<Vec<Arc<PlayerEvent>>>(64);
    let next_ping_sequence_number_for_sender = next_ping_sequence_number.clone();
    tokio::spawn(async move {
        let next_ping_sequence_number = next_ping_sequence_number_for_sender;
        let mut ticks_since_last_ping = 0;
        loop {
            tokio::select! {
                events = event_receiver.recv() => {
                    if let Some(events) = events {
                        let envelope = PlayerEventEnvelope { events };
                        send_player_event(&envelope, &mut ws_sink).await;
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
                            let envelope = PlayerEventEnvelope {
                                events: vec![Arc::new(event)]
                            };
                            send_player_event(&envelope, &mut ws_sink).await;

                            ticks_since_last_ping = 0;
                        }
                    }
                }
            }
        }
        ws_sink.close().await.unwrap(); // TODO: unwrap
    });

    server_actor_sender
        .send(server_actor::Message::PlayerConnected { player_id, connection: event_sender })
        .await
        .unwrap();

    while let Some(Ok(message)) = ws_stream.next().await {
        if let ws::Message::Binary(bytes) = message {
            let command = postcard::from_bytes(&bytes).unwrap(); // TODO unwrap

            match command {
                PlayerCommand::GlobalCommand {
                    command: GlobalCommand::Pong { sequence_number, .. },
                } => {
                    let expected = next_ping_sequence_number.load(Ordering::SeqCst) - 1;
                    if expected == sequence_number {
                    } else {
                        tracing::debug!(
                            player_id,
                            "Overdue pong from: got {sequence_number}, expected {expected}"
                        );
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
            tracing::warn!("Unexpected websocket message type");
            break;
        }
    }
    server_actor_sender
        .send(server_actor::Message::PlayerDisconnected { player_id })
        .await
        .unwrap(); // TODO: unwrap
    tracing::debug!("Receiver closed");
}

async fn send_player_event(
    envelope: &PlayerEventEnvelope<Arc<PlayerEvent>>,
    ws_sink: &mut SplitSink<WebSocket, ws::Message>,
) {
    let encoded = postcard::to_stdvec(envelope).unwrap();
    // TODO: this happens with ConnectionClosed sometimes
    ws_sink.send(ws::Message::Binary(encoded)).await.unwrap();
}
