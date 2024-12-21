use nalgebra::Vector2;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub struct ObjectId(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub enum ObjectType {
    Player,
    Mob,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub enum Direction {
    Right,
    Down,
    Left,
    Up,
}

pub static ALL_DIRECTIONS: [Direction; 4] = [
    Direction::Right,
    Direction::Down,
    Direction::Left,
    Direction::Up,
];

impl Direction {
    pub fn to_vector(&self) -> Vector2<f32> {
        match *self {
            Direction::Right => Vector2::new(1., 0.),
            Direction::Down => Vector2::new(0., 1.),
            Direction::Left => Vector2::new(-1., 0.),
            Direction::Up => Vector2::new(0., -1.),
        }
    }

    pub fn from_vector(v: Vector2<f32>) -> Self {
        if v.x.abs() > v.y.abs() {
            if v.x > 0.0 {
                Direction::Right
            } else {
                Direction::Left
            }
        } else if v.y > 0.0 {
            Direction::Down
        } else {
            Direction::Up
        }
    }
}
