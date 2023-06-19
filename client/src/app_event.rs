use mmo_common::player_command::PlayerCommand;
use mmo_common::player_event::{PlayerEvent, PlayerEventEnvelope};

pub enum AppEvent {
    KeyDown { code: String },
    KeyUp { code: String },
    WebsocketConnected { sender: Box<dyn Fn(PlayerCommand)> },
    WebsocketDisconnected,
    WebsocketMessage { message: PlayerEventEnvelope<Box<PlayerEvent>> },
}
