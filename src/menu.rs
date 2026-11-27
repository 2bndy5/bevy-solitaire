//! The main menu and the win screen, adapted from `bevy-52-card-pickup`.
use bevy::{prelude::*, ui_widgets::Button};

use crate::{
    games::GameKind,
    hud::format_duration,
    progress,
    state::GameState,
    table::{NextDeal, Table},
};

const FONT_PATH: &str = "fonts/UbuntuNerdFont-Medium.ttf";
/// The color of buttons.
const BUTTON_COLOR: Color = Color::srgb(0.125, 0.6, 0.125);

pub fn font(asset_server: &AssetServer, size: f32) -> TextFont {
    TextFont {
        font: FontSource::Handle(asset_server.load(FONT_PATH)),
        font_size: FontSize::Px(size),
        ..default()
    }
}

/// Marks buttons that deal a new game of the current kind when pressed. See [`on_menu_buttons`].
#[derive(Component)]
pub struct StartButton;

/// Marks buttons that return to the main menu when pressed.
#[derive(Component)]
pub struct HomeButton;

/// A main menu button that plays a game of its kind, picking up the saved one if there is one.
#[derive(Component)]
pub struct PlayButton(GameKind);

/// The look of a button: a rounded, green box.
pub fn button_node(padding: UiRect) -> impl Bundle {
    (
        Button,
        Node {
            padding,
            border_radius: BorderRadius::all(Val::Px(8.0)),
            align_items: AlignItems::Center,
            ..default()
        },
        BackgroundColor(BUTTON_COLOR),
    )
}

/// A button labeled `label`, tagged with `marker`.
fn text_button(label: &str, font: TextFont, marker: impl Bundle) -> impl Bundle {
    (
        button_node(UiRect::axes(Val::Px(16.0), Val::Px(8.0))),
        marker,
        // TextFont component (pointing to nerd font asset) must be in same bundle as
        // the Text component for the font to render the unicode code-point correctly.
        children![(Text::new(label), TextColor(Color::WHITE), font)],
    )
}

/// Spawns a full-screen, semi-transparent overlay for `state` with a panel in the middle, and
/// returns the panel, for the caller to fill.
fn overlay(commands: &mut Commands, state: GameState, dim: f32) -> Entity {
    let mut panel = Entity::PLACEHOLDER;
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(16.0)),
                ..default()
            },
            BackgroundColor(Color::BLACK.with_alpha(dim)),
            GlobalZIndex(1),
            DespawnOnExit(state),
        ))
        .with_children(|root| {
            // A panel behind the text keeps it readable over whatever is on the table.
            panel = root
                .spawn((
                    Node {
                        // Fill narrow screens (wrapping the text) but don't sprawl on wide ones.
                        width: Val::Percent(100.0),
                        max_width: Val::Px(760.0),
                        align_items: AlignItems::Center,
                        flex_direction: FlexDirection::Column,
                        row_gap: Val::Px(12.0),
                        padding: UiRect::all(Val::Px(24.0)),
                        border_radius: BorderRadius::all(Val::Px(16.0)),
                        ..default()
                    },
                    // Blending happens in linear color, so it takes a high alpha to dim white
                    // cards enough for white text to stand out.
                    BackgroundColor(Color::BLACK.with_alpha(0.85)),
                ))
                .id();
        });
    panel
}

/// A line of centered text.
fn line(text: impl Into<String>, font: TextFont) -> impl Bundle {
    (
        Text::new(text),
        TextColor(Color::WHITE),
        TextLayout::justify(Justify::Center),
        font,
    )
}

/// What the main menu says about the saved game of `kind`.
fn saved_game_status(kind: GameKind) -> String {
    match progress::load(kind) {
        Some(saved) if saved.game.moves > 0 => format!(
            // embedded the FontAwesome play icon via unicode code-point
            "\u{F04B} Continue: {} moves, {}",
            saved.game.moves,
            format_duration(saved.seconds)
        ),
        _ => "\u{F067} New game".to_string(),
    }
}

/// Shows the main menu: a button for each game.
pub fn main_menu(mut commands: Commands, asset_server: Res<AssetServer>) {
    let panel = overlay(&mut commands, GameState::Menu, 0.75);
    commands.entity(panel).with_children(|panel| {
        // embedded the MD hand wave emoji via unicode code-point
        panel.spawn(line(
            "\u{F1821} Welcome to Solitaire!",
            font(&asset_server, 36.0),
        ));
        panel.spawn(line("Choose a game:", font(&asset_server, 20.0)));
        for kind in GameKind::ALL {
            panel.spawn((
                Button,
                PlayButton(kind),
                Node {
                    width: Val::Percent(100.0),
                    max_width: Val::Px(560.0),
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Start,
                    row_gap: Val::Px(4.0),
                    padding: UiRect::axes(Val::Px(16.0), Val::Px(10.0)),
                    border_radius: BorderRadius::all(Val::Px(8.0)),
                    ..default()
                },
                BackgroundColor(BUTTON_COLOR),
                children![
                    (
                        Text::new(kind.name()),
                        TextColor(Color::WHITE),
                        font(&asset_server, 24.0),
                    ),
                    (
                        Text::new(kind.summary()),
                        TextColor(Color::WHITE.with_alpha(0.85)),
                        font(&asset_server, 16.0),
                    ),
                    (
                        Text::new(saved_game_status(kind)),
                        TextColor(Color::srgb(0.8, 1.0, 0.8)),
                        font(&asset_server, 16.0),
                    ),
                ],
            ));
        }
        panel.spawn(line(
            "Drag cards to move them, or tap a card to send it to the first place it fits.",
            font(&asset_server, 16.0),
        ));
    });
}

