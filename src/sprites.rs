extern crate image;
extern crate glob;

use crate::animation::Animation;

use image::GenericImageView;

use std::fs::File;
use std::io::Read;
use std::collections::HashMap;
use std::error::Error;

use sdl2::render::BlendMode;
use sdl2::render::Texture;
use sdl2::render::TextureCreator;
use sdl2::pixels::PixelFormatEnum;

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

#[derive(Debug)]
pub struct Entity<'a> {
    pub sprite: &'a Sprite,
    pub pos: Vector3<f64>,
    pub dir: Vector2<f64>,
    pub collidable: bool,
    pub collision_radius: f64,
    pub animation: Option<Animation>,
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

impl Sprite {
    // Get starting point for correct rotated sprite from a sheet based on angle
    // Assumes 8 equally spaced sprites per row on the sheet.
    pub fn get_x_offset (&self, angle: f64) -> i32 {
        let step = 2.0 * std::f64::consts::PI / 8.0;
        let step_num = ((angle + std::f64::consts::PI) / step) as i32;
        let img_step = self.width as i32 / 8;

        img_step * step_num
    }
}
