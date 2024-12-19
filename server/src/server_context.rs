use std::{collections::HashMap, sync::Arc};

use mmo_common::room::RoomId;

use crate::{assets::AssetPaths, room_state::RoomMap};

#[derive(Debug, Clone)]
pub struct ServerContext {
    pub asset_paths: AssetPaths,
    pub room_maps: HashMap<RoomId, Arc<RoomMap>>,
    pub player_velocity: f32,
}
