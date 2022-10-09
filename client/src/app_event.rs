use mmo_common::{MoveCommand, PlayerMovedEvent};

pub enum AppEvent {
    KeyDown { code: String },
    WebsocketConnected { sender: Box<dyn Fn(MoveCommand)> },
    WebsocketDisconnected,
    WebsocketMessage { message: PlayerMovedEvent },
}
