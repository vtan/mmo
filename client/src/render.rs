use nalgebra::{Orthographic3, Scale2, Scale3, Vector2, Vector4};
use web_sys::WebGl2RenderingContext as GL;

use crate::{
    app_state::AppState,
    vertex_buffer::{LineVertexBuffer, TileVertexBuffer, VertexBuffer},
};

const PLAYER_OFFSET: Vector2<f32> = Vector2::new(-0.5, -2.0);

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

    let mut tileset_vertices = TileVertexBuffer::new(Vector2::new(1.0, 1.0), Vector2::new(16, 16));
    let mut charset_vertices = TileVertexBuffer::new(Vector2::new(1.0, 1.0), Vector2::new(16, 16));

    for tile in game_state.room.tiles.iter().copied() {
        tileset_vertices.push_tile(tile.position.cast(), tile.tile_index.0 as u32);
    }

    charset_vertices.push_tile_multi(
        game_state.self_movement.position + PLAYER_OFFSET,
        Vector2::new(1, 2),
        0,
    );

    for movement in game_state.local_movements.values() {
        charset_vertices.push_tile_multi(movement.position + PLAYER_OFFSET, Vector2::new(1, 2), 0);
    }

    let mut line_vertices = LineVertexBuffer::new();

    for remote_movement in game_state.remote_movements.values() {
        line_vertices.push_rect(
            remote_movement.position - Vector2::new(0.2, 0.05),
            Vector2::new(0.4, 0.1),
            Vector4::new(1.0, 0.0, 1.0, 1.0),
        );
    }

    let tileset_vertices = tileset_vertices.vertex_buffer;
    let charset_vertices = charset_vertices.vertex_buffer;
    let line_vertices = line_vertices.vertex_buffer;
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

    gl.uniform1i(Some(&state.uniform_locations.sampler), 0);
    gl.active_texture(GL::TEXTURE0);
    gl.bind_texture(GL::TEXTURE_2D, Some(&assets.tileset.texture));

    state.vertex_buffer_renderer.render_triangles(&tileset_vertices, gl);

    gl.bind_texture(GL::TEXTURE_2D, Some(&assets.charset.texture));

    state.vertex_buffer_renderer.render_triangles(&charset_vertices, gl);

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
    gl.uniform1i(Some(&state.uniform_locations.text_sampler), 0);
    gl.active_texture(GL::TEXTURE0);
    gl.bind_texture(GL::TEXTURE_2D, Some(&assets.font.texture));

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
    for (player_id, movement) in game_state.local_movements.iter() {
        let xy = tile_to_pixel_2d * movement.position;
        let color = Vector4::new(0.0, 1.0, 0.0, 1.0);
        let str = player_id.0.to_string();
        assets.font_atlas.push_text(&str, xy + eps, 6.0, black, &mut text_vertices);
        assets.font_atlas.push_text(&str, xy, 6.0, color, &mut text_vertices);
    }
    {
        let player_id = game_state.self_id;
        let xy = tile_to_pixel_2d * game_state.self_movement.position;
        let color = Vector4::new(0.0, 1.0, 1.0, 1.0);
        let str = player_id.0.to_string();
        assets.font_atlas.push_text(&str, xy + eps, 6.0, black, &mut text_vertices);
        assets.font_atlas.push_text(&str, xy, 6.0, color, &mut text_vertices);
    }

    state.vertex_buffer_renderer.render_triangles(&text_vertices, gl);
}
