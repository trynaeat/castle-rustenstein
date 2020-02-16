extern crate image;
extern crate glob;

use glob::glob;

use std::fs::File;
use std::io::Read;
use std::error::Error;

pub fn load_map(filename: &str) -> [[u32; 24]; 24] {
    let mut file = File::open(filename).unwrap();
    let mut data = String::new();
    file.read_to_string(&mut data).unwrap();

    return serde_json::from_str(&data).unwrap();
}

pub fn gen_textures(tex_width: u32, tex_height: u32) -> [[[u32; 64]; 64]; 8] {
    let mut textures = [[[0; 64]; 64]; 8];
    for x in 0..tex_width {
        for y in 0..tex_height {
            let xorcolor = (x * 256 / tex_width) ^ (y * 256 / tex_height);
            //int xcolor = x * 256 / texWidth;
            let ycolor = y * 256 / tex_height;
            let xycolor = y * 128 / tex_height + x * 128 / tex_width;
            textures[0][y as usize][x as usize] =
                65536 * 254 * ((x != y && x != tex_width - y) as u32); //flat red texture with black cross
            textures[1][y as usize][x as usize] = xycolor + 256 * xycolor + 65536 * xycolor; //sloped greyscale
            textures[2][y as usize][x as usize] = 256 * xycolor + 65536 * xycolor; //sloped yellow gradient
            textures[3][y as usize][x as usize] = xorcolor + 256 * xorcolor + 65536 * xorcolor; //xor greyscale
            textures[4][y as usize][x as usize] = 256 * xorcolor; //xor green
            textures[5][y as usize][x as usize] =
                65536 * 192 * ((x % 16 != 0) && (y % 16 != 0)) as u32; //red bricks
            textures[6][y as usize][x as usize] = 65536 * ycolor; //red gradient
            textures[7][y as usize][x as usize] = 128 + 256 * 128 + 65536 * 128;
            //flat grey texture
        }
    }

    return textures;
}

pub fn get_textures_from_file() -> Result<Vec<Vec<u8>>, Box<dyn Error>> {
    let mut textures = vec![];
    let mut v = glob("./src/data/textures/walls/*.png")?
        .filter_map(Result::ok)
        .map(|p| match p.to_str() {
            Some(s) => String::from(s),
            None => String::from(""),
        })
        .collect::<Vec<_>>();
    v.sort_by(|a, b| a.cmp(b));
    for path in v {
        let img = image::open(path)?;
        textures.push(img.to_rgba().into_vec());
    }
    
    return Ok(textures);
}
