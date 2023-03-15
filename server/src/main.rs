mod client_connection;
mod room_actor;
mod server_actor;

use std::time::{Duration, SystemTime};

use tokio::io;
use tokio::sync::{broadcast, mpsc};
use tokio::time::{Instant, MissedTickBehavior};
use warp::Filter;

#[tokio::main]
async fn main() -> io::Result<()> {
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

    let routes = warp::path!("api" / "ws").and(warp::ws()).map(move |ws: warp::ws::Ws| {
        let message_sender = server_actor_sender.clone();
        let tick_receiver = tick_sender.subscribe();
        ws.on_upgrade(move |websocket| {
            client_connection::handle(websocket, message_sender, tick_receiver)
        })
    });

    let socket_addr = ([0, 0, 0, 0], 8081);
    warp::serve(routes).run(socket_addr).await;
    Ok(())
}
