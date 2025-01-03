use mmo_common::{
    object::ObjectType,
    room::{ForegroundTile, TileIndex},
};
use nalgebra::{Vector2, Vector4};
use web_sys::WebGl2RenderingContext as GL;

use crate::{
    app_state::AppState,
    assets::Assets,
    camera::{self},
    font_atlas::Align,
    game_state::GameState,
    metrics::Metrics,
    vertex_buffer::{LineVertexBuffer, TileVertexBuffer, VertexBuffer},
};

pub fn init(state: &mut AppState) {
    let gl = &state.gl;

    gl.use_program(Some(&state.program));
    gl.uniform1iv_with_i32_array(Some(&state.uniform_locations.sampler), &[0, 1]);

    gl.use_program(Some(&state.text_program));
    gl.uniform1iv_with_i32_array(Some(&state.uniform_locations.text_sampler), &[0, 1]);
}

pub fn render(state: &mut AppState) {
    let gl = &state.gl;
    gl.clear_color(0.0, 0.0, 0.0, 1.0);
    gl.clear(GL::COLOR_BUFFER_BIT);

    let assets = match &state.assets {
        Some(assets) => assets,
        None => return,
    };

    let game_state = match &state.game_state {
        Ok(game_state) => game_state,
        Err(_) => return,
    };

    gl.use_program(Some(&state.program));

    {
        let mut vertex_buffer = TileVertexBuffer::new(Vector2::new(
            assets.tileset.width / camera::PIXELS_PER_TILE,
            assets.tileset.height / camera::PIXELS_PER_TILE,
        ));

        for layer in &game_state.room.bg_dense_layers {
            render_dense_tile_layer(layer, game_state.room.size, &mut vertex_buffer);
        }
        render_sparse_tile_layer(&game_state.room.bg_sparse_layer, &mut vertex_buffer);
        render_foreground(game_state, &mut vertex_buffer);

        let vertex_buffer = vertex_buffer.vertex_buffer;

        gl.uniform_matrix3fv_with_f32_array(
            Some(&state.uniform_locations.view_projection),
            false,
            game_state.camera.world_to_ndc.as_slice(),
        );

        gl.active_texture(GL::TEXTURE0);
        gl.bind_texture(GL::TEXTURE_2D, Some(&assets.tileset.texture));
        gl.active_texture(GL::TEXTURE1);
        gl.bind_texture(GL::TEXTURE_2D, Some(&assets.charset.texture));

        state
            .vertex_buffer_renderer
            .render_triangles(&vertex_buffer, gl);
    }
    {
        let mut vertex_buffer = VertexBuffer::new();
        render_health_bars(game_state, &mut vertex_buffer);
        render_attack_markers(game_state, &mut vertex_buffer);

        gl.uniform_matrix3fv_with_f32_array(
            Some(&state.uniform_locations.view_projection),
            false,
            game_state.camera.world_to_ndc.as_slice(),
        );
        gl.active_texture(GL::TEXTURE0);
        gl.bind_texture(GL::TEXTURE_2D, Some(&assets.white.texture));

        state
            .vertex_buffer_renderer
            .render_triangles(&vertex_buffer, gl);
    }
    {
        let mut vertex_buffer = LineVertexBuffer::new();
        render_debug_lines(game_state, &mut vertex_buffer);
        let line_vertices = vertex_buffer.vertex_buffer;

        gl.active_texture(GL::TEXTURE0);
        gl.bind_texture(GL::TEXTURE_2D, Some(&assets.white.texture));
        state
            .vertex_buffer_renderer
            .render_lines(&line_vertices, gl);
    }

    gl.use_program(Some(&state.text_program));

    {
        let mut vertex_buffer = VertexBuffer::new();
        render_world_text(game_state, assets, &mut vertex_buffer);

        gl.uniform1f(
            Some(&state.uniform_locations.text_distance_range),
            assets.font_atlas.distance_range,
        );
        gl.uniform_matrix3fv_with_f32_array(
            Some(&state.uniform_locations.text_view_projection),
            false,
            game_state.camera.logical_screen_to_ndc.as_slice(),
        );
        gl.active_texture(GL::TEXTURE0);
        gl.bind_texture(GL::TEXTURE_2D, Some(&assets.font.texture));

        state
            .vertex_buffer_renderer
            .render_triangles(&vertex_buffer, gl);
    }
    {
        let mut text_vertices = VertexBuffer::new();
        render_debug_ui(
            state,
            game_state,
            &state.metrics.borrow(),
            assets,
            &mut text_vertices,
        );
        state
            .vertex_buffer_renderer
            .render_triangles(&text_vertices, gl);
    }
}

