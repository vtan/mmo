use std::sync::atomic::{AtomicU64, Ordering};

use server_actor::Message;
use tokio::{
    io::{self, AsyncReadExt, AsyncWriteExt},
    net::TcpListener,
    sync::mpsc,
};

mod server_actor;

static NEXT_PLAYER_ID: AtomicU64 = AtomicU64::new(0);

#[tokio::main]
async fn main() -> io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:11001").await?;

    let (actor_sender, actor_receiver) = mpsc::channel::<server_actor::Message>(4096);
    tokio::spawn(async move { server_actor::run(actor_receiver).await });

    loop {
        let (socket, _) = listener.accept().await?;
        let (mut socket_reader, mut socket_writer) = tokio::io::split(socket);
        let actor_sender = actor_sender.clone();
        let player_id = NEXT_PLAYER_ID.fetch_add(1, Ordering::SeqCst);
        let (client_sender, mut client_receiver) = mpsc::channel::<(u64, u8)>(4096);
        actor_sender
            .send(Message::PlayerConnected { player_id, connection: client_sender })
            .await
            .unwrap();

        tokio::spawn(async move {
            while let Some((player_id, position)) = client_receiver.recv().await {
                let mut buffer = vec![];
                buffer.extend(player_id.to_be_bytes());
                buffer.push(position);
                socket_writer.write_all(buffer.as_slice()).await.unwrap();
            }
        });
        tokio::spawn(async move {
            loop {
                let mut buffer = [0; 1];
                let read_bytes = socket_reader.read(&mut buffer[..]).await.unwrap();
                if read_bytes == 0 {
                    actor_sender.send(Message::PlayerDisconnected { player_id }).await.unwrap();
                    break;
                } else {
                    actor_sender
                        .send(Message::PlayerCommand { player_id, position: buffer[0] })
                        .await
                        .unwrap();
                }
            }
        });
    }
}
