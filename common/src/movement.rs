use nalgebra::Vector2;
use serde::{Deserialize, Serialize};

#[derive(Debug, Copy, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub enum Direction {
    Right,
    Down,
    Left,
    Up,
}

impl Direction {
    pub fn to_vector(&self) -> Vector2<f32> {
        match *self {
            Direction::Right => Vector2::new(1., 0.),
            Direction::Down => Vector2::new(0., 1.),
            Direction::Left => Vector2::new(-1., 0.),
            Direction::Up => Vector2::new(0., -1.),
        }
    }
}
