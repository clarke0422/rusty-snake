use ggez::{
    conf, event, glam::*, graphics::{self, Text, TextLayout}, input::keyboard::{KeyCode, KeyInput}, Context, GameResult
};

mod config;
use config as cn;

mod snake;
use snake::Snake;

mod fruit;
use fruit::Fruit;

#[derive (Clone, Copy)]
pub enum Direction {
    Up,
    Left,
    Down,
    Right,
}

enum GamePhase {
    Start,
    Play,
    Over,
}

struct PlayerState {
    input_direction: Direction,
    snake: Snake,
}

struct MainState {
    phase: GamePhase,
    fruit: Fruit,
    host_player: PlayerState,
    guest_player: PlayerState,
}

impl MainState {
    fn new(ctx: &mut Context) -> GameResult<MainState> {

        let host_snake = Snake::new(ctx, cn::HOST_STARTING_POSITION, cn::HOST_STARTING_DIRECTION, cn::HOST_COLOR_FN)?;
        let guest_snake = Snake::new(ctx, cn::GUEST_STARTING_POSITION, cn::GUEST_STARTING_DIRECTION, cn::GUEST_COLOR_FN)?;

        let host_player = PlayerState {
            input_direction: cn::HOST_STARTING_DIRECTION,
            snake: host_snake
        };
        let guest_player = PlayerState {
            input_direction: cn::GUEST_STARTING_DIRECTION,
            snake: guest_snake
        };


        Ok(
            MainState {
                phase: GamePhase::Start,
                fruit: Fruit::new(ctx)?,
                host_player,
                guest_player,
            }
        )
    }
}

impl event::EventHandler<ggez::GameError> for MainState {
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        const TARGET_FPS: u32 = 6;

        while ctx.time.check_update_time(TARGET_FPS) {

            if let GamePhase::Play = self.phase {
                self.host_player.snake.push(ctx, self.host_player.input_direction)?;
                self.guest_player.snake.push(ctx, self.guest_player.input_direction)?;

                if !(self.host_player.snake.head_pos == self.fruit.pos){
                    self.host_player.snake.pull();
                }
                if !(self.guest_player.snake.head_pos == self.fruit.pos){
                    self.guest_player.snake.pull();
                }

                if (self.host_player.snake.head_pos == self.fruit.pos)
                || (self.guest_player.snake.head_pos == self.fruit.pos) {
                    self.fruit.reposition((&self.host_player.snake, &self.guest_player.snake));
                }

                if Snake::check_out_of_bounds(self.host_player.snake.head_pos)
                || Snake::check_out_of_bounds(self.guest_player.snake.head_pos)
                || self.host_player.snake.check_self_collision()
                || self.host_player.snake.check_collision(self.guest_player.snake.head_pos)
                || self.guest_player.snake.check_self_collision()
                || self.guest_player.snake.check_collision(self.host_player.snake.head_pos) {
                    self.phase = GamePhase::Over;

                    self.host_player.input_direction = cn::HOST_STARTING_DIRECTION;
                    self.host_player.snake = Snake::new(ctx, cn::HOST_STARTING_POSITION, cn::HOST_STARTING_DIRECTION, cn::HOST_COLOR_FN)?;
                    
                    self.guest_player.input_direction = cn::GUEST_STARTING_DIRECTION;
                    self.guest_player.snake = Snake::new(ctx, cn::GUEST_STARTING_POSITION, cn::GUEST_STARTING_DIRECTION, cn::GUEST_COLOR_FN)?;

                    self.fruit = Fruit::new(ctx)?;
                }
            }
        }

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        let mut canvas =
            graphics::Canvas::from_frame(ctx, graphics::Color::BLACK);

