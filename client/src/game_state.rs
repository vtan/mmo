use mmo_common::{
    animation::AnimationAction,
    client_config::ClientConfig,
    object::{Direction, ObjectId, ObjectType},
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
    pub objects: Vec<Object>,
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

#[derive(Debug, Clone)]
pub struct Object {
    pub id: ObjectId,
    pub typ: ObjectType,
    pub remote_position: Vector2<f32>,
    pub remote_position_received_at: f32,
    pub local_position: Vector2<f32>,
    pub direction: Option<Direction>,
    pub look_direction: Direction,
    pub animation_id: usize,
    pub animation: Option<ObjectAnimation>,
    pub velocity: f32,
    pub health: i32,
    pub max_health: i32,
}

#[derive(Debug, Clone)]
pub struct ObjectAnimation {
    pub action: AnimationAction,
    pub started_at: f32,
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
            objects: vec![],
        })
    }
}
