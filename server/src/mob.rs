use serde::Deserialize;

use crate::tick::TickDuration;

#[derive(Debug, Clone, Deserialize)]
pub struct MobTemplate {
    pub id: String,
    pub animation_id: String,
    pub velocity: f32,
    pub movement_range: f32,
    pub attack_range: f32,
    pub max_health: i32,
    pub damage: i32,
    pub attack_telegraph_length: TickDuration,
    pub attack_length: TickDuration,
    pub attack_cooldown: TickDuration,
}
