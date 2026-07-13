use std::collections::{HashSet, VecDeque};

use macroquad::{
    color::{BLACK, BLUE, Color, DARKPURPLE, GREEN, RED, WHITE, YELLOW},
    input::{KeyCode, get_last_key_pressed},
    main,
    math::{IVec2, Vec2, ivec2, vec2},
    rand::{gen_range, srand},
    shapes::draw_rectangle,
    text::draw_text,
    time::get_frame_time,
    window::{clear_background, next_frame, screen_height, screen_width},
};
use serde::{Deserialize, Serialize};
use strum::{EnumCount, EnumIter, IntoEnumIterator};

const GRID_SIZE: IVec2 = ivec2(30, 30);
const STEP_TIME: f32 = 0.1;
const FOOD_COUNT: usize = 5;

#[derive(Clone, Copy, PartialEq, EnumIter, EnumCount)]
enum FoodColor {
    Red,
    Blue,
    Green,
}

impl FoodColor {
    fn random() -> Self {
        Self::iter().nth(gen_range(0, Self::COUNT)).unwrap()
    }

    fn to_color(self) -> Color {
        match self {
            FoodColor::Red => RED,
            FoodColor::Blue => BLUE,
            FoodColor::Green => GREEN,
        }
    }
}

struct Food {
    pos: IVec2,
    color: FoodColor,
}

struct Game {
    snake: Snake,
    food: [Food; FOOD_COUNT],
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
            color: None,
            grow: 0,
        };
        let mut occupied: Vec<IVec2> = snake.body.iter().copied().collect();
        let food: [Food; FOOD_COUNT] = std::array::from_fn(|_| {
            let f = spawn_food(&occupied).expect("a fresh grid is never full");
            occupied.push(f.pos);
            f
        });

        Self {
            snake,
            food,
            next_dir: ivec2(1, 0),
            phase: Phase::Start,
        }
    }
}

#[cfg(debug_assertions)]
fn check_invariants(g: &Game) {
    // dir is a canonical unit step  ← would have caught wrapped_delta instantly
    assert_eq!(
        g.snake.dir.abs().element_sum(),
        1,
        "dir not a unit step: {:?}",
        g.snake.dir
    );
    assert_eq!(g.next_dir.abs().element_sum(), 1);
    assert_ne!(g.next_dir, -g.snake.dir, "buffered a U-turn");

    // every cell on the board
    for c in &g.snake.body {
        assert!(c.x >= 0 && c.x < GRID_SIZE.x && c.y >= 0 && c.y < GRID_SIZE.y);
    }
    // body is a contiguous path (each pair one wrapped step apart)
    for (a, b) in g.snake.body.iter().zip(g.snake.body.iter().skip(1)) {
        assert_eq!(
            wrapped_delta(*b, *a).abs().element_sum(),
            1,
            "body not contiguous"
        );
    }
    // while alive, no self-overlap
    if matches!(g.phase, Phase::Playing) {
        let set: HashSet<_> = g.snake.body.iter().collect();
        assert_eq!(set.len(), g.snake.body.len(), "body overlaps itself");
    }
    // food never on the snake, never on each other
    for (i, f) in g.food.iter().enumerate() {
        assert!(!g.snake.body.contains(&f.pos), "food on snake");
        assert!(
            g.food.iter().skip(i + 1).all(|o| o.pos != f.pos),
            "food on food"
        );
    }
}

