use ggez::{
    conf, event, glam::*, graphics::{self, Text, TextLayout}, input::keyboard::{KeyCode, KeyInput}, winit::event_loop::DeviceEventFilter, Context, GameError, GameResult
};
use std::{env, io::{Read, Write}, net::{TcpListener, TcpStream}, time::{SystemTime, UNIX_EPOCH}};

mod config;
use config as cn;

mod snake;
use snake::Snake;

mod fruit;
use fruit::Fruit;

#[repr(u8)]
#[derive (Clone, Copy, Debug)]
pub enum Direction {
    Up,
    Left,
    Down,
    Right,
}

impl TryFrom<u8> for Direction {
    type Error = GameError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Direction::Up),
            1 => Ok(Direction::Left),
            2 => Ok(Direction::Down),
            3 => Ok(Direction::Right),
            _ => Err(GameError::CustomError(String::from("Invalid input token received"))),
        }
    }
}

impl From<Direction> for u8 {
    fn from(direction: Direction) -> Self {
        direction as u8
    }
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
    my_player: PlayerState,
    opponent_player: PlayerState,
    stream: TcpStream,
}

impl MainState {
    fn new(ctx: &mut Context, stream: TcpStream, is_server: bool, seed: u64) -> GameResult<MainState> {

        let my_snake;
        let opponent_snake;
        let my_player;
        let opponent_player;
        if is_server {
            my_snake = Snake::new(ctx, cn::MY_STARTING_POSITION, cn::MY_STARTING_DIRECTION, cn::MY_COLOR_FN)?;
            opponent_snake = Snake::new(ctx, cn::OPPONENT_STARTING_POSITION, cn::OPPONENT_STARTING_DIRECTION, cn::OPPONENT_COLOR_FN)?;

            my_player = PlayerState {
                input_direction: cn::MY_STARTING_DIRECTION,
                snake: my_snake
            };
            opponent_player = PlayerState {
                input_direction: cn::OPPONENT_STARTING_DIRECTION,
                snake: opponent_snake
            };
        } else {
            my_snake = Snake::new(ctx, cn::OPPONENT_STARTING_POSITION, cn::OPPONENT_STARTING_DIRECTION, cn::OPPONENT_COLOR_FN)?;
            opponent_snake = Snake::new(ctx, cn::MY_STARTING_POSITION, cn::MY_STARTING_DIRECTION, cn::MY_COLOR_FN)?;

            my_player = PlayerState {
                input_direction: cn::OPPONENT_STARTING_DIRECTION,
                snake: my_snake
            };
            opponent_player = PlayerState {
                input_direction: cn::MY_STARTING_DIRECTION,
                snake: opponent_snake
            };
        }

        Ok(
            MainState {
                phase: GamePhase::Start,
                fruit: Fruit::new(ctx, seed)?,
                my_player,
                opponent_player,
                stream,
            }
        )
    }

    fn synchronize(&mut self) -> GameResult {
        let mut buf = vec![0];
        buf[0] = self.my_player.input_direction as u8;
        self.stream.write(&buf)?;
        println!("Input sent");
        println!("{:?}", self.my_player.input_direction);

        println!("Waiting...");
        self.stream.read(&mut buf)?;
        println!("Input received");
        let received_direction = Direction::try_from(buf[0])?;
        println!("{received_direction:?}");
        self.opponent_player.input_direction = received_direction;
        Ok(())
    }
}

impl event::EventHandler<ggez::GameError> for MainState {
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        const TARGET_FPS: u32 = 6;

