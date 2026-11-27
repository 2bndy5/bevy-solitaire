//! Start and win screens, adapted from `bevy-52-card-pickup`.
use crate::state::GameState;
use bevy::{prelude::*, ui_widgets::Button};

const FONT_PATH: &str = "fonts/UbuntuNerdFont-Medium.ttf";

pub fn font(asset_server: &AssetServer, size: f32) -> TextFont {
    TextFont {
        font: FontSource::Handle(asset_server.load(FONT_PATH)),
        font_size: FontSize::Px(size),
        ..default()
    }
}

/// Marks buttons that (re)start the game when pressed. See [`on_start_button`].
#[derive(Component)]
pub struct StartButton;

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
        BackgroundColor(Color::srgb(0.125, 0.6, 0.125)),
    )
}

/// A button that (re)starts the game when pressed.
pub fn start_button(label: &str, font: TextFont) -> impl Bundle {
    (
        button_node(UiRect::axes(Val::Px(16.0), Val::Px(8.0))),
        StartButton,
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
    dim: f32,
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
            BackgroundColor(Color::BLACK.with_alpha(dim)),
            GlobalZIndex(1),
            DespawnOnExit(state),
        ))
        .with_children(|root| {
            // A panel behind the text keeps it readable over whatever is on the table.
            root.spawn((
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
                // Blending happens in linear color, so it takes a high alpha to dim white cards
                // enough for white text to stand out.
                BackgroundColor(Color::BLACK.with_alpha(0.85)),
            ))
            .with_children(|panel| {
                for &(line, size) in lines {
                    panel.spawn((
                        Text::new(line),
                        TextColor(Color::WHITE),
                        TextLayout::justify(Justify::Center),
                        font(asset_server, size),
                    ));
                }
                panel.spawn(start_button(button_label, font(asset_server, 24.0)));
            });
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
                 Click the stock to draw. Click a card to move it to the first place it fits.",
                20.0,
            ),
        ],
        // embedded the FontAwesome Play icon (circle variant) via unicode code-point
        "\u{F01D} Start Game",
        0.75,
    );
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
    overlay(
        commands,
        asset_server,
        GameState::Win,
        &[
            // embedded the Octicons rocket icon via unicode code-point
            ("Congratulations! \u{f427}", 40.0),
            (&stats, 28.0),
        ],
        "\u{F01D} Play Again",
        0.45,
    );
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

/// Starts a new deal whenever a start button is pressed.
///
/// Buttons act on the pointer's press event, which arrives once per press. (The button widget's
/// own `Activate` event can be missed when a press and release land in the same frame.)
pub fn on_start_button(
    press: On<PointerPress>,
    start_buttons: Query<(), With<StartButton>>,
    state: Res<State<GameState>>,
    mut game_state: ResMut<NextState<GameState>>,
) {
    if start_buttons.contains(press.entity) && *state.get() != GameState::Loading {
        info!("Starting a new game");
        game_state.set(GameState::Deal);
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