#[cfg(debug_assertions)]
fn maybe_dump_replay(seed: u64, history: &[Msg]) {
    #[cfg(not(target_arch = "wasm32"))]
    #[cfg(not(target_arch = "wasm32"))]
    if macroquad::input::is_key_pressed(KeyCode::F1) {
        let replay = Replay {
            seed,
            msgs: history.to_vec(),
        };
        let text = ron::ser::to_string_pretty(&replay, Default::default()).unwrap();
        std::fs::write("replay.ron", text).unwrap();
        println!("saved replay.ron ({} msgs)", history.len());
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
    color: Option<FoodColor>,
    grow: usize,
}

fn scale(color: Color, factor: f32) -> Color {
    Color::new(
        (color.r * factor).clamp(0.0, 1.0),
        (color.g * factor).clamp(0.0, 1.0),
        (color.b * factor).clamp(0.0, 1.0),
        color.a,
    )
}

fn cell_to_pixel(cell: IVec2, cell_size: f32, offset: Vec2) -> Vec2 {
    offset + cell.as_vec2() * cell_size
}

fn spawn_food(occupied: &[IVec2]) -> Option<Food> {
    let free: Vec<IVec2> = (0..GRID_SIZE.x)
        .flat_map(|x| (0..GRID_SIZE.y).map(move |y| ivec2(x, y)))
        .filter(|cell| !occupied.contains(cell))
        .collect();
    if free.is_empty() {
        None
    } else {
        let pos = free[gen_range(0, free.len())];
        let color = FoodColor::random();
        Some(Food { pos, color })
    }
}

fn wrapped_delta(from: IVec2, to: IVec2) -> IVec2 {
    let half = GRID_SIZE / 2;
    (to - from + half).rem_euclid(GRID_SIZE) - half
}

fn view(state: &Game) {
    let screen = vec2(screen_width(), screen_height());
    let cell_size = (screen / GRID_SIZE.as_vec2()).min_element();
    let board = GRID_SIZE.as_vec2() * cell_size;
    let offset = (screen - board) / 2.0;

    clear_background(BLACK);
    draw_rectangle(offset.x, offset.y, board.x, board.y, DARKPURPLE);
    let color = state.snake.color.map_or(YELLOW, |c| c.to_color());
    for (i, part) in state.snake.body.iter().enumerate().rev() {
        let color = scale(color, if i == 0 { 1.075 } else { 0.925 });
        let pixel = cell_to_pixel(*part, cell_size, offset);
        draw_rectangle(pixel.x, pixel.y, cell_size, cell_size, color);
    }

    for food in &state.food {
        let pixel = cell_to_pixel(food.pos, cell_size, offset);
        draw_rectangle(
            pixel.x,
            pixel.y,
            cell_size,
            cell_size,
            food.color.to_color(),
        );
    }

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

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
enum Msg {
    Start,
    Turn(IVec2),
    Tick,
    SelectSame,
    Reverse,
    Restart,
    Quit,
}

#[derive(Serialize, Deserialize)]
struct Replay {
    seed: u64,
    msgs: Vec<Msg>,
}

fn input(msgs: &mut Vec<Msg>, timer: &mut f32, phase: &Phase) {
    let key = get_last_key_pressed();

    if key == Some(KeyCode::Q) {
        msgs.push(Msg::Quit);
        return;
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
                Some(KeyCode::S) => msgs.push(Msg::SelectSame),
                Some(KeyCode::B) => msgs.push(Msg::Reverse),
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
            if let Some(i) = state.food.iter().position(|f| f.pos == new_head) {
                state.snake.color = Some(state.food[i].color);
                state.snake.grow += 1;

                let mut occupied: Vec<IVec2> = state.snake.body.iter().copied().collect();
                for (j, f) in state.food.iter().enumerate() {
                    if j != i {
                        occupied.push(f.pos);
                    }
                }
                match spawn_food(&occupied) {
                    Some(f) => state.food[i] = f,
                    None => state.phase = Phase::Won,
                }
            }

            if state.snake.grow > 0 {
                state.snake.grow -= 1;
            } else {
                state.snake.body.pop_back();
            }

            if state.snake.body.iter().skip(1).any(|&c| c == new_head) {
                state.phase = Phase::Lost;
            }
        }
        Msg::SelectSame => {
            let matches: Vec<usize> = state
                .food
                .iter()
                .enumerate()
                .filter(|(_, f)| Some(f.color) == state.snake.color)
                .map(|(i, _)| i)
                .collect();

            if matches.is_empty() {
                state.phase = Phase::Lost;
            } else {
                state.snake.grow += matches.len();

                let mut occupied: Vec<IVec2> = state.snake.body.iter().copied().collect();
                for (j, f) in state.food.iter().enumerate() {
                    if !matches.contains(&j) {
                        occupied.push(f.pos);
                    }
                }
                for i in matches {
                    match spawn_food(&occupied) {
                        Some(f) => {
                            occupied.push(f.pos);
                            state.food[i] = f;
                        }
                        None => state.phase = Phase::Won,
                    }
                }

                state.snake.color = None;
            }
        }
        Msg::Reverse => {
            state.snake.body.make_contiguous().reverse();

            let head = state.snake.body[0];
            let neck = state.snake.body[1];
            let dir = wrapped_delta(neck, head);

            state.snake.dir = dir;
            state.next_dir = dir;
        }
        Msg::Restart => state = Game::new(),
        Msg::Quit => state.phase = Phase::Quit,
    }
    state
}

#[main("Helix Snake")]
async fn main() {
    let seed = macroquad::miniquad::date::now() as u64;
    srand(seed);
    let mut history: Vec<Msg> = Vec::new();

    let mut msgs: Vec<Msg> = Vec::with_capacity(10);
    let mut timer = 0.0;
    let mut state = Game::new();

    loop {
        msgs.clear();
        input(&mut msgs, &mut timer, &state.phase);

        for msg in msgs.drain(..) {
            #[cfg(debug_assertions)]
            history.push(msg);

            state = update(state, msg);

            #[cfg(debug_assertions)]
            check_invariants(&state);
        }

        maybe_dump_replay(seed, &history);

        if matches!(state.phase, Phase::Quit) {
            break;
        }

        view(&state);
        next_frame().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use macroquad::rand::srand;
    use proptest::prelude::*;

    fn any_msg() -> impl Strategy<Value = Msg> {
        prop_oneof![
            8 => Just(Msg::Tick),
            1 => Just(Msg::SelectSame),
            2 => Just(Msg::Reverse),
            4 => Just(Msg::Turn(ivec2(1, 0))),
            4 => Just(Msg::Turn(ivec2(-1, 0))),
            4 => Just(Msg::Turn(ivec2(0, 1))),
            4 => Just(Msg::Turn(ivec2(0, -1))),
        ]
    }

    proptest! {
        #[test]
        fn invariants_hold_under_any_input(
            seed: u64,
            msgs in prop::collection::vec(any_msg(), 0..500),
        ) {
            srand(seed);
            let mut g = update(Game::new(), Msg::Start);

            for m in msgs {
                if !matches!(g.phase, Phase::Playing) {
                    break;   // input only emits Tick/Turn while Playing — mimic that
                }
                g = update(g, m);
                check_invariants(&g);
            }
        }
    }

    #[test]
    fn replay_regressions() {
        let dir = std::path::Path::new("tests/replays");
        if !dir.exists() {
            return;
        }

        for entry in std::fs::read_dir(dir).unwrap() {
            let path = entry.unwrap().path();
            if path.extension().is_none_or(|e| e != "ron") {
                continue;
            }

            let replay: Replay = ron::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();

            srand(replay.seed); // same order as the shell!
            let mut g = Game::new();
            for m in replay.msgs.into_iter() {
                g = update(g, m);
                check_invariants(&g); // panics at the first bad message; `i` tells you where
            }
        }
    }
}
