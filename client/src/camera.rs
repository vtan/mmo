use nalgebra::{Matrix3, Scale2, Vector2};

pub static PIXELS_PER_TILE: u32 = 16;
pub static TILES_PER_PIXEL: f32 = 1.0 / PIXELS_PER_TILE as f32;

#[derive(Debug, Clone)]
pub struct Camera {
    pub from_world: Matrix3<f32>,
    pub from_screen: Matrix3<f32>,
}

impl Camera {
    pub fn new(focus: Vector2<f32>, map_size: Vector2<u32>) -> Self {
        let screen_w = 480.0;
        let screen_h = 270.0;

        let from_screen = ortographic_2d(Vector2::new(screen_w, screen_h));

        let world_to_screen =
            Scale2::new(PIXELS_PER_TILE as f32, PIXELS_PER_TILE as f32).to_homogeneous();
        let from_world = from_screen * world_to_screen;

        Self { from_world, from_screen }
    }

    pub fn px_to_world(&self, px: f32) -> f32 {
        px * TILES_PER_PIXEL
    }

    pub fn world_to_px(&self, world: f32) -> f32 {
        world * PIXELS_PER_TILE as f32
    }
}

fn ortographic_2d(max: Vector2<f32>) -> Matrix3<f32> {
    Matrix3::from_vec(vec![
        2.0 / max[0],
        0.0,
        0.0,
        //
        0.0,
        -2.0 / max[1],
        0.0,
        //
        -1.0,
        1.0,
        1.0,
    ])
}
