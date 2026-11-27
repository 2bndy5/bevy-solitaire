use bevy::prelude::*;

mod cards;
mod celebrate;
mod game;
mod games;
mod hud;
mod layout;
mod menu;
mod progress;
mod scene;
mod settings;
mod state;
mod storage;
mod table;

use celebrate::CelebratePlugin;
use hud::HudPlugin;
use menu::{fit_ui_to_window, main_menu, on_menu_buttons};
use progress::ProgressPlugin;
use scene::ScenePlugin;
use settings::SettingsPlugin;
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
        .add_plugins((
            SettingsPlugin,
            ScenePlugin,
            TablePlugin,
            HudPlugin,
            CelebratePlugin,
            ProgressPlugin,
        ))
        .add_systems(OnEnter(GameState::Menu), main_menu)
        .add_observer(on_menu_buttons)
        .add_systems(Update, fit_ui_to_window)
        .run();
}
