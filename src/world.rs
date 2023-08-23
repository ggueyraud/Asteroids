use macroquad::window::{screen_height, screen_width};

use crate::entity::{self, Entity};

pub struct World {
    pub entities: Vec<Box<dyn Entity>>,
    entities_tmp: Vec<Box<dyn Entity>>,
}

impl World {
    pub fn new() -> Self {
        Self {
            entities: Vec::new(),
            entities_tmp: Vec::new(),
        }
    }

    pub fn size(&self) -> usize {
        self.entities.len() + self.entities_tmp.len()
    }

    pub fn add(&mut self, entity: impl Entity + 'static) {
        self.entities_tmp.push(Box::new(entity));
    }

    pub fn update(&mut self, dt: f32) -> u32 {
        if !self.entities_tmp.is_empty() {
            self.entities.append(&mut self.entities_tmp);
        }

        let mut updates = vec![];
        for entity in self.entities.iter_mut() {
            if let Some(update) = entity.update(dt) {
                updates.push(update);
            }
        }

        for update in updates.into_iter() {
            update(self);
        }

        // Handle if entity is outside bounds
        for entity in self.entities.iter_mut() {
            let mut position = entity.get_data().position;

            if position.x < -entity.get_data().texture.borrow().width() {
                position.x = screen_width();
                // position.y = screen_height() - position.y;
            } else if position.x > screen_width() {
                position.x = -(entity.get_data().texture.borrow().width());
                // position.y = screen_height() - position.y;
            }

            if position.y < -entity.get_data().texture.borrow().height() {
                position.y = screen_height();
            } else if position.y > screen_height() {
                position.y = -entity.get_data().texture.borrow().height();
            }

            entity.set_position(position);
        }

        let mut a = vec![];
        for (i, entity_i) in self.entities.iter().enumerate() {
            for (j, entity_j) in self.entities.iter().skip(i + 1).enumerate() {
                if entity_i.is_alive() && entity_i.is_collide(entity_j.as_ref()) {
                    a.push(i);
                }

                if entity_j.is_alive() && entity_j.is_collide(entity_i.as_ref()) {
                    a.push(j + i + 1);
                }
            }
        }

        for index in a.into_iter() {
            if let Some(entity) = self.entities.get_mut(index) {
                if let Some(destroy) = entity.on_destroy() {
                    destroy(self);
                }
            }
        }

        let mut remove_entities_index = vec![];

        for (index, entity) in self.entities.iter_mut().enumerate() {
            if !entity.is_alive() {
                remove_entities_index.push(index);
            }
        }

        let mut pts = 0;
        for id in remove_entities_index.into_iter() {
            if let Some(entity) = self.entities.get(id) {
                if let entity::Type::Meteor(p) = entity.get_type() {
                    pts += p;
                }
                self.entities.remove(id);
            }
        }

        pts
    }

    pub fn clear(&mut self) {
        self.entities.clear();
    }

    pub fn draw(&self) {
        for entity in self.entities.iter() {
            entity.draw();
        }
    }

    // pub fn is_collide(&self, other: impl Entity) -> bool {
    //     for entity in self.entities.iter() {
    //         if other.is_collide(entity.as_ref()) {
    //             return true;
    //         }
    //     }

    //     false
    // }
}
