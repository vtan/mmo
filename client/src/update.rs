use mmo_common::client_config::ClientConfig;
use mmo_common::object::{Direction4, Direction8};
use mmo_common::player_command::{GlobalCommand, PlayerCommand, RoomCommand};
use mmo_common::player_event::{PlayerEvent, PlayerEventEnvelope};
use mmo_common::room::RoomSync;
use mmo_common::{rle, room};
use nalgebra::Vector2;

use crate::app_event::{AppEvent, MouseButton};
use crate::app_state::AppState;
use crate::camera::Camera;
use crate::game_state::{
    AttackMarker, GameState, HealthChangeLabel, LastPing, Object, ObjectAnimation,
    PartialGameState, Room,
};
use crate::{assets, console_error, console_warn};

pub fn update(state: &mut AppState, events: Vec<AppEvent>) {
    update_camera(state);

    for event in events {
        match event {
            AppEvent::KeyDown { code } => {
                if let Ok(game_state) = &mut state.game_state {
                    match code.as_str() {
                        "KeyW" => direction_pressed(game_state, Direction4::Up, true),
                        "KeyA" => direction_pressed(game_state, Direction4::Left, true),
                        "KeyS" => direction_pressed(game_state, Direction4::Down, true),
                        "KeyD" => direction_pressed(game_state, Direction4::Right, true),
                        "Space" => start_attack(game_state),
                        "KeyP" => game_state.show_debug = !game_state.show_debug,
                        _ => (),
                    }
                }
            }
            AppEvent::KeyUp { code } => {
                if let Ok(game_state) = &mut state.game_state {
                    match code.as_str() {
                        "KeyW" => direction_pressed(game_state, Direction4::Up, false),
                        "KeyA" => direction_pressed(game_state, Direction4::Left, false),
                        "KeyS" => direction_pressed(game_state, Direction4::Down, false),
                        "KeyD" => direction_pressed(game_state, Direction4::Right, false),
                        _ => (),
                    }
                }
            }
            AppEvent::MouseDown { x, y, button } => {
                if let Ok(game_state) = &mut state.game_state {
                    if button == MouseButton::Left {
                        mouse_left_pressed(game_state, Vector2::new(x as f32, y as f32));
                    }
                }
            }
            AppEvent::WebsocketConnected => {}
            AppEvent::WebsocketDisconnected => state.game_state = Err(PartialGameState::new()),
            AppEvent::WebsocketMessage { message, received_at } => {
                update_async(state, &message);

                match &mut state.game_state {
                    Ok(game_state) => {
                        handle_server_events(game_state, received_at, message);
                    }
                    Err(partial) => {
                        update_partial(partial, message);
                        if let Some(mut full) = partial.to_full() {
                            for remaining in &partial.remaining_events {
                                handle_server_events(&mut full, received_at, remaining.clone());
                            }
                            state.game_state = Ok(full);
                        }
                    }
                }
            }
            AppEvent::AssetsLoaded { assets } => {
                state.assets = Some(assets);
            }
        }
    }

    if let Ok(game_state) = &mut state.game_state {
        update_self_movement(game_state);
        update_remote_movement(game_state);
        add_ping_if_needed(game_state);

        game_state.objects.sort_unstable_by(|a, b| {
            a.local_position.y.partial_cmp(&b.local_position.y).expect("NaN")
        });
        game_state
            .health_change_labels
            .retain(|label| game_state.time.now - label.received_at < 1.0);
        game_state
            .attack_markers
            .retain(|marker| game_state.time.now - marker.received_at < marker.length);
    }
}

fn update_async(state: &mut AppState, message: &PlayerEventEnvelope<PlayerEvent>) {
    for event in message.events.iter() {
        if let PlayerEvent::Initial { client_config, .. } = event {
            let gl = state.gl.clone();
            let events = state.events.clone();
            let asset_paths = client_config.asset_paths.clone();
            wasm_bindgen_futures::spawn_local(async move {
                let assets = assets::load(&gl, &asset_paths).await.unwrap();
                (*events).borrow_mut().push(AppEvent::AssetsLoaded { assets });
            });
        }
    }
}

