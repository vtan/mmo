use std::sync::Arc;

use mmo_common::{
    client_config::{AssetPaths, ClientConfig},
    player_event::PlayerEvent,
};
use tokio::sync::mpsc;

pub type PlayerConnection = mpsc::Sender<Vec<Arc<PlayerEvent>>>;

pub fn client_config() -> ClientConfig {
    ClientConfig {
        player_velocity: 3.0,
        asset_paths: AssetPaths {
            tileset: "/assets/tileset.png".to_string(),
            charset: "/assets/charset.png".to_string(),
            font: "/assets/notosans.png".to_string(),
            font_meta: "/assets/notosans.json".to_string(),
        },
    }
}
