use std::fs::File;
use byteorder::{ WriteBytesExt, ReadBytesExt, LittleEndian };

use console_engine::pixel;
use console_engine::Color;
use console_engine::KeyCode;
use rand::Rng;

const WIDTH: u32 = 22;
const HEIGHT: u32 = 10;
const PLAYER_CHAR: char = 'B';
const MIN_PIPE_GAP: i32 = 2;

struct Pipe {
    x: i32,
    y: i32,
    width: i32,
    height: i32,
}

struct Game {
    player_x: i32,
    player_y: i32,
    score: i32,
    high_score: i32,
    pipes: Vec<Pipe>,

    spawn_pipe_time: f32,
    time_since_last_pipe_spawned: f32,
    pipe_speed: f32,
    time_since_last_pipe_moved: f32,
    max_pipes: usize,

    gravity_time: f32,
    time_since_last_gravity: f32,
    time_since_last_jump: f32,
    jump_time: f32,

    last_score_time: f32,
    update_score_time: f32,
}

enum GameState {
    Menu,
    Playing,
    Paused,
}

fn draw_pipe(engine: &mut console_engine::ConsoleEngine, x: i32, y: i32, width: i32, height: i32) {
    for i in 0..width {
        for j in 0..height {
            engine.set_pxl(x + i, y + j, pixel::pxl_fg('#', Color::Green));
        }
    }
}

fn draw_score(engine: &mut console_engine::ConsoleEngine, score: i32) {
    let score_str = format!("{}", score);
    for (i, c) in score_str.chars().enumerate() {
        let x = (i as i32) + ((WIDTH as i32) - (score_str.len() as i32)) / 2;
        engine.set_pxl(x, 0, pixel::pxl_fg(c, Color::White));
    }
}

fn draw_player(engine: &mut console_engine::ConsoleEngine, x: i32, y: i32) {
    engine.set_pxl(x, y, pixel::pxl_fg(PLAYER_CHAR, Color::Red));
}

fn draw_menu(engine: &mut console_engine::ConsoleEngine, high_score: i32) {
    let title = "Flappy Bird";
    let start = "Press Enter to start";
    let quit = "Press Q to quit";
    let high_score = format!("High Score: {}", high_score);

    for (i, c) in title.chars().enumerate() {
        let x = (i as i32) + ((WIDTH as i32) - (title.len() as i32)) / 2;
        engine.set_pxl(x, 2, pixel::pxl_fg(c, Color::White));
    }

    for (i, c) in start.chars().enumerate() {
        let x = (i as i32) + ((WIDTH as i32) - (start.len() as i32)) / 2;
        engine.set_pxl(x, 4, pixel::pxl_fg(c, Color::White));
    }

    for (i, c) in quit.chars().enumerate() {
        let x = (i as i32) + ((WIDTH as i32) - (quit.len() as i32)) / 2;
        engine.set_pxl(x, 6, pixel::pxl_fg(c, Color::White));
    }

    for (i, c) in high_score.chars().enumerate() {
        let x = (i as i32) + ((WIDTH as i32) - (high_score.len() as i32)) / 2;
        engine.set_pxl(x, 8, pixel::pxl_fg(c, Color::Rgb { r: 255, g: 215, b: 0 }));
    }
}

fn draw_pause(engine: &mut console_engine::ConsoleEngine, high_score: i32) {
    let title = "Paused";
    let high_score = format!("High Score: {}", high_score);
    let resume = "Press Enter to resume";
    let quit = "Press Q to quit";

    for (i, c) in title.chars().enumerate() {
        let x = (i as i32) + ((WIDTH as i32) - (title.len() as i32)) / 2;
        engine.set_pxl(x, 2, pixel::pxl_fg(c, Color::White));
    }

    for (i, c) in high_score.chars().enumerate() {
        let x = (i as i32) + ((WIDTH as i32) - (high_score.len() as i32)) / 2;
        engine.set_pxl(x, 4, pixel::pxl_fg(c, Color::Rgb { r: 255, g: 215, b: 0 }));
    }

    for (i, c) in resume.chars().enumerate() {
        let x = (i as i32) + ((WIDTH as i32) - (resume.len() as i32)) / 2;
        engine.set_pxl(x, 6, pixel::pxl_fg(c, Color::White));
    }

    for (i, c) in quit.chars().enumerate() {
        let x = (i as i32) + ((WIDTH as i32) - (quit.len() as i32)) / 2;
        engine.set_pxl(x, 8, pixel::pxl_fg(c, Color::White));
    }
}

