use std::{collections::HashMap, sync::Arc};

use mmo_common::{animation::AnimationSet, room::RoomId};
use serde::Deserialize;

use crate::{assets::AssetPaths, room_state::RoomMap};

#[derive(Debug, Clone)]
pub struct ServerContext {
    pub asset_paths: AssetPaths,
    pub room_maps: HashMap<RoomId, Arc<RoomMap>>,
    pub player_animation: AnimationSet,
    pub player_velocity: f32,
}

impl ServerContext {
    pub fn new(
        server_config: ServerConfig,
        asset_paths: AssetPaths,
        room_maps: HashMap<RoomId, Arc<RoomMap>>,
    ) -> Self {
        Self {
            asset_paths,
            room_maps,
            player_animation: server_config.player_animation,
            player_velocity: server_config.player_velocity,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct ServerConfig {
    pub player_animation: AnimationSet,
    pub player_velocity: f32,
}

impl ServerConfig {
    pub fn load(path: &str) -> eyre::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let config = toml::from_str(&content)?;
        Ok(config)
    }
}
