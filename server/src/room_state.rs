use crate::{
    mob::MobTemplate, player::PlayerConnection, server_context::ServerContext, tick::Tick, util,
};
use std::{collections::HashMap, sync::Arc};

use mmo_common::{
    object::{Direction4, Direction8, ObjectId},
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
    pub last_damaged_at: Tick,
}

#[derive(Debug, Clone)]
pub struct Mob {
    pub id: ObjectId,
    pub template: Arc<MobTemplate>,
    pub spawn: Arc<MobSpawn>,
    pub animation_id: u32,
    pub movement: RemoteMovement,
    pub velocity: f32,
    pub attack_state: Option<MobAttackState>,
    pub health: i32,
    pub last_attacked_at: Tick,
}

#[derive(Debug, Clone, Copy)]
pub enum MobAttackState {
    Targeting {
        target_id: ObjectId,
        attack_index: u8,
    },
    Telegraphed {
        target_id: ObjectId,
        attack_index: u8,
        attack_started_at: Tick,
        attack_position: Vector2<f32>,
    },
    DamageDealt {
        attack_index: u8,
        attack_started_at: Tick,
    },
}

impl Mob {
    pub fn in_movement_range(&self, v: Vector2<f32>) -> bool {
        util::in_distance(
            v,
            self.spawn.position.cast().add_scalar(0.5),
            self.template.movement_range,
        )
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
    pub direction: Option<Direction8>,
    pub look_direction: Direction4,
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
pub enum UpstreamMessage {
    PlayerLeftRoom {
        sender_room_id: RoomId,
        player: Player,
        target_room_id: RoomId,
        target_position: Vector2<f32>,
    },
}
