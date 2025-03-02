use ggez::{
    glam::*, graphics::{self, Color, Rect}, Context, GameResult
};

use rand::{rngs::StdRng, Rng, SeedableRng};
use crate::config as cn;
use crate::snake::Snake;

pub struct Fruit {
    pub pos: (i32, i32),
    pub mesh: graphics::Mesh,
    rng: StdRng,
}

impl Fruit {
    pub fn new(ctx: &mut Context, seed: u64) -> GameResult<Self> {
        let mesh = graphics::Mesh::new_rectangle(
            ctx,
            graphics::DrawMode::fill(),
            Rect::new(0., 0., cn::TILE_SIZE, cn::TILE_SIZE),
            Color::from_rgb(cn::FRUIT_COLOR.0, cn::FRUIT_COLOR.1, cn::FRUIT_COLOR.2),
        )?;
        let mut rng = StdRng::seed_from_u64(seed);
        let pos = (rng.gen_range(0..=cn::GRID_SIZE-1), rng.gen_range(0..=cn::GRID_SIZE-1));
        Ok(Fruit{pos, mesh, rng})
    }

    pub fn reposition(self: &mut Self, snakes: (&Snake, &Snake)) {
        loop {
            self.pos = (self.rng.gen_range(0..=cn::GRID_SIZE-1), self.rng.gen_range(0..=cn::GRID_SIZE-1));
            if !snakes.0.check_collision(self.pos) && !snakes.1.check_collision(self.pos) {
                break;
            }
        }
    }
}