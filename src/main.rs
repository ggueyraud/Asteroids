use asteroids::{assets_manager::AssetManager, state_manager::StateManager};
use macroquad::{
    audio::{play_sound, PlaySoundParams},
    window::next_frame,
};
use std::{cell::RefCell, rc::Rc};

#[macroquad::main("Asteroids")]
async fn main() {
    let assets_manager = Rc::new(RefCell::new(AssetManager::new()));
    let mut state_manager = StateManager::new(assets_manager.clone()).await;

    // #[cfg(not(debug_assertions))]
    // {
    let sound = (*assets_manager.borrow_mut())
        .get_sound("res/theme.ogg")
        .await;
    play_sound(
        *sound.unwrap().borrow(),
        PlaySoundParams {
            looped: true,
            volume: 1.0,
        },
    );
    // }

    let mut running = true;

    loop {
        running = state_manager.update();
        state_manager.draw();

        if !running {
            break;
        }

        next_frame().await;
    }

    // let game = Game::new().await;
    // game.run().await;
}
