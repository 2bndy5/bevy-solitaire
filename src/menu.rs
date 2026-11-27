//! The main menu and the other screens: win, best times and about.
use bevy::{prelude::*, ui_widgets::Button};

use crate::{
    games::GameKind,
    hud::format_duration,
    links::{self, Link},
    progress,
    state::GameState,
    stats::{self, LastWin, Stats},
    table::{NextDeal, Table},
};

const FONT_PATH: &str = "fonts/UbuntuNerdFont-Medium.ttf";
/// The color of buttons.
const BUTTON_COLOR: Color = Color::srgb(0.125, 0.6, 0.125);
/// The color of a button that isn't selected (e.g. the other games on the stats screen).
const UNSELECTED_COLOR: Color = Color::srgb(0.25, 0.27, 0.26);
/// The color that picks out a new best time.
const HIGHLIGHT_COLOR: Color = Color::srgb(1.0, 0.85, 0.3);

/// Where the source code lives.
const SOURCE_URL: &str = "https://github.com/2bndy5/bevy-solitaire";
/// Ways to support the author, as listed in `.github/FUNDING.yml`.
const SPONSOR_URL: &str = "https://github.com/sponsors/2bndy5";
const PAYPAL_URL: &str = "https://www.paypal.me/Brendan884";
const BEVY_URL: &str = "https://bevy.org";
const CARD_ART_URL: &str = "https://hazmat-game-studios.itch.io/hand-drawn-playing-cards";

pub fn font(asset_server: &AssetServer, size: f32) -> TextFont {
    TextFont {
        font: FontSource::Handle(asset_server.load(FONT_PATH)),
        font_size: FontSize::Px(size),
        ..default()
    }
}

/// What a menu button does when pressed. See [`on_menu_buttons`].
#[derive(Component, Clone, Copy, PartialEq, Eq)]
pub enum MenuAction {
    /// Deal a new game of the current kind.
    NewGame,
    /// Return to the main menu.
    Home,
    /// Play a game of this kind, picking up the saved one if there is one.
    Play(GameKind),
    /// Show the best times screen.
    Stats,
    /// Show the best times of this kind of game.
    ShowStats(GameKind),
    /// Show the about screen.
    About,
}

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
fn text_button(label: &'static str, font: TextFont, marker: impl Bundle) -> impl Bundle {
    (
        button_node(UiRect::axes(Val::Px(16.0), Val::Px(8.0))),
        marker,
        // TextFont component (pointing to nerd font asset) must be in same bundle as
        // the Text component for the font to render the unicode code-point correctly.
        children![(
            Text::new(label),
            TextColor(Color::WHITE),
            TextLayout::no_wrap(),
            font
        )],
    )
}

