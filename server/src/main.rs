mod assets;
mod client_connection;
mod ldtk_map;
mod mob;
mod object;
mod player;
mod room_actor;
mod room_logic;
mod room_state;
mod server_actor;
mod server_context;
mod tick;

use std::sync::Arc;

use axum::extract::{Path, State, WebSocketUpgrade};
use axum::http::{HeaderValue, Response};
use axum::response::{ErrorResponse, IntoResponse};
use axum::routing::get;
use axum::Router;
use server_context::{ServerConfig, ServerContext};
use tokio::net::TcpSocket;
use tokio::sync::mpsc;

struct AppState {
    server_actor_sender: mpsc::Sender<server_actor::Message>,
    server_context: Arc<ServerContext>,
}

#[tokio::main]
async fn main() -> eyre::Result<()> {
    tracing_subscriber::fmt::init();

    tracing::info!("Loading maps...");
    let room_maps = ldtk_map::load("data/map.ldtk")?;
    tracing::info!("Loaded maps");

    tracing::info!("Loading assets...");
    let asset_paths = assets::load_assets()?;
    tracing::info!("Loaded assets");

    tracing::info!("Loading config...");
    let config = ServerConfig::load("data/config.toml")?;
    tracing::info!("Loaded config");

    let server_context = Arc::new(ServerContext::new(config, asset_paths, room_maps)?);

    let (tick_sender, _) = tick::spawn_producer();

    let (server_actor_sender, server_actor_receiver) = mpsc::channel::<server_actor::Message>(4096);
    tokio::spawn({
        let server_context = server_context.clone();
        async move { server_actor::run(server_context, server_actor_receiver, tick_sender).await }
    });

    let app_state = Box::leak(Box::new(AppState { server_actor_sender, server_context }));

    let app = Router::new()
        .route("/api/ws", get(ws_handler))
        .route("/assets/:filename", get(serve_file_handler))
        .with_state(app_state);

    let listener = {
        let socket = TcpSocket::new_v4()?;
        socket.set_reuseaddr(true)?;
        socket.set_nodelay(true)?;
        socket.bind("0.0.0.0:8081".parse()?)?;
        socket.listen(1024)?
    };
    axum::serve(listener, app).await?;

    Ok(())
}

async fn ws_handler(
    ws_upgrade: WebSocketUpgrade,
    State(app): State<&'static AppState>,
) -> impl IntoResponse {
    let message_sender = app.server_actor_sender.clone();
    ws_upgrade.on_upgrade(move |ws| client_connection::handle(ws, message_sender))
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
