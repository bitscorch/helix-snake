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
    let mut snake = Snake {
        body: VecDeque::from(vec![
            ivec2(2, GRID_SIZE.y / 2),
            ivec2(1, GRID_SIZE.y / 2),
            ivec2(0, GRID_SIZE.y / 2),
        ]),
        dir: ivec2(1, 0),
    };
    let mut food = spawn_food(&snake.body).unwrap();
    let mut next_dir = snake.dir;

    let mut timer = 0.0;

    loop {
        let cell_size = vec2(
            screen_width() / GRID_SIZE.x as f32,
            screen_height() / GRID_SIZE.y as f32,
        );

        let dt = get_frame_time();
        timer += dt;

        let proposed_dir = match get_last_key_pressed() {
            Some(KeyCode::L) => ivec2(1, 0),
            Some(KeyCode::H) => ivec2(-1, 0),
            Some(KeyCode::J) => ivec2(0, 1),
            Some(KeyCode::K) => ivec2(0, -1),
            _ => next_dir,
        };

        if proposed_dir != -snake.dir {
            next_dir = proposed_dir;
        }

        if timer >= STEP_TIME {
            timer -= STEP_TIME;
            snake.dir = next_dir;
            let new_head = (*snake.body.front().unwrap() + snake.dir).rem_euclid(GRID_SIZE);
            snake.body.push_front(new_head);
            if new_head == food {
                food = spawn_food(&snake.body).expect("You win!");
            } else {
                snake.body.pop_back();
            }

            if snake.body.iter().skip(1).any(|&c| c == new_head) {
                panic!("Game over!");
            }
        }

        if is_key_down(KeyCode::Q) {
            break;
        }

        clear_background(DARKPURPLE);
        for (i, part) in snake.body.iter().enumerate() {
            let color = if i == 0 { GOLD } else { YELLOW };
            let pixel = cell_to_pixel(*part, cell_size);
            draw_rectangle(pixel.x, pixel.y, cell_size.x, cell_size.y, color);
        }

        let pixel = cell_to_pixel(food, cell_size);
        draw_rectangle(pixel.x, pixel.y, cell_size.x, cell_size.y, BLUE);
        next_frame().await
    }
}
