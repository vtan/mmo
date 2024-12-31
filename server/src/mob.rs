use serde::Deserialize;

use crate::tick::TickDuration;

#[derive(Debug, Clone, Deserialize)]
pub struct MobTemplate {
    pub id: String,
    pub animation_id: String,
    pub velocity: f32,
    pub chase_velocity: f32,
    pub movement_range: f32,
    pub max_health: i32,
    pub attack_cooldown: TickDuration,
    pub attacks: Vec<MobAttack>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MobAttack {
    pub target_type: MobAttackTargetType,
    pub range: f32,
    pub damage: i32,
    pub telegraph_length: TickDuration,
    pub length: TickDuration,
    pub animation_index: u8,
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(tag = "type")]
pub enum MobAttackTargetType {
    Single,
    Area { radius: f32 },
}