        match self.phase {
            GamePhase::Start => {
                let mut text: Text = Text::new("RUSTY SNAKE");
                    text.add("\nPress SPACE to start")
                    .add("\nPress ESC to exit")
                    .set_bounds(Vec2::new(cn::GRID_SIZE as f32 * cn::TILE_SIZE, cn::GRID_SIZE as f32 * cn::TILE_SIZE))
                    .set_layout(TextLayout {
                        h_align: graphics::TextAlign::Middle,
                        v_align: graphics::TextAlign::Middle,
                    })
                    .set_font("Retro font");
                canvas.draw(&text, Vec2::new((cn::GRID_SIZE as f32 / 2.) * cn::TILE_SIZE, (cn::GRID_SIZE as f32 / 2.) * cn::TILE_SIZE))
            },
            GamePhase::Play => {
                for segment in self.host_player.snake.segments.iter().chain(self.guest_player.snake.segments.iter()) {
                    let x_offset;
                    let y_offset;
                    match segment.facing {
                        Direction::Up => {
                            x_offset = (cn::TILE_SIZE / cn::MARGIN_RATIO) / 2.;
                            y_offset = (cn::TILE_SIZE / cn::MARGIN_RATIO) / 2.;
                        }
                        Direction::Left => {
                            x_offset = (cn::TILE_SIZE / cn::MARGIN_RATIO) / 2.;
                            y_offset = (cn::TILE_SIZE / cn::MARGIN_RATIO) / 2.;
                        }
                        Direction::Down => {
                            x_offset = (cn::TILE_SIZE / cn::MARGIN_RATIO) / 2.;
                            y_offset = -(cn::TILE_SIZE / cn::MARGIN_RATIO) / 2.;
                        }
                        Direction::Right => {
                            x_offset = -(cn::TILE_SIZE / cn::MARGIN_RATIO) / 2.;
                            y_offset = (cn::TILE_SIZE / cn::MARGIN_RATIO) / 2.;
                        }
                    }
                    canvas.draw(&segment.mesh, Vec2::new((segment.pos.0 as f32 * cn::TILE_SIZE) + x_offset, (segment.pos.1 as f32 * cn::TILE_SIZE) + y_offset));
                }
                canvas.draw(&self.fruit.mesh, Vec2::new(self.fruit.pos.0 as f32 * cn::TILE_SIZE, self.fruit.pos.1 as f32 * cn::TILE_SIZE));
            }
            GamePhase::Over => {
                let mut text: Text = Text::new("GAME OVER");
                    text.add("\nPress SPACE to restart")
                    .add("\nPress ESC to exit")
                    .set_bounds(Vec2::new(cn::GRID_SIZE as f32 * cn::TILE_SIZE, cn::GRID_SIZE as f32 * cn::TILE_SIZE))
                    .set_layout(TextLayout {
                        h_align: graphics::TextAlign::Middle,
                        v_align: graphics::TextAlign::Middle,
                    })
                    .set_font("Retro font");
                canvas.draw(&text, Vec2::new((cn::GRID_SIZE as f32 / 2.) * cn::TILE_SIZE, (cn::GRID_SIZE as f32 / 2.) * cn::TILE_SIZE))
            },
        }

        canvas.finish(ctx)?;

        Ok(())
    }

    fn key_down_event(
        &mut self,
        ctx: &mut Context,
        input: KeyInput,
        _repeated: bool,
    ) -> GameResult {
        match input.keycode {
            Some(KeyCode::W) => {
                if let Direction::Left | Direction::Right = self.host_player.snake.head_facing {
                    self.host_player.input_direction = Direction::Up;
                }
            }
            Some(KeyCode::A) => {
                if let Direction::Up | Direction::Down = self.host_player.snake.head_facing {
                    self.host_player.input_direction = Direction::Left;
                }
            }
            Some(KeyCode::S) => {
                if let Direction::Left | Direction::Right = self.host_player.snake.head_facing {
                    self.host_player.input_direction = Direction::Down;
                }
            }
            Some(KeyCode::D) => {
                if let Direction::Up | Direction::Down = self.host_player.snake.head_facing {
                    self.host_player.input_direction = Direction::Right;
                }
            }
            Some(KeyCode::Up) => {
                if let Direction::Left | Direction::Right = self.guest_player.snake.head_facing {
                    self.guest_player.input_direction = Direction::Up;
                }
            }
            Some(KeyCode::Left) => {
                if let Direction::Up | Direction::Down = self.guest_player.snake.head_facing {
                    self.guest_player.input_direction = Direction::Left;
                }
            }
            Some(KeyCode::Down) => {
                if let Direction::Left | Direction::Right = self.guest_player.snake.head_facing {
                    self.guest_player.input_direction = Direction::Down;
                }
            }
            Some(KeyCode::Right) => {
                if let Direction::Up | Direction::Down = self.guest_player.snake.head_facing {
                    self.guest_player.input_direction = Direction::Right;
                }
            }
            Some(KeyCode::Space) => {
                if let GamePhase::Start | GamePhase::Over = self.phase {
                    self.phase = GamePhase::Play;
                }
            }
            Some(KeyCode::Escape) => ctx.request_quit(),
            _ => (), // Do nothing
        }
        Ok(())
    }
}

pub fn main() -> GameResult {
    let cb = ggez::ContextBuilder::new("snake", "Clarke Kennedy")
        .window_mode(conf::WindowMode::default().dimensions(cn::GRID_SIZE as f32 * cn::TILE_SIZE, cn::GRID_SIZE as f32 * cn::TILE_SIZE))
        .window_setup(conf::WindowSetup::default().title("Rusty Snake"));
    let (mut ctx, event_loop) = cb.build()?;
    ctx.gfx.add_font(
        "Retro font",
        graphics::FontData::from_path(&ctx, "\\nintendo-nes-font.ttf")?,
    );
    let state = MainState::new(&mut ctx)?;
    event::run(ctx, event_loop, state)
}