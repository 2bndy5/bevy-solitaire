use bevy::prelude::*;

mod cards;
mod celebrate;
mod game;
mod hud;
mod layout;
mod menu;
mod scene;
mod state;
mod table;

use celebrate::CelebratePlugin;
use hud::HudPlugin;
use menu::{hello_menu, on_start_button};
use scene::ScenePlugin;
use state::GameState;
use table::TablePlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Solitaire".into(),
                fit_canvas_to_parent: true,
                ..default()
            }),
            ..default()
        }))
        .init_state::<GameState>()
        .add_plugins((ScenePlugin, TablePlugin, HudPlugin, CelebratePlugin))
        .add_systems(OnEnter(GameState::Menu), hello_menu)
        .add_observer(on_start_button)
        .run();
}
