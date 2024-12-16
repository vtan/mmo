use mmo_common::player_event::{PlayerEvent, PlayerEventEnvelope};

use crate::assets::Assets;

pub enum AppEvent {
    KeyDown {
        code: String,
    },
    KeyUp {
        code: String,
    },
    WebsocketConnected,
    WebsocketDisconnected,
    WebsocketMessage {
        message: PlayerEventEnvelope<Box<PlayerEvent>>,
        received_at: f32,
    },
    AssetsLoaded {
        assets: Assets,
    },
}
