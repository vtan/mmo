use nalgebra::Vector2;

pub fn in_distance(v1: Vector2<f32>, v2: Vector2<f32>, distance: f32) -> bool {
    (v1 - v2).norm_squared() <= distance.powi(2)
}
