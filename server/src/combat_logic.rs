use mmo_common::{
    object::{Direction, ObjectId},
    player_event::PlayerEvent,
};
use nalgebra::Vector2;

use crate::{
    room_state::{Mob, Player, RoomState, RoomWriter},
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
            state.server_context.player_attack_range,
            mob.movement.position,
        ) {
            let damage = state.server_context.player_damage;
            mob.health = (mob.health - damage).max(0);

            writer.broadcast(
                state.players.keys().copied(),
                PlayerEvent::ObjectDamaged { object_id: mob.id, health: mob.health, damage },
            );

            if mob.health == 0 {
                writer.broadcast(
                    state.players.keys().copied(),
                    PlayerEvent::ObjectDisappeared { object_id: mob.id },
                );
            }
        }
    }

    state.mobs.retain(|mob| mob.health > 0);
}

pub fn mob_attack(
    player: &mut Player,
    mob: &Mob,
    player_ids: &[ObjectId],
    writer: &mut RoomWriter,
) {
    let damage = mob.template.damage;
    player.health = (player.health - damage).max(0);

    writer.broadcast(
        player_ids.iter().copied(),
        PlayerEvent::ObjectDamaged { object_id: player.id, health: player.health, damage },
    );
}

fn hit_reaches(from: Vector2<f32>, direction: Direction, range: f32, target: Vector2<f32>) -> bool {
    let range_permits = util::in_distance(from, target, range);
    let angle_permits = match direction {
        Direction::Up => from.y > target.y,
        Direction::Down => from.y < target.y,
        Direction::Left => from.x > target.x,
        Direction::Right => from.x < target.x,
    };
    range_permits && angle_permits
}
