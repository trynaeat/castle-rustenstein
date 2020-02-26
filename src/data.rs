extern crate image;
extern crate glob;

use glob::glob;
use serde::{Serialize, Deserialize};

use std::fs::File;
use std::io::Read;
use std::error::Error;

// Note all textures are 1-indexed since 0 is special
#[derive(Serialize, Debug)]
pub struct MapCell {
    pub wall_tex: i32,
    pub floor_tex: i32,
    pub ceil_tex: i32,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct EntityJSON {
    pub name: String, // Name of entity template
    pub x: f64,
    pub y: f64,
    #[serde(default)]
    pub dir_x: f64,
    #[serde(default)]
    pub dir_y: f64,
    #[serde(default)]
    pub animation: String,
}

// JSON definition of map. Gets transformed into WorldMap by combining the 3 grids into 1 cell vector
#[derive(Serialize, Deserialize, Debug)]
struct WorldMapJSON {
    pub height: u32,
    pub width: u32,
    wall_grid: Vec<Vec<i32>>,
    floor_grid: Vec<Vec<i32>>,
    ceil_grid: Vec<Vec<i32>>,
    pub entities: Vec<EntityJSON>,
}

#[derive(Serialize, Debug)]
pub struct WorldMap  {
    pub height: u32,
    pub width: u32,
    grid: Vec<MapCell>,
    pub entities: Vec<EntityJSON>,
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
            entities: map_json.entities,
        };
        for i in 0..map_json.height as usize {
            for j in (0..map_json.width as usize).rev() {
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
        return &self.grid[(y * self.width + x) as usize];
    }
}
