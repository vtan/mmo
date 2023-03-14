use bincode::{Decode, Encode};
use nalgebra::Vector2;

#[derive(Debug, Clone, Copy, Encode, Decode)]
pub enum PlayerCommand {
    Move {
        #[bincode(with_serde)]
        position: Vector2<f32>,
    },
}
