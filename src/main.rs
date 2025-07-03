use ggez::conf::{WindowMode, WindowSetup};
use ggez::event::{self, EventHandler};
use ggez::glam::Vec2;
use ggez::graphics::{self, Color, DrawMode, DrawParam, Drawable, Mesh, Rect, Text};
use ggez::input::keyboard::{KeyCode, KeyInput};
use ggez::{Context, ContextBuilder, GameResult};
use rand::{thread_rng, Rng};
use std::collections::LinkedList;
use std::path;

// --- CONSTANTES DO JOGO ---
const BLOCK_SIZE: f32 = 24.0;
const MAIN_FONT: &str = "main_font"; // Nome para registrar e usar a fonte.

// --- ESTADOS DO JOGO ---
enum GameMode {
    Menu,
    Playing,
    GameOver,
}

// --- ESTRUTURAS E ENUMS DO JOGO ---

#[derive(Clone, Copy, PartialEq, Debug)]
enum Direction {impl Direction {
    Up,
    Down,
    Left,
    Right,
}

impl Direction {
    fn opposite(&self) -> Direction {
        match *self {
            Direction::Up => Direction::Down,
            Direction::Down => Direction::Up,
            Direction::Left => Direction::Right,
            Direction::Right => Direction::Left,
        }
    }
}

#[derive(Debug, Clone)]
struct Block {
    x: i32,
    y: i32,
}

struct Snake {
    direction: Direction,
    body: LinkedList<Block>,
    tail: Option<Block>,
}

impl Snake {
    fn new(x: i32, y: i32) -> Snake {
        let mut body: LinkedList<Block> = LinkedList::new();
        body.push_back(Block { x, y });
        body.push_back(Block { x: x - 1, y });
        body.push_back(Block { x: x - 2, y });

        Snake {
            direction: Direction::Right,
            body,
            tail: None,
        }
    }

    fn move_forward(&mut self) {
        let head = self.body.front().expect("A cobra não tem corpo.").clone();
        let new_head = match self.direction {
            Direction::Up => Block { x: head.x, y: head.y - 1 },
            Direction::Down => Block { x: head.x, y: head.y + 1 },
            Direction::Left => Block { x: head.x - 1, y: head.y },
            Direction::Right => Block { x: head.x + 1, y: head.y },
        };
        self.body.push_front(new_head);
        self.tail = self.body.pop_back();
    }

    fn head_position(&self) -> (i32, i32) {
        let head = self.body.front().unwrap();
        (head.x, head.y)
    }

    fn is_overlapping_tail(&self) -> bool {
        let (hx, hy) = self.head_position();
        for block in self.body.iter().skip(1) {
            if block.x == hx && block.y == hy {
                return true;
            }
        }
        false
    }
}

// A estrutura principal do jogo agora implementa o `EventHandler` da ggez
struct GameState {
    mode: GameMode,
    snake: Snake,
    food_x: i32,
    food_y: i32,
    score: u32,
    time_since_last_update: f32,
    grid_width: i32,
    grid_height: i32,
    // CORRIGIDO: A fonte não é mais guardada no estado. Ela é registrada no contexto gráfico.
}

impl GameState {
    fn new(ctx: &mut Context) -> GameResult<Self> {
        let (screen_w, screen_h) = ctx.gfx.drawable_size();
        let grid_width = (screen_w / BLOCK_SIZE) as i32;
        let grid_height = (screen_h / BLOCK_SIZE) as i32;

        let font_data = graphics::FontData::from_path(ctx, "/BungeeShade-Regular.ttf")?;
        ctx.gfx.add_font(MAIN_FONT, font_data);

        let mut state = GameState {
            mode: GameMode::Menu,
            snake: Snake::new(3, 2),
            food_x: 0,
            food_y: 0,
            score: 0,
            time_since_last_update: 0.0,
            grid_width,
            grid_height,
        };
        state.add_food();
        Ok(state)
    }

    fn add_food(&mut self) {
        let mut rng = thread_rng();
        if self.grid_width > 2 && self.grid_height > 2 {
            let mut new_x = rng.gen_range(1..(self.grid_width - 1));
            let mut new_y = rng.gen_range(1..(self.grid_height - 1));
            while self.snake.body.iter().any(|b| b.x == new_x && b.y == new_y) {
                new_x = rng.gen_range(1..(self.grid_width - 1));
                new_y = rng.gen_range(1..(self.grid_height - 1));
            }
            self.food_x = new_x;
            self.food_y = new_y;
        }
    }

    /// Reinicia o estado do jogo para começar uma nova partida.
    fn restart(&mut self) {
        self.snake = Snake::new(3, 2);
        self.score = 0;
        self.mode = GameMode::Playing;
        self.add_food();
    }

    /// Redefine o estado do jogo e volta para a tela de menu.
    fn reset_to_menu(&mut self) {
        self.snake = Snake::new(3, 2);
        self.score = 0;
        self.mode = GameMode::Menu;
        self.add_food();
    }
}

impl EventHandler for GameState {
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        if !matches!(self.mode, GameMode::Playing) {
            return Ok(());
        }

