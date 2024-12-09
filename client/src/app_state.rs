use std::cell::RefCell;
use std::rc::Rc;

use web_sys::{WebGl2RenderingContext, WebGlProgram, WebGlUniformLocation};

use crate::app_event::AppEvent;
use crate::assets::Assets;
use crate::fps_counter::FpsCounter;
use crate::game_state::{GameState, PartialGameState};
use crate::vertex_buffer_renderer::VertexBufferRenderer;

pub struct AppState {
    pub gl: WebGl2RenderingContext,
    pub program: WebGlProgram,
    pub text_program: WebGlProgram,
    pub uniform_locations: UniformLocations,
    pub assets: Option<Assets>,
    pub vertex_buffer_renderer: VertexBufferRenderer,
    pub time: Timestamps,
    pub fps_counter: FpsCounter,
    pub events: Rc<RefCell<Vec<AppEvent>>>,
    pub game_state: Result<GameState, PartialGameState>,
}

pub struct UniformLocations {
    pub view_projection: WebGlUniformLocation,
    pub sampler: WebGlUniformLocation,
    pub text_view_projection: WebGlUniformLocation,
    pub text_sampler: WebGlUniformLocation,
    pub text_distance_range: WebGlUniformLocation,
}

pub struct Timestamps {
    pub now_ms: f64,
    pub now: f32,
    pub frame_delta: f32,
}
