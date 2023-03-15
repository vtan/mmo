use std::sync::atomic::{AtomicU64, Ordering};

use bincode::config::{Limit, LittleEndian, Varint, WriteFixedArrayLength};
use futures_util::{SinkExt, StreamExt};
use mmo_common::player_event::PlayerEvent;
use tokio::sync::mpsc;
use warp::ws::WebSocket;

use crate::server_actor;

static NEXT_PLAYER_ID: AtomicU64 = AtomicU64::new(0);

const BINCODE_CONFIG: bincode::config::Configuration<
    LittleEndian,
    Varint,
    WriteFixedArrayLength,
    Limit<32_768>,
> = bincode::config::standard().with_limit::<32_768>();

pub async fn handle(ws: WebSocket, actor_sender: mpsc::Sender<server_actor::Message>) {
    log::debug!("New connection");
    let player_id = NEXT_PLAYER_ID.fetch_add(1, Ordering::SeqCst);
    let (mut ws_sink, mut ws_stream) = ws.split();

    let (event_sender, mut event_receiver) = mpsc::channel::<PlayerEvent>(64);
    tokio::spawn(async move {
        while let Some(event) = event_receiver.recv().await {
            let encoded = bincode::encode_to_vec(event, BINCODE_CONFIG)
                .map_err(|e| e.to_string())
                .unwrap();
            ws_sink.send(warp::ws::Message::binary(encoded)).await.unwrap();
        }
        ws_sink.close().await.unwrap(); // TODO: unwrap
        log::debug!("Sender closed");
    });

    actor_sender
        .send(server_actor::Message::PlayerConnected { player_id, connection: event_sender })
        .await
        .unwrap();

    while let Some(Ok(message)) = ws_stream.next().await {
        if message.is_binary() {
            let bytes = message.as_bytes();
            let (command, _) = bincode::decode_from_slice(bytes, BINCODE_CONFIG).unwrap();
            log::debug!("{player_id} {command:?}");
            actor_sender
                .send(server_actor::Message::PlayerCommand { player_id, command })
                .await
                .unwrap(); // TODO: unwrap
        } else {
            log::warn!("Unexpected websocket message type");
            break;
        }
    }
    actor_sender
        .send(server_actor::Message::PlayerDisconnected { player_id })
        .await
        .unwrap(); // TODO: unwrap
    log::debug!("Receiver closed");
}
