extern crate image;
extern crate glob;

use glob::glob;
use serde::{Serialize, Deserialize};

use std::fs::File;
use std::io::Read;
use std::error::Error;

#[derive(Serialize, Debug)]
pub struct MapCell {
    pub wall_tex: u32,
    pub floor_tex: u32,
    pub ceil_tex: u32,
}

// JSON definition of map. Gets transformed into WorldMap by combining the 3 grids into 1 cell vector
#[derive(Serialize, Deserialize, Debug)]
struct WorldMapJSON {
    pub height: u32,
    pub width: u32,
    wall_grid: Vec<Vec<u32>>,
    floor_grid: Vec<Vec<u32>>,
    ceil_grid: Vec<Vec<u32>>,
}

#[derive(Serialize, Debug)]
pub struct WorldMap {
    pub height: u32,
    pub width: u32,
    grid: Vec<MapCell>,
}

impl WorldMap {
    pub fn load_map(mapname: &str) -> Result<WorldMap, Box<dyn Error>> {
        let path = format!("./data/maps/{}/{}.json", mapname, mapname);
        let mut file = File::open(path)?;
        let mut data = String::new();
        file.read_to_string(&mut data)?;

        let map_json: WorldMapJSON = serde_json::from_str(&data)?;
        let mut map = WorldMap{
            height: map_json.height,
            width: map_json.width,
            grid: vec![],
        };
        for i in 0..map_json.width as usize {
            for j in 0..map_json.height as usize {
                map.grid.push(
                    MapCell {
                        wall_tex: map_json.wall_grid[i][j],
                        floor_tex: map_json.floor_grid[i][j],
                        ceil_tex: map_json.ceil_grid[i][j],
                    },
                );
            }
        }

        return Ok(map);
    }

    pub fn get_cell(&self, x: u32, y: u32) -> &MapCell {
        return &self.grid[(y * self.height + x) as usize];
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
