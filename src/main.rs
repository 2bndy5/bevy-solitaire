use bevy::prelude::*;

mod cards;
mod celebrate;
mod game;
mod games;
mod hud;
mod layout;
mod links;
mod menu;
mod progress;
mod scene;
mod settings;
mod state;
mod stats;
mod storage;
mod table;

use celebrate::CelebratePlugin;
use hud::HudPlugin;
use menu::MenuPlugin;
use progress::ProgressPlugin;
use scene::ScenePlugin;
use settings::SettingsPlugin;
use state::GameState;
use stats::StatsPlugin;
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
            StatsPlugin,
            MenuPlugin,
        ))
        .run();
}
