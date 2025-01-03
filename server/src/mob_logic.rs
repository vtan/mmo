use std::sync::Arc;

use fastrand::Rng;
use mmo_common::{
    object::{Direction4, Direction8, ObjectType, ALL_DIRECTIONS_8},
    player_event::PlayerEvent,
};
use tokio::time::Instant;

use crate::{
    combat_logic,
    mob::{MobAttackTargetType, MobTemplate},
    object,
    room_state::{
        Mob, MobAttackState, MobRespawn, MobSpawn, Player, RemoteMovement, RoomMap, RoomState,
    },
    room_writer::{RoomWriter, RoomWriterTarget},
    server_context::ServerContext,
    tick::{self, Tick},
    util,
};

pub fn populate_mobs(map: &RoomMap, ctx: &ServerContext, now: Instant) -> Vec<Mob> {
    map.mob_spawns
        .iter()
        .filter_map(|mob_spawn| spawn_mob(mob_spawn, ctx, now))
        .collect()
}

pub fn respawn_mobs(state: &mut RoomState, writer: &mut RoomWriter) {
    let should_respawn = |respawn: &MobRespawn| respawn.respawn_at <= state.last_tick.tick;
    for mob_respawn in state.mob_respawns.iter() {
        if should_respawn(mob_respawn) {
            if let Some(mob) = spawn_mob(
                &mob_respawn.spawn,
                &state.server_context,
                state.last_tick.monotonic_time,
            ) {
                writer.tell_many(RoomWriterTarget::All, &mob_appeared_events(&mob));
                state.mobs.push(mob);
            }
        }
    }
    state
        .mob_respawns
        .retain(|mob_respawn| !should_respawn(mob_respawn));
}

pub fn mob_appeared_events(mob: &Mob) -> [PlayerEvent; 2] {
    [
        PlayerEvent::ObjectAppeared {
            object_id: mob.id,
            object_type: ObjectType::Mob,
            animation_id: mob.animation_id,
            health: mob.health,
            max_health: mob.template.max_health,
        },
        PlayerEvent::ObjectMovementChanged {
            object_id: mob.id,
            position: mob.movement.position,
            velocity: mob.velocity,
            direction: mob.movement.direction,
            look_direction: mob.movement.look_direction,
        },
    ]
}

fn spawn_mob(mob_spawn: &Arc<MobSpawn>, ctx: &ServerContext, now: Instant) -> Option<Mob> {
    let resolve = || -> Option<(Arc<MobTemplate>, u32)> {
        let mob_template = ctx.mob_templates.get(&mob_spawn.mob_template)?;
        let animation_id = ctx.mob_animations.get(&mob_template.animation_id)?;
        Some((mob_template.clone(), *animation_id))
    };
    if let Some((mob_template, animation_id)) = resolve() {
        let position = mob_spawn.position.cast().add_scalar(0.5);
        let velocity = mob_template.velocity;
        let health = mob_template.max_health;
        let mob = Mob {
            id: object::next_object_id(),
            animation_id,
            template: mob_template,
            spawn: mob_spawn.clone(),
            movement: RemoteMovement {
                position,
                direction: None,
                look_direction: Direction4::Down,
                received_at: now,
            },
            velocity,
            attack_state: None,
            health,
            last_attacked_at: Tick(0),
        };
        Some(mob)
    } else {
        None
    }
}

