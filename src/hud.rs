//! The heads-up display: move count, game clock, and the game buttons (home, how to play,
//! settings, undo, undo all and new game).
use bevy::{prelude::*, window::PrimaryWindow};

use crate::{
    menu::{MenuAction, button_node, font},
    settings::SettingsButton,
    state::GameState,
    table::Table,
};

/// Windows narrower than this (in logical pixels) show only icons on the HUD's buttons.
const NARROW_HUD_PX: f32 = 560.0;

/// Time played on the current game.
///
/// The clock starts on the first move, pauses while the game isn't being played (on the main menu,
/// say) and stops when the game is won.
#[derive(Resource, Default)]
pub struct GameClock {
    /// Seconds played before the clock last started.
    banked: f64,
    /// When the clock last started, in seconds since the app started, if it is running.
    running_since: Option<f64>,
}

impl GameClock {
    /// A stopped clock showing `seconds` already played.
    pub fn resumed(seconds: f64) -> Self {
        Self {
            banked: seconds,
            running_since: None,
        }
    }

    /// Seconds played so far, given the current time `now`.
    pub fn elapsed(&self, now: f64) -> f64 {
        self.banked + self.running_since.map_or(0.0, |start| now - start)
    }

    fn start(&mut self, now: f64) {
        self.running_since.get_or_insert(now);
    }

    fn pause(&mut self, now: f64) {
        self.banked = self.elapsed(now);
        self.running_since = None;
    }
}

/// Formats `seconds` as `m:ss`, or `h:mm:ss` from an hour on.
pub fn format_duration(seconds: f64) -> String {
    let total = seconds.max(0.0) as u64;
    let (hours, minutes, seconds) = (total / 3600, total / 60 % 60, total % 60);
    if hours > 0 {
        format!("{hours}:{minutes:02}:{seconds:02}")
    } else {
        format!("{minutes}:{seconds:02}")
    }
}

/// What the player asked to take back.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Undo {
    /// The last move.
    Last,
    /// Every move: back to the deal.
    All,
}

/// Set when the player asks to undo; the table handles (and clears) it.
#[derive(Resource, Default)]
pub struct UndoRequested(pub Option<Undo>);

#[derive(Component)]
struct MovesText;

/// A button label that shrinks to just its icon on narrow windows.
#[derive(Component)]
struct ResponsiveLabel {
    icon: &'static str,
    words: &'static str,
}

impl ResponsiveLabel {
    fn text(&self, narrow: bool) -> String {
        if narrow || self.words.is_empty() {
            self.icon.to_string()
        } else {
            format!("{} {}", self.icon, self.words)
        }
    }
}

/// A HUD button with a [`ResponsiveLabel`], tagged with `marker`.
fn hud_button(label: ResponsiveLabel, font: TextFont, marker: impl Bundle) -> impl Bundle {
    (
        button_node(UiRect::axes(Val::Px(10.0), Val::Px(6.0))),
        marker,
        children![(
            Text::new(label.text(false)),
            TextColor(Color::WHITE),
            TextLayout::no_wrap(),
            font,
            label,
        )],
    )
}

#[derive(Component)]
struct ClockText;

/// The moves and time texts.
type StatsText = Or<(With<MovesText>, With<ClockText>)>;

/// Holds the moves and time.
#[derive(Component)]
struct StatsBox;

pub struct HudPlugin;

impl Plugin for HudPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GameClock>()
            .init_resource::<UndoRequested>()
            .add_systems(Startup, spawn_hud)
            .add_systems(OnExit(GameState::Play), pause_clock)
            .add_systems(
                Update,
                (start_clock.run_if(in_state(GameState::Play)), update_hud).chain(),
            )
            .add_systems(Update, fit_labels);
    }
}

