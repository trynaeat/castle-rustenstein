extern crate sdl2;
extern crate cgmath;

use cgmath::Vector2;
use cgmath::Vector3;
use cgmath::InnerSpace;

use sdl2::render::Canvas;
use sdl2::render::Texture;
use sdl2::rect::Rect;
use sdl2::keyboard::Keycode;

use std::collections::HashSet;

use crate::data::WorldMap;
use crate::textures::TextureManager;
use crate::sprites::Entity;
use crate::sprites::SpriteManager;

const SCREEN_HEIGHT: i32 = 600;
const SCREEN_WIDTH: i32 = 800;
const TEX_WIDTH: u32 = 64;
const TEX_HEIGHT: u32 = 64;
const WALL_HEIGHT_SCALE: f64 = 1.0;
const MOVE_SPEED: f64 = 4.0;
const ROT_SPEED: f64 = 2.0;
const ACCELERATION: f64 = 0.1;
const DRAG: f64 = 3.0;
const PLAYER_RADIUS: f64 = 0.2;

struct Player {
    pos: Vector3<f64>,
    dir: Vector2<f64>,
    velocity: Vector3<f64>,
    camera_plane: Vector2<f64>,
}

#[derive(PartialEq)]
enum WallSide {
    X,
    Y,
}

#[derive(Debug)]
struct SpriteSortable<'a> {
    entity: &'a Entity<'a>,
    distance: f64,
}

pub struct Game<'a, 'b, 'c> {
    player: Player,
    world_map: WorldMap,
    texture_manager: &'a TextureManager<'a>,
    sprite_manager: &'c SpriteManager<'c>,
    floor_texture: &'b mut Texture<'a>,
    z_buffer: [f64; SCREEN_WIDTH as usize],
    entities: Vec<Entity<'c>>,
}

