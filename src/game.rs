extern crate sdl2;
extern crate cgmath;

use cgmath::Vector2;
use cgmath::Vector3;

use sdl2::render::Canvas;
use sdl2::render::Texture;
use sdl2::rect::Rect;
use sdl2::keyboard::Keycode;

use std::collections::HashSet;

use crate::data::WorldMap;
use crate::textures::TextureManager;

const SCREEN_HEIGHT: i32 = 600;
const SCREEN_WIDTH: i32 = 800;
const TEX_WIDTH: u32 = 64;
const TEX_HEIGHT: u32 = 64;
const WALL_HEIGHT_SCALE: f64 = 1.0;
const MOVE_SPEED: f64 = 4.0;
const ROT_SPEED: f64 = 2.0;

struct Player {
    pos: Vector3<f64>,
    dir: Vector2<f64>,
    camera_plane: Vector2<f64>,
}

#[derive(PartialEq)]
enum WallSide {
    X,
    Y,
}

pub struct Game<'a, 'b> {
    player: Player,
    world_map: WorldMap,
    texture_manager: &'a TextureManager<'a>,
    floor_texture: &'b mut Texture<'a>,
    z_buffer: [f64; SCREEN_WIDTH as usize],
}

impl<'a, 'b> Game<'a, 'b> {
    pub fn new(map: WorldMap, manager: &'a TextureManager, floor_tex: &'b mut Texture<'a>) -> Game<'a, 'b> {
        // Init Player and Camera
        let player = Player {
            pos: Vector3::new(6.5, 3.5, 0.0),
            dir: Vector2::new(-1.0, 0.0),
            camera_plane: Vector2::new(0.0, 0.66),
        };
        Game {
            player: player,
            world_map: map,
            texture_manager: manager,
            floor_texture: floor_tex,
            z_buffer: [0.0; SCREEN_WIDTH as usize],
        }
    }

    pub fn draw(&mut self, canvas: &mut Canvas<sdl2::video::Window>) {
        self.render_floor(canvas);
        self.render_walls(canvas);
    }

    // Actually renders the floor AND ceiling
    // Horizontally raycasts
    fn render_floor(&mut self, canvas: &mut sdl2::render::Canvas<sdl2::video::Window>) {
        let new_data = &mut vec![128; (SCREEN_WIDTH * SCREEN_HEIGHT * 4) as usize];
        let left_ray = self.player.dir - self.player.camera_plane;
        let right_ray = self.player.dir + self.player.camera_plane;
        for y in SCREEN_HEIGHT / 2..SCREEN_HEIGHT {
            // Current y distance to middle of screen
            let p = y - SCREEN_HEIGHT / 2;
            // Height of camera
            let pos_z = 0.5 * SCREEN_HEIGHT as f64;
            // Horizontal distance from camera to floor for current row
            let row_dist = pos_z / p as f64;

            let floor_step = (right_ray - left_ray) * row_dist / SCREEN_WIDTH as f64;

            let mut floor_pos = Vector2::new(
                self.player.pos.x + row_dist * left_ray.x,
                self.player.pos.y + row_dist * left_ray.y,
            );

            for x in 0..SCREEN_WIDTH {
                // Take integer portion for cell #
                let floor_cell = Vector2::new(
                    floor_pos.x as i32,
                    floor_pos.y as i32,
                );

                let f_cell = self.world_map.get_cell(floor_cell.x as u32 & (self.world_map.width - 1), floor_cell.y as u32 & (self.world_map.height - 1)).floor_tex - 1;
                let c_cell = self.world_map.get_cell(floor_cell.x as u32 & (self.world_map.width - 1), floor_cell.y as u32 & (self.world_map.height - 1)).ceil_tex - 1;

                // Get fractional part of coordiate (how far in cell)
                let tex_x = (TEX_WIDTH as f64 * (floor_pos.x - floor_cell.x as f64)) as u32 & (TEX_WIDTH - 1);
                let tex_y = (TEX_HEIGHT as f64 * (floor_pos.y - floor_cell.y as f64)) as u32 & (TEX_HEIGHT - 1);

                floor_pos = floor_pos + floor_step;

                // Yeah I gotta copy 4 bytes at a time here so for efficiency's sake we gotta go unsafe for the memcpy :O
                // One RGBA pixel = 4 bytes, so we copy 4 bytes from src texture to destination
                // Trust me...
                unsafe {
                    // Floor
                    let tex_start = &self.texture_manager.get_raw_tex(f_cell as u32)[((TEX_WIDTH * tex_y + tex_x) * 4) as usize] as *const u8;
                    let floor_start = &mut new_data[((y * SCREEN_WIDTH + x) * 4) as usize] as *mut u8;
                    std::ptr::copy(tex_start, floor_start, 4);
                    // Ceiling
                    let tex_start = &self.texture_manager.get_raw_tex(c_cell as u32)[((TEX_WIDTH * tex_y + tex_x) * 4) as usize] as *const u8;
                    let ceil_start = &mut new_data[(((SCREEN_HEIGHT - y) * SCREEN_WIDTH + x) * 4) as usize] as *mut u8;
                    std::ptr::copy(tex_start, ceil_start, 4);
                }
            }
        }
        // Faster than texture.update?
        self.floor_texture.with_lock(None, |dat, _| {
            dat.copy_from_slice(new_data);
        }).unwrap();

        canvas.copy(self.floor_texture, None, None).unwrap();
    }

