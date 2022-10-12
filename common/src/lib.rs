use bincode::{Decode, Encode};

#[derive(Debug, Clone, Copy, Encode, Decode)]
pub enum PlayerCommand {
    Move { x: f32, y: f32 },
}

#[derive(Debug, Clone, Copy, Encode, Decode)]
pub enum PlayerEvent {
    PlayerMoved { player_id: u64, x: f32, y: f32 },
}
