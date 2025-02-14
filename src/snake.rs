use std::collections::VecDeque;
use ggez::{
    glam::*, graphics::{self, Color, Rect}, Context, GameResult
};
use crate::Direction;
use crate::config as cn;

pub struct Segment {
    pub pos_x: i32,
    pub pos_y: i32,
    pub facing: Direction,
    pub mesh: graphics::Mesh,
}

impl Segment {
    fn new(ctx: &mut Context, pos_x: i32, pos_y: i32, facing: Direction, color: (f32, f32, f32)) -> GameResult<Self> {
        let width;
        let height;
        
        match facing {
            Direction::Up | Direction::Down => {
                width = cn::TILE_SIZE - (cn::TILE_SIZE / cn::MARGIN_RATIO);
                height = cn::TILE_SIZE;
            }
            Direction::Left | Direction::Right => {
                width = cn::TILE_SIZE;
                height = cn::TILE_SIZE - (cn::TILE_SIZE / cn::MARGIN_RATIO)
            }
        }

        let mesh = graphics::Mesh::new_rectangle(
            ctx,
            graphics::DrawMode::fill(),
            Rect::new(0., 0., width, height),
            Color::new(color.0, color.1, color.2, 1.),
        )?;
        
        Ok(Segment{pos_x, pos_y, facing, mesh})
    }
}

pub struct Snake {
    pub segments: VecDeque<Segment>,
    pub head_pos_x: i32,
    pub head_pos_y: i32,
    pub head_facing: Direction,
    color_fn: fn(usize) -> (f32, f32, f32),
}

impl Snake {
    pub fn new(ctx: &mut Context, head_pos_x: i32, head_pos_y: i32, head_facing: Direction, color_fn: fn(usize) -> (f32, f32, f32)) -> GameResult<Self> {
        Ok(Snake {
            head_pos_x,
            head_pos_y,
            head_facing,
            color_fn,
            segments: VecDeque::from([Segment::new(ctx, head_pos_x, head_pos_y, head_facing, color_fn(0))?]),
        })
    }

    pub fn push(self: &mut Self, ctx: &mut Context, input_direction: Direction) -> GameResult<> {
        let new_pos_x;
        let new_pos_y;
        match input_direction {
            Direction::Up => {
                new_pos_x = self.head_pos_x;
                new_pos_y = self.head_pos_y - 1;
            }
            Direction::Left => {
                new_pos_x = self.head_pos_x - 1;
                new_pos_y = self.head_pos_y;
            }
            Direction::Down => {
                new_pos_x = self.head_pos_x;
                new_pos_y = self.head_pos_y + 1;
            }
            Direction::Right => {
                new_pos_x = self.head_pos_x + 1;
                new_pos_y = self.head_pos_y;
            }
        }
        
        let new_segment = Segment::new(ctx, new_pos_x, new_pos_y, input_direction, (self.color_fn)(self.segments.len()))?;
        self.segments.push_back(new_segment);

        self.head_pos_x = new_pos_x;
        self.head_pos_y = new_pos_y;
        self.head_facing = input_direction;

        Ok(())
    }

    pub fn pull(self: &mut Self) {
        self.segments.pop_front();
    }

    pub fn check_collision(self: &Self, pos_x: i32, pos_y: i32) -> bool {
        for segment in self.segments.iter() {
            if pos_x == segment.pos_x && pos_y == segment.pos_y {
                return true;
            }
        }
        false
    }

    pub fn check_self_collision(self: &Self) -> bool {
        let mut segments_iter = self.segments.iter();
        segments_iter.next_back();

        for segment in segments_iter {
            if self.head_pos_x == segment.pos_x && self.head_pos_y == segment.pos_y {
                return true;
            }
        }
        false
    }

    pub fn check_out_of_bounds(pos_x: i32, pos_y: i32) -> bool {
        if pos_x < 0 || pos_x >= cn::GRID_SIZE || pos_y < 0 || pos_y >= cn::GRID_SIZE {
            return true;
        }
        false
    }

}