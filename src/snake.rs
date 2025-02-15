use std::collections::VecDeque;
use ggez::{
    glam::*, graphics::{self, Color, Rect}, Context, GameResult
};
use crate::Direction;
use crate::config as cn;

pub struct Segment {
    pub pos: (i32, i32),
    pub facing: Direction,
    pub mesh: graphics::Mesh,
}

impl Segment {
    fn new(ctx: &mut Context, pos: (i32, i32), facing: Direction, color: (f32, f32, f32)) -> GameResult<Self> {
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
        
        Ok(Segment{pos, facing, mesh})
    }
}

pub struct Snake {
    pub segments: VecDeque<Segment>,
    pub head_pos: (i32, i32),
    pub head_facing: Direction,
    color_fn: fn(usize) -> (f32, f32, f32),
}

impl Snake {
    pub fn new(ctx: &mut Context, head_pos: (i32, i32), head_facing: Direction, color_fn: fn(usize) -> (f32, f32, f32)) -> GameResult<Self> {
        Ok(Snake {
            head_pos,
            head_facing,
            color_fn,
            segments: VecDeque::from([Segment::new(ctx, head_pos, head_facing, color_fn(0))?]),
        })
    }

    pub fn push(self: &mut Self, ctx: &mut Context, input_direction: Direction) -> GameResult<> {
        let pos_delta;
        match input_direction {
            Direction::Up => { pos_delta = (0, -1); }
            Direction::Left => { pos_delta = (-1, 0); }
            Direction::Down => { pos_delta = (0, 1); }
            Direction::Right => { pos_delta = (1, 0); }
        }
        let new_pos = (self.head_pos.0 + pos_delta.0, self.head_pos.1 + pos_delta.1);
        
        let new_segment = Segment::new(ctx, new_pos, input_direction, (self.color_fn)(self.segments.len()))?;
        self.segments.push_back(new_segment);

        self.head_pos = new_pos;
        self.head_facing = input_direction;

        Ok(())
    }

    pub fn pull(self: &mut Self) {
        self.segments.pop_front();
    }

    pub fn check_collision(self: &Self, pos: (i32, i32)) -> bool {
        for segment in self.segments.iter() {
            if pos == segment.pos {
                return true;
            }
        }
        false
    }

    pub fn check_self_collision(self: &Self) -> bool {
        let mut segments_iter = self.segments.iter();
        segments_iter.next_back();

        for segment in segments_iter {
            if self.head_pos == segment.pos {
                return true;
            }
        }
        false
    }

    pub fn check_out_of_bounds(pos: (i32, i32)) -> bool {
        if pos.0 < 0 || pos.0 >= cn::GRID_SIZE || pos.1 < 0 || pos.1 >= cn::GRID_SIZE {
            return true;
        }
        false
    }

}