use bincode::{Decode, Encode};
use nalgebra::Vector2;

#[derive(Debug, Clone, Copy, Encode, Decode)]
pub enum PlayerCommand {
    Move {
        #[bincode(with_serde)]
        position: Vector2<f32>,
    },
}

#[derive(Debug, Clone, Copy, Encode, Decode)]
pub enum PlayerEvent {
    PlayerMoved {
        player_id: u64,
        #[bincode(with_serde)]
        position: Vector2<f32>,
    },
}
