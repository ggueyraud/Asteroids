use std::{
    cell::RefCell,
    f32::consts::PI,
    rc::Rc,
};

use macroquad::{
    audio::{play_sound, play_sound_once, PlaySoundParams},
    prelude::{is_key_down, is_key_released, rand, KeyCode, Vec2, RED, WHITE},
    shapes::{draw_circle_lines},
    texture::{draw_texture_ex, DrawTextureParams, Texture2D},
    window::{screen_height, screen_width},
};

use crate::{assets_manager::AssetManager, collision::circle_test, world::World};

pub struct Data {
    alive: bool,
    pub position: Vec2,
    impulse: Vec2,
    pub texture: Rc<RefCell<Texture2D>>,
}

#[derive(PartialEq, Clone)]
pub enum ShootType {
    Player,
    Enemy,
}

#[derive(PartialEq)]
pub enum Type {
    Meteor(u32),
    Player,
    Enemy,
    Shoot(ShootType),
}

pub trait Entity {
    fn get_type(&self) -> Type;
    fn get_data(&self) -> &Data;

    fn set_position(&mut self, position: Vec2);

    fn draw(&self);

    fn update(&mut self, dt: f32) -> Option<Box<dyn FnOnce(&mut World)>>;

    fn is_collide(&self, _entity: &dyn Entity) -> bool {
        false
    }

    fn on_destroy(&mut self) -> Option<Box<dyn FnOnce(&mut World)>> {
        None
    }

    fn is_alive(&self) -> bool;
}

pub struct Player {
    data: Data,
    rotation: f32,
    is_moving: bool,
    time_since_last_shoot: f32,
    asset_manager: Rc<RefCell<AssetManager>>,
    lives: Rc<RefCell<u8>>,
    last_touch: f32,
}

impl Entity for Player {
    fn set_position(&mut self, position: Vec2) {
        self.data.position = position;
    }

    fn get_data(&self) -> &Data {
        &self.data
    }

    fn get_type(&self) -> Type {
        Type::Player
    }

    fn is_alive(&self) -> bool {
        self.data.alive
    }

    fn is_collide(&self, entity: &dyn Entity) -> bool {
        if entity.get_type() == Type::Shoot(ShootType::Player) {
            return false;
        }

        circle_test(&self.data, entity.get_data())
        // if self.get_type() == Type::Player {
        //     // Entity is of type Player
        //     // Do something specific for Player
        //     true

        // } else {
        //     false
        // }
    }

    fn update(&mut self, dt: f32) -> Option<Box<dyn FnOnce(&mut World)>> {
        self.time_since_last_shoot += dt;
        // println!("time last shoot: {}", self.time_since_last_shoot);
        self.is_moving = false;
        // self.rotation = 0.0;

        if is_key_down(KeyCode::Up) {
            self.is_moving = true;
        }

        if is_key_released(KeyCode::Down) {
            self.go_to_hyperspace();
        }

        if is_key_down(KeyCode::Right) {
            self.rotation += 250.0 * dt;
        } else if is_key_down(KeyCode::Left) {
            self.rotation -= 250.0 * dt;
        }

        // if self.rotation != 0.0 {
        //     let angle = self.rotation * 250 * dt;
        //     self.rota
        // }

        if self.is_moving {
            let angle = self.rotation / 180.0 * PI - PI / 2.0;
            self.data.impulse += Vec2::new(f32::cos(angle), f32::sin(angle)) * 300.0 * dt;
        }

        self.data.position += dt * self.data.impulse;

        // let texture_width = (*self.data.texture.borrow()).width();
        // if self.data.position.x < -texture_width {
        //     self.data.position.x = screen_width();
        // } else if self.data.position.x > screen_width() {
        //     self.data.position.x = -texture_width;
        // }

        // let texture_height = (*self.data.texture.borrow()).height();
        // if self.data.position.y < -texture_height {
        //     self.data.position.y = screen_height();
        // } else if self.data.position.y > screen_height() {
        //     self.data.position.y = -texture_height;
        // }

        if is_key_down(KeyCode::Space) && self.shoot() {
            let a = self.asset_manager.clone();
            let r = self.rotation;
            let position = self.data.position;

            return Some(Box::new(move |world| {
                world.add(Shoot::new(a, r, position, ShootType::Player))
            }));
        }

        self.last_touch += dt;

        None
    }

    fn on_destroy(&mut self) -> Option<Box<dyn FnOnce(&mut World)>> {
        if self.last_touch > 1.0 && *self.lives.borrow() > 0 {
            // #[cfg(not(debug_assertions))]
            // {
            let sound = pollster::block_on(
                self.asset_manager
                    .borrow_mut()
                    .get_sound("res/sounds/boom.ogg"),
            )
            .unwrap();
            play_sound_once(*sound.borrow());
            // }
            *self.lives.borrow_mut() -= 1;
            self.last_touch = 0.0;
        }

        None
    }

