use std::cell::RefCell;
use std::rc::Rc;

use nalgebra::Vector2;
use web_sys::{WebGl2RenderingContext, WebGlProgram, WebGlUniformLocation};

use crate::app_event::AppEvent;
use crate::assets::Assets;
use crate::game_state::{GameState, PartialGameState};
use crate::metrics::Metrics;
use crate::vertex_buffer_renderer::VertexBufferRenderer;

pub struct AppState {
    pub client_git_sha: &'static str,
    pub gl: WebGl2RenderingContext,
    pub program: WebGlProgram,
    pub text_program: WebGlProgram,
    pub uniform_locations: UniformLocations,
    pub assets: Option<Assets>,
    pub vertex_buffer_renderer: VertexBufferRenderer,
    pub metrics: Rc<RefCell<Metrics>>,
    pub viewport: Vector2<u32>,
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
