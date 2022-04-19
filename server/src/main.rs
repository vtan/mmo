use std::sync::atomic::{AtomicU64, Ordering};

use mmo_common::{MoveCommand, PlayerMovedEvent};
use server_actor::Message;
use tokio::{
    io::{self, AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt},
    net::TcpListener,
    sync::mpsc,
};

mod server_actor;

static NEXT_PLAYER_ID: AtomicU64 = AtomicU64::new(0);

#[tokio::main]
async fn main() -> io::Result<()> {
    let bincode_config = bincode::config::standard().with_limit::<32_768>();

    let listener = TcpListener::bind("127.0.0.1:11001").await?;

    let (actor_sender, actor_receiver) = mpsc::channel::<server_actor::Message>(4096);
    tokio::spawn(async move { server_actor::run(actor_receiver).await });

    loop {
        let (socket, _) = listener.accept().await?;
        socket.set_nodelay(true).unwrap();
        let (mut socket_reader, mut socket_writer) = tokio::io::split(socket);
        let actor_sender = actor_sender.clone();
        let player_id = NEXT_PLAYER_ID.fetch_add(1, Ordering::SeqCst);
        let (client_sender, mut client_receiver) = mpsc::channel::<PlayerMovedEvent>(4096);
        actor_sender
            .send(Message::PlayerConnected { player_id, connection: client_sender })
            .await
            .unwrap();

        tokio::spawn(async move {
            while let Some(event) = client_receiver.recv().await {
                write_event(event, &mut socket_writer, bincode_config).await;
            }
        });
        tokio::spawn(async move {
            let mut command_size_buf = [0; 4];
            let mut command_buf = [0; 65_536];
            loop {
                let result = read_command(
                    &mut socket_reader,
                    &mut command_size_buf,
                    &mut command_buf,
                    bincode_config,
                )
                .await;
                match result {
                    Ok(command) => {
                        actor_sender
                            .send(Message::PlayerCommand { player_id, command })
                            .await
                            .unwrap();
                    }
                    Err(err) => {
                        println!("{} {}", player_id, err);
                        actor_sender.send(Message::PlayerDisconnected { player_id }).await.unwrap();
                        break;
                    }
                }
            }
        });
    }
}

async fn read_command<R, C>(
    socket_reader: &mut R,
    command_size_buf: &mut [u8; 4],
    command_buf: &mut [u8; 65_536],
    bincode_config: C,
) -> Result<MoveCommand, String>
where
    R: AsyncRead + Unpin,
    C: bincode::config::Config,
{
    socket_reader.read_exact(command_size_buf).await.map_err(|e| e.to_string())?;
    let command_size = u32::from_le_bytes(*command_size_buf) as usize;
    if command_size > command_buf.len() {
        return Err("Too large command".to_string());
    }

    let command_slice = &mut command_buf[0..command_size];
    socket_reader.read_exact(command_slice).await.map_err(|e| e.to_string())?;

    let (command, read_bytes): (MoveCommand, _) =
        bincode::decode_from_slice(command_slice, bincode_config).map_err(|e| e.to_string())?;

    if read_bytes == command_size {
        Ok(command)
    } else {
        Err("Read fewer bytes than expected".to_string())
    }
}

async fn write_event<W, C>(
    event: PlayerMovedEvent,
    writer: &mut W,
    bincode_config: C,
) -> Result<(), String>
where
    W: AsyncWrite + Unpin,
    C: bincode::config::Config,
{
    // TODO: use a pre-allocated Vec?
    let encoded = bincode::encode_to_vec(event, bincode_config).map_err(|e| e.to_string())?;
    let encoded_size = (encoded.len() as u32).to_le_bytes();
    writer.write_all(&encoded_size).await.map_err(|e| e.to_string())?;
    writer.write_all(encoded.as_slice()).await.map_err(|e| e.to_string())?;
    Ok(())
}
