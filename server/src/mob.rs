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
    pub attack_cooldown: TickDuration,
    pub attacks: Vec<MobAttack>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MobAttack {
    pub damage: i32,
    pub telegraph_length: TickDuration,
    pub length: TickDuration,
}