    fn draw(&self) {
        draw_texture_ex(
            *self.data.texture.borrow(),
            self.data.position.x - ((*self.data.texture.borrow()).width() / 2.0),
            self.data.position.y - ((*self.data.texture.borrow()).height() / 2.0),
            WHITE,
            DrawTextureParams {
                rotation: self.rotation * PI / 180.0,
                pivot: Some(Vec2 {
                    x: self.data.position.x,
                    y: self.data.position.y,
                }),
                ..Default::default()
            },
        );
        // draw_texture(self.data.texture, self.data.rect.x, self.data.rect.y, WHITE);
    }
}

impl Player {
    pub async fn new(asset_manager: Rc<RefCell<AssetManager>>, lives: Rc<RefCell<u8>>) -> Self {
        let texture = asset_manager
            .borrow_mut()
            .get_texture("res/Player/Ship.png")
            .await
            .unwrap();

        let position = Vec2 {
            x: screen_width() / 2.0,
            y: screen_height() / 2.0,
        };

        Self {
            lives,
            data: Data {
                alive: true,
                position,
                impulse: Vec2::default(),
                texture,
            },
            // entity: Entity::new(texture),
            rotation: 0.0,
            is_moving: false,
            time_since_last_shoot: 0.0,
            asset_manager,
            last_touch: 0.0,
        }
    }

    fn shoot(&mut self) -> bool {
        if self.time_since_last_shoot > 0.5 {
            self.time_since_last_shoot = 0.0;
            return true;
        }

        false
    }

    pub fn go_to_hyperspace(&mut self) {
        self.data.impulse = Vec2::splat(0.0);
        self.data.position = Vec2 {
            x: rand::gen_range(0.0, screen_width() - (*self.data.texture.borrow()).width()),
            y: rand::gen_range(
                0.0,
                screen_height() - (*self.data.texture.borrow()).height(),
            ),
        };
        let sound = pollster::block_on(
            self.asset_manager
                .borrow_mut()
                .get_sound("res/sounds/hyperspace.ogg"),
        )
        .unwrap();
        play_sound_once(*sound.borrow());
    }
}

pub struct Shoot {
    duration: f32,
    data: Data,
    rotation: f32,
    type_: ShootType,
}

impl Entity for Shoot {
    fn set_position(&mut self, position: Vec2) {
        self.data.position = position;
    }

    fn is_collide(&self, entity: &dyn Entity) -> bool {
        if let Type::Enemy | Type::Meteor(_) = entity.get_type() {
            return circle_test(&self.data, entity.get_data());
        }

        false
    }

    fn get_data(&self) -> &Data {
        &self.data
    }

    fn get_type(&self) -> Type {
        Type::Shoot(self.type_.clone())
    }

    fn is_alive(&self) -> bool {
        self.data.alive
    }

    fn update(&mut self, dt: f32) -> Option<Box<dyn FnOnce(&mut World)>> {
        // if self.duration < 0.0 {
        //     self.data.alive = false
        // }
        self.data.position += dt * self.data.impulse;
        self.duration -= dt;

        if self.duration < 0.0 {
            self.data.alive = false;
        }

        None
    }

    fn on_destroy(&mut self) -> Option<Box<dyn FnOnce(&mut World)>> {
        self.data.alive = false;
        println!("Desotry shoot");

        None
    }

    fn draw(&self) {
        let texture = self.data.texture.borrow();
        let pivot = Some(Vec2 {
            x: self.data.position.x + ((*texture).width() / 2.0),
            y: self.data.position.y + ((*texture).height() / 2.0),
        });

        draw_texture_ex(
            *texture,
            self.data.position.x,
            self.data.position.y,
            WHITE,
            DrawTextureParams {
                rotation: self.rotation * PI / 180.0,
                pivot,
                ..Default::default()
            },
        );

        let texture = *self.data.texture.borrow();
        let radius = (texture.width() + texture.height()) / 4.0;
        draw_circle_lines(
            self.data.position.x + (texture.width() / 2.0),
            self.data.position.y + (texture.height() / 2.0),
            radius,
            2.0,
            RED,
        );

        // draw_rectangle_lines(self.data.position.x, self.data.position.y, width, height, 2.0, RED);
    }
}

