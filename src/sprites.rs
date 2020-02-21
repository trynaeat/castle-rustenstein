extern crate image;
extern crate glob;

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

const SPRITE_WIDTH: u32 = 64;
const SPRITE_HEIGHT: u32 = 64;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Sprite {
    pub name: String,
    pub tex_id: String,
}

#[derive(Debug)]
pub struct Entity<'a> {
    pub sprite: &'a Sprite,
    pub pos: Vector3<f64>,
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
                let sprite: Sprite = serde_json::from_str(&data)?;

                // Init sprite_textures map
                let sprite_name = &sprite.name;
                let path = format!("./data/textures/sprites/{}.png", sprite_name);
                let img = image::open(path)?;
                let img_raw = img.to_rgba().into_vec();
                let mut texture = creator.create_texture_static(PixelFormatEnum::RGBA32, SPRITE_WIDTH, SPRITE_HEIGHT).unwrap();
                texture.update(None, &img_raw, (SPRITE_WIDTH * 4) as usize)?;
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
