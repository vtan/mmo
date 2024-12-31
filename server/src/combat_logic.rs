use std::collections::HashMap;

use mmo_common::{
    object::{Direction4, ObjectId},
    player_event::PlayerEvent,
};
use nalgebra::Vector2;

use crate::{
    mob::MobAttack,
    room_state::{Mob, MobRespawn, Player, RoomState},
    room_writer::{RoomWriter, RoomWriterTarget},
    tick::{Tick, TickEvent},
    util,
};

pub fn player_attack(player_id: ObjectId, state: &mut RoomState, writer: &mut RoomWriter) {
    let player = if let Some(player) = state.players.get(&player_id) {
        player
    } else {
        return;
    };

    for mob in state.mobs.iter_mut() {
        if hit_reaches(
            player.local_movement.position,
            player.remote_movement.look_direction,
            state.server_context.player.attack_range,
            mob.movement.position,
        ) {
            let damage = state.server_context.player.damage;
            mob.health = (mob.health - damage).max(0);

            writer.tell(
                RoomWriterTarget::All,
                PlayerEvent::ObjectHealthChanged {
                    object_id: mob.id,
                    health: mob.health,
                    change: -damage,
                },
            );

            if mob.health == 0 {
                writer.tell(
                    RoomWriterTarget::All,
                    PlayerEvent::ObjectDisappeared { object_id: mob.id },
                );
            }
        }
    }

    // TODO: maybe belongs to mob_logic
    state.mobs.retain(|mob| {
        if mob.health > 0 {
            true
        } else {
            let respawn = MobRespawn {
                spawn: mob.spawn.clone(),
                respawn_at: state.last_tick.tick + mob.template.respawn_rate,
            };
            state.mob_respawns.push(respawn);
            false
        }
    });
}

pub fn mob_attack_player(
    tick: TickEvent,
    player: &mut Player,
    mob: &Mob,
    attack: &MobAttack,
    writer: &mut RoomWriter,
) {
    let attack_direction =
        Direction4::from_vector(player.local_movement.position - mob.movement.position);
    let in_attack_range = util::in_distance(
        player.local_movement.position,
        mob.movement.position,
        attack.range,
    );
    if in_attack_range && attack_direction == mob.movement.look_direction {
        hurt_player(player, attack.damage, tick.tick, writer);
    }
}

pub fn mob_attack_area(
    tick: TickEvent,
    attack: &MobAttack,
    attack_position: Vector2<f32>,
    attack_radius: f32,
    players: &mut HashMap<ObjectId, Player>,
    writer: &mut RoomWriter,
) {
    for player in players.values_mut() {
        let in_attack_range = util::in_distance(
            player.local_movement.position,
            attack_position,
            attack_radius,
        );
        if in_attack_range {
            hurt_player(player, attack.damage, tick.tick, writer);
        }
    }
}

fn hurt_player(player: &mut Player, damage: i32, tick: Tick, writer: &mut RoomWriter) {
    player.health = (player.health - damage).max(0);
    player.last_damaged_at = tick;

    writer.tell(
        RoomWriterTarget::All,
        PlayerEvent::ObjectHealthChanged {
            object_id: player.id,
            health: player.health,
            change: -damage,
        },
    );
}

pub fn heal_players(state: &mut RoomState, writer: &mut RoomWriter) {
    let tick = state.last_tick.tick;
    if tick.is_nth(state.server_context.player.heal_rate) {
        for player in state.players.values_mut() {
            if player.health < player.max_health
                && tick - player.last_damaged_at > state.server_context.player.heal_after
            {
                let heal = (state.server_context.player.heal_amount as i32)
                    .min(player.max_health - player.health);
                player.health += heal;

                writer.tell(
                    RoomWriterTarget::All,
                    PlayerEvent::ObjectHealthChanged {
                        object_id: player.id,
                        health: player.health,
                        change: heal,
                    },
                );
            }
        }
    }
}

fn hit_reaches(
    from: Vector2<f32>,
    direction: Direction4,
    range: f32,
    target: Vector2<f32>,
) -> bool {
    let range_permits = util::in_distance(from, target, range);
    let angle_permits = match direction {
        Direction4::Up => from.y > target.y,
        Direction4::Down => from.y < target.y,
        Direction4::Left => from.x > target.x,
        Direction4::Right => from.x < target.x,
    };
    range_permits && angle_permits
}
