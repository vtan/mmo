use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
pub struct ClientConfig {
    pub player_velocity: f32,
}
