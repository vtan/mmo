use bincode::{Decode, Encode};
use nalgebra::Vector2;

#[derive(Debug, Clone, Copy, Encode, Decode)]
pub enum PlayerCommand {
    Pong {
        sequence_number: u32,
        ping_sent_at: u64,
    },
    Move {
        #[bincode(with_serde)]
        position: Vector2<f32>,
    },
}