fn render_dense_tile_layer(
    layer: &[TileIndex],
    room_size: Vector2<u32>,
    tileset_vertices: &mut TileVertexBuffer,
) {
    for (i, tile_index) in layer.iter().copied().enumerate() {
        if let Some(tile_index) = tile_index.0 {
            let i = i as u32;
            let x = i % room_size.x;
            let y = i / room_size.x;
            let xy = Vector2::new(x as f32, y as f32);
            tileset_vertices.push_tile(xy, tile_index.get() as u32, 0);
        }
    }
}

fn render_sparse_tile_layer(
    layer: &[(Vector2<u32>, TileIndex)],
    tileset_vertices: &mut TileVertexBuffer,
) {
    for (position, tile_index) in layer {
        if let Some(tile_index) = tile_index.0 {
            let xy = position.map(|x| x as f32);
            tileset_vertices.push_tile(xy, tile_index.get() as u32, 0);
        }
    }
}

fn render_foreground(game_state: &GameState, tile_vertices: &mut TileVertexBuffer) {
    let mut fg_y_lower_bound = f32::NEG_INFINITY;

    // TODO: traversing the foreground layer multiple times could be optimized
    for obj in game_state.objects.iter() {
        render_foreground_tile_layer(
            &game_state.room.fg_sparse_layer,
            (fg_y_lower_bound, obj.local_position.y),
            tile_vertices,
        );
        fg_y_lower_bound = obj.local_position.y;

        if let Some(animation) = &game_state.client_config.animations.get(obj.animation_id) {
            let sprite_size = animation.sprite_size;
            let position = obj.local_position - (sprite_size.cast() - animation.anchor);

            let (animation, started_at) = match &obj.animation {
                Some(obj_animation) => {
                    let animation = &animation.custom[obj_animation.animation_index as usize];
                    (animation, obj_animation.started_at)
                }
                None if obj.direction.is_some() => {
                    (&animation.walk, obj.remote_position_received_at)
                }
                None => (&animation.idle, obj.remote_position_received_at),
            };
            let direction = obj.look_direction;
            let animation_time = game_state.time.now - started_at;
            if let Some(sprite_index) = animation.get(direction, animation_time) {
                tile_vertices.push_tile_multi(position, sprite_size, sprite_index.0 as _, 1);
            }
        }
    }

    render_foreground_tile_layer(
        &game_state.room.fg_sparse_layer,
        (fg_y_lower_bound, f32::INFINITY),
        tile_vertices,
    );
}

fn render_foreground_tile_layer(
    layer: &[ForegroundTile],
    y_bounds: (f32, f32),
    tile_vertices: &mut TileVertexBuffer,
) {
    for fg_tile in layer.iter() {
        let fg_y = fg_tile.position.y as f32;
        let fg_dy = fg_tile.height as f32 + 1.0; // adding 1 so the bottom of the tile is the reference point
        if fg_y + fg_dy >= y_bounds.0 && fg_y + fg_dy < y_bounds.1 {
            if let TileIndex(Some(fg_tile_index)) = fg_tile.tile_index {
                let xy = fg_tile.position.cast();
                tile_vertices.push_tile(xy, fg_tile_index.get() as _, 0);
            }
        }
    }
}

fn render_health_bars(game_state: &GameState, vertex_buffer: &mut VertexBuffer) {
    for obj in game_state.objects.iter() {
        if let Some(animation) = game_state.client_config.animations.get(obj.animation_id) {
            if obj.health < obj.max_health {
                let zero = Vector2::new(0.0, 0.0);
                let xy = obj.local_position - Vector2::new(0.5, animation.sprite_size.y as _);
                let wh = Vector2::new(1.0, 1.0 / 8.0);
                let color = Vector4::new(0, 0, 0, 0xff);
                vertex_buffer.push_quad(xy, wh, zero, zero, color, 0);
                let wh = Vector2::new(obj.health as f32 / obj.max_health as f32, 1.0 / 8.0);
                let color = Vector4::new(0xff, 0, 0, 0xff);
                vertex_buffer.push_quad(xy, wh, zero, zero, color, 0);
            }
        }
    }
}

fn render_debug_lines(game_state: &GameState, vertex_buffer: &mut LineVertexBuffer) {
    if game_state.show_debug {
        for obj in game_state.objects.iter() {
            if obj.id != game_state.self_id {
                vertex_buffer.push_rect(
                    obj.remote_position - Vector2::new(0.2, 0.05),
                    Vector2::new(0.4, 0.1),
                    Vector4::new(0xff, 0, 0, 0xff),
                );
            }
        }
    }
}