pub fn on_tick(state: &mut RoomState, writer: &mut RoomWriter) {
    let mut rng = Rng::new();
    let tick = state.last_tick;
    for mob in &mut state.mobs {
        let mut crossed_tile = false;
        let mut changed_direction = false;

        if let Some(direction) = mob.movement.direction {
            let new_position = mob.movement.position
                + direction.to_unit_vector() * mob.velocity * tick::TICK_INTERVAL.as_secs_f32();

            let collides =
                mmo_common::room::collision_at(state.map.size, &state.map.collisions, new_position);
            let in_movement_range = mob.in_movement_range(new_position);

            if !collides && in_movement_range {
                let prev_position = mob.movement.position;
                mob.movement.position = new_position;
                crossed_tile =
                    prev_position.map(|x| x as u32) != mob.movement.position.map(|x| x as u32);
            } else {
                mob.movement.direction = None;
                changed_direction = true;
            }
        }

        match mob.attack_state {
            None => {
                if mob.movement.direction.is_none() || (crossed_tile && rng.bool()) {
                    mob.movement.direction = choose_direction(mob, &state.map);
                    mob.movement.look_direction = mob
                        .movement
                        .direction
                        .unwrap_or(Direction8::Down)
                        .to_direction4();
                    changed_direction = true;
                }

                let target = state
                    .players
                    .values()
                    .find(|player| is_valid_attack_target(mob, player));
                if let Some(target) = target {
                    let target_id = target.id;
                    let attack_index = choose_attack(mob);
                    mob.attack_state = Some(MobAttackState::Targeting {
                        target_id,
                        attack_index,
                    });
                }
            }

            Some(MobAttackState::Targeting {
                target_id,
                attack_index,
            }) => {
                let target = state.players.get(&target_id);
                let target = target.filter(|target| is_valid_attack_target(mob, target));
                if let Some(target) = target {
                    let attack = mob.template.attacks[attack_index as usize].clone();
                    let in_attack_range = util::in_distance(
                        target.local_movement.position,
                        mob.movement.position,
                        attack.range,
                    );

                    if in_attack_range {
                        changed_direction |= change_direction_for_attack(mob, target);

                        if tick.tick - mob.last_attacked_at >= mob.template.attack_cooldown {
                            writer.tell(
                                RoomWriterTarget::All,
                                PlayerEvent::ObjectAnimationAction {
                                    object_id: mob.id,
                                    animation_index: attack.animation_index,
                                },
                            );
                            if let MobAttackTargetType::Area { radius } = attack.target_type {
                                writer.tell(
                                    RoomWriterTarget::All,
                                    PlayerEvent::AttackTargeted {
                                        attacker_object_id: mob.id,
                                        position: target.local_movement.position,
                                        radius,
                                        length: attack.telegraph_length.as_secs_f32(),
                                    },
                                );
                            }

                            mob.attack_state = Some(MobAttackState::Telegraphed {
                                target_id,
                                attack_index,
                                attack_started_at: tick.tick,
                                attack_position: target.local_movement.position,
                            });
                            mob.last_attacked_at = tick.tick;
                        }
                    } else {
                        changed_direction |= change_direction_to_target(mob, target, &state.map);
                    }
                } else {
                    mob.attack_state = None;
                }
            }

            Some(MobAttackState::Telegraphed {
                target_id,
                attack_index,
                attack_started_at,
                attack_position,
            }) => {
                let attack = &mob.template.attacks[attack_index as usize];
                if tick.tick - attack_started_at >= attack.telegraph_length {
                    match attack.target_type {
                        MobAttackTargetType::Single => {
                            if let Some(target) = state.players.get_mut(&target_id) {
                                combat_logic::mob_attack_player(tick, target, mob, attack, writer);
                            }
                        }
                        MobAttackTargetType::Area { radius } => {
                            combat_logic::mob_attack_area(
                                tick,
                                attack,
                                attack_position,
                                radius,
                                &mut state.players,
                                writer,
                            );
                        }
                    }
                    mob.attack_state = Some(MobAttackState::DamageDealt {
                        attack_index,
                        attack_started_at,
                    });
                }
            }

            Some(MobAttackState::DamageDealt {
                attack_index,
                attack_started_at,
            }) => {
                let attack = &mob.template.attacks[attack_index as usize];
                if tick.tick - attack_started_at >= attack.length {
                    mob.attack_state = None;
                }
            }
        }

        let changed_velocity = {
            if mob.attack_state.is_some() && mob.velocity != mob.template.chase_velocity {
                mob.velocity = mob.template.chase_velocity;
                true
            } else if mob.attack_state.is_none() && mob.velocity != mob.template.velocity {
                mob.velocity = mob.template.velocity;
                true
            } else {
                false
            }
        };

        if crossed_tile || changed_direction || changed_velocity {
            writer.tell(
                RoomWriterTarget::All,
                PlayerEvent::ObjectMovementChanged {
                    object_id: mob.id,
                    position: mob.movement.position,
                    velocity: mob.velocity,
                    direction: mob.movement.direction,
                    look_direction: mob.movement.look_direction,
                },
            );
        }
    }
}

fn is_valid_attack_target(mob: &Mob, player: &Player) -> bool {
    mob.in_movement_range(player.local_movement.position)
}

fn change_direction_to_target(mob: &mut Mob, attack_target: &Player, map: &RoomMap) -> bool {
    let direction = attack_target.local_movement.position - mob.movement.position;
    let direction = Direction8::from_vector(direction);
    let next_tile = mob.movement.position + direction.to_unit_vector();
    let can_move = !mmo_common::room::collision_at(map.size, &map.collisions, next_tile);
    if can_move {
        if mob.movement.direction != Some(direction) {
            mob.movement.direction = Some(direction);
            mob.movement.look_direction = direction.to_direction4();
            true
        } else {
            false
        }
    } else if mob.movement.direction.is_some() {
        mob.movement.direction = None;
        true
    } else {
        false
    }
}

fn change_direction_for_attack(mob: &mut Mob, attack_target: &Player) -> bool {
    let mut changed_direction = false;
    if mob.movement.direction.is_some() {
        mob.movement.direction = None;
        changed_direction = true;
    }
    let attack_direction =
        Direction4::from_vector(attack_target.local_movement.position - mob.movement.position);
    if mob.movement.look_direction != attack_direction {
        mob.movement.look_direction = attack_direction;
        changed_direction = true;
    }
    changed_direction
}

fn choose_direction(mob: &Mob, map: &RoomMap) -> Option<Direction8> {
    let mut rng = fastrand::Rng::new();
    let current_tile = mob.movement.position;
    let candidates = ALL_DIRECTIONS_8
        .iter()
        .copied()
        .filter(|dir| {
            let next_tile = current_tile + dir.to_neighbor_vector();
            let in_movement_range = mob.in_movement_range(next_tile);
            let collides = mmo_common::room::collision_at(map.size, &map.collisions, next_tile);
            in_movement_range && !collides
        })
        .collect::<Vec<_>>();
    rng.choice(&candidates).copied()
}

fn choose_attack(mob: &Mob) -> u8 {
    let mut rng = fastrand::Rng::new();
    rng.u8(0..mob.template.attacks.len() as u8)
}
