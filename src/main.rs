use std::collections::VecDeque;

use macroquad::{
    color::{BLACK, BLUE, DARKPURPLE, GOLD, WHITE, YELLOW},
    input::{KeyCode, get_last_key_pressed},
    main,
    math::{IVec2, Vec2, ivec2, vec2},
    rand::gen_range,
    shapes::draw_rectangle,
    text::draw_text,
    time::get_frame_time,
    window::{clear_background, next_frame, screen_height, screen_width},
};

const GRID_SIZE: IVec2 = ivec2(20, 20);
const STEP_TIME: f32 = 0.1;

struct Game {
    snake: Snake,
    food: IVec2,
    next_dir: IVec2,
    phase: Phase,
}

impl Game {
    fn new() -> Self {
        let snake = Snake {
            body: VecDeque::from(vec![
                ivec2(2, GRID_SIZE.y / 2),
                ivec2(1, GRID_SIZE.y / 2),
                ivec2(0, GRID_SIZE.y / 2),
            ]),
            dir: ivec2(1, 0),
        };
        let food = spawn_food(&snake.body).expect("a fresh grid is never full");

        Self {
            snake,
            food,
            next_dir: ivec2(1, 0),
            phase: Phase::Start,
        }
    }
}

enum Phase {
    Playing,
    Lost,
    Won,
    Start,
    Quit,
}

struct Snake {
    body: VecDeque<IVec2>,
    dir: IVec2,
}

fn cell_to_pixel(cell: IVec2, cell_size: f32, offset: Vec2) -> Vec2 {
    offset + cell.as_vec2() * cell_size
}

fn spawn_food(body: &VecDeque<IVec2>) -> Option<IVec2> {
    let free: Vec<IVec2> = (0..GRID_SIZE.x)
        .flat_map(|x| (0..GRID_SIZE.y).map(move |y| ivec2(x, y)))
        .filter(|cell| !body.contains(cell))
        .collect();
    if free.is_empty() {
        None
    } else {
        Some(free[gen_range(0, free.len())])
    }
}

fn view(state: &Game) {
    let screen = vec2(screen_width(), screen_height());
    let cell_size = (screen / GRID_SIZE.as_vec2()).min_element();
    let board = GRID_SIZE.as_vec2() * cell_size;
    let offset = (screen - board) / 2.0;

    clear_background(BLACK);
    draw_rectangle(offset.x, offset.y, board.x, board.y, DARKPURPLE);
    for (i, part) in state.snake.body.iter().enumerate().rev() {
        let color = if i == 0 { GOLD } else { YELLOW };
        let pixel = cell_to_pixel(*part, cell_size, offset);
        draw_rectangle(pixel.x, pixel.y, cell_size, cell_size, color);
    }

    let pixel = cell_to_pixel(state.food, cell_size, offset);
    draw_rectangle(pixel.x, pixel.y, cell_size, cell_size, BLUE);

    match state.phase {
        Phase::Start => {
            draw_text(
                "Press any key to start",
                screen_width() / 2.0,
                screen_height() / 2.0,
                40.0,
                WHITE,
            );
        }
        Phase::Lost => {
            draw_text(
                "Game Over - R to restart",
                screen_width() / 2.0,
                screen_height() / 2.0,
                40.0,
                WHITE,
            );
        }
        Phase::Won => {
            draw_text(
                "You Win! - R to restart",
                screen_width() / 2.0,
                screen_height() / 2.0,
                40.0,
                WHITE,
            );
        }
        Phase::Playing | Phase::Quit => {}
    };
}

enum Msg {
    Start,
    Turn(IVec2),
    Tick,
    Restart,
    Quit,
}

fn input(msgs: &mut Vec<Msg>, timer: &mut f32, phase: &Phase) {
    let key = get_last_key_pressed();

    if key == Some(KeyCode::Q) {
        msgs.push(Msg::Quit);
    }

    match phase {
        Phase::Start => {
            *timer = 0.0;
            if key.is_some() {
                msgs.push(Msg::Start)
            }
        }
        Phase::Playing => {
            match key {
                Some(KeyCode::L) => msgs.push(Msg::Turn(ivec2(1, 0))),
                Some(KeyCode::H) => msgs.push(Msg::Turn(ivec2(-1, 0))),
                Some(KeyCode::J) => msgs.push(Msg::Turn(ivec2(0, 1))),
                Some(KeyCode::K) => msgs.push(Msg::Turn(ivec2(0, -1))),
                _ => {}
            }

            *timer += get_frame_time();
            if *timer >= STEP_TIME {
                *timer -= STEP_TIME;
                msgs.push(Msg::Tick);
            }
        }
        Phase::Lost | Phase::Won => {
            *timer = 0.0;
            if key == Some(KeyCode::R) {
                msgs.push(Msg::Restart);
            }
        }
        Phase::Quit => {}
    }
}

fn update(mut state: Game, msg: Msg) -> Game {
    match msg {
        Msg::Start => state.phase = Phase::Playing,
        Msg::Turn(dir) => {
            if dir != -state.snake.dir {
                state.next_dir = dir
            }
        }
        Msg::Tick => {
            state.snake.dir = state.next_dir;
            let new_head =
                (*state.snake.body.front().unwrap() + state.snake.dir).rem_euclid(GRID_SIZE);
            state.snake.body.push_front(new_head);
            if new_head == state.food {
                match spawn_food(&state.snake.body) {
                    Some(f) => state.food = f,
                    None => state.phase = Phase::Won,
                }
            } else {
                state.snake.body.pop_back();
            }
            if state.snake.body.iter().skip(1).any(|&c| c == new_head) {
                state.phase = Phase::Lost;
            }
        }
        Msg::Restart => state = Game::new(),
        Msg::Quit => state.phase = Phase::Quit,
    }
    state
}

#[main("Helix Snake")]
async fn main() {
    let mut msgs: Vec<Msg> = Vec::with_capacity(10);
    let mut timer = 0.0;
    let mut state = Game::new();

    loop {
        msgs.clear();
        input(&mut msgs, &mut timer, &state.phase);

        for msg in msgs.drain(..) {
            state = update(state, msg);
        }

        if matches!(state.phase, Phase::Quit) {
            break;
        }

        view(&state);
        next_frame().await
    }
}
