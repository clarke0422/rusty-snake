use ggez::{
    glam::*, graphics::{self, Color, Rect}, Context, GameResult
};

use rand::Rng;
use crate::config as cn;
use crate::snake::Snake;

pub struct Fruit {
    pub pos_x: i32,
    pub pos_y: i32,
    pub mesh: graphics::Mesh,
}

impl Fruit {
    pub fn new(ctx: &mut Context) -> GameResult<Self> {
        let mesh = graphics::Mesh::new_rectangle(
            ctx,
            graphics::DrawMode::fill(),
            Rect::new(0., 0., cn::TILE_SIZE, cn::TILE_SIZE),
            Color::from_rgb(cn::FRUIT_COLOR.0, cn::FRUIT_COLOR.1, cn::FRUIT_COLOR.2),
        )?;
        
        let pos_x = rand::thread_rng().gen_range(0..=cn::GRID_SIZE-1);
        let pos_y = rand::thread_rng().gen_range(0..=cn::GRID_SIZE-1);
        Ok(Fruit{pos_x, pos_y, mesh})
    }

    pub fn reposition(self: &mut Self, snakes: (&Snake, &Snake)) {
        loop {
            self.pos_x = rand::thread_rng().gen_range(0..=cn::GRID_SIZE-1);
            self.pos_y = rand::thread_rng().gen_range(0..=cn::GRID_SIZE-1);
            if !snakes.0.check_collision(self.pos_x, self.pos_y) && !snakes.1.check_collision(self.pos_x, self.pos_y) {
                break;
            }
        }
    }
}