fn render_attack_markers(game_state: &GameState, vertex_buffer: &mut VertexBuffer) {
    for marker in &game_state.attack_markers {
        let wh = Vector2::new(2.0, 2.0) * marker.radius;
        let xy = marker.position - wh / 2.0;
        let color = Vector4::new(0xff, 0, 0, 0x1f);
        vertex_buffer.push_quad(
            xy,
            wh,
            Vector2::new(0.0, 0.0),
            Vector2::new(0.0, 0.0),
            color,
            0,
        );

        let t = (game_state.time.now - marker.received_at) / marker.length;
        let wh = wh * t;
        let xy = marker.position - wh / 2.0;
        vertex_buffer.push_quad(
            xy,
            wh,
            Vector2::new(0.0, 0.0),
            Vector2::new(0.0, 0.0),
            color,
            0,
        );
    }
}

fn render_world_text(game_state: &GameState, assets: &Assets, vertex_buffer: &mut VertexBuffer) {
    let black = Vector4::new(0, 0, 0, 0xff);
    let eps = Vector2::new(0.4, 0.4);
    for obj in game_state.objects.iter() {
        if obj.typ == ObjectType::Player {
            let xy = game_state.camera.world_point_to_screen(obj.local_position);
            let color = if obj.id == game_state.self_id {
                Vector4::new(0xff, 0xff, 0, 0xff)
            } else {
                Vector4::new(0xff, 0xff, 0xff, 0xff)
            };
            let str = obj.id.0.to_string();
            assets
                .font_atlas
                .push_text(&str, xy + eps, 6.0, black, Align::Center, vertex_buffer);
            assets
                .font_atlas
                .push_text(&str, xy, 6.0, color, Align::Center, vertex_buffer);
        }
    }

    for label in &game_state.health_change_labels {
        let dt = game_state.time.now - label.received_at;
        let dy = 5.0 + 10.0 * dt * dt;
        let xy = game_state.camera.world_point_to_screen(label.position) - Vector2::new(0.0, dy);
        let color = if label.health_change > 0 {
            Vector4::new(0, 0xff, 0xff, 0xff)
        } else if label.object_type == ObjectType::Mob {
            Vector4::new(0xff, 0xff, 0xff, 0xff)
        } else {
            Vector4::new(0xff, 0, 0, 0xff)
        };
        let str = label.health_change.abs().to_string();
        assets
            .font_atlas
            .push_text(&str, xy + eps, 8.0, black, Align::Center, vertex_buffer);
        assets
            .font_atlas
            .push_text(&str, xy, 8.0, color, Align::Center, vertex_buffer);
    }
}

fn render_debug_ui(
    app_state: &AppState,
    game_state: &GameState,
    metrics: &Metrics,
    assets: &Assets,
    buf: &mut VertexBuffer,
) {
    let x = game_state.camera.logical_screen_size.x - 48.0;
    let lines = [
        ("FPS:", &format!("{:.}", metrics.fps_stats.fps)),
        ("p50:", &format!("{:.1}ms", metrics.fps_stats.median_ms)),
        ("p100:", &format!("{:.1} ms", metrics.fps_stats.max_ms)),
        ("ping:", &format!("{:.1} ms", game_state.ping_rtt * 1000.0)),
        ("in:", &format!("{} B/s", metrics.net_stats.in_bytes)),
        ("", &format!("{} evt/s", metrics.net_stats.in_events)),
        ("", &format!("{} frame/s", metrics.net_stats.in_frames)),
        ("out:", &format!("{} B/s", metrics.net_stats.out_bytes)),
        ("", &format!("{} cmd/s", metrics.net_stats.out_commands)),
        ("", &format!("{} frame/s", metrics.net_stats.out_frames)),
    ];
    for (i, (str1, str2)) in lines.iter().enumerate() {
        let y = i as f32 * 5.5;
        let xy = Vector2::new(x, y);
        let color = Vector4::new(0xff, 0xff, 0xff, 0xff);
        let fa = &assets.font_atlas;
        fa.push_text(str1, xy, 6.0, color, Align::Left, buf);
        let xy = xy + Vector2::new(16.0, 0.0);
        fa.push_text(str2, xy, 6.0, color, Align::Left, buf);
    }

    let y = game_state.camera.logical_screen_size.y - 12.0;
    let lines = [
        ("client:", &format!("{:.7}", &app_state.client_git_sha)),
        (
            "server:",
            &format!("{:.7}", game_state.client_config.server_git_sha),
        ),
    ];
    for (i, (str1, str2)) in lines.iter().enumerate() {
        let y = y + i as f32 * 5.5;
        let xy = Vector2::new(x, y);
        let color = Vector4::new(0xff, 0xff, 0xff, 0xff);
        let fa = &assets.font_atlas;
        fa.push_text(str1, xy, 6.0, color, Align::Left, buf);
        let xy = xy + Vector2::new(16.0, 0.0);
        fa.push_text(str2, xy, 6.0, color, Align::Left, buf);
    }
}
