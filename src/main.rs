use std::collections::VecDeque;

use macroquad::{
    color::{DARKPURPLE, YELLOW},
    input::{KeyCode, get_keys_down, get_last_key_pressed, is_key_down},
    main,
    math::{IVec2, Vec2, ivec2, vec2},
    shapes::draw_rectangle,
    time::get_frame_time,
    window::{clear_background, next_frame, screen_height, screen_width},
};

const GRID_SIZE: IVec2 = ivec2(20, 20);
const STEP_TIME: f32 = 0.1;

struct Snake {
    body: VecDeque<IVec2>,
    dir: IVec2,
}

fn cell_to_pixel(cell: IVec2, cell_size: Vec2) -> Vec2 {
    cell.as_vec2() * cell_size
}

#[main("Helix Snake")]
async fn main() {
    let mut snake = Snake {
        body: VecDeque::from(vec![
            ivec2(2, GRID_SIZE.y / 2),
            ivec2(1, GRID_SIZE.y / 2),
            ivec2(0, GRID_SIZE.y / 2),
        ]),
        dir: ivec2(1, 0),
    };
    let mut timer = 0.0;

    loop {
        let cell_size = vec2(
            screen_width() / GRID_SIZE.x as f32,
            screen_height() / GRID_SIZE.y as f32,
        );

        let dt = get_frame_time();
        timer += dt;

        if timer >= STEP_TIME {
            timer -= STEP_TIME;
            snake.body.pop_back();
            let new_head = *snake.body.front().unwrap() + snake.dir;
            snake.body.push_front(new_head.rem_euclid(GRID_SIZE));
        }

        let new_dir = match get_last_key_pressed() {
            Some(KeyCode::L) => ivec2(1, 0),
            Some(KeyCode::H) => ivec2(-1, 0),
            Some(KeyCode::J) => ivec2(0, 1),
            Some(KeyCode::K) => ivec2(0, -1),
            _ => snake.dir,
        };
        if snake.dir.dot(new_dir) == 0 {
            snake.dir = new_dir;
        }

        if is_key_down(KeyCode::Q) {
            break;
        }

        clear_background(DARKPURPLE);
        for part in &snake.body {
            let pixel = cell_to_pixel(*part, cell_size);
            draw_rectangle(pixel.x, pixel.y, cell_size.x, cell_size.y, YELLOW);
        }
        next_frame().await
    }
}
