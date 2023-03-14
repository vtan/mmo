use mmo_common::player_command::PlayerCommand;
use mmo_common::player_event::PlayerEvent;

pub enum AppEvent {
    KeyDown { code: String },
    WebsocketConnected { sender: Box<dyn Fn(PlayerCommand)> },
    WebsocketDisconnected,
    WebsocketMessage { message: PlayerEvent },
}
