use nalgebra::{Matrix3, Point2, Scale2, Translation2, Vector2};

pub static PIXELS_PER_TILE: u32 = 16;

static MAX_X_TILES_ON_SCREEN: f32 = 30.0;
static MAX_Y_TILES_ON_SCREEN: f32 = 16.875;

#[derive(Debug, Clone)]
pub struct Camera {
    pub world_to_ndc: Matrix3<f32>,
    pub world_to_logical_screen: Matrix3<f32>,
    pub logical_screen_to_ndc: Matrix3<f32>,
    pub logical_screen_size: Vector2<f32>,
    world_to_camera: Translation2<f32>,
    camera_to_screen: Scale2<f32>,
}

impl Camera {
    pub fn new(focus: Vector2<f32>, map_size: Vector2<u32>, screen_size: Vector2<u32>) -> Self {
        // FIXME: some very wide aspect ratios have a striping artifact

        let pixels_per_tile = PIXELS_PER_TILE as f32;
        let screen_size: Vector2<f32> = screen_size.cast();

        let aspect_ratio = screen_size[0] / screen_size[1];
        let pixels_per_world_unit = {
            let bounded = if aspect_ratio >= 16.0 / 9.0 {
                screen_size[0] / MAX_X_TILES_ON_SCREEN
            } else {
                screen_size[1] / MAX_Y_TILES_ON_SCREEN
            };
            (bounded / pixels_per_tile).ceil() * pixels_per_tile
        };
        let pixels_per_logical_pixel = pixels_per_world_unit / pixels_per_tile;

        let world_to_camera = {
            let focus = focus.map(|a| (a * pixels_per_world_unit).round() / pixels_per_world_unit);
            let map_size = map_size.cast();
            let world_viewport = screen_size / pixels_per_world_unit;
            Translation2::new(
                camera_translation(focus[0], world_viewport[0], map_size[0]),
                camera_translation(focus[1], world_viewport[1], map_size[1]),
            )
        };
        let camera_to_logical_screen = Scale2::new(pixels_per_tile, pixels_per_tile);
        let logical_screen_to_screen =
            Scale2::new(pixels_per_logical_pixel, pixels_per_logical_pixel);
        let camera_to_screen = logical_screen_to_screen * camera_to_logical_screen;

        let world_to_logical_screen =
            camera_to_logical_screen.to_homogeneous() * world_to_camera.to_homogeneous();

        let logical_screen_to_ndc =
            ortographic_2d(screen_size) * logical_screen_to_screen.to_homogeneous();
        let world_to_ndc = logical_screen_to_ndc * world_to_logical_screen;

        let logical_screen_size = screen_size / pixels_per_logical_pixel;

        Self {
            world_to_ndc,
            world_to_logical_screen,
            logical_screen_to_ndc,
            logical_screen_size,
            world_to_camera,
            camera_to_screen,
        }
    }

    pub fn world_point_to_screen(&self, p: Vector2<f32>) -> Vector2<f32> {
        self.world_to_logical_screen.transform_point(&Point2::from(p)).coords
    }

    pub fn screen_point_to_world(&self, p: Vector2<f32>) -> Vector2<f32> {
        self.world_to_camera
            .inverse_transform_point(
                &self.camera_to_screen.pseudo_inverse().transform_point(&Point2::from(p)),
            )
            .coords
    }
}

fn camera_translation(focus: f32, world_viewport: f32, map_size: f32) -> f32 {
    if map_size <= world_viewport {
        (world_viewport - map_size) / 2.0
    } else {
        let max_translate = map_size - world_viewport;
        (world_viewport / 2.0 - focus).min(0.0).max(-max_translate)
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
