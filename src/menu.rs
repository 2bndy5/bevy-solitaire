//! Start and win screens, adapted from `bevy-52-card-pickup`.
use crate::{state::GameState, table::Table};
use bevy::{prelude::*, ui::Pressed, ui_widgets::Button};

const FONT_PATH: &str = "fonts/UbuntuNerdFont-Medium.ttf";

fn font(asset_server: &AssetServer, size: f32) -> TextFont {
    TextFont {
        font: FontSource::Handle(asset_server.load(FONT_PATH)),
        font_size: FontSize::Px(size),
        ..default()
    }
}

/// A button that (re)starts the game when pressed. See [`button_detector`].
pub fn start_button(label: &str, font: TextFont) -> impl Bundle {
    (
        Button,
        Node {
            padding: UiRect::axes(Val::Px(16.0), Val::Px(8.0)),
            border_radius: BorderRadius::all(Val::Px(8.0)),
            ..default()
        },
        BackgroundColor(Color::srgb(0.125, 0.6, 0.125)),
        // TextFont component (pointing to nerd font asset) must be in same bundle as
        // the Text component for the font to render the unicode code-point correctly.
        children![(Text::new(label), TextColor(Color::WHITE), font)],
    )
}

/// A full-screen, semi-transparent overlay holding `lines` of text and a start button.
fn overlay(
    commands: &mut Commands,
    asset_server: &AssetServer,
    state: GameState,
    lines: &[(&str, f32)],
    button_label: &str,
) {
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(12.0),
                padding: UiRect::all(Val::Px(16.0)),
                ..default()
            },
            BackgroundColor(Color::BLACK.with_alpha(0.75)),
            GlobalZIndex(1),
            DespawnOnExit(state),
        ))
        .with_children(|root| {
            for &(line, size) in lines {
                root.spawn((
                    Text::new(line),
                    TextColor(Color::WHITE),
                    TextLayout::justify(Justify::Center),
                    font(asset_server, size),
                ));
            }
            root.spawn(start_button(button_label, font(asset_server, 24.0)));
        });
}

/// System that runs when entering the [`GameState::Menu`] state (on game startup only).
pub fn hello_menu(mut commands: Commands, asset_server: Res<AssetServer>) {
    overlay(
        &mut commands,
        &asset_server,
        GameState::Menu,
        &[
            // embedded the MD hand wave emoji via unicode code-point
            ("\u{F1821} Welcome to Solitaire!", 36.0),
            (
                "Build each foundation up by suit from Ace to King.\n\
                 Drag cards to move them; build the tableau down in alternating colors.\n\
                 Click the stock to draw. Double-click a card to send it to a foundation.",
                20.0,
            ),
        ],
        // embedded the FontAwesome Play icon (circle variant) via unicode code-point
        "\u{F01D} Start Game",
    );
}

/// System that spawns the menu when entering the [`GameState::Win`] state.
pub fn win_menu(mut commands: Commands, asset_server: Res<AssetServer>, table: Res<Table>) {
    let moves = format!("You cleared the table in {} moves.", table.0.moves);
    overlay(
        &mut commands,
        &asset_server,
        GameState::Win,
        &[
            // embedded the Octicons rocket icon via unicode code-point
            ("Congratulations! \u{f427}", 40.0),
            (&moves, 28.0),
        ],
        "\u{F01D} Play Again",
    );
}

/// Starts a new deal whenever a start button is pressed.
pub fn button_detector(
    query: Query<&Pressed, (Changed<Pressed>, With<Button>)>,
    mut game_state: ResMut<NextState<GameState>>,
) {
    for _ in query {
        info!("Starting a new game");
        game_state.set(GameState::Deal);
    }
}
