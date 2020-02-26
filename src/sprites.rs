extern crate image;
extern crate glob;

use crate::animation::Animation;
use crate::animation::AnimationManager;

use image::GenericImageView;

use std::fs::File;
use std::io::Read;
use std::collections::HashMap;
use std::error::Error;

use sdl2::render::BlendMode;
use sdl2::render::Texture;
use sdl2::render::TextureCreator;
use sdl2::pixels::PixelFormatEnum;
use sdl2::rect::Rect;

use glob::glob;
use serde::{Serialize, Deserialize};

use cgmath::Vector3;
use cgmath::Vector2;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Sprite {
    pub name: String,
    pub tex_id: String,
    #[serde(default)]
    pub width: u32,
    #[serde(default)]
    pub height: u32,
    pub u_scale: f64, // horizontal scale
    pub v_scale: f64, // vertical scale
    pub u_move: i32, // horizontal move
    pub v_move: i32, // vertical move
    #[serde(default)]
    pub rotating: bool,
}

#[derive(Clone, Debug)]
pub struct Entity {
    pub id: u32,
    pub name: String,
    pub sprite: Sprite,
    pub pos: Vector3<f64>,
    pub dir: Vector2<f64>,
    pub collidable: bool,
    pub collision_radius: f64,
    pub animation: Option<Animation>,
    pub dead: bool,
}

// Template for instantiating entities
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct EntityTmpl {
    pub name: String,
    pub sprite_name: String,
    #[serde(default)]
    pub collidable: bool,
    #[serde(default)]
    pub collision_radius: f64,
}

pub struct EntityManager<'a> {
    entity_tmpls: HashMap<String, EntityTmpl>,
    sprite_manager: &'a SpriteManager<'a>,
    id_counter: u32,
}

impl<'a> EntityManager<'_> {
    pub fn new(manager: &'a SpriteManager) -> EntityManager<'a> {
        EntityManager {
            entity_tmpls: HashMap::new(),
            sprite_manager: manager,
            id_counter: 0,
        }
    }

    pub fn init(&mut self) -> Result<&Self, Box<dyn Error>> {
        let mut map = HashMap::new();
        let paths = glob("./data/entities/*.json")?
            .filter_map(Result::ok);
        for path in paths {
            let mut file = File::open(path)?;
            let mut data = String::new();
            file.read_to_string(&mut data)?;
            let ent: EntityTmpl = serde_json::from_str(&data)?;

            map.insert(ent.name.clone(), ent);
        }
        self.entity_tmpls = map;
        return Ok(self);
    }

    pub fn create_entity(&mut self, name: &str) -> Option<Entity> {
        let ent_tmpl = self.entity_tmpls.get(name)?;
        let sprite = self.sprite_manager.get_sprite(&ent_tmpl.sprite_name)?;
        let ent = Entity {
            id: self.id_counter,
            name: String::from(name),
            sprite: sprite.clone(),
            pos: Vector3::new(0.0, 0.0, 0.0),
            dir: Vector2::new(0.0, 0.0),
            collidable: ent_tmpl.collidable,
            collision_radius: ent_tmpl.collision_radius,
            animation: None,
            dead: false,
        };
        self.id_counter += 1;
        return Some(ent);
    }
}

pub struct SpriteManager<'a> {
    sprite_textures: HashMap<String, Texture<'a>>,
    sprites: HashMap<String, Sprite>,
}

