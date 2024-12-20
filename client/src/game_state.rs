use std::collections::HashMap;

use mmo_common::{
    animation::AnimationAction,
    client_config::ClientConfig,
    object::{Direction, ObjectId},
    player_command::PlayerCommand,
    player_event::{PlayerEvent, PlayerEventEnvelope},
    room::{ForegroundTile, RoomId, TileIndex},
};
use nalgebra::Vector2;

pub struct GameState {
    pub time: Timestamps,
    pub ws_commands: Vec<PlayerCommand>,
    pub last_ping: Option<LastPing>,
    pub ping_rtt: f32,
    pub self_id: ObjectId,
    pub client_config: ClientConfig,
    pub room: Room,
    pub self_movement: SelfMovement,
    pub remote_movements: HashMap<ObjectId, RemoteMovement>,
    pub local_movements: Vec<LocalMovement>,
}

#[derive(Debug, Clone, Copy)]
pub struct Timestamps {
    pub now: f32,
    pub frame_delta: f32,
}

#[derive(Debug, Clone)]
pub struct Room {
    pub room_id: RoomId,
    pub size: Vector2<u32>,
    pub bg_dense_layers: Vec<Vec<TileIndex>>,
    pub bg_sparse_layer: Vec<(Vector2<u32>, TileIndex)>,
    pub fg_sparse_layer: Vec<ForegroundTile>,
    pub collisions: Vec<bool>,
}

// TODO: consolidate with RemoteMovement?
#[derive(Debug, Clone)]
pub struct SelfMovement {
    pub position: Vector2<f32>,
    pub direction: Option<Direction>,
    pub look_direction: Direction,
    pub action: Option<MovementAction>,
    pub changed_at: f32,
}

#[derive(Debug, Clone)]
pub struct RemoteMovement {
    pub position: Vector2<f32>,
    pub direction: Option<Direction>,
    pub look_direction: Direction,
    pub action: Option<MovementAction>,
    pub started_at: f32,
    pub velocity: f32,
}

#[derive(Debug, Clone, Copy)]
pub struct MovementAction {
    pub action: AnimationAction,
    pub started_at: f32,
}

#[derive(Debug, Clone, Copy)]
pub struct LocalMovement {
    pub object_id: ObjectId,
    pub position: Vector2<f32>,
    pub direction: Option<Direction>,
    pub look_direction: Direction,
    pub animation_action: Option<AnimationAction>,
    pub animation_time: f32,
}

#[derive(Debug, Clone, Copy)]
pub struct LastPing {
    pub sequence_number: u32,
    pub sent_at: f32,
}

pub struct PartialGameState {
    pub time: Timestamps,
    pub self_id: Option<ObjectId>,
    pub client_config: Option<ClientConfig>,
    pub room: Option<Room>,
    pub remaining_events: Vec<PlayerEventEnvelope<PlayerEvent>>,
}

impl PartialGameState {
    pub fn new() -> Self {
        Self {
            time: Timestamps { now: 0.0, frame_delta: 0.0 },
            self_id: None,
            client_config: None,
            room: None,
            remaining_events: vec![],
        }
    }

    pub fn to_full(&self) -> Option<GameState> {
        let self_id = self.self_id?;
        let client_config = self.client_config.clone()?;
        let room = self.room.clone()?;
        Some(GameState {
            time: self.time,
            ws_commands: Vec::new(),
            last_ping: None,
            ping_rtt: 0.0,
            self_id,
            client_config,
            room,
            self_movement: SelfMovement {
                position: Vector2::new(0.0, 0.0),
                direction: None,
                look_direction: Direction::Down,
                action: None,
                changed_at: 0.0,
            },
            remote_movements: HashMap::new(),
            local_movements: vec![],
        })
    }
}
