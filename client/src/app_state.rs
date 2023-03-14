use std::collections::HashMap;

use mmo_common::PlayerCommand;
use nalgebra::Vector2;
use web_sys::{
    WebGl2RenderingContext, WebGlBuffer, WebGlProgram, WebGlUniformLocation, WebGlVertexArrayObject,
};

use crate::texture::Texture;

pub struct AppState {
    pub gl: WebGl2RenderingContext,
    pub program: WebGlProgram,
    pub attrib_locations: AttribLocations,
    pub uniform_locations: UniformLocations,
    pub textures: Textures,
    pub vaos: Vaos,
    pub buffers: Buffers,
    pub ticks: u64,
    pub connection: Option<Box<dyn Fn(PlayerCommand)>>,
    pub player_position: Vector2<f32>,
    pub other_positions: HashMap<u64, Vector2<f32>>,
}

pub struct AttribLocations {
    pub position: u32,
    pub instance_translation: u32,
    pub instance_texture_coord_offset: u32,
    pub instance_texture_index: u32,
}

pub struct UniformLocations {
    pub view_projection: WebGlUniformLocation,
    pub sampler: WebGlUniformLocation,
}

pub struct Textures {
    pub tileset: Texture,
    pub charset: Texture,
}

pub struct Vaos {
    pub tile: WebGlVertexArrayObject,
}

pub struct Buffers {
    pub quad_vertex: WebGlBuffer,
    pub tile_attrib: WebGlBuffer,
    pub tile_attrib_data: Vec<TileAttribs>,
}

#[repr(C)]
pub struct TileAttribs {
    pub world_position: Vector2<f32>,
    pub texture_position: Vector2<f32>,
    pub texture_index: u32,
}
