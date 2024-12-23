use std::{collections::HashMap, sync::Arc};

use crate::{mob::MobTemplate, player::PlayerConnection, server_context::ServerContext, util};

use mmo_common::{
    object::{Direction, ObjectId},
    player_event::PlayerEvent,
    room::{ForegroundTile, RoomId, RoomSync, TileIndex},
};
use nalgebra::Vector2;
use tokio::time::Instant;

#[derive(Debug, Clone)]
pub struct RoomState {
    pub server_context: Arc<ServerContext>,
    pub map: Arc<RoomMap>,
    pub room: RoomSync,
    pub players: HashMap<ObjectId, Player>,
    pub mobs: Vec<Mob>,
}

#[derive(Debug, Clone)]
pub struct RoomMap {
    pub size: Vector2<u32>,
    pub bg_dense_layers: Vec<Vec<TileIndex>>,
    pub bg_sparse_layer: Vec<(Vector2<u32>, TileIndex)>,
    pub fg_sparse_layer: Vec<ForegroundTile>,
    pub collisions: Vec<bool>,
    pub portals: Vec<Portal>,
    pub mob_spawns: Vec<Arc<MobSpawn>>,
}

#[derive(Debug, Clone)]
pub struct Player {
    pub id: ObjectId,
    pub connection: PlayerConnection,
    pub local_movement: LocalMovement,
    pub remote_movement: RemoteMovement,
    pub health: i32,
    pub max_health: i32,
}

#[derive(Debug, Clone)]
pub struct Mob {
    pub id: ObjectId,
    pub template: Arc<MobTemplate>,
    pub spawn: Arc<MobSpawn>,
    pub animation_id: u32,
    pub movement: RemoteMovement,
    pub attack_target: Option<ObjectId>,
    pub health: i32,
    pub last_attacked_at: u32,
}

impl Mob {
    pub fn in_movement_range(&self, v: Vector2<f32>) -> bool {
        util::in_distance(
            v,
            self.spawn.position.cast().add_scalar(0.5),
            self.template.movement_range,
        )
    }

    pub fn in_attack_range(&self, v: Vector2<f32>) -> bool {
        util::in_distance(v, self.movement.position, self.template.attack_range)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct LocalMovement {
    pub position: Vector2<f32>,
    pub updated_at: Instant,
}

#[derive(Debug, Clone, Copy)]
pub struct RemoteMovement {
    pub position: Vector2<f32>,
    pub direction: Option<Direction>,
    pub look_direction: Direction,
    pub received_at: Instant,
}

#[derive(Debug, Clone)]
pub struct Portal {
    pub position: Vector2<u32>,
    pub target_room_id: RoomId,
    pub target_position: Vector2<f32>,
}

#[derive(Debug, Clone)]
pub struct MobSpawn {
    pub position: Vector2<u32>,
    pub mob_template: String,
}

#[derive(Debug, Clone)]
pub struct RoomWriter {
    pub events: HashMap<ObjectId, Vec<Arc<PlayerEvent>>>,
    pub upstream_messages: Vec<UpstreamMessage>,
}

impl RoomWriter {
    pub fn new() -> Self {
        Self { events: HashMap::new(), upstream_messages: vec![] }
    }

    pub fn tell(&mut self, player_id: ObjectId, event: PlayerEvent) {
        self.tell_many(player_id, &[event]);
    }

    pub fn tell_many(&mut self, player_id: ObjectId, events: &[PlayerEvent]) {
        for event in events {
            self.events.entry(player_id).or_default().push(Arc::new(event.clone()));
        }
    }

    pub fn broadcast(&mut self, player_ids: impl Iterator<Item = ObjectId>, event: PlayerEvent) {
        let event = Arc::new(event);
        for player_id in player_ids {
            self.events.entry(player_id).or_default().push(event.clone());
        }
    }

    pub fn broadcast_many(
        &mut self,
        player_ids: impl Iterator<Item = ObjectId>,
        events: &[PlayerEvent],
    ) {
        let events = events.iter().map(|event| Arc::new(event.clone())).collect::<Vec<_>>();
        for player_id in player_ids {
            self.events.entry(player_id).or_default().extend(events.iter().cloned());
        }
    }
}

#[derive(Debug, Clone)]
pub enum UpstreamMessage {
    PlayerLeftRoom {
        sender_room_id: RoomId,
        player: Player,
        target_room_id: RoomId,
        target_position: Vector2<f32>,
    },
}
