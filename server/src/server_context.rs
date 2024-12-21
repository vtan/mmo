use std::{collections::HashMap, sync::Arc};

use eyre::{eyre, Result};
use mmo_common::{animation::AnimationSet, room::RoomId};
use serde::Deserialize;

use crate::{assets::AssetPaths, mob::MobTemplate, room_state::RoomMap};

#[derive(Debug, Clone)]
pub struct ServerContext {
    pub asset_paths: AssetPaths,
    pub room_maps: HashMap<RoomId, Arc<RoomMap>>,
    pub mob_templates: HashMap<String, Arc<MobTemplate>>,
    pub animations: Vec<AnimationSet>,
    pub player_animation: u32,
    pub mob_animations: HashMap<String, u32>,
    pub player_velocity: f32,
    pub player_max_health: i32,
    pub player_damage: i32,
    pub player_attack_range: f32,
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

        let mut mob_animations = HashMap::new();
        for name in server_config.mob_templates.keys() {
            let index = animation_keys
                .iter()
                .position(|animation_name| animation_name == name)
                .ok_or_else(|| eyre!("Mob animation not found"))? as u32;
            mob_animations.insert(name.clone(), index);
        }

        Ok(Self {
            asset_paths,
            room_maps,
            mob_templates: server_config.mob_templates,
            animations,
            player_animation,
            mob_animations,
            player_velocity: server_config.player_velocity,
            player_max_health: server_config.player_max_health,
            player_damage: server_config.player_damage,
            player_attack_range: server_config.player_attack_range,
        })
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct ServerConfig {
    pub animations: HashMap<String, AnimationSet>,
    pub mob_templates: HashMap<String, Arc<MobTemplate>>,
    pub player_animation: String,
    pub player_velocity: f32,
    pub player_max_health: i32,
    pub player_damage: i32,
    pub player_attack_range: f32,
}

impl ServerConfig {
    pub fn load(path: &str) -> Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let config = toml::from_str(&content)?;
        Ok(config)
    }
}
