use bevy::prelude::*;

mod cards;
mod game;
mod layout;
mod menu;
mod scene;
mod state;
mod table;

use menu::{button_detector, hello_menu, win_menu};
use scene::ScenePlugin;
use state::GameState;
use table::{TablePlugin, spawn_hud};

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
        .add_plugins((ScenePlugin, TablePlugin))
        .add_systems(Startup, spawn_hud)
        .add_systems(OnEnter(GameState::Menu), hello_menu)
        .add_systems(OnEnter(GameState::Win), win_menu)
        .add_systems(
            Update,
            button_detector.run_if(not(in_state(GameState::Loading))),
        )
        .run();
}
