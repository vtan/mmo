use bincode::{Decode, Encode};

#[derive(Debug, Clone, Copy, Encode, Decode)]
pub struct MoveCommand {
    pub position: u8,
}

#[derive(Debug, Clone, Copy, Encode, Decode)]
pub struct PlayerMovedEvent {
    pub player_id: u64,
    pub position: u8,
}
