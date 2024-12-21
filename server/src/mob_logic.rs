use std::sync::Arc;

use mmo_common::{
    object::{Direction, ALL_DIRECTIONS},
    player_event::PlayerEvent,
};
use tokio::time::Instant;

use crate::{
    mob::MobTemplate,
    object,
    room_state::{Mob, RemoteMovement, RoomMap, RoomState, RoomWriter},
    server_context::ServerContext,
    tick::{self, Tick},
};

pub fn populate_mobs(map: &RoomMap, ctx: &ServerContext, now: Instant) -> Vec<Mob> {
    map.mob_spawns
        .iter()
        .filter_map(|mob_spawn| {
            let resolve = || -> Option<(Arc<MobTemplate>, u32)> {
                let mob_template = ctx.mob_templates.get(&mob_spawn.mob_template)?;
                let animation_id = ctx.mob_animations.get(&mob_template.animation_id)?;
                Some((mob_template.clone(), *animation_id))
            };
            if let Some((mob_template, animation_id)) = resolve() {
                let mob = Mob {
                    id: object::next_object_id(),
                    animation_id,
                    template: mob_template,
                    movement: RemoteMovement {
                        position: mob_spawn.position.cast(),
                        direction: None,
                        look_direction: Direction::Down,
                        received_at: now,
                    },
                };
                Some(mob)
            } else {
                None
            }
        })
        .collect()
}

pub fn on_tick(tick: Tick, state: &mut RoomState, writer: &mut RoomWriter) {
    let player_ids = state.players.keys().copied().collect::<Vec<_>>();

    for mob in &mut state.mobs {
        let mut crossed_tile = false;
        if let Some(direction) = mob.movement.direction {
            let prev_position = mob.movement.position;
            mob.movement.position +=
                direction.to_vector() * mob.template.velocity * tick::TICK_INTERVAL.as_secs_f32();
            crossed_tile =
                prev_position.map(|x| x as u32) != mob.movement.position.map(|x| x as u32);
        }

        let mut changed_direction = false;
        if crossed_tile || mob.movement.direction.is_none() {
            mob.movement.direction = choose_direction(mob, &state.map);
            mob.movement.look_direction = mob.movement.direction.unwrap_or(Direction::Down);
            changed_direction = true;
        }

        if crossed_tile || changed_direction {
            writer.broadcast(
                player_ids.iter().copied(),
                PlayerEvent::ObjectMovementChanged {
                    object_id: mob.id,
                    position: mob.movement.position,
                    direction: mob.movement.direction,
                    look_direction: mob.movement.look_direction,
                },
            );
        }
    }
}

fn choose_direction(mob: &Mob, map: &RoomMap) -> Option<Direction> {
    let mut rng = fastrand::Rng::new();
    let current_tile = mob.movement.position;
    let candidates = ALL_DIRECTIONS
        .iter()
        .copied()
        .filter(|dir| {
            let next_tile = current_tile + dir.to_vector();
            !mmo_common::room::collision_at(map.size, &map.collisions, next_tile)
        })
        .collect::<Vec<_>>();
    rng.choice(&candidates).copied()
}
