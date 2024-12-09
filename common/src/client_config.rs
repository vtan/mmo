use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ClientConfig {
    pub asset_paths: AssetPaths,
    pub player_velocity: f32,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AssetPaths {
    pub tileset: String,
    pub charset: String,
    pub font: String,
    pub font_meta: String,
}
