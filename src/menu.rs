//! The main menu and the other screens: win, best times and about.
use bevy::{prelude::*, ui_widgets::Button};

use crate::{
    games::GameKind,
    hud::{Undo, UndoRequested, format_duration},
    links::{self, Link},
    progress,
    state::GameState,
    stats::{self, LastWin, Stats},
    storage,
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
    /// Show the ways to play this family of games (e.g. Spider's suit counts).
    Variants(GameKind),
    /// Show how to play the current game.
    HowToPlay,
    /// Close the pop-up.
    ClosePopup,
    /// Take back the last move.
    Undo,
    /// Take back every move, back to the deal.
    UndoAll,
}

/// The pop-up showing over everything else, if any.
#[derive(States, Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Popup {
    #[default]
    Closed,
    /// How to play the current game.
    HowToPlay,
    /// The ways to play a family of games.
    Variants(GameKind),
    /// The game is stuck: no moves are left.
    NoMoves,
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
        // A definite width, so the row wraps (and grows taller) when its buttons don't fit,
        // instead of spilling over what comes after it.
        width: Val::Percent(100.0),
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
                        // Never wider than the screen, whatever is inside.
                        min_width: Val::Px(0.0),
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

/// Spawns `tiles` into `parent`, two to a row: each a button with a name and a status line, that
/// does its action when pressed.
fn spawn_tiles(
    parent: &mut ChildSpawnerCommands,
    asset_server: &AssetServer,
    tiles: Vec<(&'static str, String, MenuAction)>,
) {
    for pair in tiles.chunks(2) {
        parent
            .spawn(Node {
                width: Val::Percent(100.0),
                max_width: Val::Px(560.0),
                column_gap: Val::Px(10.0),
                ..default()
            })
            .with_children(|row| {
                for (name, status, action) in pair {
                    row.spawn((
                        Button,
                        *action,
                        Node {
                            flex_basis: Val::Px(0.0),
                            flex_grow: 1.0,
                            min_width: Val::Px(0.0),
                            flex_direction: FlexDirection::Column,
                            row_gap: Val::Px(2.0),
                            padding: UiRect::axes(Val::Px(14.0), Val::Px(8.0)),
                            border_radius: BorderRadius::all(Val::Px(8.0)),
                            ..default()
                        },
                        BackgroundColor(BUTTON_COLOR),
                        children![
                            (
                                Text::new(*name),
                                TextColor(Color::WHITE),
                                font(asset_server, 22.0),
                            ),
                            (
                                Text::new(status.clone()),
                                TextColor(Color::srgb(0.8, 1.0, 0.8)),
                                font(asset_server, 15.0),
                            ),
                        ],
                    ));
                }
                // Keep a lone last tile half width.
                if pair.len() == 1 {
                    row.spawn(Node {
                        flex_basis: Val::Px(0.0),
                        flex_grow: 1.0,
                        // As wide as a tile, padding included.
                        padding: UiRect::horizontal(Val::Px(14.0)),
                        ..default()
                    });
                }
            });
    }
}

/// Spawns `buttons` into `parent` side by side, sharing the width equally.
fn spawn_equal_row(
    parent: &mut ChildSpawnerCommands,
    asset_server: &AssetServer,
    buttons: &[(&'static str, MenuAction)],
) {
    parent
        .spawn(Node {
            width: Val::Percent(100.0),
            max_width: Val::Px(560.0),
            column_gap: Val::Px(10.0),
            ..default()
        })
        .with_children(|row| {
            for &(label, action) in buttons {
                row.spawn((
                    Button,
                    action,
                    Node {
                        flex_basis: Val::Px(0.0),
                        flex_grow: 1.0,
                        min_width: Val::Px(0.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        padding: UiRect::axes(Val::Px(8.0), Val::Px(8.0)),
                        border_radius: BorderRadius::all(Val::Px(8.0)),
                        ..default()
                    },
                    BackgroundColor(BUTTON_COLOR),
                    children![(
                        Text::new(label),
                        TextColor(Color::WHITE),
                        TextLayout::no_wrap(),
                        font(asset_server, 20.0)
                    )],
                ));
            }
        });
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
        let tiles = GameKind::MENU
            .iter()
            .map(|&kind| match kind.variants() {
                [] => (kind.name(), saved_game_status(kind), MenuAction::Play(kind)),
                variants => (
                    kind.family_name(),
                    // embedded the FontAwesome bars icon via unicode code-point
                    format!("\u{F0C9} {} ways to play", variants.len()),
                    MenuAction::Variants(kind),
                ),
            })
            .collect();
        spawn_tiles(panel, &asset_server, tiles);
        // Two equal buttons side by side. (Not a wrapping row: some phones measured its
        // wrapped height too short, so the About button overlapped the text below it.)
        spawn_equal_row(
            panel,
            &asset_server,
            &[
                // embedded the FontAwesome trophy icon via unicode code-point
                ("\u{F091} Best Times", MenuAction::Stats),
                // embedded the FontAwesome info icon via unicode code-point
                ("\u{F05A} About", MenuAction::About),
            ],
        );
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
    screens: Query<&ChildOf, With<StatsScreen>>,
) {
    if !shown.is_changed() {
        return;
    }
    // The marker is on the panel; remove the whole overlay it sits in, or the old overlays'
    // shading would build up.
    for screen in &screens {
        commands.entity(screen.parent()).despawn();
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
            .init_state::<Popup>()
            .add_systems(OnEnter(Popup::HowToPlay), show_how_to_play)
            .add_systems(OnEnter(Popup::NoMoves), show_no_moves)
            .add_systems(Update, open_no_moves.run_if(in_state(GameState::Play)))
            .add_systems(OnExit(GameState::Play), close_no_moves)
            .add_systems(Update, show_variants)
            .add_systems(
                Update,
                close_popup_on_escape.run_if(not(in_state(Popup::Closed))),
            )
            .add_systems(OnEnter(GameState::Play), first_time_help)
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

/// A pop-up's panel: over everything else, gone when `popup` closes.
fn popup_panel(commands: &mut Commands, popup: Popup) -> Entity {
    let mut panel = Entity::PLACEHOLDER;
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                padding: UiRect::all(Val::Px(16.0)),
                ..default()
            },
            BackgroundColor(Color::BLACK.with_alpha(0.6)),
            GlobalZIndex(3),
            DespawnOnExit(popup),
        ))
        .with_children(|root| {
            panel = root
                .spawn((
                    Node {
                        width: Val::Percent(100.0),
                        max_width: Val::Px(600.0),
                        min_width: Val::Px(0.0),
                        align_items: AlignItems::Center,
                        flex_direction: FlexDirection::Column,
                        row_gap: Val::Px(12.0),
                        padding: UiRect::all(Val::Px(24.0)),
                        border_radius: BorderRadius::all(Val::Px(16.0)),
                        ..default()
                    },
                    // Opaque: pop-ups sit over busy screens.
                    BackgroundColor(Color::srgb(0.06, 0.08, 0.07)),
                ))
                .id();
        });
    panel
}

/// Shows how to play the current game.
fn show_how_to_play(mut commands: Commands, asset_server: Res<AssetServer>, table: Res<Table>) {
    let kind = table.0.kind;
    let panel = popup_panel(&mut commands, Popup::HowToPlay);
    commands.entity(panel).with_children(|panel| {
        // embedded the FontAwesome question icon via unicode code-point
        panel.spawn(line(
            format!("\u{F059} How to play {}", kind.name()),
            font(&asset_server, 28.0),
        ));
        // Games where taps select cards explain their taps themselves.
        let taps_send = !table.0.rules().any_tap_selects();
        let points = kind
            .how_to_play()
            .iter()
            .copied()
            .chain(taps_send.then_some(
                "Drag cards to move them, or tap a card to send it to the first place it fits. If \
             it has nowhere to go, it shakes.",
            ));
        for point in points {
            panel.spawn((
                Node {
                    width: Val::Percent(100.0),
                    column_gap: Val::Px(8.0),
                    ..default()
                },
                children![
                    (
                        Text::new("\u{2022}"),
                        TextColor(Color::WHITE),
                        font(&asset_server, 18.0),
                    ),
                    (
                        Node {
                            flex_grow: 1.0,
                            flex_basis: Val::Px(0.0),
                            min_width: Val::Px(0.0),
                            ..default()
                        },
                        children![(
                            Text::new(point),
                            TextColor(Color::WHITE),
                            font(&asset_server, 18.0),
                        )],
                    ),
                ],
            ));
        }
        // embedded the FontAwesome check icon via unicode code-point
        panel.spawn(text_button(
            "\u{F00C} Got it",
            font(&asset_server, 20.0),
            MenuAction::ClosePopup,
        ));
    });
}

/// Opens the "no moves left" pop-up when a move (or undo) leaves the game stuck.
fn open_no_moves(
    table: Res<Table>,
    popup: Res<State<Popup>>,
    mut next_popup: ResMut<NextState<Popup>>,
) {
    if table.is_changed() && *popup.get() == Popup::Closed && table.0.is_lost() {
        next_popup.set(Popup::NoMoves);
    }
}

fn close_no_moves(popup: Res<State<Popup>>, mut next_popup: ResMut<NextState<Popup>>) {
    if *popup.get() == Popup::NoMoves {
        next_popup.set(Popup::Closed);
    }
}

/// Shows that the game is stuck, with ways out.
fn show_no_moves(mut commands: Commands, asset_server: Res<AssetServer>) {
    let panel = popup_panel(&mut commands, Popup::NoMoves);
    commands.entity(panel).with_children(|panel| {
        panel.spawn(line("No moves left", font(&asset_server, 30.0)));
        panel.spawn(line(
            "There's nowhere left to play a card.",
            font(&asset_server, 18.0),
        ));
        spawn_equal_row(
            panel,
            &asset_server,
            &[
                // embedded the FontAwesome undo icon via unicode code-point
                ("\u{F0E2} Undo", MenuAction::Undo),
                // embedded the FontAwesome plus icon via unicode code-point
                ("\u{F067} New Game", MenuAction::NewGame),
                // embedded the FontAwesome house icon via unicode code-point
                ("\u{F015} Home", MenuAction::Home),
            ],
        );
    });
}

/// Shows the ways to play a family of games, when that pop-up opens.
fn show_variants(mut commands: Commands, asset_server: Res<AssetServer>, popup: Res<State<Popup>>) {
    let Popup::Variants(family) = *popup.get() else {
        return;
    };
    if !popup.is_changed() {
        return;
    }
    let panel = popup_panel(&mut commands, Popup::Variants(family));
    commands.entity(panel).with_children(|panel| {
        panel.spawn(line(family.family_name(), font(&asset_server, 30.0)));
        panel.spawn(line("Choose how to play:", font(&asset_server, 20.0)));
        let tiles = family
            .variants()
            .iter()
            .map(|&kind| {
                (
                    kind.variant_name(),
                    saved_game_status(kind),
                    MenuAction::Play(kind),
                )
            })
            .collect();
        spawn_tiles(panel, &asset_server, tiles);
        // embedded the FontAwesome arrow-left icon via unicode code-point
        panel.spawn(text_button(
            "\u{F060} Back",
            font(&asset_server, 20.0),
            MenuAction::ClosePopup,
        ));
    });
}

fn close_popup_on_escape(keys: Res<ButtonInput<KeyCode>>, mut popup: ResMut<NextState<Popup>>) {
    if keys.just_pressed(KeyCode::Escape) {
        popup.set(Popup::Closed);
    }
}

/// Shows how to play a game the first time it is played.
fn first_time_help(table: Res<Table>, mut popup: ResMut<NextState<Popup>>) {
    let name = format!("seen-{}", table.0.kind.key());
    if storage::load(&name).is_none() {
        storage::save(&name, "yes");
        popup.set(Popup::HowToPlay);
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
#[allow(clippy::too_many_arguments)]
pub fn on_menu_buttons(
    press: On<PointerPress>,
    actions: Query<&MenuAction>,
    table: Res<Table>,
    state: Res<State<GameState>>,
    mut next_deal: ResMut<NextDeal>,
    mut shown: ResMut<StatsShown>,
    mut game_state: ResMut<NextState<GameState>>,
    mut popup: ResMut<NextState<Popup>>,
    mut undo: ResMut<UndoRequested>,
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
            popup.set(Popup::Closed);
            *next_deal = NextDeal {
                kind: table.0.kind,
                resume: false,
            };
            game_state.set(GameState::Deal);
        }
        MenuAction::Play(kind) => {
            *next_deal = NextDeal { kind, resume: true };
            popup.set(Popup::Closed);
            game_state.set(GameState::Deal);
        }
        MenuAction::Home if state != GameState::Menu => {
            popup.set(Popup::Closed);
            game_state.set(GameState::Menu);
        }
        MenuAction::Home => {}
        MenuAction::Stats => game_state.set(GameState::Stats),
        MenuAction::ShowStats(kind) => {
            if shown.0 != kind {
                shown.0 = kind;
            }
        }
        MenuAction::About => game_state.set(GameState::About),
        MenuAction::Variants(kind) => popup.set(Popup::Variants(kind)),
        MenuAction::HowToPlay => popup.set(Popup::HowToPlay),
        MenuAction::ClosePopup => popup.set(Popup::Closed),
        MenuAction::Undo | MenuAction::UndoAll => {
            undo.0 = Some(if action == MenuAction::Undo {
                Undo::Last
            } else {
                Undo::All
            });
            popup.set(Popup::Closed);
        }
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
