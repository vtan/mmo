mod assets;
mod client_connection;
mod player;
mod room_actor;
mod room_logic;
mod room_state;
mod server_actor;
mod server_context;

use std::sync::Arc;
use std::time::{Duration, SystemTime};

use axum::extract::{Path, State, WebSocketUpgrade};
use axum::http::{HeaderValue, Response};
use axum::response::{ErrorResponse, IntoResponse};
use axum::routing::get;
use axum::Router;
use server_context::ServerContext;
use tokio::sync::{broadcast, mpsc};
use tokio::time::{Instant, MissedTickBehavior};

struct AppState {
    server_actor_sender: mpsc::Sender<server_actor::Message>,
    tick_sender: broadcast::Sender<(SystemTime, Duration)>,
    server_context: Arc<ServerContext>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    tracing::info!("Loading assets...");
    let asset_paths = assets::load_assets()?;
    tracing::info!("Loaded assets");

    let server_context = Arc::new(ServerContext { asset_paths });

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
    tokio::spawn({
        let server_context = server_context.clone();
        async move { server_actor::run(server_context, server_actor_receiver).await }
    });

    let app_state = Box::leak(Box::new(AppState {
        server_actor_sender,
        tick_sender,
        server_context,
    }));

    let app = Router::new()
        .route("/api/ws", get(ws_handler))
        .route("/assets/:filename", get(serve_file_handler))
        .with_state(app_state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8081").await?;
    axum::serve(listener, app).await?;

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

async fn serve_file_handler(
    path: Path<String>,
    State(app): State<&'static AppState>,
) -> Result<impl IntoResponse, ErrorResponse> {
    tracing::debug!("Serving asset: {}", path.as_str());

    let asset_path = app
        .server_context
        .asset_paths
        .lookup
        .get(path.as_str())
        .ok_or_else(|| ErrorResponse::from(axum::http::StatusCode::NOT_FOUND))?;

    // TODO: stream response
    // TODO: use correct content type
    let local_path = &asset_path.local_path;
    let file_bytes = tokio::fs::read(local_path).await.map_err(|err| {
        tracing::error!("Failed to read file: {local_path} ({err})");
        ErrorResponse::from(axum::http::StatusCode::INTERNAL_SERVER_ERROR)
    })?;
    let mut response: Response<_> = file_bytes.into_response();
    response.headers_mut().insert(
        "Content-Type",
        HeaderValue::from_static("application/octet-stream"),
    );
    Ok(response)
}
