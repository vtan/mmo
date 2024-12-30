use serde::{Deserialize, Serialize};

use crate::animation::AnimationSet;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ClientConfig {
    pub server_git_sha: String,
    pub asset_paths: AssetPaths,
    pub animations: Vec<AnimationSet>,
    pub player_attack_animation_index: u8,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AssetPaths {
    pub tileset: String,
    pub charset: String,
    pub font: String,
    pub font_meta: String,
}
