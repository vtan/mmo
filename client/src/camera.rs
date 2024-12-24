use nalgebra::{Matrix3, Point2, Scale2, Translation2, Vector2};

pub static PIXELS_PER_TILE: u32 = 16;

#[derive(Debug, Clone)]
pub struct Camera {
    pub world_to_ndc: Matrix3<f32>,
    pub world_to_screen: Matrix3<f32>,
    pub screen_to_ndc: Matrix3<f32>,
}

impl Camera {
    pub fn new(focus: Vector2<f32>, map_size: Vector2<u32>) -> Self {
        let screen_w = 480.0;
        let screen_h = 270.0;

        let world_to_camera = {
            let map_size = map_size.cast();
            let world_viewport = Vector2::new(screen_w, screen_h) / (PIXELS_PER_TILE as f32);
            Translation2::new(
                Self::camera_translation(focus[0], world_viewport[0], map_size[0]),
                Self::camera_translation(focus[1], world_viewport[1], map_size[1]),
            )
        };
        let camera_to_screen = Scale2::new(PIXELS_PER_TILE as f32, PIXELS_PER_TILE as f32);
        let world_to_screen = camera_to_screen.to_homogeneous() * world_to_camera.to_homogeneous();

        let screen_to_ndc = ortographic_2d(Vector2::new(screen_w, screen_h));
        let world_to_ndc = screen_to_ndc * world_to_screen;

        Self { world_to_ndc, world_to_screen, screen_to_ndc }
    }

    pub fn world_point_to_screen(&self, p: Vector2<f32>) -> Vector2<f32> {
        self.world_to_screen.transform_point(&Point2::from(p)).coords
    }

    fn camera_translation(focus: f32, world_viewport: f32, map_size: f32) -> f32 {
        if map_size <= world_viewport {
            (world_viewport - map_size) / 2.0
        } else {
            let max_translate = map_size - world_viewport;
            (world_viewport / 2.0 - focus).min(0.0).max(-max_translate)
        }
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
