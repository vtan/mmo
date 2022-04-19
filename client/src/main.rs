use std::{
    collections::HashMap,
    io::{Read, Write},
    net::TcpStream,
    sync::mpsc,
};

use mmo_common::{MoveCommand, PlayerMovedEvent};
use sdl2::{event::Event, keyboard::Keycode, pixels::Color, rect::Rect};

fn main() {
    let bincode_config = bincode::config::standard().with_limit::<32_768>();

    let mut socket = TcpStream::connect("127.0.0.1:11001").unwrap();
    socket.set_nodelay(true).unwrap();

    let mut local_player_x: u8 = 0;
    let mut players: HashMap<u64, u8> = HashMap::new();
    let socket_receiver = spawn_socket_reader(socket.try_clone().unwrap());

    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem.window("mmo", 960, 720).position_centered().build().unwrap();

    let mut canvas = window.into_canvas().accelerated().present_vsync().build().unwrap();

    let mut event_pump = sdl_context.event_pump().unwrap();
    let mut running = true;
    while running {
        socket_receiver.try_iter().for_each(|event| match event {
            PlayerMovedEvent { player_id, position } => {
                players.insert(player_id, position);
            }
        });

        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } | Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    running = false
                }
                Event::KeyDown { keycode: Some(Keycode::Right), .. } => {
                    local_player_x = local_player_x.wrapping_add(1);
                    let command = MoveCommand { position: local_player_x };
                    // TODO: use a pre-allocated Vec?
                    let encoded = bincode::encode_to_vec(command, bincode_config).unwrap();
                    let encoded_size = (encoded.len() as u32).to_le_bytes();
                    socket.write_all(&encoded_size).unwrap();
                    socket.write_all(encoded.as_slice()).unwrap();
                }
                _ => (),
            }
        }

        canvas.set_draw_color(Color::RGB(0, 0, 0));
        canvas.clear();

        canvas.set_draw_color(Color::RGB(255, 0, 0));
        canvas.fill_rect(Rect::new(local_player_x as i32, 400, 32, 32)).unwrap();

        canvas.set_draw_color(Color::RGB(0, 0, 255));
        players.values().for_each(|player_x| {
            canvas.fill_rect(Rect::new(*player_x as i32, 400, 32, 32)).unwrap();
        });

        canvas.present();
    }
}

fn spawn_socket_reader(mut socket: TcpStream) -> mpsc::Receiver<PlayerMovedEvent> {
    let bincode_config = bincode::config::standard().with_limit::<32_768>();
    let (connection_sender, connection_receiver) = mpsc::channel::<PlayerMovedEvent>();
    std::thread::spawn(move || loop {
        let mut event_size_buf = [0; 4];
        socket.read_exact(&mut event_size_buf).unwrap();
        let event_size = u32::from_le_bytes(event_size_buf) as usize;

        let mut event_buf = vec![0; event_size];
        socket.read_exact(&mut event_buf).unwrap();

        let (event, _) = bincode::decode_from_slice(&event_buf[..], bincode_config).unwrap();
        connection_sender.send(event).unwrap();
    });
    connection_receiver
}
