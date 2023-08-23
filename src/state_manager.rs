use macroquad::prelude::*;
use macroquad::time::get_frame_time;
use macroquad::ui::{hash, root_ui, widgets};
use std::collections::HashMap;
use std::{cell::RefCell, rc::Rc};

use crate::entity::{Entity, Meteor, MeteorSize, Player};
use crate::{assets_manager::AssetManager, world::World};

#[derive(PartialEq, Eq, Hash, Debug)]
pub enum Type {
    MainMenu,
    Game,
    Lose,
    Win,
}

pub trait State {
    fn update(&mut self) -> Box<dyn FnOnce(&mut StateManager) -> bool>;
    fn draw(&self);
}

pub struct StateManager {
    states: HashMap<Type, Box<dyn State>>,
    type_: Type,
}

impl StateManager {
    pub async fn new(assets_manager: Rc<RefCell<AssetManager>>) -> Self {
        let mut manager = Self {
            states: HashMap::new(),
            type_: Type::MainMenu,
        };

        manager.states.insert(
            Type::Game,
            Box::new(GameState::new(assets_manager.clone()).await),
        );
        let font = assets_manager
            .borrow_mut()
            .get_font("res/trs-million.ttf")
            .await
            .unwrap();
        manager
            .states
            .insert(Type::Lose, Box::new(LoseState::new(font).await));
        manager
            .states
            .insert(Type::MainMenu, Box::new(MainState::new()));

        manager
    }

    pub fn update(&mut self) -> bool {
        if self.states.is_empty() {
            return false;
        }

        // let mut update = None;

        if let Some(state) = self.states.get_mut(&self.type_) {
            let update = state.update();

            if !update(self) {
                return false;
            }
        }

        // if let Some(update) = update {
        //     update(self)
        // }

        true
    }

    pub fn switch_to(&mut self, type_: Type) {
        self.type_ = type_;
    }

    pub fn draw(&self) {
        if self.states.is_empty() {
            return;
        }

        if let Some(state) = self.states.get(&self.type_) {
            state.draw();
        }
    }
}

#[derive(Clone)]
enum Level {
    One = 1,
    Two = 2,
    Three = 3,
    Four = 4,
    Five = 5,
}

impl Level {
    fn from_u32(value: u32) -> Level {
        match value {
            1 => Self::One,
            2 => Self::Two,
            3 => Self::Three,
            4 => Self::Four,
            _ => Self::Five,
        }
    }
}

struct GameState {
    assets_manager: Rc<RefCell<AssetManager>>,
    world: World,
    level: Level,
    lives: Rc<RefCell<u8>>,
    score: u32,
}

impl GameState {
    pub async fn new(assets_manager: Rc<RefCell<AssetManager>>) -> Self {
        let mut state = Self {
            assets_manager,
            world: World::new(),
            level: Level::One,
            lives: Rc::new(RefCell::new(3)),
            score: 0,
        };

        state
            .world
            .add(Player::new(state.assets_manager.clone(), state.lives.clone()).await);

        state.init_level().await;

        state
    }

    async fn init_level(&mut self) {
        let nb_meteors = match self.level {
            Level::One => 4,
            Level::Two => 5,
            Level::Three => 7,
            Level::Four => 9,
            Level::Five => 11,
        };

        for _ in 0..nb_meteors {
            let mut meteor = Meteor::new(self.assets_manager.clone(), MeteorSize::Big).await;
            meteor.set_position(Vec2 {
                x: rand::gen_range(0.0, screen_width()),
                y: rand::gen_range(0.0, screen_height()),
            });
            self.world.add(meteor);
        }
    }

    async fn reset(&mut self) {
        self.level = Level::One;
        self.score = 0;
        *self.lives.borrow_mut() = 3;
        self.world = World::new();
        self.world
            .add(Player::new(self.assets_manager.clone(), self.lives.clone()).await);

        self.init_level().await;
    }
}