fn update_partial(partial: &mut PartialGameState, events: PlayerEventEnvelope<PlayerEvent>) {
    let mut remaining = events.clone();
    remaining.events.clear();

    for event in events.events {
        match event {
            PlayerEvent::Initial { self_id, client_config } => {
                partial.self_id = Some(self_id);
                partial.client_config = Some(*client_config);
            }
            PlayerEvent::RoomEntered { room } => {
                partial.room = Some(load_room_map(*room));
            }
            PlayerEvent::Pong { .. }
            | PlayerEvent::ObjectAppeared { .. }
            | PlayerEvent::ObjectMovementChanged { .. }
            | PlayerEvent::ObjectAnimationAction { .. }
            | PlayerEvent::ObjectHealthChanged { .. }
            | PlayerEvent::AttackTargeted { .. }
            | PlayerEvent::ObjectDisappeared { .. } => {
                remaining.events.push(event);
            }
        }
    }
    partial.remaining_events.push(remaining);
}

fn handle_server_events(
    game_state: &mut GameState,
    received_at: f32,
    events: PlayerEventEnvelope<PlayerEvent>,
) {
    for event in events.events {
        /*
        if !matches!(event, PlayerEvent::Pong { .. }) {
            console_info!("{event:?}");
        }
        */
        handle_server_event(game_state, received_at, event);
    }
}

fn handle_server_event(game_state: &mut GameState, received_at: f32, event: PlayerEvent) {
    match event {
        PlayerEvent::Pong { sequence_number } => {
            if let Some(last_ping) = &mut game_state.last_ping {
                if sequence_number == last_ping.sequence_number {
                    game_state.ping_rtt = received_at - last_ping.sent_at;
                } else {
                    console_warn!("Unexpected pong sequence number, received: {sequence_number}, expected: {}", last_ping.sequence_number);
                }
            }
        }
        PlayerEvent::Initial { .. } => {}
        PlayerEvent::RoomEntered { room } => {
            game_state.room = load_room_map(*room);
            game_state.objects.clear();
            game_state.health_change_labels.clear();
            game_state.attack_markers.clear();
        }
        PlayerEvent::ObjectAppeared {
            object_id,
            animation_id,
            velocity,
            object_type,
            health,
            max_health,
        } => {
            let object = Object {
                id: object_id,
                typ: object_type,
                remote_position: Vector2::new(0.0, 0.0),
                remote_position_received_at: f32::NEG_INFINITY,
                local_position: Vector2::new(0.0, 0.0),
                direction: None,
                look_direction: Direction4::Down,
                animation_id: animation_id as usize,
                animation: None,
                velocity,
                health,
                max_health,
            };
            if game_state.objects.iter().any(|o| o.id == object_id) {
                console_warn!(
                    "Got ObjectAppeared for {object_id:?} {object_type:?} but already had object"
                );
            } else {
                game_state.objects.push(object);
            }
        }
        PlayerEvent::ObjectMovementChanged { object_id, position, direction, look_direction } => {
            if let Some(obj) = game_state.objects.iter_mut().find(|o| o.id == object_id) {
                obj.remote_position = position;
                obj.remote_position_received_at = received_at;
                obj.direction = direction;
                obj.look_direction = look_direction;
                if obj.id == game_state.self_id {
                    obj.local_position = position;
                }
            } else {
                console_warn!("Got ObjectMovementChanged for {object_id:?} but no object");
            }
        }
        PlayerEvent::ObjectAnimationAction { object_id, animation_index } => {
            if let Some(obj) = game_state.objects.iter_mut().find(|o| o.id == object_id) {
                obj.animation =
                    Some(ObjectAnimation { animation_index, started_at: game_state.time.now });
            } else {
                console_warn!("Got ObjectAnimationAction for {object_id:?} but no object");
            }
        }
        PlayerEvent::ObjectHealthChanged { object_id, change: damage, health } => {
            if let Some(obj) = game_state.objects.iter_mut().find(|o| o.id == object_id) {
                obj.health = health;

                let obj_height = game_state
                    .client_config
                    .animations
                    .get(obj.animation_id)
                    .map(|a| a.sprite_size.y as f32)
                    .unwrap_or(0.0);
                game_state.health_change_labels.push(HealthChangeLabel {
                    health_change: damage,
                    position: obj.local_position - Vector2::new(0.0, obj_height),
                    received_at: game_state.time.now,
                });
            } else {
                console_warn!("Got ObjectDamaged for {object_id:?} but no object");
            }
        }
        PlayerEvent::AttackTargeted { position, radius, length } => {
            game_state.attack_markers.push(AttackMarker {
                position,
                radius,
                length,
                received_at: game_state.time.now,
            });
        }
        PlayerEvent::ObjectDisappeared { object_id } => {
            game_state.objects.retain(|o| o.id != object_id);
        }
    }
}