fn spawn_pipe(pipes: &mut Vec<Pipe>) {
    let mut rng = rand::thread_rng();
    let upper_pipe_height = rng.gen_range(1..(HEIGHT as i32) - MIN_PIPE_GAP);
    let lower_pipe_height = (HEIGHT as i32) - upper_pipe_height - MIN_PIPE_GAP;

    let upper_pipe = Pipe {
        x: WIDTH as i32,
        y: 0,
        width: 1,
        height: upper_pipe_height,
    };

    let lower_pipe = Pipe {
        x: WIDTH as i32,
        y: upper_pipe_height + MIN_PIPE_GAP,
        width: 1,
        height: lower_pipe_height,
    };

    pipes.push(upper_pipe);
    pipes.push(lower_pipe);
}

fn update_pipes(engine: &mut console_engine::ConsoleEngine, game: &mut Game, dt: f32) {
    game.time_since_last_pipe_spawned += dt;
    game.time_since_last_pipe_moved += dt;

    let pipes_len = game.pipes.len();

    for pipe in game.pipes.iter_mut() {
        draw_pipe(engine, pipe.x, pipe.y, pipe.width, pipe.height);

        if game.time_since_last_pipe_moved > 1.0 / game.pipe_speed {
            pipe.x -= 1;
        }
    }

    if game.time_since_last_pipe_moved > 1.0 / game.pipe_speed {
        game.time_since_last_pipe_moved = 0.0;
    }

    if pipes_len < game.max_pipes && game.time_since_last_pipe_spawned > game.spawn_pipe_time {
        spawn_pipe(&mut game.pipes);
        game.time_since_last_pipe_spawned = 0.0;
    }

    game.pipes.retain(|pipe| pipe.x + pipe.width > 0);
}

fn update_player(engine: &mut console_engine::ConsoleEngine, game: &mut Game, dt: f32) {
    game.time_since_last_gravity += dt;
    game.time_since_last_jump += dt;

    draw_player(engine, game.player_x, game.player_y);

    let mut y = game.player_y;

    if game.time_since_last_gravity > game.gravity_time {
        y += 1;

        game.time_since_last_gravity = 0.0;
    }

    if
        (engine.is_key_pressed(KeyCode::Char(' ')) || engine.is_key_pressed(KeyCode::Up)) &&
        game.time_since_last_jump > game.jump_time
    {
        y -= 1;
        game.time_since_last_jump = 0.0;
    }

    game.player_y = y;
}

fn check_collision(
    first_x: i32,
    first_y: i32,
    first_width: i32,
    first_height: i32,
    second_x: i32,
    second_y: i32,
    second_width: i32,
    second_height: i32
) -> bool {
    first_x < second_x + second_width &&
        first_x + first_width > second_x &&
        first_y < second_y + second_height &&
        first_y + first_height > second_y
}

fn check_collision_with_pipes(player_x: i32, player_y: i32, pipes: &Vec<Pipe>) -> bool {
    for pipe in pipes.iter() {
        if check_collision(player_x, player_y, 1, 1, pipe.x, pipe.y, pipe.width, pipe.height) {
            return true;
        }
    }

    false
}

fn check_collision_with_screen(player_x: i32, player_y: i32) -> bool {
    player_x < 0 || player_x >= (WIDTH as i32) || player_y < 0 || player_y >= (HEIGHT as i32)
}