        while ctx.time.check_update_time(TARGET_FPS) {

            if let GamePhase::Play = self.phase {
                self.synchronize()?;

                self.my_player.snake.push(ctx, self.my_player.input_direction)?;
                self.opponent_player.snake.push(ctx, self.opponent_player.input_direction)?;

                if !(self.my_player.snake.head_pos == self.fruit.pos){
                    self.my_player.snake.pull();
                }
                if !(self.opponent_player.snake.head_pos == self.fruit.pos){
                    self.opponent_player.snake.pull();
                }

                if (self.my_player.snake.head_pos == self.fruit.pos)
                || (self.opponent_player.snake.head_pos == self.fruit.pos) {
                    self.fruit.reposition((&self.my_player.snake, &self.opponent_player.snake));
                }

                if Snake::check_out_of_bounds(self.my_player.snake.head_pos)
                || Snake::check_out_of_bounds(self.opponent_player.snake.head_pos)
                || self.my_player.snake.check_self_collision()
                || self.my_player.snake.check_collision(self.opponent_player.snake.head_pos)
                || self.opponent_player.snake.check_self_collision()
                || self.opponent_player.snake.check_collision(self.my_player.snake.head_pos) {
                    self.phase = GamePhase::Over;

                    self.my_player.input_direction = cn::MY_STARTING_DIRECTION;
                    self.my_player.snake = Snake::new(ctx, cn::MY_STARTING_POSITION, cn::MY_STARTING_DIRECTION, cn::MY_COLOR_FN)?;
                    
                    self.opponent_player.input_direction = cn::OPPONENT_STARTING_DIRECTION;
                    self.opponent_player.snake = Snake::new(ctx, cn::OPPONENT_STARTING_POSITION, cn::OPPONENT_STARTING_DIRECTION, cn::OPPONENT_COLOR_FN)?;

                    self.fruit.reposition((&self.my_player.snake, &self.opponent_player.snake));
                }
            }
        }

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        println!("Draw");
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
                for segment in self.my_player.snake.segments.iter().chain(self.opponent_player.snake.segments.iter()) {
                    let mut x_offset = (cn::TILE_SIZE / cn::MARGIN_RATIO) / 2.;
                    let mut y_offset = (cn::TILE_SIZE / cn::MARGIN_RATIO) / 2.;
                    match segment.facing {
                        Direction::Down => {
                            y_offset *= -1.;
                        }
                        Direction::Right => {
                            x_offset *= -1.;
                        }
                        _ => {}
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
                if let Direction::Left | Direction::Right = self.my_player.snake.head_facing {
                    self.my_player.input_direction = Direction::Up;
                }
            }
            Some(KeyCode::A) => {
                if let Direction::Up | Direction::Down = self.my_player.snake.head_facing {
                    self.my_player.input_direction = Direction::Left;
                }
            }
            Some(KeyCode::S) => {
                if let Direction::Left | Direction::Right = self.my_player.snake.head_facing {
                    self.my_player.input_direction = Direction::Down;
                }
            }
            Some(KeyCode::D) => {
                if let Direction::Up | Direction::Down = self.my_player.snake.head_facing {
                    self.my_player.input_direction = Direction::Right;
                }
            }
            // Some(KeyCode::Up) => {
            //     if let Direction::Left | Direction::Right = self.opponent_player.snake.head_facing {
            //         self.opponent_player.input_direction = Direction::Up;
            //     }
            // }
            // Some(KeyCode::Left) => {
            //     if let Direction::Up | Direction::Down = self.opponent_player.snake.head_facing {
            //         self.opponent_player.input_direction = Direction::Left;
            //     }
            // }
            // Some(KeyCode::Down) => {
            //     if let Direction::Left | Direction::Right = self.opponent_player.snake.head_facing {
            //         self.opponent_player.input_direction = Direction::Down;
            //     }
            // }
            // Some(KeyCode::Right) => {
            //     if let Direction::Up | Direction::Down = self.opponent_player.snake.head_facing {
            //         self.opponent_player.input_direction = Direction::Right;
            //     }
            // }
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
    let args: Vec<String> = env::args().collect();
    let mut stream;
    let mut window_title = String::from("Rusty Snake");
    let is_server;
    let seed;
    match args[1].as_str() {
        "s" => {
            let listener = TcpListener::bind("127.0.0.1:7878").unwrap();
            println!("Waiting for client to connect...");
            stream = listener.accept().unwrap().0;
            println!("Connection established");

            let start = SystemTime::now();
            let duration = start.duration_since(UNIX_EPOCH).expect("Time went backwards");
            seed = duration.as_nanos() as u64;
            let buf = seed.to_be_bytes();
            stream.write_all(&buf)?;

            window_title.push_str(" Server");
            is_server = true;
        }
        "c" => {
            stream = TcpStream::connect("192.168.0.45:7878").unwrap();
            println!("Connection established");

            let mut buf = [0; 8];
            stream.read(&mut buf)?;
            seed = u64::from_be_bytes(buf);

            window_title.push_str(" Client");
            is_server = false;
        }
        _ => {
            panic!("Invalid arguments");
        }
    }
    let cb = ggez::ContextBuilder::new("snake", "Clarke Kennedy")
        .window_mode(conf::WindowMode::default().dimensions(cn::GRID_SIZE as f32 * cn::TILE_SIZE, cn::GRID_SIZE as f32 * cn::TILE_SIZE))
        .window_setup(conf::WindowSetup::default().title(&window_title));
    let (mut ctx, event_loop) = cb.build()?;
    event_loop.set_device_event_filter(DeviceEventFilter::Never);
    ctx.gfx.add_font(
        "Retro font",
        graphics::FontData::from_path(&ctx, "\\nintendo-nes-font.ttf")?,
    );
    let state = MainState::new(&mut ctx, stream, is_server, seed)?;
    event::run(ctx, event_loop, state)
}