fn update_camera(state: &mut AppState) {
    if let Ok(ref mut game_state) = &mut state.game_state {
        let focus = game_state
            .objects
            .iter()
            .find(|o| o.id == game_state.self_id)
            .map(|o| o.local_position)
            .unwrap_or_default();
        let map_size = game_state.room.size;
        let viewport = state.viewport;
        game_state.camera = Camera::new(focus, map_size, viewport);
    }
}

fn direction_pressed(game_state: &mut GameState, pressed_direction: Direction4, pressed: bool) {
    if let Some(obj) = game_state.objects.iter_mut().find(|o| o.id == game_state.self_id) {
        game_state.directions_pressed[pressed_direction as usize] = pressed;
        let new_direction = direction_from_pressed(&game_state.directions_pressed);
        if new_direction != obj.direction {
            obj.direction = new_direction;
            if let Some(dir) = new_direction {
                obj.look_direction = dir.to_direction4();
            }
            obj.remote_position_received_at = game_state.time.now;

            let command = PlayerCommand::RoomCommand {
                room_id: game_state.room.room_id,
                command: RoomCommand::Move {
                    position: obj.remote_position,
                    direction: obj.direction,
                    look_direction: obj.look_direction,
                },
            };
            game_state.ws_commands.push(command);
        }
    } else {
        console_error!("No self object found");
    }
}

fn direction_from_pressed(pressed: &[bool; 4]) -> Option<Direction8> {
    let down = pressed[Direction4::Down as usize];
    let up = pressed[Direction4::Up as usize];
    let left = pressed[Direction4::Left as usize];
    let right = pressed[Direction4::Right as usize];
    match (down, up, left, right) {
        (true, false, false, false) => Some(Direction8::Down),
        (false, true, false, false) => Some(Direction8::Up),
        (false, false, true, false) => Some(Direction8::Left),
        (false, false, false, true) => Some(Direction8::Right),
        (true, false, false, true) => Some(Direction8::RightDown),
        (false, true, false, true) => Some(Direction8::RightUp),
        (true, false, true, false) => Some(Direction8::LeftDown),
        (false, true, true, false) => Some(Direction8::LeftUp),
        _ => None,
    }
}

fn mouse_left_pressed(game_state: &mut GameState, mouse: Vector2<f32>) {
    if let Some(player) = game_state.objects.iter_mut().find(|o| o.id == game_state.self_id) {
        let to_click = game_state.camera.screen_point_to_world(mouse) - player.local_position;
        let look_direction = Direction4::from_vector(to_click);
        if look_direction != player.look_direction {
            player.look_direction = look_direction;
            game_state.ws_commands.push(PlayerCommand::RoomCommand {
                room_id: game_state.room.room_id,
                command: RoomCommand::Move {
                    position: player.local_position,
                    direction: player.direction,
                    look_direction,
                },
            });
        }
    }
    start_attack(game_state);
}

