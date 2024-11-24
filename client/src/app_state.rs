use web_sys::{WebGl2RenderingContext, WebGlProgram, WebGlUniformLocation};

use crate::font_atlas::FontAtlas;
use crate::game_state::{GameState, PartialGameState};
use crate::texture::Texture;
use crate::vertex_buffer_renderer::VertexBufferRenderer;

pub struct AppState {
    pub gl: WebGl2RenderingContext,
    pub program: WebGlProgram,
    pub text_program: WebGlProgram,
    pub uniform_locations: UniformLocations,
    pub textures: Textures,
    pub font_atlas: FontAtlas,
    pub vertex_buffer_renderer: VertexBufferRenderer,
    pub time: Timestamps,
    pub game_state: Result<GameState, PartialGameState>,
}

pub struct UniformLocations {
    pub view_projection: WebGlUniformLocation,
    pub sampler: WebGlUniformLocation,
    pub text_view_projection: WebGlUniformLocation,
    pub text_sampler: WebGlUniformLocation,
}

pub struct Textures {
    pub tileset: Texture,
    pub charset: Texture,
    pub font: Texture,
    pub white: Texture,
}

pub struct Timestamps {
    pub now_ms: f64,
    pub now: f32,
    pub frame_delta: f32,
}
