use mmo_common::{
    animation::AnimationAction,
    object::ObjectType,
    room::{ForegroundTile, TileIndex},
};
use nalgebra::{Orthographic3, Scale2, Scale3, Vector2, Vector4};
use web_sys::WebGl2RenderingContext as GL;

use crate::{
    app_state::AppState,
    vertex_buffer::{LineVertexBuffer, TileVertexBuffer, VertexBuffer},
};

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

    let mut tile_vertices = TileVertexBuffer::new(Vector2::new(1.0, 1.0), Vector2::new(16, 16));

    for layer in &game_state.room.bg_dense_layers {
        render_dense_tile_layer(layer, game_state.room.size, &mut tile_vertices);
    }
    render_sparse_tile_layer(&game_state.room.bg_sparse_layer, &mut tile_vertices);

    let mut fg_y_lower_bound = f32::NEG_INFINITY;

    // TODO: traversing the foreground layer multiple times could be optimized
    for movement in game_state.local_movements.iter() {
        render_foreground_tile_layer(
            &game_state.room.fg_sparse_layer,
            (fg_y_lower_bound, movement.position.y),
            &mut tile_vertices,
        );
        fg_y_lower_bound = movement.position.y;

        {
            let animation = &game_state.client_config.animations[movement.animation_id];
            let sprite_size = animation.sprite_size;
            let position = movement.position - (sprite_size.cast() - animation.anchor);

            let animation = match movement.animation_action {
                Some(AnimationAction::Attack) => &animation.attack,
                None if movement.direction.is_some() => &animation.walk,
                None => &animation.idle,
            };
            let direction = movement.direction.unwrap_or(movement.look_direction);
            if let Some(sprite_index) = animation.get(direction, movement.animation_time) {
                tile_vertices.push_tile_multi(position, sprite_size, sprite_index.0 as _, 1);
            }
        }
    }
    render_foreground_tile_layer(
        &game_state.room.fg_sparse_layer,
        (fg_y_lower_bound, f32::INFINITY),
        &mut tile_vertices,
    );

    let tileset_vertices = tile_vertices.vertex_buffer;
    gl.use_program(Some(&state.program));

    let logical_screen_to_ndc =
        Orthographic3::new(0.0, 480.0, 270.0, 0.0, -1.0, 1.0).to_homogeneous();
    let tile_to_pixel_2d = Scale2::new(16.0, 16.0);
    let tile_to_pixel = Scale3::new(16.0, 16.0, 1.0).to_homogeneous();
    let tile_to_ndc = logical_screen_to_ndc * tile_to_pixel;
    gl.uniform_matrix4fv_with_f32_array(
        Some(&state.uniform_locations.view_projection),
        false,
        tile_to_ndc.as_slice(),
    );

    gl.uniform1iv_with_i32_array(Some(&state.uniform_locations.sampler), &[0, 1]);
    gl.active_texture(GL::TEXTURE0);
    gl.bind_texture(GL::TEXTURE_2D, Some(&assets.tileset.texture));
    gl.active_texture(GL::TEXTURE1);
    gl.bind_texture(GL::TEXTURE_2D, Some(&assets.charset.texture));

    state.vertex_buffer_renderer.render_triangles(&tileset_vertices, gl);

    // lines

    let mut line_vertices = LineVertexBuffer::new();

    for remote_movement in game_state.remote_movements.values() {
        line_vertices.push_rect(
            remote_movement.position - Vector2::new(0.2, 0.05),
            Vector2::new(0.4, 0.1),
            Vector4::new(1.0, 0.0, 1.0, 1.0),
        );
    }

    let line_vertices = line_vertices.vertex_buffer;

    gl.active_texture(GL::TEXTURE0);
    gl.bind_texture(GL::TEXTURE_2D, Some(&assets.white.texture));
    state.vertex_buffer_renderer.render_lines(&line_vertices, gl);

    gl.uniform_matrix4fv_with_f32_array(
        Some(&state.uniform_locations.view_projection),
        false,
        logical_screen_to_ndc.as_slice(),
    );

    gl.use_program(Some(&state.text_program));

    gl.uniform1f(
        Some(&state.uniform_locations.text_distance_range),
        assets.font_atlas.distance_range,
    );

    gl.uniform_matrix4fv_with_f32_array(
        Some(&state.uniform_locations.text_view_projection),
        false,
        logical_screen_to_ndc.as_slice(),
    );
    gl.uniform1iv_with_i32_array(Some(&state.uniform_locations.text_sampler), &[0, 1]);
    gl.active_texture(GL::TEXTURE0);
    gl.bind_texture(GL::TEXTURE_2D, Some(&assets.font.texture));

    // text

    let mut text_vertices = VertexBuffer::new();

    let fps_lines = [
        ("FPS:", &format!("{:.}", state.fps_counter.agg.fps)),
        ("p50:", &format!("{:.1}ms", state.fps_counter.agg.median_ms)),
        ("p100:", &format!("{:.1}ms", state.fps_counter.agg.max_ms)),
        ("ping:", &format!("{:.1}ms", game_state.ping_rtt * 1000.0)),
    ];
    for (i, (str1, str2)) in fps_lines.iter().enumerate() {
        let y = i as f32 * 5.5;
        let color = Vector4::new(1.0, 1.0, 1.0, 0.6);
        let fa = &assets.font_atlas;
        fa.push_text(str1, Vector2::new(420.0, y), 6.0, color, &mut text_vertices);
        fa.push_text(str2, Vector2::new(432.0, y), 6.0, color, &mut text_vertices);
    }

    let black = Vector4::new(0.0, 0.0, 0.0, 1.0);
    let eps = Vector2::new(0.4, 0.4);
    for movement in game_state.local_movements.iter() {
        if movement.object_type == ObjectType::Player {
            let xy = tile_to_pixel_2d * movement.position;
            let color = if movement.object_id == game_state.self_id {
                Vector4::new(1.0, 1.0, 0.0, 1.0)
            } else {
                Vector4::new(1.0, 1.0, 1.0, 1.0)
            };
            let str = movement.object_id.0.to_string();
            assets.font_atlas.push_text(&str, xy + eps, 6.0, black, &mut text_vertices);
            assets.font_atlas.push_text(&str, xy, 6.0, color, &mut text_vertices);
        }
    }

    state.vertex_buffer_renderer.render_triangles(&text_vertices, gl);
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
