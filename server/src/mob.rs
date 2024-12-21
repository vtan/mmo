use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct MobTemplate {
    pub id: String,
    pub animation_id: String,
    pub velocity: f32,
    pub movement_range: f32,
    pub attack_range: f32,
}
