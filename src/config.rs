use crate::Direction;
use std::f64::consts::PI;

pub const TILE_SIZE: f32 = 24.;
pub const MARGIN_RATIO: f32 = 16.;
pub const GRID_SIZE: i32 = 30;

pub const HOST_STARTING_POSITION: (i32, i32) = (15, 10);
pub const HOST_STARTING_DIRECTION: Direction = Direction::Right;
pub const HOST_COLOR_FN: fn(usize) -> (f32, f32, f32) = |n| -> (f32, f32, f32) {
    (
        ((f64::cos((n as f64 / 5.) + PI) + 1.) / 2.) as f32,
        ((f64::cos(n as f64 / 5.) + 1.) / 2.) as f32,
        1.,
    )
};

pub const GUEST_STARTING_POSITION: (i32, i32) = (15, 20);
pub const GUEST_STARTING_DIRECTION: Direction = Direction::Left;
pub const GUEST_COLOR_FN: fn(usize) -> (f32, f32, f32) = |n| -> (f32, f32, f32) {
    (
        ((f64::cos(n as f64 / 5.) + 1.) / 2.) as f32,
        1.,
        ((f64::cos((n as f64 / 5.) + PI) + 1.) / 4.) as f32,
    )
};

pub const FRUIT_COLOR: (u8, u8, u8) = (255, 20, 147);