fn check_collision_with_gap(player_x: i32, player_y: i32, pipes: &Vec<Pipe>) -> bool {
    for pipe in pipes.iter() {
        if
            player_x == pipe.x &&
            ((player_y < pipe.y + pipe.height && player_y > pipe.y + MIN_PIPE_GAP) ||
                (player_y > pipe.y + pipe.height && player_y < pipe.y + pipe.height + MIN_PIPE_GAP))
        {
            return true;
        }
    }

    false
}

fn handle_collision(game: &mut Game) {
    game.score = 0;
    game.pipes.clear();
    game.player_x = 0;
    game.player_y = (HEIGHT as i32) / 2;
}

fn handle_collisions(game: &mut Game, dt: f32) {
    game.last_score_time += dt;

    if check_collision_with_pipes(game.player_x, game.player_y, &game.pipes) {
        handle_collision(game);
    }

    if check_collision_with_screen(game.player_x, game.player_y) {
        handle_collision(game);
    }

    if
        check_collision_with_gap(game.player_x, game.player_y, &game.pipes) &&
        game.last_score_time > game.update_score_time
    {
        game.score += 1;
        game.last_score_time = 0.0;
    }
}

fn save_score(score: i32) -> std::io::Result<()> {
    let mut file = File::create("data.bin")?;
    file.write_i32::<LittleEndian>(score)?;
    Ok(())
}

fn read_score() -> std::io::Result<i32> {
    let mut file = File::open("data.bin")?;
    let score = file.read_i32::<LittleEndian>()?;
    Ok(score)
}

fn handle_save_score(game: &mut Game) {
    if game.score > game.high_score {
        game.high_score = game.score;
    }

    match save_score(game.high_score) {
        Ok(_) => {}
        Err(e) => {
            println!("Error saving score: {}", e);
        }
    }
}

fn main() {
    let mut engine = console_engine::ConsoleEngine::init(WIDTH, HEIGHT, 60).unwrap();

    let score = match read_score() {
        Ok(score) => score,
        Err(_) => 0,
    };

    let mut game = Game {
        player_x: 0,
        player_y: (HEIGHT as i32) / 2,
        score: 0,
        high_score: score,
        pipes: vec![],
        spawn_pipe_time: 3.0,
        time_since_last_pipe_spawned: 0.0,
        pipe_speed: 2.0,
        time_since_last_pipe_moved: 0.0,
        max_pipes: 5,
        gravity_time: 1.0,
        time_since_last_gravity: 0.0,
        jump_time: 0.25,
        time_since_last_jump: 0.0,
        last_score_time: 0.0,
        update_score_time: 2.0,
    };

    let mut state = GameState::Menu;

    let mut last_frame_time = std::time::Instant::now();
    let mut dt;

    loop {
        let now = std::time::Instant::now();
        dt = (now - last_frame_time).as_secs_f32();
        engine.wait_frame();
        engine.clear_screen();

        match state {
            GameState::Menu => {
                draw_menu(&mut engine, game.high_score);

                if engine.is_key_pressed(KeyCode::Enter) {
                    state = GameState::Playing;
                }
            }
            GameState::Playing => {
                handle_collisions(&mut game, dt);

                update_pipes(&mut engine, &mut game, dt);
                update_player(&mut engine, &mut game, dt);

                draw_score(&mut engine, game.score);

                if engine.is_key_pressed(KeyCode::Char('p')) || engine.is_key_pressed(KeyCode::Esc) {
                    state = GameState::Paused;
                }
            }
            GameState::Paused => {
                draw_pause(&mut engine, game.high_score);

                if engine.is_key_pressed(KeyCode::Enter) {
                    state = GameState::Playing;
                }

                if engine.is_key_pressed(KeyCode::Char('q')) {
                    state = GameState::Menu;
                }

                handle_save_score(&mut game);
            }
        }

        if engine.is_key_pressed(KeyCode::Char('q')) {
            break;
        }

        engine.draw();
        last_frame_time = now;
    }

    handle_save_score(&mut game);
}
