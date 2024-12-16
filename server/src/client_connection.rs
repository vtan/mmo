use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::SystemTime;

use axum::extract::ws;
use axum::extract::ws::WebSocket;
use futures_util::stream::SplitSink;
use futures_util::{SinkExt, StreamExt};
use mmo_common::player_event::{PlayerEvent, PlayerEventEnvelope};
use tokio::sync::{broadcast, mpsc};
use tracing::instrument;

use crate::server_actor;

static NEXT_PLAYER_ID: AtomicU64 = AtomicU64::new(0);

#[instrument(skip_all)]
pub async fn handle(
    ws: WebSocket,
    server_actor_sender: mpsc::Sender<server_actor::Message>,
    tick_receiver: broadcast::Receiver<SystemTime>,
) {
    let player_id = NEXT_PLAYER_ID.fetch_add(1, Ordering::SeqCst);
    handle_with_id(ws, server_actor_sender, tick_receiver, player_id).await;
}

#[instrument(skip_all, fields(player_id = player_id))]
pub async fn handle_with_id(
    ws: WebSocket,
    server_actor_sender: mpsc::Sender<server_actor::Message>,
    mut tick_receiver: broadcast::Receiver<SystemTime>,
    player_id: u64,
) {
    tracing::debug!("Client connected");
    let (mut ws_sink, mut ws_stream) = ws.split();

    let (event_sender, mut event_receiver) = mpsc::channel::<Vec<Arc<PlayerEvent>>>(64);
    tokio::spawn(async move {
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
                    if let Ok(_now) = tick {
                        //
                    }
                }
            }
        }
        tracing::debug!("Closing sender");
        ws_sink.close().await.unwrap(); // TODO: unwrap
    });

    server_actor_sender
        .send(server_actor::Message::PlayerConnected { player_id, connection: event_sender })
        .await
        .unwrap();

    while let Some(Ok(message)) = ws_stream.next().await {
        if let ws::Message::Binary(bytes) = message {
            let command = postcard::from_bytes(&bytes).unwrap(); // TODO unwrap

            server_actor_sender
                .send(server_actor::Message::PlayerCommand { player_id, command })
                .await
                .unwrap(); // TODO: unwrap
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