/// A centered row of buttons that wraps on narrow screens.
fn button_row() -> Node {
    Node {
        column_gap: Val::Px(12.0),
        row_gap: Val::Px(12.0),
        flex_wrap: FlexWrap::Wrap,
        justify_content: JustifyContent::Center,
        ..default()
    }
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

/// Shows the main menu: a button for each game, then the stats and about buttons.
pub fn main_menu(mut commands: Commands, asset_server: Res<AssetServer>) {
    let panel = overlay(&mut commands, GameState::Menu, 0.75);
    commands.entity(panel).with_children(|panel| {
        // embedded the MD hand wave emoji via unicode code-point
        panel.spawn(line(
            "\u{F1821} Welcome to Solitaire!",
            font(&asset_server, 36.0),
        ));
        panel.spawn(line("Choose a game:", font(&asset_server, 20.0)));
        // A tile per game, two to a row.
        panel
            .spawn(Node {
                width: Val::Percent(100.0),
                max_width: Val::Px(560.0),
                flex_wrap: FlexWrap::Wrap,
                justify_content: JustifyContent::SpaceBetween,
                row_gap: Val::Px(10.0),
                ..default()
            })
            .with_children(|tiles| {
                for kind in GameKind::ALL {
                    tiles.spawn((
                        Button,
                        MenuAction::Play(kind),
                        Node {
                            width: Val::Percent(48.5),
                            flex_direction: FlexDirection::Column,
                            row_gap: Val::Px(2.0),
                            padding: UiRect::axes(Val::Px(14.0), Val::Px(8.0)),
                            border_radius: BorderRadius::all(Val::Px(8.0)),
                            ..default()
                        },
                        BackgroundColor(BUTTON_COLOR),
                        children![
                            (
                                Text::new(kind.name()),
                                TextColor(Color::WHITE),
                                font(&asset_server, 22.0),
                            ),
                            (
                                Text::new(saved_game_status(kind)),
                                TextColor(Color::srgb(0.8, 1.0, 0.8)),
                                font(&asset_server, 15.0),
                            ),
                        ],
                    ));
                }
            });
        panel.spawn((
            button_row(),
            children![
                // embedded the FontAwesome trophy icon via unicode code-point
                text_button(
                    "\u{F091} Best Times",
                    font(&asset_server, 20.0),
                    MenuAction::Stats
                ),
                // embedded the FontAwesome info icon via unicode code-point
                text_button(
                    "\u{F05A} About",
                    font(&asset_server, 20.0),
                    MenuAction::About
                ),
            ],
        ));
        panel.spawn(line(
            "Drag cards to move them, or tap a card to send it to the first place it fits.",
            font(&asset_server, 16.0),
        ));
    });
}

/// Spawns a table of the best times in `stats` into `parent`, with the one at `highlight` (if
/// any) picked out.
fn spawn_best_times(
    parent: &mut ChildSpawnerCommands,
    asset_server: &AssetServer,
    stats: &Stats,
    highlight: Option<usize>,
) {
    if stats.best.is_empty() {
        parent.spawn(line("No wins yet.", font(asset_server, 18.0)));
        return;
    }
    parent
        .spawn(Node {
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(4.0),
            ..default()
        })
        .with_children(|table| {
            for (place, win) in stats.best.iter().enumerate() {
                let color = if Some(place) == highlight {
                    HIGHLIGHT_COLOR
                } else {
                    Color::WHITE
                };
                let cell = |text: String, width: f32, justify: Justify| {
                    (
                        Node {
                            width: Val::Px(width),
                            ..default()
                        },
                        children![(
                            Text::new(text),
                            TextColor(color),
                            TextLayout::new(justify, LineBreak::NoWrap),
                            font(asset_server, 20.0),
                        )],
                    )
                };
                table.spawn((
                    Node {
                        column_gap: Val::Px(16.0),
                        ..default()
                    },
                    children![
                        cell(format!("{}.", place + 1), 28.0, Justify::Right),
                        cell(format_duration(win.seconds), 80.0, Justify::Right),
                        cell(format!("{} moves", win.moves), 110.0, Justify::Right),
                    ],
                ));
            }
        });
}

/// Shows the win screen with the game's statistics and best times.
///
/// The background is only lightly dimmed, so the celebration stays visible behind it.
pub fn spawn_win_overlay(
    commands: &mut Commands,
    asset_server: &AssetServer,
    moves: u32,
    time: &str,
    last: &LastWin,
) {
    let stats = format!("You cleared the table in {moves} moves and {time}.");
    // A first win is trivially the best; only call out beating an earlier best.
    let title = if last.place == Some(0) && last.stats.won > 1 {
        // embedded the FontAwesome trophy icon via unicode code-point
        "New best time! \u{F091}"
    } else {
        // embedded the Octicons rocket icon via unicode code-point
        "Congratulations! \u{f427}"
    };
    let panel = overlay(commands, GameState::Win, 0.45);
    commands.entity(panel).with_children(|panel| {
        panel.spawn(line(title, font(asset_server, 40.0)));
        panel.spawn(line(stats, font(asset_server, 28.0)));
        panel.spawn(line("Best times", font(asset_server, 22.0)));
        spawn_best_times(panel, asset_server, &last.stats, last.place);
        panel.spawn((
            button_row(),
            children![
                // embedded the FontAwesome Play icon (circle variant) via unicode code-point
                text_button(
                    "\u{F01D} Play Again",
                    font(asset_server, 24.0),
                    MenuAction::NewGame
                ),
                // embedded the FontAwesome house icon via unicode code-point
                text_button("\u{F015} Home", font(asset_server, 24.0), MenuAction::Home),
            ],
        ));
    });
}

/// The game whose best times the stats screen shows.
#[derive(Resource, Default)]
pub struct StatsShown(pub GameKind);

/// Marks the stats screen, which is rebuilt when [`StatsShown`] changes.
#[derive(Component)]
struct StatsScreen;

/// Opens the stats screen on the game last played.
fn open_stats(table: Res<Table>, mut shown: ResMut<StatsShown>) {
    shown.0 = table.0.kind;
}

/// (Re)builds the stats screen for the game in [`StatsShown`].
fn show_stats(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    shown: Res<StatsShown>,
    screens: Query<Entity, With<StatsScreen>>,
) {
    if !shown.is_changed() {
        return;
    }
    for screen in &screens {
        commands.entity(screen).despawn();
    }
    let kind = shown.0;
    let record = stats::load(kind);
    let panel = overlay(&mut commands, GameState::Stats, 0.75);
    commands.entity(panel).insert(StatsScreen);
    commands.entity(panel).with_children(|panel| {
        panel.spawn(line("\u{F091} Best Times", font(&asset_server, 32.0)));
        // A tab for each game, when there's more than one.
        if GameKind::ALL.len() > 1 {
            panel
                .spawn(Node {
                    column_gap: Val::Px(8.0),
                    row_gap: Val::Px(8.0),
                    flex_wrap: FlexWrap::Wrap,
                    justify_content: JustifyContent::Center,
                    ..default()
                })
                .with_children(|tabs| {
                    for tab in GameKind::ALL {
                        let color = if tab == kind {
                            BUTTON_COLOR
                        } else {
                            UNSELECTED_COLOR
                        };
                        // The button already has a background color: replace it, rather than
                        // spawn it with two (which panics).
                        tabs.spawn(text_button(
                            tab.name(),
                            font(&asset_server, 16.0),
                            MenuAction::ShowStats(tab),
                        ))
                        .insert(BackgroundColor(color));
                    }
                });
        }
        panel.spawn(line(kind.name(), font(&asset_server, 24.0)));
        panel.spawn(line(
            format!("Games won: {}", record.won),
            font(&asset_server, 18.0),
        ));
        spawn_best_times(panel, &asset_server, &record, None);
        // embedded the FontAwesome house icon via unicode code-point
        panel.spawn(text_button(
            "\u{F015} Home",
            font(&asset_server, 20.0),
            MenuAction::Home,
        ));
    });
}

/// Shows the about screen: what the app is, who made it, and links.
fn show_about(mut commands: Commands, asset_server: Res<AssetServer>) {
    let link_button = |label: &'static str, url: &'static str| {
        text_button(label, font(&asset_server, 18.0), Link(url))
    };
    let panel = overlay(&mut commands, GameState::About, 0.75);
    commands.entity(panel).with_children(|panel| {
        panel.spawn(line("\u{F05A} About", font(&asset_server, 32.0)));
        panel.spawn(line(
            format!("Solitaire, version {}", env!("CARGO_PKG_VERSION")),
            font(&asset_server, 22.0),
        ));
        panel.spawn(line(
            "A free and open source collection of solitaire games.",
            font(&asset_server, 18.0),
        ));
        panel.spawn(line("Made by Brendan (2bndy5).", font(&asset_server, 18.0)));
        panel.spawn(line(
            "Built with the Bevy game engine. Hand-drawn cards by Hazmat Game Studios.",
            font(&asset_server, 16.0),
        ));
        panel.spawn((
            button_row(),
            children![
                // embedded the FontAwesome GitHub icon via unicode code-point
                link_button("\u{F09B} Source code", SOURCE_URL),
                // embedded the FontAwesome heart icon via unicode code-point
                link_button("\u{F004} Sponsor", SPONSOR_URL),
                // embedded the FontAwesome PayPal icon via unicode code-point
                link_button("\u{F1ED} PayPal", PAYPAL_URL),
            ],
        ));
        panel.spawn((
            button_row(),
            children![
                link_button("Bevy", BEVY_URL),
                link_button("Card art", CARD_ART_URL),
            ],
        ));
        panel.spawn(text_button(
            "\u{F015} Home",
            font(&asset_server, 20.0),
            MenuAction::Home,
        ));
    });
}

pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<StatsShown>()
            .add_observer(on_menu_buttons)
            .add_observer(links::on_link_press)
            .add_systems(OnEnter(GameState::Menu), main_menu)
            .add_systems(OnEnter(GameState::Stats), open_stats)
            .add_systems(Update, show_stats.run_if(in_state(GameState::Stats)))
            .add_systems(OnEnter(GameState::About), show_about)
            .add_systems(
                Update,
                back_on_escape.run_if(|state: Res<State<GameState>>| {
                    matches!(state.get(), GameState::Stats | GameState::About)
                }),
            )
            .add_systems(Update, fit_ui_to_window);
    }
}

