use mmo_common::MoveCommand;
use nalgebra::Vector2;
use web_sys::{
    WebGl2RenderingContext, WebGlBuffer, WebGlProgram, WebGlTexture, WebGlUniformLocation,
    WebGlVertexArrayObject,
};

pub struct AppState {
    pub gl: WebGl2RenderingContext,
    pub program: WebGlProgram,
    pub attrib_locations: AttribLocations,
    pub uniform_locations: UniformLocations,
    pub textures: Textures,
    pub vaos: Vaos,
    pub buffers: Buffers,
    pub ticks: u64,
    pub connection: Option<Box<dyn Fn(MoveCommand)>>,
    pub player_position: Vector2<f32>,
}

pub struct AttribLocations {
    pub position: u32,
    pub instance_translation: u32,
    pub instance_texture_coord_offset: u32,
}

pub struct UniformLocations {
    pub view_projection: WebGlUniformLocation,
    pub sampler: WebGlUniformLocation,
}

pub struct Textures {
    pub tileset: WebGlTexture,
}

pub struct Vaos {
    pub tile: WebGlVertexArrayObject,
}

pub struct Buffers {
    pub quad_vertex: WebGlBuffer,
    pub tile_attrib: WebGlBuffer,
    pub tile_attrib_data: Vec<f32>,
}