fn spawn_hud(mut commands: Commands, asset_server: Res<AssetServer>) {
    let text_font = font(&asset_server, 20.0);
    let button_font = font(&asset_server, 18.0);
    commands.spawn((
        Node {
            width: Val::Percent(100.0),
            padding: UiRect::axes(Val::Px(12.0), Val::Px(5.0)),
            justify_content: JustifyContent::SpaceBetween,
            align_items: AlignItems::Center,
            column_gap: Val::Px(8.0),
            ..default()
        },
        children![
            (
                // Moves and time: side by side, or one above the other on narrow windows (see
                // `fit_labels`).
                Node {
                    column_gap: Val::Px(14.0),
                    ..default()
                },
                StatsBox,
                children![
                    (
                        Text::new("Moves 0"),
                        TextColor(Color::WHITE),
                        TextLayout::no_wrap(),
                        text_font.clone(),
                        MovesText
                    ),
                    (
                        // embedded the FontAwesome clock icon via unicode code-point
                        Text::new("\u{F017} 0:00"),
                        TextColor(Color::WHITE),
                        TextLayout::no_wrap(),
                        text_font,
                        ClockText
                    ),
                ],
            ),
            (
                Node {
                    column_gap: Val::Px(8.0),
                    ..default()
                },
                children![
                    hud_button(
                        // embedded the FontAwesome house icon via unicode code-point
                        ResponsiveLabel {
                            icon: "\u{F015}",
                            words: ""
                        },
                        button_font.clone(),
                        MenuAction::Home
                    ),
                    hud_button(
                        // embedded the FontAwesome question icon via unicode code-point
                        ResponsiveLabel {
                            icon: "\u{F128}",
                            words: ""
                        },
                        button_font.clone(),
                        MenuAction::HowToPlay
                    ),
                    hud_button(
                        // embedded the FontAwesome cog icon via unicode code-point
                        ResponsiveLabel {
                            icon: "\u{F013}",
                            words: ""
                        },
                        button_font.clone(),
                        SettingsButton
                    ),
                    hud_button(
                        // embedded the FontAwesome undo icon via unicode code-point
                        ResponsiveLabel {
                            icon: "\u{F0E2}",
                            words: "Undo"
                        },
                        button_font.clone(),
                        MenuAction::Undo
                    ),
                    hud_button(
                        // embedded the FontAwesome fast-backward icon via unicode code-point
                        ResponsiveLabel {
                            icon: "\u{F049}",
                            words: ""
                        },
                        button_font.clone(),
                        MenuAction::UndoAll
                    ),
                    hud_button(
                        // embedded the FontAwesome plus icon via unicode code-point
                        ResponsiveLabel {
                            icon: "\u{F067}",
                            words: "New Game"
                        },
                        button_font,
                        MenuAction::NewGame
                    ),
                ],
            ),
        ],
    ));
}

/// Shortens the button labels on narrow windows, so the HUD fits on one line.
fn fit_labels(
    window: Single<&Window, (With<PrimaryWindow>, Changed<Window>)>,
    mut labels: Query<(&ResponsiveLabel, &mut Text)>,
    mut stats_box: Single<&mut Node, With<StatsBox>>,
    mut stats_fonts: Query<&mut TextFont, StatsText>,
) {
    let narrow = window.width() < NARROW_HUD_PX;
    for (label, mut text) in &mut labels {
        let wanted = label.text(narrow);
        if text.0 != wanted {
            text.0 = wanted;
        }
    }
    // Stack the moves and time on narrow windows, to leave room for the buttons.
    let direction = if narrow {
        FlexDirection::Column
    } else {
        FlexDirection::Row
    };
    if stats_box.flex_direction != direction {
        stats_box.flex_direction = direction;
    }
    let size = FontSize::Px(if narrow { 15.0 } else { 20.0 });
    for mut font in &mut stats_fonts {
        if font.font_size != size {
            font.font_size = size;
        }
    }
}

/// Starts the clock on the game's first move (or right away, for a resumed game).
fn start_clock(table: Res<Table>, time: Res<Time>, mut clock: ResMut<GameClock>) {
    if clock.running_since.is_none() && table.0.moves > 0 {
        clock.start(time.elapsed_secs_f64());
    }
}

fn pause_clock(time: Res<Time>, mut clock: ResMut<GameClock>) {
    clock.pause(time.elapsed_secs_f64());
}

fn update_hud(
    table: Res<Table>,
    time: Res<Time>,
    clock: Res<GameClock>,
    mut moves: Single<&mut Text, (With<MovesText>, Without<ClockText>)>,
    mut clock_text: Single<&mut Text, (With<ClockText>, Without<MovesText>)>,
) {
    if table.is_changed() {
        moves.0 = format!("Moves {}", table.0.moves);
    }
    let shown = format!(
        "\u{F017} {}",
        format_duration(clock.elapsed(time.elapsed_secs_f64()))
    );
    if clock_text.0 != shown {
        clock_text.0 = shown;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats_durations() {
        assert_eq!(format_duration(0.0), "0:00");
        assert_eq!(format_duration(59.9), "0:59");
        assert_eq!(format_duration(61.0), "1:01");
        assert_eq!(format_duration(3600.0 + 62.0), "1:01:02");
    }

    #[test]
    fn clock_runs_from_start_until_paused() {
        let mut clock = GameClock::default();
        assert_eq!(clock.elapsed(100.0), 0.0);
        clock.start(10.0);
        assert_eq!(clock.elapsed(25.0), 15.0);
        clock.start(20.0); // already running
        assert_eq!(clock.elapsed(25.0), 15.0);
        clock.pause(30.0);
        assert_eq!(clock.elapsed(100.0), 20.0);
        // Resuming carries on from where it left off.
        clock.start(200.0);
        assert_eq!(clock.elapsed(205.0), 25.0);

        let resumed = GameClock::resumed(61.0);
        assert_eq!(resumed.elapsed(1000.0), 61.0);
    }
}