impl Shoot {
    fn new(
        assets: Rc<RefCell<AssetManager>>,
        rotation: f32,
        position: Vec2,
        type_: ShootType,
    ) -> Self {
        let angle = rotation / 180.0 * PI - PI / 2.0;
        let sound =
            pollster::block_on(assets.borrow_mut().get_sound("res/sounds/laser1.ogg")).unwrap();
        play_sound_once(*sound.borrow());

        Self {
            type_,
            duration: 0.5,
            rotation,
            data: Data {
                alive: true,
                position,
                impulse: Vec2 {
                    x: f32::cos(angle),
                    y: f32::sin(angle),
                } * 500.0,
                // texture: assets.borrow().get_texture("res/Shoot/Player.png"),
                texture: pollster::block_on(
                    assets.borrow_mut().get_texture("res/Shoot/Player.png"),
                )
                .unwrap(),
            },
        }
    }
}

#[derive(Clone)]
pub enum MeteorSize {
    Big,
    Medium,
    Small,
}

pub struct Meteor {
    data: Data,
    assets: Rc<RefCell<AssetManager>>,
    size: MeteorSize,
}

impl Meteor {
    pub async fn new(assets: Rc<RefCell<AssetManager>>, size: MeteorSize) -> Self {
        let angle: f32 = rand::gen_range(0.0, 2.0 * PI);

        let texture = match size {
            MeteorSize::Big => vec!["Big1.png", "Big2.png", "Big3.png", "Big4.png"],
            MeteorSize::Medium => vec!["Medium1.png", "Medium2.png"],
            MeteorSize::Small => vec!["Small1.png", "Small2.png", "Small3.png", "Small4.png"],
        };
        let texture = texture.get(rand::gen_range(0, texture.len() - 1)).unwrap();
        let texture = assets
            .borrow_mut()
            .get_texture(&format!("res/Meteor/{}", texture))
            .await
            .unwrap();

        Self {
            size,
            assets,
            data: Data {
                alive: true,
                position: Vec2::default(),
                impulse: Vec2 {
                    x: angle.cos(),
                    y: angle.sin(),
                },
                texture,
            },
        }
    }
}

impl Entity for Meteor {
    fn set_position(&mut self, position: Vec2) {
        self.data.position = position;
    }

    fn is_collide(&self, entity: &dyn Entity) -> bool {
        match entity.get_type() {
            Type::Shoot(ShootType::Player) => circle_test(&self.data, entity.get_data()),
            _ => false,
        }
    }
    fn get_data(&self) -> &Data {
        &self.data
    }

    fn get_type(&self) -> Type {
        Type::Meteor(match self.size {
            MeteorSize::Big => 10,
            MeteorSize::Medium => 5,
            MeteorSize::Small => 1,
        })
    }

    fn is_alive(&self) -> bool {
        self.data.alive
    }

    fn update(&mut self, dt: f32) -> Option<Box<dyn FnOnce(&mut World)>> {
        self.data.position += dt * self.data.impulse * 30.0;

        None
    }

    fn draw(&self) {
        let texture = self.data.texture.borrow();

        draw_texture_ex(
            *texture,
            self.data.position.x,
            self.data.position.y,
            WHITE,
            DrawTextureParams {
                // rotation: self.rotation * PI / 180.0,
                // pivot,
                ..Default::default()
            },
        );

        // draw_rectangle_lines(self.data.position.x, self.data.position.y, width, height, 1.0, RED);
        let texture = *self.data.texture.borrow();
        let radius = (texture.width() + texture.height()) / 4.0;
        draw_circle_lines(
            self.data.position.x + (texture.width() / 2.0),
            self.data.position.y + (texture.height() / 2.0),
            radius,
            2.0,
            RED,
        );
    }

    fn on_destroy(&mut self) -> Option<Box<dyn FnOnce(&mut World)>> {
        self.data.alive = false;
        let sound = match self.size {
            MeteorSize::Big => "explosion1.ogg",
            MeteorSize::Medium => "explosion2.ogg",
            _ => "explosion3.ogg",
        };
        let sound = pollster::block_on(
            self.assets
                .borrow_mut()
                .get_sound(&format!("res/sounds/{}", sound)),
        )
        .unwrap();
        play_sound(
            *sound.borrow(),
            PlaySoundParams {
                looped: false,
                volume: 0.1,
            },
        );

        // play_sound_once(*sound.borrow());
        println!("Desotry meteor");
        let size = self.size.clone();
        let assets = self.assets.clone();
        let position = self.data.position.clone();
        let nb = rand::gen_range(2, 3);

        Some(Box::new(move |world| match size {
            MeteorSize::Big => {
                for _ in 0..nb {
                    let mut entity =
                        pollster::block_on(Meteor::new(assets.clone(), MeteorSize::Medium));
                    entity.data.position = position;
                    world.add(entity);
                }
            }
            MeteorSize::Medium => {
                for _ in 0..nb {
                    let mut entity =
                        pollster::block_on(Meteor::new(assets.clone(), MeteorSize::Small));
                    entity.data.position = position;
                    world.add(entity);
                }
            }
            _ => (),
        }))
        // None
    }
}
