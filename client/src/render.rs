use std::f32::consts::PI;

use nalgebra::{Orthographic3, Scale3, Vector2, Vector4};
use web_sys::WebGl2RenderingContext as GL;

use crate::{
    app_state::AppState,
    vertex_buffer::{LineVertexBuffer, TileVertexBuffer, VertexBuffer},
};

pub fn render(state: &mut AppState) {
    let gl = &state.gl;
    gl.clear_color(0.0, 0.0, 0.0, 1.0);
    gl.clear(GL::COLOR_BUFFER_BIT);

    let game_state = match &state.game_state {
        Ok(game_state) => game_state,
        Err(_) => return,
    };

    let mut tileset_vertices = TileVertexBuffer::new(Vector2::new(1.0, 1.0), Vector2::new(16, 16));
    let mut charset_vertices = TileVertexBuffer::new(Vector2::new(1.0, 1.0), Vector2::new(16, 16));

    for tile in game_state.room.tiles.iter().copied() {
        tileset_vertices.push_tile(tile.position.cast(), tile.tile_index.0 as u32);
    }

    charset_vertices.push_tile(game_state.self_movement.position, 0);

    for other_position in game_state.other_positions.values() {
        let current_position = match other_position.direction {
            Some(dir) => {
                let mov_distance =
                    other_position.velocity * (state.time.now - other_position.started_at);
                other_position.position + mov_distance * dir.to_vector()
            }
            None => other_position.position,
        };

        charset_vertices.push_tile(current_position, 5 + 16 * 1);
    }

    let mut line_vertices = LineVertexBuffer::new();

    for i in 0..16 {
        let (y, x) = ((i as f32) / 16.0 * PI).sin_cos();
        let r = (i % 2) as f32;
        let g = (i % 3) as f32;
        let b = (i % 5) as f32;
        let start = Vector2::new(100.0, 135.0);
        let end = Vector2::new(100.0 + 100.0 * x, 135.0 + 100.0 * y).map(|x| x.round());
        line_vertices.push_line(start, end, Vector4::new(r, g, b, 1.0));
    }

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
    gl.bind_texture(GL::TEXTURE_2D, Some(&state.textures.tileset.texture));

    state.vertex_buffer_renderer.render_triangles(&tileset_vertices, gl);

    gl.bind_texture(GL::TEXTURE_2D, Some(&state.textures.charset.texture));

    state.vertex_buffer_renderer.render_triangles(&charset_vertices, gl);

    gl.uniform_matrix4fv_with_f32_array(
        Some(&state.uniform_locations.view_projection),
        false,
        logical_screen_to_ndc.as_slice(),
    );

    gl.bind_texture(GL::TEXTURE_2D, Some(&state.textures.white.texture));
    state.vertex_buffer_renderer.render_lines(&line_vertices, gl);

    gl.use_program(Some(&state.text_program));

    gl.uniform_matrix4fv_with_f32_array(
        Some(&state.uniform_locations.text_view_projection),
        false,
        logical_screen_to_ndc.as_slice(),
    );
    gl.uniform1i(Some(&state.uniform_locations.text_sampler), 0);
    gl.active_texture(GL::TEXTURE0);
    gl.bind_texture(GL::TEXTURE_2D, Some(&state.textures.font.texture));

    let mut text_vertices = VertexBuffer::new();
    for i in 0..8 {
        let i = i as f32;
        let pos = Vector2::new(140.0, 8.0 * i.powf(1.5));
        let h = 6.0 + 4.0 * i;
        state.font_atlas.push_text("Árvíztűrő tükörfúrógép", pos, h, &mut text_vertices);
    }
    state.vertex_buffer_renderer.render_triangles(&text_vertices, gl);
}
