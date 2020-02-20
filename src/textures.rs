extern crate image;
extern crate glob;

use std::error::Error;

use sdl2::render::Texture;
use sdl2::render::TextureCreator;
use sdl2::pixels::PixelFormatEnum;

use glob::glob;

use image::GenericImageView;

pub struct TextureManager<'a> {
    raw_textures: Vec<Vec<u8>>,
    textures: Vec<Texture<'a>>,
    dark_textures: Vec<Texture<'a>>,
}

impl<'a> TextureManager<'a> {
    pub fn new() -> TextureManager<'a> {
        TextureManager {
            raw_textures: vec![],
            textures: vec![],
            dark_textures: vec![],
        }
    }

    pub fn init(&mut self, creator: &'a TextureCreator<sdl2::video::WindowContext>) -> Result<&Self, Box<dyn Error>> {
        let tex_paths = glob("./data/textures/walls/*.png")?
            .filter_map(Result::ok);
        for path in tex_paths {
            let img = image::open(path)?;
            let img_raw = img.to_rgba().into_vec();
            let dim = img.dimensions(); // (width, height)
            let mut texture = creator.create_texture_static(PixelFormatEnum::RGBA32, dim.0, dim.1).unwrap();
            texture.update(None, &img_raw, (dim.0 * 4) as usize)?;

            // Divide color by 2 for dark texture
            let mut dark_bytes = vec![];
            for byte in &img_raw {
                dark_bytes.push(byte / 2);
            }
            let mut dark_texture = creator.create_texture_static(PixelFormatEnum::RGBA32, dim.0, dim.1).unwrap();
            dark_texture.update(None, &dark_bytes, (dim.0 * 4) as usize).unwrap();
            // Push texture to each vector
            self.raw_textures.push(img_raw);
            self.textures.push(texture);
            self.dark_textures.push(dark_texture);
        }

        return Ok(self);
    }

    pub fn get_tex(&self, index: u32) -> &Texture {
        &self.textures[index as usize]
    }

    pub fn get_dark_tex(&self, index: u32) -> &Texture {
        &self.dark_textures[index as usize]
    }

    pub fn get_raw_tex(&self, index: u32) -> &Vec<u8> {
        &self.raw_textures[index as usize]
    }
}
