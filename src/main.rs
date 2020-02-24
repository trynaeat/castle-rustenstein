#![windows_subsystem = "windows"]

extern crate cgmath;
extern crate sdl2;
mod data;
mod sprites;
mod textures;
mod game;
mod animation;

use crate::game::Game;
use crate::data::WorldMap;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::pixels::PixelFormatEnum;
use sdl2::rect::Rect;
use sdl2::render::Texture;
use sdl2::video::WindowContext;

use std::collections::HashMap;
use std::str;
use std::env;

const SCREEN_WIDTH: i32 = 800;
const SCREEN_HEIGHT: i32 = 600;

pub fn main() {
    // Get map name
    let args: Vec<String> = env::args().collect();
    let mut map_name = "test_map_small";
    if args.len() > 1 {
        map_name = &args[1];
    }
    // Init map
    let world_map = WorldMap::load_map(map_name).unwrap();

    // SDL setup and loop
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem
        .window("rust-sdl2 demo", SCREEN_WIDTH as u32, SCREEN_HEIGHT as u32)
        .position_centered()
        .resizable()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().build().unwrap();
    canvas.set_logical_size(SCREEN_WIDTH as u32, SCREEN_HEIGHT as u32).unwrap();
    // Load textures
    // Wall/Floor textures
    let creator = canvas.texture_creator();
    let mut texture_manager = textures::TextureManager::new();
    texture_manager.init(&creator).unwrap();
    let mut floor_texture = creator.create_texture_streaming(PixelFormatEnum::RGBA32, SCREEN_WIDTH as u32, SCREEN_HEIGHT as u32).unwrap();

    // Load sprites
    let mut sprite_manager = sprites::SpriteManager::new();
    sprite_manager.init(&creator).unwrap();

    // Load animations
    let mut animation_manager = animation::AnimationManager::new();
    animation_manager.init().unwrap();

    // Init game
    let mut game = Game::new(world_map, &texture_manager, &sprite_manager, &animation_manager, &mut floor_texture);

    // Font textures
    let font_textures = generate_font_textures(&creator);

    canvas.clear();
    let mut event_pump = sdl_context.event_pump().unwrap();
    // Time counter for last frame
    let mut old_time: u32 = 0;
    let mut frames = 0;
    let mut fps = 0.0;
    // Buffer of wall distance for each x-stripe. Used later for sprite occlusion
    'running: loop {
        // Clear screen
        canvas.set_draw_color(Color::RGB(128, 128, 128));
        canvas.clear();
        // Render Game frame
        game.draw(&mut canvas);
        // Get frame time
        let time = sdl_context.timer().unwrap().ticks();
        let frame_time = (time - old_time) as f64 / 1000.0; // in seconds
        old_time = time;
        // Draw FPS counter
        if frames % 30 == 0 {
            fps = get_fps(frame_time);
        }
        draw_fps(&mut canvas, fps, &font_textures);
        // Read keyboard state and move the player/camera accordingly
        game.move_player(&event_pump, frame_time);

        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,
                _ => {}
            }
        }

        canvas.present();
        // ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
        frames += 1;
    }
}

pub fn draw_fps(canvas: &mut sdl2::render::Canvas<sdl2::video::Window>, fps: f64, font_textures: &HashMap<char, Texture>) {
    render_string(&format!("fps: {0:.1}", fps), Rect::new(30, 30, 20, 35), canvas, font_textures);
}

pub fn get_fps (frame_time: f64) -> f64 {
    return 1.0 / frame_time;
}

fn generate_font_textures (texture_creator: &sdl2::render::TextureCreator<WindowContext>) -> HashMap<char, Texture> {
    let mut textures = HashMap::new();
    let ttf = sdl2::ttf::init().unwrap();
    let font = ttf.load_font("./data/fonts/ARIAL.TTF", 35).unwrap();
    let valid_chars = "0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ .:";
    for c in valid_chars.chars() {
        let surface = font.render(str::from_utf8(&[(c as u8)]).unwrap()).blended(Color::RGBA(255, 255, 0, 255)).unwrap();
        let texture = texture_creator.create_texture_from_surface(surface).unwrap();
        textures.insert(c, texture);
    }

    return textures;
}

fn render_string (s: &str, position: Rect, canvas: &mut sdl2::render::Canvas<sdl2::video::Window>, font_textures: &HashMap<char, Texture>) {
    let mut start_x = position.x;
    for c in s.chars() {
        if c == ' ' {
            start_x += 10;
            continue;
        }
        let width = &font_textures.get(&c).unwrap().query().width;
        canvas.copy(&font_textures.get(&c).unwrap(), None, Rect::new(start_x, position.y, position.width(), position.height())).unwrap();
        start_x += *width as i32 + 5;
    }
}