impl<'a> SpriteManager<'a> {
    pub fn new() -> SpriteManager<'a> {
        SpriteManager {
            sprite_textures: HashMap::new(),
            sprites: HashMap::new(),
        }
    }

    pub fn init(&mut self, creator: &'a TextureCreator<sdl2::video::WindowContext>) -> Result<&Self, Box<dyn Error>> {
        let mut map = HashMap::new();
        let mut tex_map = HashMap::new();
        let meta_paths = glob("./data/textures/sprites/*_meta.json")?
            .filter_map(Result::ok);
        for meta in meta_paths {
                // Init 'sprites' map with all sprite metadata
                let mut file = File::open(meta)?;
                let mut data = String::new();
                file.read_to_string(&mut data)?;
                let mut sprite: Sprite = serde_json::from_str(&data)?;

                // Init sprite_textures map
                let sprite_name = &sprite.name;
                let path = format!("./data/textures/sprites/{}.png", sprite_name);
                let img = image::open(path)?;
                let dim = img.dimensions(); // (width, height)
                sprite.width = dim.0;
                sprite.height = dim.1;
                let img_raw = img.to_rgba().into_vec();
                let mut texture = creator.create_texture_static(PixelFormatEnum::RGBA32, sprite.width, sprite.height).unwrap();
                texture.update(None, &img_raw, (sprite.width * 4) as usize)?;
                texture.set_blend_mode(BlendMode::Blend);
                tex_map.insert(sprite.tex_id.clone(), texture);

                map.insert(sprite.name.clone(), sprite);
        }
        self.sprites = map;
        self.sprite_textures = tex_map;

        return Ok(self);
    }

    pub fn get_texture(&self, id: &str) -> Option<&Texture> {
        self.sprite_textures.get(id)
    }

    pub fn get_sprite(&self, id: &str) -> Option<&Sprite> {
        self.sprites.get(id)
    }
}

impl Entity {
    // Get the correct rectangle to render from the sprite sheet
    // Based on current angle to player and animation frame (if applicable)
    // x_slice: desired slice of sprite on screen
    // sprite_screen_x: sprite's starting point on screen
    // sprite_screen_width: width of sprite on screen (used to calc starting x coord on sprite sheet)
    // angle: angle between sprite's dir and player
    pub fn get_frame_rect (&self, x_slice: i32, sprite_screen_x: i32, sprite_screen_width: i32, angle: f64) -> Rect {
        let mut width = self.sprite.width; // Width of sprite
        let mut height = self.sprite.height; // Height of sprite
        let mut x = 0; // Starting x pos of sprite (top left)
        if self.sprite.rotating {
            width = width / 8;
            height = height / 7 + 1;
            if !self.dead {
                let step = 2.0 * std::f64::consts::PI / 8.0;
                let step_num = ((angle + std::f64::consts::PI) / step) as i32;
                let img_step = self.sprite.width as i32 / 8;

                x = img_step * step_num;
            }
        }

        let y = match &self.animation {
            None => 0,
            Some(a) => height * a.get_current_frame_immut().y_pos,
        } as i32;

        x = match &self.animation {
            None => x,
            Some(a) => x + (width * a.get_current_frame_immut().x_pos) as i32,
        };

        x = x + ((x_slice - (-sprite_screen_width / 2 + sprite_screen_x)) * width as i32 / sprite_screen_width) as i32;

        // Returns a vertical strip at the right location
        return Rect::new(x, y, 1, height);
    }

    // Move animation by 1 frame (if there IS an active animation)
    // 1. decrement time in current frame
    // 2. if time is up, go to next frame
    // 3. if there is no next frame, either loop or remove the animation entirely
    // frame_time: time that the last frame took in seconds
    pub fn tick_animation (&mut self, frame_time: f64) {
        match &mut self.animation {
            None => return,
            Some(a) => {
                let curr_frame = a.get_current_frame();
                curr_frame.time_remaining -= frame_time;
                if curr_frame.time_remaining <= 0.0 {
                    // Reset duration on frame in case we're looping
                    curr_frame.time_remaining = curr_frame.duration;
                    a.curr_frame += 1;
                    if a.curr_frame >= a.get_num_frames() {
                        if a.do_loop {
                            a.curr_frame = 0;
                        } else if !a.perm {
                            self.animation = None;
                        } else {
                            // Stay on last frame
                            a.curr_frame = a.get_num_frames() - 1;
                        }
                    }
                }
            }
        };
    }

    pub fn kill (&mut self, manager: &AnimationManager) {
        self.dead = true;
        self.collidable = false;
        self.animation = manager.get_animation("die");
    }

    pub fn revive (&mut self) {
        self.dead = false;
        self.animation = None;
    }
}
