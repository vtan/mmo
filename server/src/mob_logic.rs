use std::{collections::HashMap, sync::Arc};

use mmo_common::{
    object::{Direction, ObjectId, ALL_DIRECTIONS},
    player_event::PlayerEvent,
};
use tokio::time::Instant;

use crate::{
    combat_logic,
    mob::MobTemplate,
    object,
    room_state::{Mob, Player, RemoteMovement, RoomMap, RoomState, RoomWriter},
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
                let position = mob_spawn.position.cast().add_scalar(0.5);
                let health = mob_template.max_health;
                let mob = Mob {
                    id: object::next_object_id(),
                    animation_id,
                    template: mob_template,
                    spawn: mob_spawn.clone(),
                    movement: RemoteMovement {
                        position,
                        direction: None,
                        look_direction: Direction::Down,
                        received_at: now,
                    },
                    attack_target: None,
                    health,
                    last_attacked_at: 0,
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
        // update position
        let mut crossed_tile = false;
        if let Some(direction) = mob.movement.direction {
            let prev_position = mob.movement.position;
            mob.movement.position +=
                direction.to_vector() * mob.template.velocity * tick::TICK_INTERVAL.as_secs_f32();
            crossed_tile =
                prev_position.map(|x| x as u32) != mob.movement.position.map(|x| x as u32);
        }

        // change direction if needed
        let mut changed_direction = false;
        let attack_target = choose_attack_target(&mut state.players, mob);

        #[allow(clippy::collapsible_else_if)]
        if let Some(attack_target) = attack_target {
            if mob.in_attack_range(attack_target.local_movement.position) {
                if mob.movement.direction.is_some() {
                    mob.movement.direction = None;
                    changed_direction = true;
                }

                if tick.tick - mob.last_attacked_at >= mob.template.attack_cooldown_ticks {
                    combat_logic::mob_attack(attack_target, mob, &player_ids, writer);
                    mob.last_attacked_at = tick.tick;
                }
            } else {
                let direction = attack_target.local_movement.position - mob.movement.position;
                let direction = Direction::from_vector(direction);
                let next_tile = mob.movement.position + direction.to_vector();
                let can_move = !mmo_common::room::collision_at(
                    state.map.size,
                    &state.map.collisions,
                    next_tile,
                );
                if can_move {
                    if mob.movement.direction != Some(direction) {
                        mob.movement.direction = Some(direction);
                        mob.movement.look_direction = direction;
                        changed_direction = true;
                    }
                } else {
                    if mob.movement.direction.is_some() {
                        mob.movement.direction = None;
                        changed_direction = true;
                    }
                }
            }
        } else {
            if crossed_tile || mob.movement.direction.is_none() {
                mob.movement.direction = choose_direction(mob, &state.map);
                mob.movement.look_direction = mob.movement.direction.unwrap_or(Direction::Down);
                changed_direction = true;
            }
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

fn choose_attack_target<'a>(
    players: &'a mut HashMap<ObjectId, Player>,
    mob: &mut Mob,
) -> Option<&'a mut Player> {
    // clear invalid attack target
    if let Some(attack_target_id) = mob.attack_target {
        if let Some(attack_target) = players.get_mut(&attack_target_id) {
            if mob.in_movement_range(attack_target.local_movement.position) {
                return Some(attack_target);
            } else {
                mob.attack_target = None;
            }
        } else {
            mob.attack_target = None;
        }
    }
    // find someone to attack
    else {
        for player in players.values_mut() {
            if mob.in_movement_range(player.local_movement.position) {
                mob.attack_target = Some(player.id);
                return Some(player);
            }
        }
    }
    None
}

fn choose_direction(mob: &Mob, map: &RoomMap) -> Option<Direction> {
    let mut rng = fastrand::Rng::new();
    let current_tile = mob.movement.position;
    let candidates = ALL_DIRECTIONS
        .iter()
        .copied()
        .filter(|dir| {
            let next_tile = current_tile + dir.to_vector();
            let in_movement_range = mob.in_movement_range(next_tile);
            let collides = mmo_common::room::collision_at(map.size, &map.collisions, next_tile);
            in_movement_range && !collides
        })
        .collect::<Vec<_>>();
    rng.choice(&candidates).copied()
}