        self.time_since_last_update += ctx.time.delta().as_secs_f32();
        let update_interval = (0.15 - (self.score as f32 * 0.005)).max(0.05);

        if self.time_since_last_update > update_interval {
            self.snake.move_forward();
            self.time_since_last_update = 0.0;

            let (head_x, head_y) = self.snake.head_position();
            if head_x == self.food_x && head_y == self.food_y {
                if let Some(tail) = self.snake.tail.take() {
                    self.snake.body.push_back(tail);
                }
                self.score += 1;
                self.add_food();
            }

            if head_x <= 0
                || head_x >= self.grid_width - 1
                || head_y <= 0
                || head_y >= self.grid_height - 1
                || self.snake.is_overlapping_tail()
            {
                self.mode = GameMode::GameOver;
            }
        }
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        let mut canvas = graphics::Canvas::from_frame(ctx, Color::from([0.15, 0.17, 0.18, 1.0]));
        draw_background(self, ctx, &mut canvas)?;

        match self.mode {
            GameMode::Menu => {
                draw_menu(self, ctx, &mut canvas)?;
            }
            GameMode::Playing => {
                draw_gameplay(self, ctx, &mut canvas)?;
            }
            GameMode::GameOver => {
                draw_gameplay(self, ctx, &mut canvas)?;
                draw_game_over(self, ctx, &mut canvas)?;
            }
        }

        canvas.finish(ctx)?;
        Ok(())
    }

    fn resize_event(&mut self, _ctx: &mut Context, width: f32, height: f32) -> GameResult {
        self.grid_width = (width / BLOCK_SIZE) as i32;
        self.grid_height = (height / BLOCK_SIZE) as i32;
        self.reset_to_menu();
        Ok(())
    }

    fn key_down_event(&mut self, _ctx: &mut Context, input: KeyInput, _repeat: bool) -> GameResult {
        if let Some(keycode) = input.keycode {
            match self.mode {
                GameMode::Menu => {
                    if keycode == KeyCode::Return {
                        self.restart();
                    }
                }
                GameMode::Playing => {
                    let dir = match keycode {
                        KeyCode::Up     |   KeyCode::W  => Some(Direction::Up),
                        KeyCode::Down   |   KeyCode::S  => Some(Direction::Down),
                        KeyCode::Left   |   KeyCode::A  => Some(Direction::Left),
                        KeyCode::Right  |   KeyCode::D  => Some(Direction::Right),
                        _ => None,
                    };

                    if let Some(d) = dir {
                        if d != self.snake.direction.opposite() {
                            self.snake.direction = d;
                        }
                    }
                }
                GameMode::GameOver => {
                    self.reset_to_menu();
                }
            }
        }
        Ok(())
    }
}

// --- FUNÇÕES DE DESENHO AUXILIARES ---

fn draw_background(gs: &GameState, ctx: &mut Context, canvas: &mut graphics::Canvas) -> GameResult {
    let wall_mesh = Mesh::new_rectangle(
        ctx,
        DrawMode::fill(),
        Rect::new(0.0, 0.0, BLOCK_SIZE, BLOCK_SIZE),
        Color::from([0.4, 0.4, 0.4, 1.0]),
    )?;
    let grid_mesh = Mesh::new_rectangle(
        ctx,
        DrawMode::stroke(0.5),
        Rect::new(0.0, 0.0, BLOCK_SIZE, BLOCK_SIZE),
        Color::from([0.2, 0.22, 0.23, 1.0]),
    )?;

    for y in 0..gs.grid_height {
        for x in 0..gs.grid_width {
            if x == 0 || x == gs.grid_width - 1 || y == 0 || y == gs.grid_height - 1 {
                canvas.draw(
                    &wall_mesh,
                    Vec2::new(x as f32 * BLOCK_SIZE, y as f32 * BLOCK_SIZE),
                );
            } else {
                canvas.draw(
                    &grid_mesh,
                    Vec2::new(x as f32 * BLOCK_SIZE, y as f32 * BLOCK_SIZE),
                );
            }
        }
    }
    Ok(())
}

