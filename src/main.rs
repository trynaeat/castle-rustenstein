extern crate sdl2;
extern crate cgmath;
mod data;

use sdl2::pixels::Color;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::rect::Point;
use std::time::Duration;

use cgmath::Vector2;
use cgmath::Vector3;

const SCREEN_WIDTH: u32 = 800;
const SCREEN_HEIGHT: u32 = 600;
const WALL_HEIGHT: i32 = 600;
const MOVE_SPEED: f64 = 6.0;
const ROT_SPEED: f64 = 3.0;

struct MapGrid {
    x: i32,
    y: i32
}

enum WallSide {
    X,
    Y,
}

pub fn main() {
    // Init map
    let world_map = crate::data::load_map("./src/data/maps/map.json");
    // Init Player and Camera
    let mut player_pos = Vector3::new(22.0, 12.0, 0.0);
    let mut player_dir = Vector2::new(-1.0, 0.0);
    let mut camera_plane = Vector2::new(0.0, 0.66);
    // SDL setup and loop
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
 
    let window = video_subsystem.window("rust-sdl2 demo", SCREEN_WIDTH, SCREEN_HEIGHT)
        .position_centered()
        .build()
        .unwrap();
 
    let mut canvas = window.into_canvas().build().unwrap();
 
    canvas.clear();
    canvas.present();
    let mut event_pump = sdl_context.event_pump().unwrap();
    // Time counter for last frame
    let mut old_time: u32 = 0;
    'running: loop {
        // canvas.set_draw_color(Color::RGB(i, 64, 255 - i));
        canvas.set_draw_color(Color::RGB(0, 0, 0));
        canvas.clear();
        // The rest of the game loop goes here...
        // Perform raycasting
        for i in 0..SCREEN_WIDTH {
            // Calculate incoming ray position/direction
            let camera_x: f64 = 2.0 * i as f64 / SCREEN_WIDTH as f64 - 1.0;
            let ray_hit_pos = camera_x * camera_plane;
            let ray_dir = player_dir + ray_hit_pos;
            // Which box we're in
            let mut curr_grid = MapGrid { x: player_pos.x as i32, y: player_pos.y as i32 };
            // Length of ray from any x/y side to next x/y side
            let delta_dist = Vector2::new((1.0 / ray_dir.x).abs(), (1.0 / ray_dir.y).abs());
            let step_x: i32;
            let step_y: i32;
            let mut side_dist_x: f64;
            let mut side_dist_y: f64;
            if ray_dir.x < 0.0 {
                step_x = -1;
                side_dist_x = (player_pos.x - curr_grid.x as f64) * delta_dist.x;
            } else {
                step_x = 1;
                side_dist_x = (curr_grid.x as f64 + 1.0 - player_pos.x) * delta_dist.x;
            }
            if ray_dir.y < 0.0 {
                step_y = -1;
                side_dist_y = (player_pos.y - curr_grid.y as f64) * delta_dist.y;
            } else {
                step_y = 1;
                side_dist_y = (curr_grid.y as f64 + 1.0 - player_pos.y) * delta_dist.y;
            }

            // start DDA
            let mut side: WallSide;
            loop {
                if side_dist_x < side_dist_y {
                    side_dist_x += delta_dist.x;
                    curr_grid.x += step_x;
                    side = WallSide::X;
                } else {
                    side_dist_y += delta_dist.y;
                    curr_grid.y += step_y;
                    side = WallSide::Y;
                }
                if world_map[curr_grid.x as usize][curr_grid.y as usize] > 0 {
                    break;
                }
            }
            let perp_wall_dist = match side {
                WallSide::X => (curr_grid.x as f64 - player_pos.x + (1.0 - step_x as f64) / 2.0) / ray_dir.x,
                WallSide::Y => (curr_grid.y as f64 - player_pos.y + (1.0 - step_y as f64) / 2.0) / ray_dir.y,
            };
            // Calculate height of line
            let line_height = (WALL_HEIGHT as f64 / perp_wall_dist) as i32;
            // Get lowest/highest pixel to draw (drawing walls in middle of screen)
            let mut draw_start = -line_height / 2 + WALL_HEIGHT / 2;
            if draw_start < 0 {
                draw_start = 0;
            }
            let mut draw_end = line_height / 2 + WALL_HEIGHT / 2;
            if draw_end >= WALL_HEIGHT {
                draw_end = WALL_HEIGHT - 1;
            }
            let mut color = match world_map[curr_grid.x as usize][curr_grid.y as usize] {
                1 => Color::RGB(255, 0, 0),
                2 => Color::RGB(0, 255, 0),
                3 => Color::RGB(0, 0, 255),
                _ => Color::RGB(128, 128, 0),
            };
            // Set y side to darker
            color = match side {
                WallSide::X => color,
                WallSide::Y => Color::RGB(color.r / 2, color.g / 2, color.b / 2),
            };
            canvas.set_draw_color(color);
            canvas.draw_line(Point::new(i as i32, draw_start), Point::new(i as i32, draw_end)).unwrap();
        }
        // Get frame time
        let time = sdl_context.timer().unwrap().ticks();
        let frame_time = (time - old_time) as f64 / 1000.0; // in seconds
        old_time = time;
        draw_fps(frame_time);

        // Todo properly encapsulate inputs and make this a function
        let mut read_player_input = |keycode: Keycode, frame_time: f64| {
            let move_speed = frame_time * MOVE_SPEED;
            let rot_speed = frame_time * ROT_SPEED;
            match keycode {
                Keycode::Up => { 
                    if world_map[(player_pos.x + player_dir.x * move_speed) as usize][player_pos.y as usize] == 0 {
                        player_pos = player_pos + Vector3::new(player_dir.x * move_speed, player_dir.y * move_speed, 0.0);
                    }
                },
                Keycode::Down => { player_pos = player_pos - Vector3::new(player_dir.x * move_speed, player_dir.y * move_speed, 0.0) },
                Keycode::Right => {
                    player_dir = Vector2::new(
                        player_dir.x * (-rot_speed).cos() - player_dir.y * (-rot_speed).sin(),
                        player_dir.x * (-rot_speed).sin() + player_dir.y * (-rot_speed).cos(),
                    );
                    camera_plane = Vector2::new(
                        camera_plane.x * (-rot_speed).cos() - camera_plane.y * (-rot_speed).sin(),
                        camera_plane.x * (-rot_speed).sin() + camera_plane.y * (-rot_speed).cos(),
                    );
                },
                Keycode::Left => {
                    player_dir = Vector2::new(
                        player_dir.x * rot_speed.cos() - player_dir.y * rot_speed.sin(),
                        player_dir.x * rot_speed.sin() + player_dir.y * rot_speed.cos(),
                    );
                    camera_plane = Vector2::new(
                        camera_plane.x * rot_speed.cos() - camera_plane.y * rot_speed.sin(),
                        camera_plane.x * rot_speed.sin() + camera_plane.y * rot_speed.cos(),
                    );
                }
                _ => (),
            }
        };

        for event in event_pump.poll_iter() {
            match event {
                Event::Quit {..} |
                Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    break 'running
                },
                Event::KeyDown { keycode: Some(keycode), .. } => {
                    read_player_input(keycode, frame_time);
                },
                _ => {}
            }
        }

        canvas.present();
        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }

    pub fn draw_fps(frame_time: f64) {
        // TODO: draw text
        // println!("{}", 1.0 / frame_time);
    }
}
