use ggez::{
    conf, event, glam::*, graphics::{self, Color, Rect, Text, TextLayout}, input::keyboard::{KeyCode, KeyInput}, Context, GameResult
};

use std::{collections::VecDeque, f64::consts::PI};
use rand::Rng;

const TILE_SIZE: f32 = 24.;
const MARGIN_RATIO: f32 = 16.;
const GRID_SIZE: i32 = 30;
const STARTING_POS_X: i32 = 0;
const STARTING_POS_Y: i32 = 10;
const STARTING_DIRECTION: Direction = Direction::Right;
const STARTING_COLOR: (f32, f32, f32) = (0., 1., 1.);
const FRUIT_COLOR: (u8, u8, u8) = (255, 20, 147);

#[derive (Clone, Copy)]
enum Direction {
    Up,
    Left,
    Down,
    Right,
}

struct Segment {
    pos_x: i32,
    pos_y: i32,
    facing: Direction,
    mesh: graphics::Mesh,
}

impl Segment {
    fn new(ctx: &mut Context, pos_x: i32, pos_y: i32, facing: Direction, color: (f32, f32, f32)) -> GameResult<Self> {
        let width;
        let height;
        
        match facing {
            Direction::Up | Direction::Down => {
                width = TILE_SIZE - (TILE_SIZE / MARGIN_RATIO);
                height = TILE_SIZE;
            }
            Direction::Left | Direction::Right => {
                width = TILE_SIZE;
                height = TILE_SIZE - (TILE_SIZE / MARGIN_RATIO)
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

struct Snake {
    segments: VecDeque<Segment>,
    head_pos_x: i32,
    head_pos_y: i32,
    head_facing: Direction,
    // head_color: (f32, f32, f32),
}

impl Snake {
    fn new(ctx: &mut Context, pos_x: i32, pos_y: i32, facing: Direction) -> GameResult<Self> {
        Ok(Snake {
            head_pos_x: pos_x,
            head_pos_y: pos_y,
            head_facing: STARTING_DIRECTION,
            segments: VecDeque::from([Segment::new(ctx, pos_x, pos_y, facing, STARTING_COLOR)?]),
        })
    }

    // Returning true represents game over due to collision or out of bounds
    fn push(self: &mut Self, ctx: &mut Context, input_direction: Direction) -> GameResult<bool> {
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
        if Snake::check_out_of_bounds(new_pos_x, new_pos_y) || self.check_collision(new_pos_x, new_pos_y) {
            return Ok(true);
        }

        // let color_difference = (GRID_SIZE as f32).powf(2.);
        let new_color_r = (f64::cos((self.segments.len() as f64 / 5.) + PI) + 1.) / 2.;
        let new_color_g = (f64::cos(self.segments.len() as f64 / 5.) + 1.) / 2.;

        let new_segment = Segment::new(ctx, new_pos_x, new_pos_y, input_direction, (new_color_r as f32, new_color_g as f32, STARTING_COLOR.2))?;
        self.segments.push_back(new_segment);

        // self.head_color = new_color;
        self.head_pos_x = new_pos_x;
        self.head_pos_y = new_pos_y;
        self.head_facing = input_direction;

        Ok(false)
    }

    fn pull(self: &mut Self) {
        self.segments.pop_front();
    }

    fn check_collision(self: &Self, pos_x: i32, pos_y: i32) -> bool {
        for segment in self.segments.iter() {
            if pos_x == segment.pos_x && pos_y == segment.pos_y {
                return true;
            }
        }
        false
    }

    fn check_out_of_bounds(pos_x: i32, pos_y: i32) -> bool {
        if pos_x < 0 || pos_x >= GRID_SIZE || pos_y < 0 || pos_y >= GRID_SIZE {
            return true;
        }
        false
    }

}

struct Fruit {
    pos_x: i32,
    pos_y: i32,
    mesh: graphics::Mesh,
}

impl Fruit {
    fn new(ctx: &mut Context) -> GameResult<Self> {
        let mesh = graphics::Mesh::new_rectangle(
            ctx,
            graphics::DrawMode::fill(),
            Rect::new(0., 0., TILE_SIZE, TILE_SIZE),
            Color::from_rgb(FRUIT_COLOR.0, FRUIT_COLOR.1, FRUIT_COLOR.2),
        )?;
        
        let pos_x = rand::thread_rng().gen_range(0..=GRID_SIZE-1);
        let pos_y = rand::thread_rng().gen_range(0..=GRID_SIZE-1);
        Ok(Fruit{pos_x, pos_y, mesh})
    }

    fn reposition(self: &mut Self, snake: &Snake) {
        loop {
            self.pos_x = rand::thread_rng().gen_range(0..=GRID_SIZE-1);
            self.pos_y = rand::thread_rng().gen_range(0..=GRID_SIZE-1);
            if !snake.check_collision(self.pos_x, self.pos_y) {
                break;
            }
        }
    }
}

enum GamePhase {
    Start,
    Play,
    Over,
}

struct MainState {
    phase: GamePhase,
    input_direction: Direction,
    snake: Snake,
    fruit: Fruit,
}

impl MainState {
    fn new(ctx: &mut Context) -> GameResult<MainState> {

        let snake = Snake::new(ctx, STARTING_POS_X, STARTING_POS_Y, STARTING_DIRECTION)?;

        Ok(
            MainState {
                phase: GamePhase::Start,
                input_direction: STARTING_DIRECTION,
                snake,
                fruit: Fruit::new(ctx)?,
            }
        )
    }
}

impl event::EventHandler<ggez::GameError> for MainState {
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        const TARGET_FPS: u32 = 6;

        while ctx.time.check_update_time(TARGET_FPS) {

            if let GamePhase::Play = self.phase {
                if self.snake.push(ctx, self.input_direction)? {
                    self.phase = GamePhase::Over;
                    self.input_direction = STARTING_DIRECTION;
                    self.snake = Snake::new(ctx, STARTING_POS_X, STARTING_POS_Y, STARTING_DIRECTION)?;
                    self.fruit = Fruit::new(ctx)?;
                } else if self.snake.head_pos_x == self.fruit.pos_x && self.snake.head_pos_y == self.fruit.pos_y {
                    self.fruit.reposition(&self.snake);
                } else {
                    self.snake.pull();
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
                    .set_bounds(Vec2::new(GRID_SIZE as f32 * TILE_SIZE, GRID_SIZE as f32 * TILE_SIZE))
                    .set_layout(TextLayout {
                        h_align: graphics::TextAlign::Middle,
                        v_align: graphics::TextAlign::Middle,
                    })
                    .set_font("Retro font");
                canvas.draw(&text, Vec2::new((GRID_SIZE as f32 / 2.) * TILE_SIZE, (GRID_SIZE as f32 / 2.) * TILE_SIZE))
            },
            GamePhase::Play => {
                for segment in self.snake.segments.iter() {
                    let x_offset;
                    let y_offset;
                    match segment.facing {
                        Direction::Up => {
                            x_offset = (TILE_SIZE / MARGIN_RATIO) / 2.;
                            y_offset = (TILE_SIZE / MARGIN_RATIO) / 2.;
                        }
                        Direction::Left => {
                            x_offset = (TILE_SIZE / MARGIN_RATIO) / 2.;
                            y_offset = (TILE_SIZE / MARGIN_RATIO) / 2.;
                        }
                        Direction::Down => {
                            x_offset = (TILE_SIZE / MARGIN_RATIO) / 2.;
                            y_offset = -(TILE_SIZE / MARGIN_RATIO) / 2.;
                        }
                        Direction::Right => {
                            x_offset = -(TILE_SIZE / MARGIN_RATIO) / 2.;
                            y_offset = (TILE_SIZE / MARGIN_RATIO) / 2.;
                        }
                    }
                    canvas.draw(&segment.mesh, Vec2::new((segment.pos_x as f32 * TILE_SIZE) + x_offset, (segment.pos_y as f32 * TILE_SIZE) + y_offset));
                }
                canvas.draw(&self.fruit.mesh, Vec2::new(self.fruit.pos_x as f32 * TILE_SIZE, self.fruit.pos_y as f32 * TILE_SIZE));
            }
            GamePhase::Over => {
                let mut text: Text = Text::new("GAME OVER");
                    text.add("\nPress SPACE to restart")
                    .add("\nPress ESC to exit")
                    .set_bounds(Vec2::new(GRID_SIZE as f32 * TILE_SIZE, GRID_SIZE as f32 * TILE_SIZE))
                    .set_layout(TextLayout {
                        h_align: graphics::TextAlign::Middle,
                        v_align: graphics::TextAlign::Middle,
                    })
                    .set_font("Retro font");
                canvas.draw(&text, Vec2::new((GRID_SIZE as f32 / 2.) * TILE_SIZE, (GRID_SIZE as f32 / 2.) * TILE_SIZE))
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
            Some(KeyCode::W) | Some(KeyCode::Up) => {
                if let Direction::Left | Direction::Right = self.snake.head_facing {
                    self.input_direction = Direction::Up;
                }
            }
            Some(KeyCode::A) | Some(KeyCode::Left) => {
                if let Direction::Up | Direction::Down = self.snake.head_facing {
                    self.input_direction = Direction::Left;
                }
            }
            Some(KeyCode::S) | Some(KeyCode::Down) => {
                if let Direction::Left | Direction::Right = self.snake.head_facing {
                    self.input_direction = Direction::Down;
                }
            }
            Some(KeyCode::D) | Some(KeyCode::Right) => {
                if let Direction::Up | Direction::Down = self.snake.head_facing {
                    self.input_direction = Direction::Right;
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
        .window_mode(conf::WindowMode::default().dimensions(GRID_SIZE as f32 * TILE_SIZE, GRID_SIZE as f32 * TILE_SIZE))
        .window_setup(conf::WindowSetup::default().title("Rusty Snake"));
    let (mut ctx, event_loop) = cb.build()?;
    ctx.gfx.add_font(
        "Retro font",
        graphics::FontData::from_path(&ctx, "\\nintendo-nes-font.ttf")?,
    );
    let state = MainState::new(&mut ctx)?;
    event::run(ctx, event_loop, state)
}