fn start_attack(game_state: &mut GameState) {
    if let Some(obj) = game_state.objects.iter_mut().find(|o| o.id == game_state.self_id) {
        obj.animation = Some(ObjectAnimation {
            animation_index: game_state.client_config.player_attack_animation_index,
            started_at: game_state.time.now,
        });

        let command = PlayerCommand::RoomCommand {
            room_id: game_state.room.room_id,
            command: RoomCommand::Attack,
        };
        game_state.ws_commands.push(command);
    } else {
        console_error!("No self object found");
    }
}

fn update_self_movement(game_state: &mut GameState) {
    let room = &game_state.room;

    // TODO: for self probably remote_position = local_position, make that more intentional
    if let Some(obj) = game_state.objects.iter_mut().find(|o| o.id == game_state.self_id) {
        if let Some(direction) = obj.direction {
            let delta = game_state.time.frame_delta * obj.velocity * direction.to_unit_vector();
            let target = obj.remote_position + delta;

            if room::collision_at(room.size, &room.collisions, target) {
                obj.direction = None;
                game_state.ws_commands.push(PlayerCommand::RoomCommand {
                    room_id: game_state.room.room_id,
                    command: RoomCommand::Move {
                        position: obj.remote_position,
                        direction: None,
                        look_direction: obj.look_direction,
                    },
                });
            } else {
                obj.remote_position = target;
                obj.local_position = target;
            }
        }
        if !is_animation_running(obj, &game_state.client_config, game_state.time.now) {
            obj.animation = None;
        }
    } else {
        console_error!("No self object found");
    }
}

fn update_remote_movement(game_state: &mut GameState) {
    for obj in game_state.objects.iter_mut() {
        if obj.id != game_state.self_id {
            obj.local_position = match obj.direction {
                Some(dir) => {
                    let mov_distance =
                        obj.velocity * (game_state.time.now - obj.remote_position_received_at);
                    obj.remote_position + mov_distance * dir.to_unit_vector()
                }
                None => obj.remote_position,
            };
        }
        if !is_animation_running(obj, &game_state.client_config, game_state.time.now) {
            obj.animation = None;
        }
    }
}

fn is_animation_running(object: &Object, client_config: &ClientConfig, now: f32) -> bool {
    if let Some(animation) = &object.animation {
        let runtime = now - animation.started_at;
        let animation = &client_config.animations[object.animation_id].custom
            [animation.animation_index as usize];
        runtime < animation.total_length
    } else {
        false
    }
}

fn add_ping_if_needed(gs: &mut GameState) {
    let should_send = if let Some(last_ping) = &gs.last_ping {
        let elapsed = gs.time.now - last_ping.sent_at;
        if elapsed >= 1.0 || (elapsed >= 0.5 && !gs.ws_commands.is_empty()) {
            Some(last_ping.sequence_number + 1)
        } else {
            None
        }
    } else {
        Some(0)
    };
    if let Some(sequence_number) = should_send {
        gs.ws_commands.push(PlayerCommand::GlobalCommand {
            command: GlobalCommand::Ping { sequence_number },
        });
        gs.last_ping = Some(LastPing { sequence_number, sent_at: gs.time.now });
    }
}

fn load_room_map(room_sync: RoomSync) -> Room {
    let bg_dense_layers = room_sync.bg_dense_layers.iter().map(rle::decode).collect();
    let collisions = rle::decode(&room_sync.collisions);
    Room {
        room_id: room_sync.room_id,
        size: room_sync.size,
        bg_dense_layers,
        bg_sparse_layer: room_sync.bg_sparse_layer,
        fg_sparse_layer: room_sync.fg_sparse_layer,
        collisions,
    }
}
