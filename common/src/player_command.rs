use bincode::{Decode, Encode};
use nalgebra::Vector2;

#[derive(Debug, Clone, Encode, Decode)]
pub enum PlayerCommand {
    GlobalCommand { command: GlobalCommand },
    RoomCommand { room_id: u64, command: RoomCommand },
}

#[derive(Debug, Clone, Encode, Decode)]
pub enum GlobalCommand {
    Pong { sequence_number: u32, ping_sent_at: u64 },
}

#[derive(Debug, Clone, Encode, Decode)]
pub enum RoomCommand {
    Move {
        #[bincode(with_serde)]
        position: Vector2<f32>,
    },
}
