use std::{collections::HashMap, sync::Arc};

use mmo_common::{
    animation::{Animation, AnimationFrame, AnimationSet, DirectionalAnimation, SpriteIndex},
    room::RoomId,
};
use nalgebra::Vector2;

use crate::{assets::AssetPaths, room_state::RoomMap};

#[derive(Debug, Clone)]
pub struct ServerContext {
    pub asset_paths: AssetPaths,
    pub room_maps: HashMap<RoomId, Arc<RoomMap>>,
    pub player_animation: AnimationSet,
    pub player_velocity: f32,
}

pub fn make_player_animation() -> AnimationSet {
    AnimationSet {
        sprite_size: Vector2::new(1, 2),
        anchor: Vector2::new(0.5, 0.0),
        idle: DirectionalAnimation([
            Animation {
                total_length: 0.0,
                frames: vec![AnimationFrame { start: 0.0, sprite_index: SpriteIndex(6) }],
            },
            Animation {
                total_length: 0.0,
                frames: vec![AnimationFrame { start: 0.0, sprite_index: SpriteIndex(0) }],
            },
            Animation {
                total_length: 0.0,
                frames: vec![AnimationFrame { start: 0.0, sprite_index: SpriteIndex(9) }],
            },
            Animation {
                total_length: 0.0,
                frames: vec![AnimationFrame { start: 0.0, sprite_index: SpriteIndex(3) }],
            },
        ]),
        walk: DirectionalAnimation([
            Animation {
                total_length: 0.4,
                frames: vec![
                    AnimationFrame { start: 0.0, sprite_index: SpriteIndex(6) },
                    AnimationFrame { start: 0.1, sprite_index: SpriteIndex(7) },
                    AnimationFrame { start: 0.2, sprite_index: SpriteIndex(6) },
                    AnimationFrame { start: 0.3, sprite_index: SpriteIndex(8) },
                ],
            },
            Animation {
                total_length: 0.4,
                frames: vec![
                    AnimationFrame { start: 0.0, sprite_index: SpriteIndex(0) },
                    AnimationFrame { start: 0.1, sprite_index: SpriteIndex(1) },
                    AnimationFrame { start: 0.2, sprite_index: SpriteIndex(0) },
                    AnimationFrame { start: 0.3, sprite_index: SpriteIndex(2) },
                ],
            },
            Animation {
                total_length: 0.4,
                frames: vec![
                    AnimationFrame { start: 0.0, sprite_index: SpriteIndex(9) },
                    AnimationFrame { start: 0.1, sprite_index: SpriteIndex(10) },
                    AnimationFrame { start: 0.2, sprite_index: SpriteIndex(9) },
                    AnimationFrame { start: 0.3, sprite_index: SpriteIndex(11) },
                ],
            },
            Animation {
                total_length: 0.4,
                frames: vec![
                    AnimationFrame { start: 0.0, sprite_index: SpriteIndex(3) },
                    AnimationFrame { start: 0.1, sprite_index: SpriteIndex(4) },
                    AnimationFrame { start: 0.2, sprite_index: SpriteIndex(3) },
                    AnimationFrame { start: 0.3, sprite_index: SpriteIndex(5) },
                ],
            },
        ]),
    }
}
