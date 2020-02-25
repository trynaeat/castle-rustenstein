extern crate glob;

use glob::glob;

use serde::{Serialize, Deserialize};

use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::error::Error;
use std::convert::From;

#[derive(Clone, Debug)]
pub struct Animation {
    pub name: String,
    pub do_loop: bool,
    pub curr_frame: usize,
    frames: Vec<AnimationFrame>,
}

#[derive(Clone, Debug)]
pub struct AnimationFrame {
    pub x_pos: u32, // X position of frame in sprite sheet
    pub y_pos: u32, // Y position of frame in sprite sheet
    pub duration: f64, // Duration in seconds
    pub time_remaining: f64, // Current time remaining in frame
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AnimationJSON {
    name: String,
    do_loop: bool,
    frames: Vec<AnimationFrameJSON>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AnimationFrameJSON {
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
            let animation_json: AnimationJSON = serde_json::from_str(&data)?;
            let animation = Animation::from(animation_json);
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

impl From<AnimationFrameJSON> for AnimationFrame {
    fn from(json: AnimationFrameJSON) -> Self {
        AnimationFrame {
            x_pos: json.x_pos,
            y_pos: json.y_pos,
            duration: json.duration,
            time_remaining: json.duration,
        }
    }
}

impl From<AnimationJSON> for Animation {
    fn from(json: AnimationJSON) -> Self {
        Animation {
            name: json.name,
            do_loop: json.do_loop,
            curr_frame: 0,
            frames: json.frames.iter().map(|f| { AnimationFrame::from(f.clone()) }).collect(),
        }
    }
}

impl Animation {
    pub fn get_current_frame (&mut self) -> &mut AnimationFrame {
        return &mut self.frames[self.curr_frame];
    }

    pub fn get_current_frame_immut (&self) -> &AnimationFrame {
        return &self.frames[self.curr_frame];
    }

    pub fn get_num_frames (&self) -> usize {
        return self.frames.len();
    }
}