use mmo_common::{
    object::{Direction4, ObjectId},
    player_event::PlayerEvent,
};
use nalgebra::Vector2;

use crate::{
    room_state::{Mob, Player, RoomState},
    room_writer::{RoomWriter, RoomWriterTarget},
    tick::TickEvent,
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

    state.mobs.retain(|mob| mob.health > 0);
}

pub fn mob_attack(tick: TickEvent, player: &mut Player, mob: &Mob, writer: &mut RoomWriter) {
    let attack_direction =
        Direction4::from_vector(player.local_movement.position - mob.movement.position);
    if mob.in_attack_range(player.local_movement.position)
        && attack_direction == mob.movement.look_direction
    {
        let damage = mob.template.damage;
        player.health = (player.health - damage).max(0);
        player.last_damaged_at = tick.tick;

        writer.tell(
            RoomWriterTarget::All,
            PlayerEvent::ObjectHealthChanged {
                object_id: player.id,
                health: player.health,
                change: -damage,
            },
        );
    }
}

pub fn heal_players(tick: TickEvent, state: &mut RoomState, writer: &mut RoomWriter) {
    if tick.tick.is_nth(state.server_context.player.heal_rate) {
        for player in state.players.values_mut() {
            if player.health < player.max_health
                && tick.tick - player.last_damaged_at > state.server_context.player.heal_after
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
