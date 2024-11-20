use std::mem::size_of;

use nalgebra::{Orthographic3, Scale3, Vector2};
use wasm_bindgen::prelude::*;
use web_sys::{WebGl2RenderingContext as GL, WebGlVertexArrayObject};

use crate::app_state::{AppState, AttribLocations, Buffers, TexturedVertex, TileAttribs};

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

    let stride = 5 * size_of::<f32>() as i32;
    gl.bind_buffer(GL::ARRAY_BUFFER, Some(&buffers.tile_attrib));
    {
        let num_components = 2;
        let typ = GL::FLOAT;
        let normalize = false;
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
    {
        let num_components = 1;
        let typ = GL::UNSIGNED_INT;
        let normalize = false;
        let offset = (2 + 2) * size_of::<f32>() as i32;
        gl.vertex_attrib_pointer_with_i32(
            attrib_locations.instance_texture_index,
            num_components,
            typ,
            normalize,
            stride,
            offset,
        );
        gl.vertex_attrib_divisor(attrib_locations.instance_texture_index, 1);
        gl.enable_vertex_attrib_array(attrib_locations.instance_texture_index);
    }
    Ok(vao)
}

pub fn create_textured_vertex_vao(
    gl: &GL,
    buffers: &Buffers,
    attrib_locations: &AttribLocations,
) -> Result<WebGlVertexArrayObject, JsValue> {
    let vao = gl.create_vertex_array().ok_or("Could not create vertex array object")?;
    gl.bind_vertex_array(Some(&vao));

    let stride = 4 * size_of::<f32>() as i32;
    gl.bind_buffer(GL::ARRAY_BUFFER, Some(&buffers.textured_vertex));
    {
        let num_components = 2;
        let typ = GL::FLOAT;
        let normalize = false;
        let offset = 0;
        gl.vertex_attrib_pointer_with_i32(
            attrib_locations.position2,
            num_components,
            typ,
            normalize,
            stride,
            offset,
        );
        gl.enable_vertex_attrib_array(attrib_locations.position2);
    }
    {
        let num_components = 2;
        let typ = GL::FLOAT;
        let normalize = false;
        let offset = 2 * size_of::<f32>() as i32;
        gl.vertex_attrib_pointer_with_i32(
            attrib_locations.texture_position2,
            num_components,
            typ,
            normalize,
            stride,
            offset,
        );
        gl.enable_vertex_attrib_array(attrib_locations.texture_position2);
    }
    Ok(vao)
}

pub fn render(state: &mut AppState) {
    let gl = &state.gl;
    gl.clear_color(0.0, 0.0, 0.0, 1.0);
    gl.clear(GL::COLOR_BUFFER_BIT);

    let game_state = match &state.game_state {
        Ok(game_state) => game_state,
        Err(_) => return,
    };

    /*
    state.buffers.tile_attrib_data.clear();

    let tileset_width = state.textures.tileset.width / PIXELS_PER_TILE;

    for tile in game_state.room.tiles.iter().copied() {
        let tex_x = (tile.tile_index.0 as u32) % tileset_width;
        let tex_y = (tile.tile_index.0 as u32) / tileset_width;
        state.buffers.tile_attrib_data.push(TileAttribs {
            world_position: Vector2::new(tile.position.x as f32, tile.position.y as f32),
            texture_position: Vector2::new(
                tex_x as f32 / (PIXELS_PER_TILE as f32),
                tex_y as f32 / (PIXELS_PER_TILE as f32),
            ),
            texture_index: 0,
        });
    }

    state.buffers.tile_attrib_data.push(TileAttribs {
        world_position: game_state.self_movement.position,
        texture_position: Vector2::new(0.0, 0.0),
        texture_index: 1,
    });
    for other_position in game_state.other_positions.values() {
        let current_position = match other_position.direction {
            Some(dir) => {
                let mov_distance =
                    other_position.velocity * (state.time.now - other_position.started_at);
                other_position.position + mov_distance * dir.to_vector()
            }
            None => other_position.position,
        };
        let attribs = TileAttribs {
            world_position: current_position,
            texture_position: Vector2::new(
                5.0 / (PIXELS_PER_TILE as f32),
                1.0 / (PIXELS_PER_TILE as f32),
            ),
            texture_index: 1,
        };
        state.buffers.tile_attrib_data.push(attribs);
    }

    gl.use_program(Some(&state.program));

    let projection = Orthographic3::new(0.0, 480.0, 270.0, 0.0, -1.0, 1.0).to_homogeneous();
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
    gl.active_texture(GL::TEXTURE1);
    gl.bind_texture(GL::TEXTURE_2D, Some(&state.textures.charset.texture));
    gl.uniform1iv_with_i32_array(Some(&state.uniform_locations.sampler), &[0, 1]);

    render_tile_vao(state);
    */

    //
    gl.use_program(Some(&state.program2));

    let vertices = vec![
        TexturedVertex {
            position: Vector2::new(16.0, 16.0),
            texture_position: Vector2::new(0.0, 0.0),
        },
        TexturedVertex {
            position: Vector2::new(16.0, 32.0),
            texture_position: Vector2::new(0.0, 1.0 / 16.0),
        },
        TexturedVertex {
            position: Vector2::new(32.0, 32.0),
            texture_position: Vector2::new(1.0 / 16.0, 1.0 / 16.0),
        },
        //
        TexturedVertex {
            position: Vector2::new(32.0, 32.0),
            texture_position: Vector2::new(1.0 / 16.0, 1.0 / 16.0),
        },
        TexturedVertex {
            position: Vector2::new(32.0, 16.0),
            texture_position: Vector2::new(1.0 / 16.0, 0.0),
        },
        TexturedVertex {
            position: Vector2::new(16.0, 16.0),
            texture_position: Vector2::new(0.0, 0.0),
        },
    ];

    gl.bind_vertex_array(Some(&state.vaos.textured_vertex));
    gl.bind_buffer(GL::ARRAY_BUFFER, Some(&state.buffers.textured_vertex));
    //
    // Unsafe: do not allocate memory until the view is dropped
    unsafe {
        let byte_slice = std::slice::from_raw_parts(
            vertices.as_ptr() as *const u8,
            vertices.len() * std::mem::size_of::<TexturedVertex>(),
        );
        let buffer_view = js_sys::Uint8Array::view(byte_slice);
        gl.buffer_data_with_array_buffer_view(GL::ARRAY_BUFFER, &buffer_view, GL::DYNAMIC_DRAW);
    }

    let projection = Orthographic3::new(0.0, 480.0, 270.0, 0.0, -1.0, 1.0).to_homogeneous();
    let view = Scale3::new(1.0, 1.0, 1.0).to_homogeneous(); // TODO delete?
    let view_projection = projection * view;
    gl.uniform_matrix4fv_with_f32_array(
        Some(&state.uniform_locations.view_projection2),
        false,
        view_projection.as_slice(),
    );

    gl.active_texture(GL::TEXTURE0);
    gl.bind_texture(GL::TEXTURE_2D, Some(&state.textures.tileset.texture));
    gl.uniform1i(Some(&state.uniform_locations.sampler2), 0);

    gl.draw_arrays(GL::TRIANGLES, 0, vertices.len() as i32);
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
