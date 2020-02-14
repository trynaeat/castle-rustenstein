extern crate cgmath;
extern crate sdl2;
mod data;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::pixels::PixelFormatEnum;
use sdl2::rect::Point;
use sdl2::rect::Rect;
use sdl2::render::Texture;
use std::collections::HashSet;
use std::time::Duration;

use cgmath::Vector2;
use cgmath::Vector3;

const TEX_WIDTH: u32 = 64;
const TEX_HEIGHT: u32 = 64;
const SCREEN_WIDTH: i32 = 1920;
const SCREEN_HEIGHT: i32 = 1080;
const WALL_HEIGHT_SCALE: i32 = 1;
const MOVE_SPEED: f64 = 4.0;
const ROT_SPEED: f64 = 2.0;

struct Player {
    pos: Vector3<f64>,
    dir: Vector2<f64>,
    camera_plane: Vector2<f64>,
}

struct MapGrid {
    x: i32,
    y: i32,
}

#[derive(PartialEq)]
enum WallSide {
    X,
    Y,
}

pub fn main() {
    // Init map
    let world_map = crate::data::load_map("./src/data/maps/map_textured.json");
    // Init Player and Camera
    let mut player = Player {
        pos: Vector3::new(22.0, 11.5, 0.0),
        dir: Vector2::new(-1.0, 0.0),
        camera_plane: Vector2::new(0.0, 0.66),
    };
    // SDL setup and loop
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem
        .window("rust-sdl2 demo", SCREEN_WIDTH as u32, SCREEN_HEIGHT as u32)
        .position_centered()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().build().unwrap();
    // Load textures
    let texture_bits = crate::data::gen_textures(TEX_WIDTH, TEX_HEIGHT);
    let creator = canvas.texture_creator();
    let mut textures: Vec<Texture> = vec![];
    let mut dark_textures: Vec<Texture> = vec![];
    for i in texture_bits.iter() {
        let mut texture = creator.create_texture_static(PixelFormatEnum::ARGB8888, TEX_WIDTH, TEX_HEIGHT).unwrap();
        let mut dark_texture = creator.create_texture_static(PixelFormatEnum::ARGB8888, TEX_WIDTH, TEX_HEIGHT).unwrap();
        let row: [u8; 64 * 64 * 4];
        unsafe {
            // Interpret 2d array of 32 bit values into 1d array of 8 bit
            row = std::mem::transmute::<[[u32; 64]; 64], [u8; 64 * 64 * 4]>(*i);
        }
        texture.update(None, &row, (TEX_WIDTH * 4) as usize).unwrap();
        textures.push(texture);
        let mut dark_bits: [[u32; 64]; 64] = [[0; 64]; 64];
        let dark_row: [u8; 64 * 64 * 4];
        // Divide color by 2 for dark texture
        for (x, row) in i.iter().enumerate() {
            for (y, bit) in row.iter().enumerate() {
                dark_bits[x as usize][y as usize] = (bit >> 1) & 8355711 as u32;
            }
        }
        unsafe {
            // Interpret 2d array of 32 bit values into 1d array of 8 bit
            dark_row = std::mem::transmute::<[[u32; 64]; 64], [u8; 64 * 64 * 4]>(dark_bits);
        }
        dark_texture.update(None, &dark_row, (TEX_WIDTH * 4) as usize).unwrap();
        dark_textures.push(dark_texture);
    }

    canvas.clear();
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
            let mut curr_grid = MapGrid {
                x: player.pos.x as i32,
                y: player.pos.y as i32,
            };
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
                WallSide::X => {
                    (curr_grid.x as f64 - player.pos.x + (1.0 - step_x as f64) / 2.0) / ray_dir.x
                }
                WallSide::Y => {
                    (curr_grid.y as f64 - player.pos.y + (1.0 - step_y as f64) / 2.0) / ray_dir.y
                }
            };
            // Calculate height of line
            let line_height =
                (WALL_HEIGHT_SCALE as f64 * SCREEN_HEIGHT as f64 / perp_wall_dist) as i32;
            // Get lowest/highest pixel to draw (drawing walls in middle of screen)
            let mut draw_start = -1 * line_height / 2 + SCREEN_HEIGHT / 2;
            if draw_start < 0 {
                draw_start = 0;
            }
            let mut draw_end = line_height / 2 + SCREEN_HEIGHT / 2;
            if draw_end >= SCREEN_HEIGHT as i32 {
                draw_end = SCREEN_HEIGHT as i32 - 1;
            }
            // Texture calculations
            let tex_num = world_map[curr_grid.x as usize][curr_grid.y as usize] - 1;

            // Exact x/y coord where it hit
            let wall_x = match side {
                WallSide::X => player.pos.y + perp_wall_dist * ray_dir.y,
                WallSide::Y => player.pos.x + perp_wall_dist * ray_dir.x,
            };
            let wall_x = wall_x - wall_x.floor();

            //x coord on the texture
            let mut tex_x = (wall_x * TEX_WIDTH as f64) as u32;
            if side == WallSide::X && ray_dir.x > 0 as f64 {
                tex_x = TEX_WIDTH - tex_x - 1;
            }
            if side == WallSide::Y && ray_dir.y < 0 as f64 {
                tex_x = TEX_WIDTH - tex_x - 1;
            }
            let texture = match side {
                WallSide::X => &textures[tex_num as usize],
                WallSide::Y => &dark_textures[tex_num as usize],
            };
            canvas.copy(
                texture, 
                Rect::new(tex_x as i32, 0, 1, TEX_HEIGHT),
                Rect::new(i as i32, SCREEN_HEIGHT - draw_end, 1, (draw_end - draw_start) as u32),
            ).unwrap();
        }
        // Get frame time
        let time = sdl_context.timer().unwrap().ticks();
        let frame_time = (time - old_time) as f64 / 1000.0; // in seconds
        old_time = time;
        draw_fps(frame_time);
        move_player(&mut player, &world_map, &event_pump, frame_time);

        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,
                _ => {}
            }
        }

        canvas.present();
        // ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }

    pub fn draw_fps(frame_time: f64) {
        // TODO: draw text
        // println!("{}", 1.0 / frame_time);
    }

    fn move_player(
        player: &mut Player,
        world_map: &[[u32; 24]; 24],
        event_pump: &sdl2::EventPump,
        frame_time: f64,
    ) {
        let move_speed = frame_time * MOVE_SPEED;
        let rot_speed = frame_time * ROT_SPEED;
        let pressed_keys: HashSet<Keycode> = event_pump
            .keyboard_state()
            .pressed_scancodes()
            .filter_map(Keycode::from_scancode)
            .collect();
        if pressed_keys.contains(&Keycode::Up) {
            let new_pos = player.pos
                + Vector3::new(player.dir.x * move_speed, player.dir.y * move_speed, 0.0);
            if world_map[new_pos.x as usize][new_pos.y as usize] == 0 {
                player.pos = new_pos;
            }
        }
        if pressed_keys.contains(&Keycode::Down) {
            let new_pos = player.pos
                - Vector3::new(player.dir.x * move_speed, player.dir.y * move_speed, 0.0);
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
                player.camera_plane.x * (-rot_speed).cos()
                    - player.camera_plane.y * (-rot_speed).sin(),
                player.camera_plane.x * (-rot_speed).sin()
                    + player.camera_plane.y * (-rot_speed).cos(),
            );
        }
    }
}
