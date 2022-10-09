use bincode::{Decode, Encode};

#[derive(Debug, Clone, Copy, Encode, Decode)]
pub struct MoveCommand {
    pub x: f32,
    pub y: f32,
}

#[derive(Debug, Clone, Copy, Encode, Decode)]
pub struct PlayerMovedEvent {
    pub player_id: u64,
    pub x: f32,
    pub y: f32,
}
