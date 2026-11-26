//! The heads-up display: move count, game clock, and the Undo and New Game buttons.
use bevy::prelude::*;

use crate::{
    menu::{button, font, start_button},
    state::GameState,
    table::Table,
};

/// When the current game's clock started and stopped, in seconds since the app started.
///
/// The clock starts on the first move and stops when the game is won.
#[derive(Resource, Default)]
pub struct GameClock {
    started: Option<f64>,
    stopped: Option<f64>,
}

impl GameClock {
    /// Seconds played so far, given the current time `now`.
    pub fn elapsed(&self, now: f64) -> f64 {
        match self.started {
            Some(start) => self.stopped.unwrap_or(now) - start,
            None => 0.0,
        }
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

/// Marks the button that takes back the last move.
#[derive(Component)]
pub struct UndoButton;

/// Set when the player asks to undo; the table handles (and clears) it.
#[derive(Resource, Default)]
pub struct UndoRequested(pub bool);

/// Requests an undo when the Undo button is pressed. See [`crate::menu::on_start_button`].
fn on_undo_button(
    press: On<PointerPress>,
    undo_buttons: Query<(), With<UndoButton>>,
    mut requested: ResMut<UndoRequested>,
) {
    if undo_buttons.contains(press.entity) {
        requested.0 = true;
    }
}

#[derive(Component)]
struct MovesText;

#[derive(Component)]
struct ClockText;

pub struct HudPlugin;

impl Plugin for HudPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GameClock>()
            .init_resource::<UndoRequested>()
            .add_observer(on_undo_button)
            .add_systems(Startup, spawn_hud)
            .add_systems(OnEnter(GameState::Deal), reset_clock)
            .add_systems(OnEnter(GameState::Win), stop_clock)
            .add_systems(Update, (start_clock, update_hud).chain());
    }
}

fn spawn_hud(mut commands: Commands, asset_server: Res<AssetServer>) {
    let text_font = font(&asset_server, 20.0);
    let button_font = font(&asset_server, 18.0);
    let padding = UiRect::axes(Val::Px(10.0), Val::Px(6.0));
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
                Node {
                    column_gap: Val::Px(14.0),
                    ..default()
                },
                children![
                    (
                        Text::new("Moves 0"),
                        TextColor(Color::WHITE),
                        text_font.clone(),
                        MovesText
                    ),
                    (
                        // embedded the FontAwesome clock icon via unicode code-point
                        Text::new("\u{F017} 0:00"),
                        TextColor(Color::WHITE),
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
                    // embedded the FontAwesome undo icon via unicode code-point
                    button("\u{F0E2} Undo", button_font.clone(), padding, UndoButton),
                    start_button("\u{F0453} New Game", button_font),
                ],
            ),
        ],
    ));
}

fn reset_clock(mut clock: ResMut<GameClock>) {
    *clock = GameClock::default();
}

/// Starts the clock on the game's first move.
fn start_clock(table: Res<Table>, time: Res<Time>, mut clock: ResMut<GameClock>) {
    if clock.started.is_none() && table.0.moves > 0 {
        clock.started = Some(time.elapsed_secs_f64());
    }
}

fn stop_clock(time: Res<Time>, mut clock: ResMut<GameClock>) {
    if clock.started.is_some() {
        clock.stopped = Some(time.elapsed_secs_f64());
    }
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
    fn clock_runs_from_start_until_stopped() {
        let mut clock = GameClock::default();
        assert_eq!(clock.elapsed(100.0), 0.0);
        clock.started = Some(10.0);
        assert_eq!(clock.elapsed(25.0), 15.0);
        clock.stopped = Some(30.0);
        assert_eq!(clock.elapsed(100.0), 20.0);
    }
}
