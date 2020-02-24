extern crate glob;

use glob::glob;

use serde::{Serialize, Deserialize};

use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::error::Error;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Animation {
    name: String,
    do_loop: bool,
    frames: Vec<AnimationFrame>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AnimationFrame {
    x_pos: u32, // X position of frame in sprite sheet
    y_pos: u32, // Y position of frame in sprite sheet
    duration: f64, // Duration in seconds
}

pub struct AnimationManager {
    animations: HashMap<String, Animation>,
}

impl AnimationManager {
    pub fn new() -> AnimationManager {
        AnimationManager {
            animations: HashMap::new(),
        }
    }

    pub fn init(&mut self) -> Result<&Self, Box<dyn Error>> {
        // Load all animation json into hashmap
        let mut map = HashMap::new();
        let anim_paths = glob("./data/animations/*.json")?
            .filter_map(Result::ok);
        for path in anim_paths {
            let mut file = File::open(path)?;
            let mut data = String::new();
            file.read_to_string(&mut data)?;
            let animation: Animation = serde_json::from_str(&data)?;
            map.insert(animation.name.clone(), animation);
        }

        self.animations = map;

        return Ok(self);
    }

    // Return a clone of the animation
    pub fn get_animation(&self, name: &str) -> Option<Animation> {
        match self.animations.get(name) {
            None => None,
            Some(anim) => Some(anim.clone()),
        }
    }
}