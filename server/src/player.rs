use std::sync::Arc;

use mmo_common::{client_config::ClientConfig, player_event::PlayerEvent};
use tokio::sync::mpsc;

use crate::server_context::ServerContext;

pub type PlayerConnection = mpsc::Sender<Vec<Arc<PlayerEvent>>>;

// TODO: use Arc?
pub fn client_config(server_context: &ServerContext) -> ClientConfig {
    ClientConfig {
        player_velocity: server_context.player_velocity,
        asset_paths: server_context.asset_paths.paths.clone(),
        animations: server_context.animations.clone(),
        player_animation: server_context.player_animation as usize,
    }
}
