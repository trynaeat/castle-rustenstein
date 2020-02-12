// use serde::{Deserialize};
use std::fs::File;
use std::io::Read;

pub fn load_map(filename: &str) -> [[u32; 24]; 24] {
    let mut file = File::open(filename).unwrap();
    let mut data = String::new();
    file.read_to_string(&mut data).unwrap();

    return serde_json::from_str(&data).unwrap();
}