impl<'a, 'b, 'c> Game<'a, 'b, 'c> {
    pub fn new(map: WorldMap, manager: &'a TextureManager, s_manager: &'c SpriteManager, floor_tex: &'b mut Texture<'a>) -> Game<'a, 'b, 'c> {
        // Init Player and Camera
        let player = Player {
            pos: Vector3::new(6.5, 3.5, 0.0),
            dir: Vector2::new(-1.0, 0.0),
            velocity: Vector3::new(0.0, 0.0, 0.0),
            camera_plane: Vector2::new(0.0, 0.66),
        };
        // Initialize starting entities based on JSON definition on the map
        let init_entities = map.entities.iter().map(|e| {
            let sprite = s_manager.get_sprite(&e.sprite).unwrap();
            Entity {
                sprite: sprite,
                pos: Vector3::new(e.x, e.y, 0.0),
                dir: Vector2::new(e.dir_x, e.dir_y),
                collidable: e.collidable,
                collision_radius: e.collision_radius,
            }
        }).collect();
        Game {
            player: player,
            world_map: map,
            texture_manager: manager,
            sprite_manager: s_manager,
            floor_texture: floor_tex,
            z_buffer: [0.0; SCREEN_WIDTH as usize],
            entities: init_entities,
        }
    }

    pub fn draw(&mut self, canvas: &mut Canvas<sdl2::video::Window>) {
        self.render_floor(canvas);
        self.render_walls(canvas);
        self.render_sprites(canvas);
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
                    if f_cell > -1 {
                        // Floor
                        let tex_start = &self.texture_manager.get_raw_tex(f_cell as u32)[((TEX_WIDTH * tex_y + tex_x) * 4) as usize] as *const u8;
                        let floor_start = &mut new_data[((y * SCREEN_WIDTH + x) * 4) as usize] as *mut u8;
                        std::ptr::copy(tex_start, floor_start, 4);
                    }
                    if c_cell > -1 {
                        // Ceiling
                        let tex_start = &self.texture_manager.get_raw_tex(c_cell as u32)[((TEX_WIDTH * tex_y + tex_x) * 4) as usize] as *const u8;
                        let ceil_start = &mut new_data[(((SCREEN_HEIGHT - y) * SCREEN_WIDTH + x) * 4) as usize] as *mut u8;
                        std::ptr::copy(tex_start, ceil_start, 4);
                    }
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
            let mut perp_wall_dist = match side {
                WallSide::X => {
                    (curr_grid.x as f64 - self.player.pos.x + (1.0 - step_x as f64) / 2.0) / ray_dir.x
                }
                WallSide::Y => {
                    (curr_grid.y as f64 - self.player.pos.y + (1.0 - step_y as f64) / 2.0) / ray_dir.y
                }
            };
            // Clamp minimum distance to avoid overflow
            perp_wall_dist = perp_wall_dist.max(0.001);
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

    // Render all current "Entities" as 2d sprites
    fn render_sprites(&mut self, canvas: &mut Canvas<sdl2::video::Window>) {
        // Get all entities' sprites and sort them
        let mut sprite_buffer = vec![];
        for ent in self.entities.iter() {
            sprite_buffer.push(SpriteSortable {
                entity: ent,
                distance: (self.player.pos.x - ent.pos.x).powf(2.0) + (self.player.pos.y - ent.pos.y).powf(2.0), // Take distance without square root (doesn't matter)
            });
        }
        sprite_buffer.sort_by(|a, b| a.distance.partial_cmp(&b.distance).unwrap());
        sprite_buffer.reverse(); // Farthest to nearest

        // draw all sprites
        for s in sprite_buffer {
            let sprite = s.entity;
            let rel_pos = sprite.pos - self.player.pos;
            //transform sprite with the inverse camera matrix
             // [ planeX   dirX ] -1                                       [ dirY      -dirX ]
            // [               ]       =  1/(planeX*dirY-dirX*planeY) *   [                 ]
             // [ planeY   dirY ]                                          [ -planeY  planeX ]
            
            let inv_det = 1.0 / (self.player.camera_plane.x * self.player.dir.y - self.player.dir.x * self.player.camera_plane.y);
            let transform_x = inv_det * (self.player.dir.y * rel_pos.x - self.player.dir.x * rel_pos.y);
            let mut transform_y = inv_det * ((-self.player.camera_plane.y) * rel_pos.x + self.player.camera_plane.x * rel_pos.y); // depth of sprite from camera
            // Clamp transform_y if ~= 0 to prevent overflows
            if transform_y.abs() < 0.0001 {
                if transform_y < 0.0 {
                    transform_y = -0.0001;
                } else {
                    transform_y = 0.0001;
                }
            }

            let mov_screen = (sprite.sprite.v_move as f64 / transform_y) as i32; // User defined sprite offset
            let sprite_screen_x = ((SCREEN_WIDTH / 2) as f64 * (1.0 + transform_x / transform_y)) as i32;

            // height of sprite on screen
            let sprite_height = (((SCREEN_HEIGHT as f64 / transform_y) * sprite.sprite.v_scale) as i32).abs();
            let sprite_width = ((SCREEN_HEIGHT as f64 / transform_y) * sprite.sprite.u_scale) as i32;
            // clamp draw start into screen with max/min
            let draw_start = Vector2::new(((-sprite_width) / 2 + sprite_screen_x).max(0), ((-sprite_height) / 2 + SCREEN_HEIGHT / 2 + mov_screen).max(0));
            let draw_end = Vector2::new((sprite_width / 2 + sprite_screen_x).min(SCREEN_WIDTH - 1), (sprite_height / 2 + SCREEN_HEIGHT / 2 + mov_screen).min(SCREEN_HEIGHT - 1));
            // Draw every vertical stripe of sprite
            for x in draw_start.x..draw_end.x {
                let tex_x: i32;
                if sprite.sprite.rotating {
                    // Get angle between entity's direction and player direction
                    let diff_ray = sprite.pos - self.player.pos;
                    let dir_angle = sprite.dir.y.atan2(sprite.dir.x);
                    // let diff_ray = Vector2::new(diff_ray.x, diff_ray.y) + sprite.dir;
                    let mut angle = diff_ray.y.atan2(diff_ray.x) + dir_angle;
                    if angle > std::f64::consts::PI {
                        angle -= 2.0 * std::f64::consts::PI;
                    }
                    if angle <= -std::f64::consts::PI {
                        angle += 2.0 * std::f64::consts::PI;
                    }
                    tex_x = ((x - (-sprite_width / 2 + sprite_screen_x)) * (sprite.sprite.width / 8) as i32 / sprite_width) as i32
                    + SpriteManager::get_sprite_x_offset(sprite.sprite.width, sprite.sprite.height, angle);
                } else {
                    tex_x = ((x - (-sprite_width / 2 + sprite_screen_x)) * sprite.sprite.width as i32 / sprite_width) as i32;
                }
                //1) it's in front of camera plane
                //2) it's on the screen (left)
                //3) it's on the screen (right)
                //4) ZBuffer, with perpendicular distance
                if transform_y > 0.0 && x > 0 && x < SCREEN_WIDTH && transform_y < self.z_buffer[x as usize] {
                    let height = match sprite.sprite.rotating {
                        false => sprite.sprite.height,
                        true => sprite.sprite.height / 7,
                    };
                    canvas.copy(
                        self.sprite_manager.get_texture(&sprite.sprite.tex_id).unwrap(),
                        Rect::new(tex_x, 0 as i32, 1, height),
                        Rect::new(x, SCREEN_HEIGHT - (draw_end.y + mov_screen), 1, sprite_height as u32)
                    ).unwrap();
                }
            }
        }
    }

    pub fn move_player(
        &mut self,
        event_pump: &sdl2::EventPump,
        frame_time: f64,
    ) {
        let rot_speed = frame_time * ROT_SPEED;
        let pressed_keys: HashSet<Keycode> = event_pump
            .keyboard_state()
            .pressed_scancodes()
            .filter_map(Keycode::from_scancode)
            .collect();
        if pressed_keys.contains(&Keycode::Up) {
            let dir_normal = self.player.dir.normalize();
            let dir = Vector3::new(dir_normal.x, dir_normal.y, 0.0);
            let new_velocity = self.player.velocity + ACCELERATION * frame_time * dir;
            if new_velocity.magnitude() < MOVE_SPEED * frame_time {
                self.player.velocity = new_velocity;
            }
        }
        if pressed_keys.contains(&Keycode::Down) {
            let dir_normal = self.player.dir.normalize();
            let dir = Vector3::new(dir_normal.x, dir_normal.y, 0.0);
            let new_velocity = self.player.velocity - ACCELERATION * frame_time * dir;
            if new_velocity.magnitude() < MOVE_SPEED * frame_time {
                self.player.velocity = new_velocity;
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

        // Apply drag
        self.player.velocity -= self.player.velocity * DRAG * frame_time;
        // Do sprite collision detection
        for e in self.entities.iter() {
            if e.collidable {
                let diff = e.pos - self.player.pos;
                if diff.magnitude() < e.collision_radius {
                    self.player.velocity = -1.0 * self.player.velocity;
                }
            }
        }
        // Draw a line from current -> direction * hitbox radius
        let collision_point = self.player.pos + self.player.velocity.normalize() * PLAYER_RADIUS;
        // Do wall collision detection
        // Move player based on current velocity
        if self.world_map.get_cell(collision_point.x as u32, collision_point.y as u32).wall_tex == 0 {
            // noop
        } else {
            // Compare current/new cells, update velocity according to which way we hit the wall
            let new_x = collision_point.x as u32;
            let curr_x = self.player.pos.x as u32;
            // We hit a wall from the x direction
            if curr_x != new_x {
                self.player.velocity = Vector3::new(0.0, self.player.velocity.y, 0.0);
            } else {
                // we hit it from the y direction
                self.player.velocity = Vector3::new(self.player.velocity.x, 0.0, 0.0);
            }
        }

        let new_pos = self.player.pos + self.player.velocity;
        self.player.pos = new_pos;
    }
}