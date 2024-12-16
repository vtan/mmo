use nalgebra::{Orthographic3, Scale3, Vector2, Vector4};
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

    // TODO: calculate the position in the update function
    for other_position in game_state.other_positions.values() {
        let current_position = match other_position.direction {
            Some(dir) => {
                let mov_distance =
                    other_position.velocity * (state.time.now - other_position.started_at);
                other_position.position + mov_distance * dir.to_vector()
            }
            None => other_position.position,
        };

        charset_vertices.push_tile_multi(current_position + PLAYER_OFFSET, Vector2::new(1, 2), 0);
    }

    let mut line_vertices = LineVertexBuffer::new();

    line_vertices.push_rect(
        game_state.self_movement.position - Vector2::new(0.2, 0.05),
        Vector2::new(0.4, 0.1),
        Vector4::new(1.0, 0.0, 1.0, 1.0),
    );

    let tileset_vertices = tileset_vertices.vertex_buffer;
    let charset_vertices = charset_vertices.vertex_buffer;
    let line_vertices = line_vertices.vertex_buffer;
    gl.use_program(Some(&state.program));

    let logical_screen_to_ndc =
        Orthographic3::new(0.0, 480.0, 270.0, 0.0, -1.0, 1.0).to_homogeneous();
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
        ("FPS:", &format!("{:.0}", state.fps_counter.agg.fps)),
        ("p50:", &format!("{:.2}ms", state.fps_counter.agg.median_ms)),
        ("max:", &format!("{:.2}ms", state.fps_counter.agg.max_ms)),
        ("ping:", &format!("{:.2}ms", game_state.ping_rtt * 1000.0)),
    ];
    for (i, (str1, str2)) in fps_lines.iter().enumerate() {
        let y = i as f32 * 5.5;
        let color = Vector4::new(1.0, 1.0, 1.0, 0.6);
        let fa = &assets.font_atlas;
        fa.push_text(str1, Vector2::new(420.0, y), 6.0, color, &mut text_vertices);
        fa.push_text(str2, Vector2::new(432.0, y), 6.0, color, &mut text_vertices);
    }

    for i in 0..8 {
        let i = i as f32;
        let pos = Vector2::new(140.0, 8.0 * i.powf(1.5));
        let h = 6.0 + 4.0 * i;
        let c = Vector4::new(1.0, 1.0, 1.0, 1.0);
        assets
            .font_atlas
            .push_text("Árvíztűrő tükörfúrógép", pos, h, c, &mut text_vertices);
    }
    state.vertex_buffer_renderer.render_triangles(&text_vertices, gl);
}
