use mmo_common::{PlayerCommand, PlayerEvent};

pub enum AppEvent {
    KeyDown { code: String },
    WebsocketConnected { sender: Box<dyn Fn(PlayerCommand)> },
    WebsocketDisconnected,
    WebsocketMessage { message: PlayerEvent },
}
