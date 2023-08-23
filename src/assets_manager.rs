use macroquad::audio::{load_sound, Sound};
use macroquad::text::{load_ttf_font, Font};
// use macroquad::audio::Sound;
use macroquad::texture::{load_texture, Texture2D};
use std::collections::HashMap;

use std::{cell::RefCell, rc::Rc};

pub struct AssetManager {
    sounds: HashMap<String, Rc<RefCell<Sound>>>,
    textures: HashMap<String, Rc<RefCell<Texture2D>>>,
    fonts: HashMap<String, Rc<RefCell<Font>>>,
}

impl AssetManager {
    pub fn new() -> Self {
        Self {
            sounds: HashMap::new(),
            textures: HashMap::new(),
            fonts: HashMap::new(),
        }
    }

    pub async fn get_texture(&mut self, name: &str) -> Option<Rc<RefCell<Texture2D>>> {
        match self.textures.get(name) {
            Some(texture) => Some((*texture).clone()),
            _ => {
                let texture = Rc::new(RefCell::new(load_texture(name).await.unwrap()));
                let t1 = texture.clone();
                self.textures.insert(name.to_string(), texture);

                Some(t1)
            }
        }
    }

    pub async fn get_sound(&mut self, name: &str) -> Option<Rc<RefCell<Sound>>> {
        match self.sounds.get(name) {
            Some(sound) => Some((*sound).clone()),
            _ => {
                let sound = Rc::new(RefCell::new(load_sound(name).await.unwrap()));
                let s1 = sound.clone();
                self.sounds.insert(name.to_string(), sound);

                Some(s1)
            }
        }
    }

    pub async fn get_font(&mut self, name: &str) -> Option<Rc<RefCell<Font>>> {
        match self.fonts.get(name) {
            Some(font) => Some((*font).clone()),
            _ => {
                let font = Rc::new(RefCell::new(load_ttf_font(name).await.unwrap()));
                let f1 = font.clone();
                self.fonts.insert(name.to_string(), font);

                Some(f1)
            }
        }
    }
}
