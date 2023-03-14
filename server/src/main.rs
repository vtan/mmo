mod room_actor;

use std::marker::Send;
use std::sync::atomic::{AtomicU64, Ordering};

use futures_util::{SinkExt, StreamExt};
use mmo_common::player_event::PlayerEvent;
use tokio::io;
use tokio::sync::mpsc;
use warp::ws::WebSocket;
use warp::Filter;

static NEXT_PLAYER_ID: AtomicU64 = AtomicU64::new(0);

#[tokio::main]
async fn main() -> io::Result<()> {
    pretty_env_logger::init();

    let bincode_config = bincode::config::standard().with_limit::<32_768>();

    let (actor_sender, actor_receiver) = mpsc::channel::<room_actor::Message>(4096);
    tokio::spawn(async move { room_actor::run(actor_receiver).await });

    let routes = warp::path!("api" / "ws").and(warp::ws()).map(move |ws: warp::ws::Ws| {
        let message_sender = actor_sender.clone();
        ws.on_upgrade(move |websocket| handle_connection(websocket, message_sender, bincode_config))
    });

    let socket_addr = ([0, 0, 0, 0], 8081);
    warp::serve(routes).run(socket_addr).await;
    Ok(())
}

async fn handle_connection<C>(
    ws: WebSocket,
    actor_sender: mpsc::Sender<room_actor::Message>,
    bincode_config: C,
) where
    C: bincode::config::Config + Send + 'static,
{
    log::debug!("New connection");
    let player_id = NEXT_PLAYER_ID.fetch_add(1, Ordering::SeqCst);
    let (mut ws_sink, mut ws_stream) = ws.split();

    let (event_sender, mut event_receiver) = mpsc::channel::<PlayerEvent>(64);
    tokio::spawn(async move {
        while let Some(event) = event_receiver.recv().await {
            let encoded = bincode::encode_to_vec(event, bincode_config)
                .map_err(|e| e.to_string())
                .unwrap();
            ws_sink.send(warp::ws::Message::binary(encoded)).await.unwrap();
        }
        ws_sink.close().await.unwrap();
        log::debug!("Sender closed");
    });

    actor_sender
        .send(room_actor::Message::PlayerConnected { player_id, connection: event_sender })
        .await
        .unwrap();

    while let Some(Ok(message)) = ws_stream.next().await {
        if message.is_binary() {
            let bytes = message.as_bytes();
            let (command, _) = bincode::decode_from_slice(bytes, bincode_config).unwrap();
            log::debug!("{player_id} {command:?}");
            actor_sender
                .send(room_actor::Message::PlayerCommand { player_id, command })
                .await
                .unwrap();
        } else {
            log::warn!("Unexpected websocket message type");
            break;
        }
    }
    actor_sender
        .send(room_actor::Message::PlayerDisconnected { player_id })
        .await
        .unwrap();
    log::debug!("Receiver closed");
}