impl State for GameState {
    fn update(&mut self) -> Box<dyn FnOnce(&mut StateManager) -> bool> {
        self.score += self.world.update(get_frame_time());

        if *self.lives.borrow() == 0 {
            pollster::block_on(self.reset());
            return Box::new(|world| {
                world.switch_to(Type::Lose);
                true
            });
        }

        if self.world.size() == 1 {
            self.level = Level::from_u32((self.level.clone() as u32) + 1);
            pollster::block_on(self.init_level());
        }

        Box::new(|_| true)
    }

    fn draw(&self) {
        // Draw score
        let font = pollster::block_on(
            self.assets_manager
                .borrow_mut()
                .get_font("res/trs-million.ttf"),
        )
        .unwrap();
        draw_text_ex(
            &format!("Score: {}", self.score),
            0.0,
            30.0,
            TextParams {
                font: *font.borrow(),
                font_size: 30,
                color: WHITE,
                ..Default::default()
            },
        );

        // Draw lives
        let texture = pollster::block_on(
            self.assets_manager
                .borrow_mut()
                .get_texture("res/Player/life.png"),
        )
        .unwrap();
        let width = screen_width();
        for i in 0..*self.lives.borrow() {
            draw_texture(
                *texture.borrow(),
                width - (*self.lives.borrow() - i) as f32 * texture.borrow().width(),
                0.0,
                WHITE,
            );
        }

        self.world.draw()
    }
}

struct MainState {}

impl MainState {
    pub fn new() -> Self {
        Self {}
    }
}

impl State for MainState {
    fn update(&mut self) -> Box<dyn FnOnce(&mut StateManager) -> bool> {
        let mut play = false;
        let mut quit = false;

        let width = screen_width();
        let height = screen_height();

        widgets::Window::new(
            hash!(),
            vec2((width - 120.) * 0.5, (height - 70.) * 0.5),
            vec2(120., 148.),
        )
        .movable(false)
        .titlebar(false)
        .ui(&mut *root_ui(), |ui| {
            if widgets::Button::new("Play").size(vec2(113., 70.)).ui(ui) {
                play = true;
            }

            if widgets::Button::new("Quit").size(vec2(113., 70.)).ui(ui) {
                quit = true;
            }
        });

        if play {
            return Box::new(|state_manager| {
                state_manager.switch_to(Type::Game);

                true
            });
        }

        Box::new(move |_| !quit)
    }

    fn draw(&self) {}
}

struct LoseState {
    font: Rc<RefCell<Font>>,
}

impl LoseState {
    pub async fn new(font: Rc<RefCell<Font>>) -> Self {
        Self { font }
    }
}

impl State for LoseState {
    fn draw(&self) {
        let text = "You lose the game!";
        let dimensions = measure_text(text, Some(*self.font.borrow()), 50, 1.0);

        draw_text_ex(
            text,
            (screen_width() - dimensions.width) * 0.5,
            (screen_height() - dimensions.height) * 0.5,
            TextParams {
                font: *self.font.borrow(),
                font_size: 50,
                color: WHITE,
                ..Default::default()
            },
        );
    }

    fn update(&mut self) -> Box<dyn FnOnce(&mut StateManager) -> bool> {
        let mut play = false;
        let mut quit = false;
        let height = screen_height() * 0.5;
        let width = screen_width();

        widgets::Window::new(
            hash!(),
            vec2((width - 120.) * 0.5, height + 80.0),
            vec2(120., 148.),
        )
        .movable(false)
        .titlebar(false)
        .ui(&mut root_ui(), |ui| {
            if widgets::Button::new("Retry").size(vec2(113., 70.)).ui(ui) {
                play = true;
            }

            if widgets::Button::new("Quit").size(vec2(113., 70.)).ui(ui) {
                quit = true;
            }
        });

        if play {
            return Box::new(|state_manager| {
                state_manager.switch_to(Type::Game);

                true
            });
        }

        Box::new(move |_| !quit)
    }
}
