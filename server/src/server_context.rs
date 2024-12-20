use std::{collections::HashMap, sync::Arc};

use eyre::{eyre, Result};
use mmo_common::{animation::AnimationSet, room::RoomId};
use serde::Deserialize;

use crate::{assets::AssetPaths, room_state::RoomMap};

#[derive(Debug, Clone)]
pub struct ServerContext {
    pub asset_paths: AssetPaths,
    pub room_maps: HashMap<RoomId, Arc<RoomMap>>,
    pub animations: Vec<AnimationSet>,
    pub player_animation: u32,
    pub player_velocity: f32,
}

impl ServerContext {
    pub fn new(
        server_config: ServerConfig,
        asset_paths: AssetPaths,
        room_maps: HashMap<RoomId, Arc<RoomMap>>,
    ) -> Result<Self> {
        let mut animations: Vec<(String, AnimationSet)> =
            server_config.animations.into_iter().collect();
        animations.sort_by_key(|(name, _)| name.clone());

        let animation_keys: Vec<String> = animations.iter().map(|(name, _)| name.clone()).collect();
        let animations: Vec<AnimationSet> =
            animations.into_iter().map(|(_, animation)| animation).collect();

        let player_animation = animation_keys
            .iter()
            .position(|name| name == &server_config.player_animation)
            .ok_or_else(|| eyre!("Player animation not found"))?
            as u32;

        Ok(Self {
            asset_paths,
            room_maps,
            animations,
            player_animation,
            player_velocity: server_config.player_velocity,
        })
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct ServerConfig {
    pub animations: HashMap<String, AnimationSet>,
    pub player_animation: String,
    pub player_velocity: f32,
}

impl ServerConfig {
    pub fn load(path: &str) -> Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let config = toml::from_str(&content)?;
        Ok(config)
    }
}
