mod client_connection;
mod room_actor;
mod server_actor;

use tokio::io;
use tokio::sync::mpsc;
use warp::Filter;

#[tokio::main]
async fn main() -> io::Result<()> {
    pretty_env_logger::init();

    let (server_actor_sender, server_actor_receiver) = mpsc::channel::<server_actor::Message>(4096);
    tokio::spawn(async move { server_actor::run(server_actor_receiver).await });

    let routes = warp::path!("api" / "ws").and(warp::ws()).map(move |ws: warp::ws::Ws| {
        let message_sender = server_actor_sender.clone();
        ws.on_upgrade(move |websocket| client_connection::handle(websocket, message_sender))
    });

    let socket_addr = ([0, 0, 0, 0], 8081);
    warp::serve(routes).run(socket_addr).await;
    Ok(())
}
