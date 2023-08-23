use super::entity::Data;
use macroquad::prelude::Vec2;

pub fn circle_test(a: &Data, b: &Data) -> bool {
    let first_rect = Vec2 {
        x: a.texture.borrow().width(),
        y: a.texture.borrow().height(),
    };
    let second_rect = Vec2 {
        x: b.texture.borrow().width(),
        y: b.texture.borrow().height(),
    };

    let radius_1 = (first_rect.x + first_rect.y) / 4.0;
    let radius_2 = (second_rect.x + second_rect.y) / 4.0;
    let xd = a.position.x - b.position.x;
    let yd = a.position.y - b.position.y;

    (xd * xd + yd * yd).sqrt() <= radius_1 + radius_2
}