/// Shows the win screen with the game's statistics.
///
/// The background is only lightly dimmed, so the celebration stays visible behind it.
pub fn spawn_win_overlay(
    commands: &mut Commands,
    asset_server: &AssetServer,
    moves: u32,
    time: &str,
) {
    let stats = format!("You cleared the table in {moves} moves and {time}.");
    let panel = overlay(commands, GameState::Win, 0.45);
    commands.entity(panel).with_children(|panel| {
        // embedded the Octicons rocket icon via unicode code-point
        panel.spawn(line("Congratulations! \u{f427}", font(asset_server, 40.0)));
        panel.spawn(line(stats, font(asset_server, 28.0)));
        panel.spawn((
            Node {
                column_gap: Val::Px(12.0),
                row_gap: Val::Px(12.0),
                flex_wrap: FlexWrap::Wrap,
                justify_content: JustifyContent::Center,
                ..default()
            },
            children![
                // embedded the FontAwesome Play icon (circle variant) via unicode code-point
                text_button("\u{F01D} Play Again", font(asset_server, 24.0), StartButton),
                // embedded the FontAwesome house icon via unicode code-point
                text_button("\u{F015} Home", font(asset_server, 24.0), HomeButton),
            ],
        ));
    });
}

/// Shrinks the whole UI on small screens, so panels fit without scrolling.
///
/// The UI is designed for windows at least [`UI_DESIGN_SIZE`] logical pixels big; smaller windows
/// scale it down to match (but no further than [`MIN_UI_SCALE`], to keep text legible).
pub fn fit_ui_to_window(
    window: Single<&Window, (With<bevy::window::PrimaryWindow>, Changed<Window>)>,
    mut ui_scale: ResMut<UiScale>,
) {
    let scale = ui_scale_for(window.size());
    if ui_scale.0 != scale {
        ui_scale.0 = scale;
    }
}

/// The smallest window (logical pixels) the UI is designed for at full size.
pub const UI_DESIGN_SIZE: Vec2 = Vec2::new(440.0, 720.0);
/// The UI never shrinks below this scale.
pub const MIN_UI_SCALE: f32 = 0.6;

fn ui_scale_for(window: Vec2) -> f32 {
    let fit = window / UI_DESIGN_SIZE;
    fit.x.min(fit.y).clamp(MIN_UI_SCALE, 1.0)
}

/// Acts on presses of the start, home and play buttons.
///
/// Buttons act on the pointer's press event, which arrives once per press. (The button widget's
/// own `Activate` event can be missed when a press and release land in the same frame.)
#[allow(clippy::too_many_arguments)]
pub fn on_menu_buttons(
    press: On<PointerPress>,
    start_buttons: Query<(), With<StartButton>>,
    home_buttons: Query<(), With<HomeButton>>,
    play_buttons: Query<&PlayButton>,
    table: Res<Table>,
    state: Res<State<GameState>>,
    mut next_deal: ResMut<NextDeal>,
    mut game_state: ResMut<NextState<GameState>>,
) {
    let state = *state.get();
    if state == GameState::Loading {
        return;
    }
    if start_buttons.contains(press.entity) {
        *next_deal = NextDeal {
            kind: table.0.kind,
            resume: false,
        };
        game_state.set(GameState::Deal);
    } else if let Ok(&PlayButton(kind)) = play_buttons.get(press.entity) {
        *next_deal = NextDeal { kind, resume: true };
        game_state.set(GameState::Deal);
    } else if home_buttons.contains(press.entity) && state != GameState::Menu {
        game_state.set(GameState::Menu);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ui_shrinks_only_on_small_windows() {
        assert_eq!(ui_scale_for(Vec2::new(1280.0, 720.0)), 1.0);
        assert_eq!(ui_scale_for(Vec2::new(2560.0, 1440.0)), 1.0);
        // A phone held upright: limited by its width.
        assert!((ui_scale_for(Vec2::new(360.0, 740.0)) - 360.0 / 440.0).abs() < 1e-6);
        // A short landscape window: limited by its height.
        assert!((ui_scale_for(Vec2::new(1474.0, 610.0)) - 610.0 / 720.0).abs() < 1e-6);
        assert_eq!(ui_scale_for(Vec2::new(100.0, 100.0)), MIN_UI_SCALE);
    }
}
