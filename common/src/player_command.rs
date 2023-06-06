use nalgebra::Vector2;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum PlayerCommand {
    GlobalCommand { command: GlobalCommand },
    RoomCommand { room_id: u64, command: RoomCommand },
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum GlobalCommand {
    Pong { sequence_number: u32, ping_sent_at: u64 },
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum RoomCommand {
    Move { position: Vector2<f32> },
}
