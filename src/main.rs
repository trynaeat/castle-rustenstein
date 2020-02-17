#![windows_subsystem = "windows"]

extern crate cgmath;
extern crate sdl2;
mod data;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::pixels::PixelFormatEnum;
use sdl2::rect::Rect;
use sdl2::rect::Point;
use sdl2::render::Texture;
use sdl2::video::WindowContext;
use std::collections::HashSet;
use std::collections::HashMap;
use std::time::Duration;
use std::str;

use cgmath::Vector2;
use cgmath::Vector3;
use cgmath::Angle;

const TEX_WIDTH: u32 = 64;
const TEX_HEIGHT: u32 = 64;
const SCREEN_WIDTH: i32 = 640;
const SCREEN_HEIGHT: i32 = 480;
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
        .resizable()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().build().unwrap();
    canvas.set_logical_size(SCREEN_WIDTH as u32, SCREEN_HEIGHT as u32);
    // Load textures
    // Wall/Floor textures
    let texture_bits = crate::data::get_textures_from_file().unwrap();
    let raw_floor = &texture_bits[1];
    let creator = canvas.texture_creator();
    let mut textures: Vec<Texture> = vec![];
    let mut dark_textures: Vec<Texture> = vec![];
    let mut floor_texture = creator.create_texture_streaming(PixelFormatEnum::RGBA32, SCREEN_WIDTH as u32, SCREEN_HEIGHT as u32).unwrap();
    for i in texture_bits.iter() {
        let mut texture = creator.create_texture_static(PixelFormatEnum::RGBA32, TEX_WIDTH, TEX_HEIGHT).unwrap();
        let mut dark_texture = creator.create_texture_static(PixelFormatEnum::RGBA32, TEX_WIDTH, TEX_HEIGHT).unwrap();
        texture.update(None, &i, (TEX_WIDTH * 4) as usize).unwrap();
        textures.push(texture);
        // Divide color by 2 for dark texture
        let mut dark_bits = vec![];
        for byte in i {
            dark_bits.push(byte / 2);
        }
        dark_texture.update(None, &dark_bits, (TEX_WIDTH * 4) as usize).unwrap();
        dark_textures.push(dark_texture);
    }

    // Font textures
    let font_textures = generate_font_textures(&creator);

    canvas.clear();
    let mut event_pump = sdl_context.event_pump().unwrap();
    // Time counter for last frame
    let mut old_time: u32 = 0;
    let mut frames = 0;
    let mut fps = 0.0;
    'running: loop {
        // Clear screen
        canvas.set_draw_color(Color::RGB(128, 128, 128));
        canvas.clear();
        // Draw floor
        // Draw ceiling
        render_floor(&mut canvas, &player, &mut floor_texture, raw_floor);
        // render_ceiling(&mut canvas);
        // Perform raycasting
        render_walls(&mut canvas, &player, &world_map, &textures, &dark_textures, &mut floor_texture);
        // Get frame time
        let time = sdl_context.timer().unwrap().ticks();
        let frame_time = (time - old_time) as f64 / 1000.0; // in seconds
        old_time = time;
        // Draw FPS counter
        if frames % 30 == 0 {
            fps = get_fps(frame_time);
        }
        draw_fps(&mut canvas, fps, &font_textures);
        // Read keyboard state and move the player/camera accordingly
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
        frames += 1;
    }

    pub fn draw_fps(canvas: &mut sdl2::render::Canvas<sdl2::video::Window>, fps: f64, font_textures: &HashMap<char, Texture>) {
        render_string(&format!("fps: {0:.1}", fps), Rect::new(30, 30, 20, 35), canvas, font_textures);
        let m = canvas.window_mut();
    }

    pub fn get_fps (frame_time: f64) -> f64 {
        return 1.0 / frame_time;
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

    fn render_walls(canvas: &mut sdl2::render::Canvas<sdl2::video::Window>, player: &Player, world_map: &[[u32; 24]; 24], textures: &Vec<Texture>, dark_textures: &Vec<Texture>, floor_texture: &mut Texture) {
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
            let mut tex_strip_start = 0;
            let mut tex_strip_height = TEX_HEIGHT as i32;
            let screen_wall_ratio = SCREEN_HEIGHT as f64 / line_height as f64;
            // Trim texture region to only be the portion visible in the viewcreen, if wall > screen height
            if screen_wall_ratio < 1.0 {
                let tex_y_drawn = (screen_wall_ratio * TEX_HEIGHT as f64) as i32;
                let offset = TEX_HEIGHT as i32 - tex_y_drawn;
                tex_strip_start += offset / 2;
                tex_strip_height -= offset;
            }
            canvas.copy(
                texture, 
                Rect::new(tex_x as i32, tex_strip_start, 1, tex_strip_height as u32),
                Rect::new(i as i32, SCREEN_HEIGHT - draw_end, 1, (draw_end - draw_start) as u32),
            ).unwrap();
        }
    }

    fn render_floor(canvas: &mut sdl2::render::Canvas<sdl2::video::Window>, player: &Player, floor_texture: &mut Texture, raw_floor: &[u8]) {
        let new_data = &mut vec![128; (SCREEN_WIDTH * SCREEN_HEIGHT * 4) as usize];
        let left_ray = player.dir - player.camera_plane;
        let right_ray = player.dir + player.camera_plane;
        for y in SCREEN_HEIGHT / 2..SCREEN_HEIGHT {
            // Current y distance to middle of screen
            let p = y - SCREEN_HEIGHT / 2;
            // Height of camera
            let pos_z = 0.5 * SCREEN_HEIGHT as f64;
            // Horizontal distance from camera to floor for current row
            let row_dist = pos_z / p as f64;

            let floor_step = (right_ray - left_ray) * row_dist / SCREEN_WIDTH as f64;

            let mut floor_pos = Vector2::new(
                player.pos.x + row_dist * left_ray.x,
                player.pos.y + row_dist * left_ray.y,
            );

            for x in 0..SCREEN_WIDTH {
                // Take interger portion for cell #
                let floor_cell = Vector2::new(
                    floor_pos.x as i32,
                    floor_pos.y as i32,
                );

                // Get fractional part of coordiate (how far in cell)
                let tex_x = (TEX_WIDTH as f64 * (floor_pos.x - floor_cell.x as f64)) as u32 & (TEX_WIDTH - 1);
                let tex_y = (TEX_HEIGHT as f64 * (floor_pos.y - floor_cell.y as f64)) as u32 & (TEX_HEIGHT - 1);

                floor_pos = floor_pos + floor_step;

                unsafe {
                    let tex_start = &raw_floor[((TEX_WIDTH * tex_y + tex_x) * 4) as usize] as *const u8;
                    let floor_start = &mut new_data[((y * SCREEN_WIDTH + x) * 4) as usize] as *mut u8;
                    std::ptr::copy(tex_start, floor_start, 4);
                }
                // let color = raw_floor[tex_start as usize];
                // new_data[(y * SCREEN_WIDTH + x) as usize] = color;
            }
        }

        // TODO get floor pixels, edit texture here
        floor_texture.with_lock(None, |mut dat, pitch| {
            dat.copy_from_slice(new_data);
        });
        // floor_texture.update(None, &new_data, (SCREEN_WIDTH * 4) as usize).unwrap();

        canvas.copy(floor_texture, Rect::new(0, SCREEN_HEIGHT / 2, SCREEN_WIDTH as u32, SCREEN_HEIGHT as u32 / 2), Rect::new(0, SCREEN_HEIGHT / 2, SCREEN_WIDTH as u32, SCREEN_HEIGHT as u32 / 2));
    }

    fn render_ceiling (canvas: &mut sdl2::render::Canvas<sdl2::video::Window>) {
        canvas.set_draw_color(Color::RGB(64, 64, 64));
        canvas.fill_rect(Rect::new(
            0,
            0,
            SCREEN_WIDTH as u32,
            SCREEN_HEIGHT as u32 / 2,
        )).unwrap();
    }

    fn generate_font_textures (texture_creator: &sdl2::render::TextureCreator<WindowContext>) -> HashMap<char, Texture> {
        let mut textures = HashMap::new();
        let ttf = sdl2::ttf::init().unwrap();
        let font = ttf.load_font("./src/data/fonts/ARIAL.TTF", 35).unwrap();
        let valid_chars = "0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ .:";
        for c in valid_chars.chars() {
            let surface = font.render(str::from_utf8(&[(c as u8)]).unwrap()).blended(Color::RGBA(255, 255, 0, 255)).unwrap();
            let texture = texture_creator.create_texture_from_surface(surface).unwrap();
            textures.insert(c, texture);
        }

        return textures;
    }

    fn render_string (s: &str, position: Rect, canvas: &mut sdl2::render::Canvas<sdl2::video::Window>, font_textures: &HashMap<char, Texture>) {
        let mut start_x = position.x;
        for c in s.chars() {
            if c == ' ' {
                start_x += 10;
                continue;
            }
            let width = &font_textures.get(&c).unwrap().query().width;
            canvas.copy(&font_textures.get(&c).unwrap(), None, Rect::new(start_x, position.y, position.width(), position.height())).unwrap();
            start_x += ((*width as i32) + 5);
        }
    }
}
