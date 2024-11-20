use nalgebra::Vector2;
use web_sys::{
    WebGl2RenderingContext, WebGlBuffer, WebGlProgram, WebGlUniformLocation, WebGlVertexArrayObject,
};

use crate::game_state::{GameState, PartialGameState};
use crate::texture::Texture;

pub struct AppState {
    pub gl: WebGl2RenderingContext,
    pub program: WebGlProgram,
    pub program2: WebGlProgram,
    pub attrib_locations: AttribLocations,
    pub uniform_locations: UniformLocations,
    pub textures: Textures,
    pub vaos: Vaos,
    pub buffers: Buffers,
    pub time: Timestamps,
    pub game_state: Result<GameState, PartialGameState>,
}

pub struct AttribLocations {
    pub position: u32,
    pub instance_translation: u32,
    pub instance_texture_coord_offset: u32,
    pub instance_texture_index: u32,
    //
    pub position2: u32,
    pub texture_position2: u32,
}

pub struct UniformLocations {
    pub view_projection: WebGlUniformLocation,
    pub sampler: WebGlUniformLocation,
    //
    pub view_projection2: WebGlUniformLocation,
    pub sampler2: WebGlUniformLocation,
}

pub struct Textures {
    pub tileset: Texture,
    pub charset: Texture,
}

pub struct Vaos {
    pub tile: WebGlVertexArrayObject,
    pub textured_vertex: WebGlVertexArrayObject,
}

pub struct Buffers {
    pub quad_vertex: WebGlBuffer,
    pub tile_attrib: WebGlBuffer,
    pub tile_attrib_data: Vec<TileAttribs>,
    pub textured_vertex: WebGlBuffer,
}

pub struct Timestamps {
    pub now_ms: f64,
    pub now: f32,
    pub frame_delta: f32,
}

#[repr(C)]
pub struct TileAttribs {
    pub world_position: Vector2<f32>,
    pub texture_position: Vector2<f32>,
    pub texture_index: u32,
}

#[repr(C)]
pub struct TexturedVertex {
    pub position: Vector2<f32>,
    pub texture_position: Vector2<f32>,
}
