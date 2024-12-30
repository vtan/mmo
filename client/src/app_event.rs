use mmo_common::player_event::{PlayerEvent, PlayerEventEnvelope};

use crate::assets::Assets;

pub enum AppEvent {
    KeyDown {
        code: String,
    },
    KeyUp {
        code: String,
    },
    MouseDown {
        x: i32,
        y: i32,
        button: MouseButton,
    },
    WebsocketConnected,
    WebsocketDisconnected,
    WebsocketMessage {
        message: PlayerEventEnvelope<PlayerEvent>,
        received_at: f32,
    },
    AssetsLoaded {
        assets: Assets,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MouseButton {
    Left,
    Middle,
    Right,
    Button4,
    Button5,
    Unknown,
}

impl From<i16> for MouseButton {
    fn from(button: i16) -> Self {
        match button {
            0 => MouseButton::Left,
            1 => MouseButton::Middle,
            2 => MouseButton::Right,
            3 => MouseButton::Button4,
            4 => MouseButton::Button5,
            _ => MouseButton::Unknown,
        }
    }
}