/// Escape leaves the stats and about screens.
fn back_on_escape(keys: Res<ButtonInput<KeyCode>>, mut game_state: ResMut<NextState<GameState>>) {
    if keys.just_pressed(KeyCode::Escape) {
        game_state.set(GameState::Menu);
    }
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

/// Acts on presses of the menu buttons.
///
/// Buttons act on the pointer's press event, which arrives once per press. (The button widget's
/// own `Activate` event can be missed when a press and release land in the same frame.)
pub fn on_menu_buttons(
    press: On<PointerPress>,
    actions: Query<&MenuAction>,
    table: Res<Table>,
    state: Res<State<GameState>>,
    mut next_deal: ResMut<NextDeal>,
    mut shown: ResMut<StatsShown>,
    mut game_state: ResMut<NextState<GameState>>,
) {
    let state = *state.get();
    let Ok(&action) = actions.get(press.entity) else {
        return;
    };
    if state == GameState::Loading {
        return;
    }
    match action {
        MenuAction::NewGame => {
            *next_deal = NextDeal {
                kind: table.0.kind,
                resume: false,
            };
            game_state.set(GameState::Deal);
        }
        MenuAction::Play(kind) => {
            *next_deal = NextDeal { kind, resume: true };
            game_state.set(GameState::Deal);
        }
        MenuAction::Home if state != GameState::Menu => game_state.set(GameState::Menu),
        MenuAction::Home => {}
        MenuAction::Stats => game_state.set(GameState::Stats),
        MenuAction::ShowStats(kind) => {
            if shown.0 != kind {
                shown.0 = kind;
            }
        }
        MenuAction::About => game_state.set(GameState::About),
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
