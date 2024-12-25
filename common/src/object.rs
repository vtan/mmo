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
pub enum Direction4 {
    Right,
    Down,
    Left,
    Up,
}

pub static ALL_DIRECTIONS_4: [Direction4; 4] = [
    Direction4::Right,
    Direction4::Down,
    Direction4::Left,
    Direction4::Up,
];

impl Direction4 {
    pub fn to_unit_vector(self) -> Vector2<f32> {
        match self {
            Direction4::Right => Vector2::new(1., 0.),
            Direction4::Down => Vector2::new(0., 1.),
            Direction4::Left => Vector2::new(-1., 0.),
            Direction4::Up => Vector2::new(0., -1.),
        }
    }

    pub fn from_vector(v: Vector2<f32>) -> Self {
        if v.x.abs() > v.y.abs() {
            if v.x > 0.0 {
                Direction4::Right
            } else {
                Direction4::Left
            }
        } else if v.y > 0.0 {
            Direction4::Down
        } else {
            Direction4::Up
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub enum Direction8 {
    Right,
    Down,
    Left,
    Up,
    RightDown,
    LeftDown,
    RightUp,
    LeftUp,
}

pub static ALL_DIRECTIONS_8: [Direction8; 8] = [
    Direction8::Right,
    Direction8::RightDown,
    Direction8::Down,
    Direction8::LeftDown,
    Direction8::Left,
    Direction8::LeftUp,
    Direction8::Up,
    Direction8::RightUp,
];

impl Direction8 {
    pub fn to_unit_vector(self) -> Vector2<f32> {
        let diag = 1.0 / 2.0_f32.sqrt();
        match self {
            Direction8::Right => Vector2::new(1., 0.),
            Direction8::Down => Vector2::new(0., 1.),
            Direction8::Left => Vector2::new(-1., 0.),
            Direction8::Up => Vector2::new(0., -1.),
            Direction8::RightDown => Vector2::new(diag, diag),
            Direction8::LeftDown => Vector2::new(-diag, diag),
            Direction8::RightUp => Vector2::new(diag, -diag),
            Direction8::LeftUp => Vector2::new(-diag, -diag),
        }
    }

    pub fn to_neighbor_vector(self) -> Vector2<f32> {
        match self {
            Direction8::Right => Vector2::new(1., 0.),
            Direction8::Down => Vector2::new(0., 1.),
            Direction8::Left => Vector2::new(-1., 0.),
            Direction8::Up => Vector2::new(0., -1.),
            Direction8::RightDown => Vector2::new(1., 1.),
            Direction8::LeftDown => Vector2::new(-1., 1.),
            Direction8::RightUp => Vector2::new(1., -1.),
            Direction8::LeftUp => Vector2::new(-1., -1.),
        }
    }

    pub fn from_vector(v: Vector2<f32>) -> Self {
        let slope = (45.0_f32 + 22.5).to_radians().tan();
        #[allow(clippy::collapsible_else_if)]
        if v.y > v.x * slope {
            if v.y > -v.x * slope {
                Direction8::Down
            } else if v.y * slope > -v.x {
                Direction8::LeftDown
            } else if v.y * slope > v.x {
                Direction8::Left
            } else {
                Direction8::LeftUp
            }
        } else {
            if v.y * slope > v.x {
                Direction8::RightDown
            } else if v.y * slope > -v.x {
                Direction8::Right
            } else if v.y > -v.x * slope {
                Direction8::RightUp
            } else {
                Direction8::Up
            }
        }
    }

    pub fn to_direction4(self) -> Direction4 {
        match self {
            Direction8::Right => Direction4::Right,
            Direction8::Down => Direction4::Down,
            Direction8::Left => Direction4::Left,
            Direction8::Up => Direction4::Up,
            Direction8::RightDown => Direction4::Right,
            Direction8::RightUp => Direction4::Right,
            Direction8::LeftDown => Direction4::Left,
            Direction8::LeftUp => Direction4::Left,
        }
    }
}
