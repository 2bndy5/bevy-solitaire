use bevy::{camera::ScalingMode, prelude::*};

mod cards;
mod game;
mod layout;
mod menu;
mod state;
mod table;

use layout::TABLE_SIZE;
use menu::{button_detector, hello_menu, win_menu};
use state::GameState;
use table::{TablePlugin, spawn_hud};

/// The color of the felt on the card table.
const FELT: Color = Color::srgb(0.11, 0.42, 0.23);

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
        .insert_resource(ClearColor(FELT))
        .init_state::<GameState>()
        .add_plugins(TablePlugin)
        .add_systems(Startup, (setup_camera, spawn_hud))
        .add_systems(OnEnter(GameState::Menu), hello_menu)
        .add_systems(OnEnter(GameState::Win), win_menu)
        .add_systems(
            Update,
            button_detector.run_if(not(in_state(GameState::Loading))),
        )
        .run();
}

/// Spawns a 2D camera that always fits the whole table on screen.
///
/// Also sets the game state to [`GameState::Menu`] when done.
fn setup_camera(mut commands: Commands, mut game_state: ResMut<NextState<GameState>>) {
    commands.spawn((
        Camera2d,
        Projection::Orthographic(OrthographicProjection {
            scaling_mode: ScalingMode::AutoMin {
                min_width: TABLE_SIZE.x,
                min_height: TABLE_SIZE.y,
            },
            ..OrthographicProjection::default_2d()
        }),
    ));
    game_state.set(GameState::Menu);
}
