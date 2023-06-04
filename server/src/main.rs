mod client_connection;
mod room_actor;
mod server_actor;

use std::net::SocketAddr;
use std::time::{Duration, SystemTime};

use axum::extract::{State, WebSocketUpgrade};
use axum::response::IntoResponse;
use axum::routing::get;
use axum::Router;
use tokio::sync::{broadcast, mpsc};
use tokio::time::{Instant, MissedTickBehavior};

struct AppState {
    server_actor_sender: mpsc::Sender<server_actor::Message>,
    tick_sender: broadcast::Sender<(SystemTime, Duration)>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    pretty_env_logger::init();

    let start_monotonic = Instant::now();

    let (tick_sender, _) = broadcast::channel(8);
    let spawn_tick_sender = tick_sender.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_millis(100));
        interval.set_missed_tick_behavior(MissedTickBehavior::Skip);
        loop {
            let now_monotonic = interval.tick().await;
            let now = SystemTime::now();
            let since_start = now_monotonic - start_monotonic;
            let _ = spawn_tick_sender.send((now, since_start));
        }
    });

    let (server_actor_sender, server_actor_receiver) = mpsc::channel::<server_actor::Message>(4096);
    tokio::spawn(async move { server_actor::run(server_actor_receiver).await });

    let app_state = Box::leak(Box::new(AppState { server_actor_sender, tick_sender }));

    let app = Router::new().route("/api/ws", get(ws_handler)).with_state(app_state);

    axum::Server::bind(&SocketAddr::from(([0, 0, 0, 0], 8081)))
        .serve(app.into_make_service_with_connect_info::<SocketAddr>())
        .await?;

    Ok(())
}

async fn ws_handler(
    ws_upgrade: WebSocketUpgrade,
    State(app): State<&'static AppState>,
) -> impl IntoResponse {
    let message_sender = app.server_actor_sender.clone();
    let tick_receiver = app.tick_sender.subscribe();
    ws_upgrade.on_upgrade(move |ws| client_connection::handle(ws, message_sender, tick_receiver))
}
