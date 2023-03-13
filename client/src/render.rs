use std::mem::size_of;

use nalgebra::{Orthographic3, Scale3, Vector2};
use wasm_bindgen::prelude::*;
use web_sys::{WebGl2RenderingContext as GL, WebGlVertexArrayObject};

use crate::app_state::{AppState, AttribLocations, Buffers, TileAttribs};

const PIXELS_PER_TILE: u32 = 16;

pub fn create_tile_vao(
    gl: &GL,
    buffers: &Buffers,
    attrib_locations: &AttribLocations,
) -> Result<WebGlVertexArrayObject, JsValue> {
    let vao = gl.create_vertex_array().ok_or("Could not create vertex array object")?;
    gl.bind_vertex_array(Some(&vao));

    gl.bind_buffer(GL::ARRAY_BUFFER, Some(&buffers.quad_vertex));
    {
        let num_components = 2;
        let typ = GL::FLOAT;
        let normalize = false;
        let stride = 0;
        let offset = 0;
        gl.vertex_attrib_pointer_with_i32(
            attrib_locations.position,
            num_components,
            typ,
            normalize,
            stride,
            offset,
        );
        gl.enable_vertex_attrib_array(attrib_locations.position);
    }

    gl.bind_buffer(GL::ARRAY_BUFFER, Some(&buffers.tile_attrib));
    {
        let num_components = 2;
        let typ = GL::FLOAT;
        let normalize = false;
        let stride = 4 * size_of::<f32>() as i32;
        let offset = 0;
        gl.vertex_attrib_pointer_with_i32(
            attrib_locations.instance_translation,
            num_components,
            typ,
            normalize,
            stride,
            offset,
        );
        gl.vertex_attrib_divisor(attrib_locations.instance_translation, 1);
        gl.enable_vertex_attrib_array(attrib_locations.instance_translation);
    }
    {
        let num_components = 2;
        let typ = GL::FLOAT;
        let normalize = false;
        let stride = 4 * size_of::<f32>() as i32;
        let offset = 2 * size_of::<f32>() as i32;
        gl.vertex_attrib_pointer_with_i32(
            attrib_locations.instance_texture_coord_offset,
            num_components,
            typ,
            normalize,
            stride,
            offset,
        );
        gl.vertex_attrib_divisor(attrib_locations.instance_texture_coord_offset, 1);
        gl.enable_vertex_attrib_array(attrib_locations.instance_texture_coord_offset);
    }
    Ok(vao)
}

pub fn render(state: &mut AppState) {
    let gl = &state.gl;
    gl.clear_color(0.0, 0.0, 0.0, 1.0);
    gl.clear(GL::COLOR_BUFFER_BIT);

    if state.connection.is_none() {
        return;
    }

    state.buffers.tile_attrib_data = vec![TileAttribs {
        world_position: state.player_position,
        texture_position: Vector2::new(0.0, 0.0),
    }];
    for other_position in state.other_positions.values() {
        let attribs = TileAttribs {
            world_position: *other_position,
            texture_position: Vector2::new(
                5.0 / (PIXELS_PER_TILE as f32),
                1.0 / (PIXELS_PER_TILE as f32),
            ),
        };
        state.buffers.tile_attrib_data.push(attribs);
    }

    gl.use_program(Some(&state.program));

    let projection = Orthographic3::new(0.0, 320.0, 180.0, 0.0, -1.0, 1.0).to_homogeneous();
    let view = Scale3::new(
        PIXELS_PER_TILE as _,
        PIXELS_PER_TILE as _,
        PIXELS_PER_TILE as _,
    )
    .to_homogeneous();
    let view_projection = projection * view;
    gl.uniform_matrix4fv_with_f32_array(
        Some(&state.uniform_locations.view_projection),
        false,
        view_projection.as_slice(),
    );

    gl.active_texture(GL::TEXTURE0);
    gl.bind_texture(GL::TEXTURE_2D, Some(&state.textures.tileset.texture));
    gl.uniform1i(Some(&state.uniform_locations.sampler), 0);

    render_tile_vao(state);
}

fn render_tile_vao(state: &AppState) {
    let gl = &state.gl;
    gl.bind_vertex_array(Some(&state.vaos.tile));
    gl.bind_buffer(GL::ARRAY_BUFFER, Some(&state.buffers.tile_attrib));

    // Unsafe: do not allocate memory until the view is dropped
    unsafe {
        let byte_slice = std::slice::from_raw_parts(
            state.buffers.tile_attrib_data.as_ptr() as *const u8,
            state.buffers.tile_attrib_data.len() * std::mem::size_of::<TileAttribs>(),
        );
        let buffer_view = js_sys::Uint8Array::view(byte_slice);
        gl.buffer_data_with_array_buffer_view(GL::ARRAY_BUFFER, &buffer_view, GL::DYNAMIC_DRAW);
    }

    let offset = 0;
    let count = 4;
    let instance_count = state.buffers.tile_attrib_data.len() as i32;
    gl.draw_arrays_instanced(GL::TRIANGLE_STRIP, offset, count, instance_count);
}
