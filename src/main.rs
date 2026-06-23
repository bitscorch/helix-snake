use std::collections::VecDeque;

use macroquad::{
    color::{BLUE, DARKPURPLE, GOLD, YELLOW},
    input::{KeyCode, get_keys_down, get_last_key_pressed, is_key_down},
    main,
    math::{IVec2, Vec2, ivec2, vec2},
    rand::gen_range,
    shapes::draw_rectangle,
    time::get_frame_time,
    window::{clear_background, next_frame, screen_height, screen_width},
};

const GRID_SIZE: IVec2 = ivec2(20, 20);
const STEP_TIME: f32 = 0.1;

struct Game {
    snake: Snake,
    food: IVec2,
    next_dir: IVec2,
    timer: f32,
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
            timer: 0.0,
            phase: Phase::Start,
        }
    }
}

enum Phase {
    Playing,
    Lost,
    Won,
    Start,
}

struct Snake {
    body: VecDeque<IVec2>,
    dir: IVec2,
}

fn cell_to_pixel(cell: IVec2, cell_size: Vec2) -> Vec2 {
    cell.as_vec2() * cell_size
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

#[main("Helix Snake")]
async fn main() {
    let mut state = Game::new();

    loop {
        let cell_size = vec2(
            screen_width() / GRID_SIZE.x as f32,
            screen_height() / GRID_SIZE.y as f32,
        );

        let dt = get_frame_time();
        state.timer += dt;

        let proposed_dir = match get_last_key_pressed() {
            Some(KeyCode::L) => ivec2(1, 0),
            Some(KeyCode::H) => ivec2(-1, 0),
            Some(KeyCode::J) => ivec2(0, 1),
            Some(KeyCode::K) => ivec2(0, -1),
            _ => state.next_dir,
        };

        if proposed_dir != -state.snake.dir {
            state.next_dir = proposed_dir;
        }

        if state.timer >= STEP_TIME {
            state.timer -= STEP_TIME;
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

        if is_key_down(KeyCode::Q) {
            break;
        }

        clear_background(DARKPURPLE);
        for (i, part) in state.snake.body.iter().enumerate() {
            let color = if i == 0 { GOLD } else { YELLOW };
            let pixel = cell_to_pixel(*part, cell_size);
            draw_rectangle(pixel.x, pixel.y, cell_size.x, cell_size.y, color);
        }

        let pixel = cell_to_pixel(state.food, cell_size);
        draw_rectangle(pixel.x, pixel.y, cell_size.x, cell_size.y, BLUE);
        next_frame().await
    }
}