    // Vertical raycast walls
    fn render_walls(&mut self, canvas: &mut sdl2::render::Canvas<sdl2::video::Window>) {
        for i in 0..SCREEN_WIDTH {
            // Calculate incoming ray position/direction
            let camera_x: f64 = 2.0 * i as f64 / SCREEN_WIDTH as f64 - 1.0;
            let ray_hit_pos = camera_x * self.player.camera_plane;
            let ray_dir = self.player.dir + ray_hit_pos;
            // Which box we're in
            let mut curr_grid = Vector2::new(
                self.player.pos.x as i32,
                self.player.pos.y as i32,
            );
            // Length of ray from any x/y side to next x/y side
            let delta_dist = Vector2::new((1.0 / ray_dir.x).abs(), (1.0 / ray_dir.y).abs());
            let step_x: i32;
            let step_y: i32;
            let mut side_dist_x: f64;
            let mut side_dist_y: f64;
            if ray_dir.x < 0.0 {
                step_x = -1;
                side_dist_x = (self.player.pos.x - curr_grid.x as f64) * delta_dist.x;
            } else {
                step_x = 1;
                side_dist_x = (curr_grid.x as f64 + 1.0 - self.player.pos.x) * delta_dist.x;
            }
            if ray_dir.y < 0.0 {
                step_y = -1;
                side_dist_y = (self.player.pos.y - curr_grid.y as f64) * delta_dist.y;
            } else {
                step_y = 1;
                side_dist_y = (curr_grid.y as f64 + 1.0 - self.player.pos.y) * delta_dist.y;
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
                if self.world_map.get_cell(curr_grid.x as u32, curr_grid.y as u32).wall_tex > 0 {
                    break;
                }
            }
            let perp_wall_dist = match side {
                WallSide::X => {
                    (curr_grid.x as f64 - self.player.pos.x + (1.0 - step_x as f64) / 2.0) / ray_dir.x
                }
                WallSide::Y => {
                    (curr_grid.y as f64 - self.player.pos.y + (1.0 - step_y as f64) / 2.0) / ray_dir.y
                }
            };
            // Save distance in z-buffer
            self.z_buffer[i as usize] = perp_wall_dist;
            // Calculate height of line
            let line_height =
                (WALL_HEIGHT_SCALE * SCREEN_HEIGHT as f64 / perp_wall_dist) as i32;
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
            let tex_num = self.world_map.get_cell(curr_grid.x as u32, curr_grid.y as u32).wall_tex - 1;

            // Exact x/y coord where it hit
            let wall_x = match side {
                WallSide::X => self.player.pos.y + perp_wall_dist * ray_dir.y,
                WallSide::Y => self.player.pos.x + perp_wall_dist * ray_dir.x,
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
                WallSide::X => self.texture_manager.get_tex(tex_num as u32),
                WallSide::Y => self.texture_manager.get_dark_tex(tex_num as u32),
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

    pub fn move_player(
        &mut self,
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
            let new_pos = self.player.pos
                + Vector3::new(self.player.dir.x * move_speed, self.player.dir.y * move_speed, 0.0);
            if self.world_map.get_cell(new_pos.x as u32, new_pos.y as u32).wall_tex == 0 {
                self.player.pos = new_pos;
            }
        }
        if pressed_keys.contains(&Keycode::Down) {
            let new_pos = self.player.pos
                - Vector3::new(self.player.dir.x * move_speed, self.player.dir.y * move_speed, 0.0);
            if self.world_map.get_cell(new_pos.x as u32, new_pos.y as u32).wall_tex == 0 {
                self.player.pos = new_pos;
            }
        }
        if pressed_keys.contains(&Keycode::Left) {
            self.player.dir = Vector2::new(
                self.player.dir.x * rot_speed.cos() - self.player.dir.y * rot_speed.sin(),
                self.player.dir.x * rot_speed.sin() + self.player.dir.y * rot_speed.cos(),
            );
            self.player.camera_plane = Vector2::new(
                self.player.camera_plane.x * rot_speed.cos() - self.player.camera_plane.y * rot_speed.sin(),
                self.player.camera_plane.x * rot_speed.sin() + self.player.camera_plane.y * rot_speed.cos(),
            );
        }
        if pressed_keys.contains(&Keycode::Right) {
            self.player.dir = Vector2::new(
                self.player.dir.x * (-rot_speed).cos() - self.player.dir.y * (-rot_speed).sin(),
                self.player.dir.x * (-rot_speed).sin() + self.player.dir.y * (-rot_speed).cos(),
            );
            self.player.camera_plane = Vector2::new(
                self.player.camera_plane.x * (-rot_speed).cos()
                    - self.player.camera_plane.y * (-rot_speed).sin(),
                self.player.camera_plane.x * (-rot_speed).sin()
                    + self.player.camera_plane.y * (-rot_speed).cos(),
            );
        }
    }
}