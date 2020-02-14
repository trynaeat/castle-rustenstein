extern crate sdl2;
extern crate cgmath;
mod data;

use sdl2::pixels::Color;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::rect::Point;
use std::time::Duration;
use std::collections::HashSet;

use cgmath::Vector2;
use cgmath::Vector3;

const SCREEN_WIDTH: i32 = 1920;
const SCREEN_HEIGHT: i32 = 1080;
const WALL_HEIGHT_SCALE: i32 = 1;
const MOVE_SPEED: f64 = 4.0;
const ROT_SPEED: f64 = 2.0;

struct Player {
    pos: Vector3<f64>,
    dir: Vector2<f64>,
    camera_plane: Vector2<f64>
}

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
    let mut player = Player {
        pos: Vector3::new(22.0, 12.0, 0.0),
        dir: Vector2::new(-1.0, 0.0),
        camera_plane: Vector2::new(0.0, 0.66)
    };
    // SDL setup and loop
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
 
    let window = video_subsystem.window("rust-sdl2 demo", SCREEN_WIDTH as u32, SCREEN_HEIGHT as u32)
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
            let ray_hit_pos = camera_x * player.camera_plane;
            let ray_dir = player.dir + ray_hit_pos;
            // Which box we're in
            let mut curr_grid = MapGrid { x: player.pos.x as i32, y: player.pos.y as i32 };
            // Length of ray from any x/y side to next x/y side
            let delta_dist = Vector2::new((1.0 / ray_dir.x).abs(), (1.0 / ray_dir.y).abs());
            let step_x: i32;
            let step_y: i32;
            let mut side_dist_x: f64;
            let mut side_dist_y: f64;
            if ray_dir.x < 0.0 {
                step_x = -1;
                side_dist_x = (player.pos.x - curr_grid.x as f64) * delta_dist.x;
            } else {
                step_x = 1;
                side_dist_x = (curr_grid.x as f64 + 1.0 - player.pos.x) * delta_dist.x;
            }
            if ray_dir.y < 0.0 {
                step_y = -1;
                side_dist_y = (player.pos.y - curr_grid.y as f64) * delta_dist.y;
            } else {
                step_y = 1;
                side_dist_y = (curr_grid.y as f64 + 1.0 - player.pos.y) * delta_dist.y;
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
                WallSide::X => (curr_grid.x as f64 - player.pos.x + (1.0 - step_x as f64) / 2.0) / ray_dir.x,
                WallSide::Y => (curr_grid.y as f64 - player.pos.y + (1.0 - step_y as f64) / 2.0) / ray_dir.y,
            };
            // Calculate height of line
            let line_height = (WALL_HEIGHT_SCALE as f64 * SCREEN_HEIGHT as f64 / perp_wall_dist) as i32;
            // Get lowest/highest pixel to draw (drawing walls in middle of screen)
            let mut draw_start = -line_height / 2 + SCREEN_HEIGHT / 2;
            if draw_start < 0 {
                draw_start = 0;
            }
            let mut draw_end = line_height / 2 + SCREEN_HEIGHT / 2;
            if draw_end >= SCREEN_HEIGHT as i32 {
                draw_end = SCREEN_HEIGHT as i32 - 1;
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
        move_player(&mut player, &world_map, &event_pump, frame_time);

        for event in event_pump.poll_iter() {
            match event {
                Event::Quit {..} |
                Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    break 'running
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

    fn move_player (player: &mut Player, world_map: &[[u32; 24]; 24], event_pump: &sdl2::EventPump, frame_time: f64) {
        let move_speed = frame_time * MOVE_SPEED;
        let rot_speed = frame_time * ROT_SPEED;
        let pressed_keys: HashSet<Keycode> = event_pump.keyboard_state()
            .pressed_scancodes()
            .filter_map(Keycode::from_scancode)
            .collect();
        if pressed_keys.contains(&Keycode::Up) {
            let new_pos = player.pos + Vector3::new(player.dir.x * move_speed, player.dir.y * move_speed, 0.0);
            if world_map[new_pos.x as usize][new_pos.y as usize] == 0 {
                player.pos = new_pos;
            }
        }
        if pressed_keys.contains(&Keycode::Down) {
            let new_pos = player.pos - Vector3::new(player.dir.x * move_speed, player.dir.y * move_speed, 0.0);
            if world_map[new_pos.x as usize][new_pos.y as usize] == 0 {
                player.pos = new_pos;
            }
        }
        if pressed_keys.contains(&Keycode::Left) {
            player.dir = Vector2::new(
                player.dir.x * rot_speed.cos() - player.dir.y * rot_speed.sin(),
                player.dir.x * rot_speed.sin() + player.dir.y * rot_speed.cos(),
            );
            player.camera_plane = Vector2::new(
                player.camera_plane.x * rot_speed.cos() - player.camera_plane.y * rot_speed.sin(),
                player.camera_plane.x * rot_speed.sin() + player.camera_plane.y * rot_speed.cos(),
            );
        }
        if pressed_keys.contains(&Keycode::Right) {
            player.dir = Vector2::new(
                player.dir.x * (-rot_speed).cos() - player.dir.y * (-rot_speed).sin(),
                player.dir.x * (-rot_speed).sin() + player.dir.y * (-rot_speed).cos(),
            );
            player.camera_plane = Vector2::new(
                player.camera_plane.x * (-rot_speed).cos() - player.camera_plane.y * (-rot_speed).sin(),
                player.camera_plane.x * (-rot_speed).sin() + player.camera_plane.y * (-rot_speed).cos(),
            );
        }
    }
}
