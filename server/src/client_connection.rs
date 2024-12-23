use std::sync::Arc;
use std::time::Duration;

use axum::extract::ws;
use axum::extract::ws::WebSocket;
use futures_util::stream::{SplitSink, SplitStream};
use futures_util::{SinkExt, StreamExt};
use mmo_common::object::ObjectId;
use mmo_common::player_command::PlayerHandshake;
use mmo_common::player_event::{PlayerEvent, PlayerEventEnvelope};
use tokio::sync::mpsc;
use tracing::instrument;

use crate::{object, server_actor};

pub async fn handle(ws: WebSocket, server_actor_sender: mpsc::Sender<server_actor::Message>) {
    let player_id = object::next_object_id();
    handle_with_id(ws, server_actor_sender, player_id).await;
}

#[instrument(skip_all, fields(player_id = player_id.0))]
pub async fn handle_with_id(
    ws: WebSocket,
    server_actor_sender: mpsc::Sender<server_actor::Message>,
    player_id: ObjectId,
) {
    tracing::debug!("Client connected");
    let (mut ws_sink, mut ws_stream) = ws.split();

    if !expect_handshake(&mut ws_stream).await {
        return;
    }
    tracing::info!("Client joined");

    let (event_sender, mut event_receiver) = mpsc::channel::<Vec<Arc<PlayerEvent>>>(64);
    tokio::spawn(async move {
        while let Some(events) = event_receiver.recv().await {
            let envelope = PlayerEventEnvelope { events };
            send_player_event(&envelope, &mut ws_sink).await;
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
            let command = match postcard::from_bytes(&bytes) {
                Ok(command) => command,
                Err(err) => {
                    tracing::warn!("Error deserializing command from {player_id:?}: {err:?}");
                    break;
                }
            };

            server_actor_sender
                .send(server_actor::Message::PlayerCommand { player_id, command })
                .await
                .unwrap(); // TODO: unwrap
        } else if let ws::Message::Close(_) = message {
            tracing::debug!("Received close message");
            break;
        } else {
            tracing::warn!("Unexpected websocket message type");
            break;
        }
    }
    server_actor_sender
        .send(server_actor::Message::PlayerDisconnected { player_id })
        .await
        .unwrap(); // TODO: unwrap

    tracing::info!("Client disconnected");
}

async fn expect_handshake(ws_stream: &mut SplitStream<WebSocket>) -> bool {
    let timeout = Duration::from_secs(3);
    let msg = tokio::time::timeout(timeout, ws_stream.next());
    match msg.await {
        Ok(Some(Ok(ws::Message::Binary(bytes)))) => {
            match postcard::from_bytes::<PlayerHandshake>(&bytes) {
                Ok(handshake) if handshake.is_valid() => true,
                Ok(_) => {
                    tracing::warn!("Invalid handshake");
                    false
                }
                Err(err) => {
                    tracing::warn!("Error deserializing handshake: {err:?}");
                    false
                }
            }
        }
        Ok(_) => {
            tracing::warn!("Unexpected message type for handshake");
            false
        }
        Err(_) => {
            tracing::warn!("Handshake timeout");
            false
        }
    }
}

async fn send_player_event(
    envelope: &PlayerEventEnvelope<Arc<PlayerEvent>>,
    ws_sink: &mut SplitSink<WebSocket, ws::Message>,
) {
    let encoded = postcard::to_stdvec(envelope).unwrap();
    // TODO: this happens with ConnectionClosed sometimes
    ws_sink.send(ws::Message::Binary(encoded)).await.unwrap();
}
