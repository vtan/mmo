use std::{
    collections::HashMap,
    io::{Read, Write},
    net::TcpStream,
    sync::mpsc,
};

use sdl2::{event::Event, keyboard::Keycode, pixels::Color, rect::Rect};

fn main() {
    let mut socket = TcpStream::connect("127.0.0.1:11001").unwrap();

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
        socket_receiver.try_iter().for_each(|(player_id, position)| {
            players.insert(player_id, position);
        });

        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } | Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    running = false
                }
                Event::KeyDown { keycode: Some(Keycode::Right), .. } => {
                    local_player_x = local_player_x.wrapping_add(1);
                    socket.write_all(&[local_player_x]).unwrap();
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

fn spawn_socket_reader(mut socket: TcpStream) -> mpsc::Receiver<(u64, u8)> {
    let (connection_sender, connection_receiver) = mpsc::channel::<(u64, u8)>();
    std::thread::spawn(move || loop {
        let mut buffer = [0; 9];
        socket.read_exact(&mut buffer).unwrap();
        let player_id = u64::from_be_bytes(buffer[0..8].try_into().unwrap());
        let position = buffer[8];
        connection_sender.send((player_id, position)).unwrap();
    });
    connection_receiver
}
