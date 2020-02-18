extern crate image;
extern crate glob;

use glob::glob;
use serde::{Serialize, Deserialize};

use std::fs::File;
use std::io::Read;
use std::error::Error;

#[derive(Serialize, Deserialize, Debug)]
pub struct WorldMap {
    pub height: u32,
    pub width: u32,
    wall_grid: Vec<Vec<u32>>,
    floor_grid: Vec<Vec<u32>>,
    ceil_grid: Vec<Vec<u32>>,
}

impl WorldMap {
    pub fn load_map(mapname: &str) -> Result<WorldMap, Box<dyn Error>> {
        let path = format!("./data/maps/{}/{}.json", mapname, mapname);
        let mut file = File::open(path)?;
        let mut data = String::new();
        file.read_to_string(&mut data)?;

        let map: WorldMap = serde_json::from_str(&data)?;

        return Ok(map);
    }

    // pub fn load_map(mapname: &str) -> Result<WorldMap, Box<dyn Error>> {
    //     return Ok(WorldMap{
    //         height: 24,
    //         width: 24,
    //         wall_grid: vec![],
    //         floor_grid: vec![],
    //         ceil_grid: vec![],
    //     });
    // }

    pub fn get_wall_cell(&self, x: u32, y: u32) -> u32 {
        return self.wall_grid[x as usize][y as usize];
    }

    pub fn get_floor_cell(&self, x: u32, y: u32) -> u32 {
        return self.floor_grid[x as usize][y as usize];
    }

    pub fn get_ceil_cell(&self, x: u32, y: u32) -> u32 {
        return self.ceil_grid[x as usize][y as usize];
    }
}

// Procedurally generate some basic textures
// pub fn gen_textures(tex_width: u32, tex_height: u32) -> [[[u32; 64]; 64]; 8] {
//     let mut textures = [[[0; 64]; 64]; 8];
//     for x in 0..tex_width {
//         for y in 0..tex_height {
//             let xorcolor = (x * 256 / tex_width) ^ (y * 256 / tex_height);
//             //int xcolor = x * 256 / texWidth;
//             let ycolor = y * 256 / tex_height;
//             let xycolor = y * 128 / tex_height + x * 128 / tex_width;
//             textures[0][y as usize][x as usize] =
//                 65536 * 254 * ((x != y && x != tex_width - y) as u32); //flat red texture with black cross
//             textures[1][y as usize][x as usize] = xycolor + 256 * xycolor + 65536 * xycolor; //sloped greyscale
//             textures[2][y as usize][x as usize] = 256 * xycolor + 65536 * xycolor; //sloped yellow gradient
//             textures[3][y as usize][x as usize] = xorcolor + 256 * xorcolor + 65536 * xorcolor; //xor greyscale
//             textures[4][y as usize][x as usize] = 256 * xorcolor; //xor green
//             textures[5][y as usize][x as usize] =
//                 65536 * 192 * ((x % 16 != 0) && (y % 16 != 0)) as u32; //red bricks
//             textures[6][y as usize][x as usize] = 65536 * ycolor; //red gradient
//             textures[7][y as usize][x as usize] = 128 + 256 * 128 + 65536 * 128;
//             //flat grey texture
//         }
//     }

//     return textures;
// }

pub fn get_textures_from_file() -> Result<Vec<Vec<u8>>, Box<dyn Error>> {
    let mut textures = vec![];
    let mut v = glob("./data/textures/walls/*.png")?
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