fn draw_gameplay(gs: &mut GameState, ctx: &mut Context, canvas: &mut graphics::Canvas) -> GameResult {
    let head_color = Color::from([0.9, 0.5, 0.2, 1.0]);
    let body_color = Color::from([0.8, 0.4, 0.1, 1.0]);

    let block_mesh = Mesh::new_rectangle(
        ctx,
        DrawMode::fill(),
        Rect::new(2.0, 2.0, BLOCK_SIZE - 4.0, BLOCK_SIZE - 4.0),
        Color::WHITE,
    )?;
    let eye_mesh = Mesh::new_rectangle(
        ctx,
        DrawMode::fill(),
        Rect::new(0.0, 0.0, 4.0, 4.0),
        Color::BLACK,
    )?;

    for (i, block) in gs.snake.body.iter().enumerate() {
        let color = if i == 0 { head_color } else { body_color };
        let pos = Vec2::new(block.x as f32 * BLOCK_SIZE, block.y as f32 * BLOCK_SIZE);
        canvas.draw(&block_mesh, DrawParam::new().dest(pos).color(color));

        if i == 0 {
            let (eye1_offset, eye2_offset) = match gs.snake.direction {
                Direction::Up => (Vec2::new(4.0, 4.0), Vec2::new(BLOCK_SIZE - 8.0, 4.0)),
                Direction::Down => {
                    (
                        Vec2::new(4.0, BLOCK_SIZE - 8.0),
                        Vec2::new(BLOCK_SIZE - 8.0, BLOCK_SIZE - 8.0),
                    )
                }
                Direction::Left => {
                    (
                        Vec2::new(4.0, 4.0),
                        Vec2::new(4.0, BLOCK_SIZE - 8.0),
                    )
                }
                Direction::Right => {
                    (
                        Vec2::new(BLOCK_SIZE - 8.0, 4.0),
                        Vec2::new(BLOCK_SIZE - 8.0, BLOCK_SIZE - 8.0),
                    )
                }
            };
            canvas.draw(&eye_mesh, pos + eye1_offset);
            canvas.draw(&eye_mesh, pos + eye2_offset);
        }
    }

    canvas.draw(
        &block_mesh,
        DrawParam::new()
            .dest(Vec2::new(
                gs.food_x as f32 * BLOCK_SIZE,
                gs.food_y as f32 * BLOCK_SIZE,
            ))
            .color(Color::RED),
    );

    let apple_mesh = Mesh::new_rectangle(
        ctx,
        DrawMode::fill(),
        Rect::new(0.0, 0.0, BLOCK_SIZE * 0.5, BLOCK_SIZE * 0.5),
        Color::RED,
    )?;
    canvas.draw(&apple_mesh, Vec2::new(10.0, 10.0));

    let mut score_text = Text::new(format!(": {}", gs.score));
    // CORRIGIDO: Usa o nome registrado da fonte.
    score_text.set_font(MAIN_FONT).set_scale(20.0);

    if let Some(text_rect) = score_text.dimensions(ctx) {
        let h = text_rect.h;
        canvas.draw(
            &score_text,
            DrawParam::new()
                .dest(Vec2::new(10.0 + BLOCK_SIZE * 0.5, 12.0 - h * 0.5))
                .color(Color::WHITE),
        );
    }

    Ok(())
}

fn draw_centered_text(
    canvas: &mut graphics::Canvas,
    ctx: &mut Context,
    text_str: &str,
    size: f32,
    y_offset: f32,
    color: Color,
) -> GameResult {
    let (screen_w, screen_h) = ctx.gfx.drawable_size();
    let mut text = Text::new(text_str);
    // CORRIGIDO: Usa o nome registrado da fonte.
    text.set_font(MAIN_FONT).set_scale(size);

    if let Some(text_rect) = text.dimensions(ctx) {
        let text_w = text_rect.w;
        let pos = Vec2::new((screen_w - text_w) / 2.0, screen_h / 2.0 + y_offset);
        canvas.draw(&text, DrawParam::new().dest(pos).color(color));
    }
    Ok(())
}

fn draw_menu(_gs: &GameState, ctx: &mut Context, canvas: &mut graphics::Canvas) -> GameResult {
    // CORRIGIDO: A fonte não é mais passada como argumento.
    draw_centered_text(
        canvas,
        ctx,
        "Rusted Snake",
        64.0,
        -100.0,
        Color::from([0.8, 0.4, 0.1, 1.0]),
    )?;
    draw_centered_text(
        canvas,
        ctx,
        "Pressione ENTER para começar",
        24.0,
        20.0,
        Color::WHITE,
    )?;
    Ok(())
}

fn draw_game_over(_gs: &GameState, ctx: &mut Context, canvas: &mut graphics::Canvas) -> GameResult {
    // CORRIGIDO: A fonte não é mais passada como argumento.
    draw_centered_text(
        canvas,
        ctx,
        "Fim de Jogo!",
        48.0,
        -50.0,
        Color::RED,
    )?;
    draw_centered_text(
        canvas,
        ctx,
        "Pressione qualquer tecla para voltar ao menu",
        24.0,
        0.0,
        Color::WHITE,
    )?;
    Ok(())
}

// --- FUNÇÃO PRINCIPAL ---
pub fn main() -> GameResult {
    let resource_dir = if let Ok(manifest_dir) = std::env::var("CARGO_MANIFEST_DIR") {
        let mut path = path::PathBuf::from(manifest_dir);
        path.push("assets");
        path
    } else {
        path::PathBuf::from("./assets")
    };

    let (mut ctx, event_loop) = ContextBuilder::new("snake_rust", "Gemini")
        .window_setup(WindowSetup::default().title("Rusted Snake"))
        .window_mode(WindowMode::default().dimensions(816.0, 600.0).resizable(true))
        .add_resource_path(resource_dir)
        .build()?;

    // A criação de GameState agora pode falhar, então usamos `?`.
    let state = GameState::new(&mut ctx)?;
    event::run(ctx, event_loop, state)
}
