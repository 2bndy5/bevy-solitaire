//! Saved progress: the last game of each kind, so it can be picked up again later.
//!
//! The game in play is saved after every move, every few seconds while its clock runs, and when
//! leaving it. Winning a game forgets it.
use bevy::prelude::*;

use crate::{game::Game, games::GameKind, hud::GameClock, state::GameState, storage, table::Table};

/// How often (in seconds) the game in play is saved, so the clock isn't far off after the app is
/// closed without leaving the game first.
const SAVE_INTERVAL: f64 = 10.0;

/// A saved game and the time played on it.
pub struct SavedGame {
    pub game: Game,
    pub seconds: f64,
}

impl SavedGame {
    fn to_text(&self) -> String {
        format!("{}seconds={:.1}\n", self.game.to_text(), self.seconds)
    }

    fn from_text(text: &str) -> Option<Self> {
        let game = Game::from_text(text)?;
        let seconds = text
            .lines()
            .find_map(|line| line.strip_prefix("seconds="))
            .and_then(|value| value.trim().parse::<f64>().ok())
            .filter(|seconds| seconds.is_finite() && *seconds >= 0.0)
            .unwrap_or_default();
        Some(Self { game, seconds })
    }
}

fn storage_name(kind: GameKind) -> String {
    format!("save-{}", kind.key())
}

/// The saved game of `kind`, if there is one.
pub fn load(kind: GameKind) -> Option<SavedGame> {
    storage::load(&storage_name(kind))
        .and_then(|text| SavedGame::from_text(&text))
        .filter(|saved| saved.game.kind == kind)
}

fn save(game: &Game, seconds: f64) {
    let saved = SavedGame {
        game: game.clone(),
        seconds,
    };
    storage::save(&storage_name(game.kind), &saved.to_text());
}

pub struct ProgressPlugin;

impl Plugin for ProgressPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, save_in_play.run_if(in_state(GameState::Play)))
            .add_systems(OnExit(GameState::Play), save_now)
            .add_systems(OnEnter(GameState::Win), forget_won_game);
    }
}

/// Saves the game after each move, and every [`SAVE_INTERVAL`] seconds.
fn save_in_play(
    table: Res<Table>,
    time: Res<Time>,
    clock: Res<GameClock>,
    mut last_saved: Local<f64>,
) {
    let now = time.elapsed_secs_f64();
    if table.is_changed() || now - *last_saved >= SAVE_INTERVAL {
        save(&table.0, clock.elapsed(now));
        *last_saved = now;
    }
}

fn save_now(table: Res<Table>, time: Res<Time>, clock: Res<GameClock>) {
    if !table.0.is_won() {
        save(&table.0, clock.elapsed(time.elapsed_secs_f64()));
    }
}

fn forget_won_game(table: Res<Table>) {
    storage::remove(&storage_name(table.0.kind));
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cards::shuffle_deck;

    #[test]
    fn saved_games_keep_their_time() {
        let mut game = Game::deal(GameKind::Klondike, shuffle_deck());
        game.draw();
        let saved = SavedGame {
            game,
            seconds: 83.24,
        };
        let restored = SavedGame::from_text(&saved.to_text()).unwrap();
        assert_eq!(restored.game.board(), saved.game.board());
        assert_eq!(restored.game.moves, 1);
        assert!((restored.seconds - 83.2).abs() < 1e-9);

        // A missing or nonsense time counts as no time played.
        let text = saved.game.to_text();
        assert_eq!(SavedGame::from_text(&text).unwrap().seconds, 0.0);
        let text = format!("{text}seconds=-5\n");
        assert_eq!(SavedGame::from_text(&text).unwrap().seconds, 0.0);
    }
}
