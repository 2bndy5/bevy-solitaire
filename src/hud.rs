//! The heads-up display: move count, game clock, and the Undo and New Game buttons.
use bevy::{prelude::*, window::PrimaryWindow};

use crate::{
    game::Game,
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

/// Marks the button that takes back the last move.
#[derive(Component)]
pub struct UndoButton;

/// Set when the player asks to undo; the table handles (and clears) it.
#[derive(Resource, Default)]
pub struct UndoRequested(pub bool);

/// Requests an undo when the Undo button is pressed. See [`crate::menu::on_menu_buttons`].
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

/// The banner along the bottom of the screen.
#[derive(Component)]
struct Banner;

#[derive(Component)]
struct BannerText;

/// What the banner should say about `game`, if anything: the game's rules until the first move,
/// and that it's stuck when it is.
fn banner_message(game: &Game) -> Option<String> {
    if game.is_lost() {
        Some("No moves left. Undo, or start a new game.".to_string())
    } else if game.moves == 0 {
        Some(format!("{}: {}", game.kind.name(), game.kind.summary()))
    } else {
        None
    }
}

pub struct HudPlugin;

impl Plugin for HudPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GameClock>()
            .init_resource::<UndoRequested>()
            .add_observer(on_undo_button)
            .add_systems(Startup, spawn_hud)
            .add_systems(OnExit(GameState::Play), pause_clock)
            .add_systems(
                Update,
                (start_clock.run_if(in_state(GameState::Play)), update_hud).chain(),
            )
            .add_systems(Update, (fit_labels, update_banner));
    }
}

fn spawn_hud(mut commands: Commands, asset_server: Res<AssetServer>) {
    // A banner along the bottom for hints. See `update_banner`.
    commands.spawn((
        Banner,
        Node {
            position_type: PositionType::Absolute,
            bottom: Val::Px(12.0),
            left: Val::Px(12.0),
            right: Val::Px(12.0),
            justify_content: JustifyContent::Center,
            ..default()
        },
        Visibility::Hidden,
        children![(
            Node {
                max_width: Val::Px(640.0),
                padding: UiRect::axes(Val::Px(14.0), Val::Px(8.0)),
                border_radius: BorderRadius::all(Val::Px(8.0)),
                ..default()
            },
            BackgroundColor(Color::BLACK.with_alpha(0.7)),
            children![(
                Text::default(),
                TextColor(Color::WHITE),
                TextLayout::justify(Justify::Center),
                font(&asset_server, 16.0),
                BannerText,
            )],
        )],
    ));

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
                Node {
                    column_gap: Val::Px(14.0),
                    ..default()
                },
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
                        UndoButton
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
) {
    let narrow = window.width() < NARROW_HUD_PX;
    for (label, mut text) in &mut labels {
        let wanted = label.text(narrow);
        if text.0 != wanted {
            text.0 = wanted;
        }
    }
}

/// Shows or hides the banner, and sets its text, while a game is being played.
fn update_banner(
    table: Res<Table>,
    state: Res<State<GameState>>,
    mut banner: Single<&mut Visibility, With<Banner>>,
    mut text: Single<&mut Text, With<BannerText>>,
) {
    let message = (*state.get() == GameState::Play)
        .then(|| banner_message(&table.0))
        .flatten();
    let visibility = if message.is_some() {
        Visibility::Inherited
    } else {
        Visibility::Hidden
    };
    if **banner != visibility {
        **banner = visibility;
    }
    if let Some(message) = message
        && text.0 != message
    {
        text.0 = message;
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
    fn banner_explains_then_warns() {
        use crate::{
            cards::{Card, Rank, Suit, new_deck},
            game::{Board, Pile, Slot},
            games::GameKind,
        };
        let mut game = Game::deal(GameKind::Golf, new_deck());
        assert!(banner_message(&game).unwrap().starts_with("Golf: "));
        assert!(game.draw());
        assert_eq!(banner_message(&game), None);

        // A Nine with nothing left to draw and a Five on the waste: stuck.
        let mut board = Board::new(GameKind::Golf.rules().piles());
        *board.pile_mut(Pile::Waste) = vec![Slot::up(Card::new(Rank::Five, Suit::Hearts))];
        *board.pile_mut(Pile::Tableau(0)) = vec![Slot::up(Card::new(Rank::Nine, Suit::Clubs))];
        let stuck = Game::with_board(GameKind::Golf, board);
        assert!(banner_message(&stuck).unwrap().starts_with("No moves left"));
    }

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
