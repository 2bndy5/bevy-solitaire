//! Best times: how many games of each kind were won, and the five fastest wins.
use bevy::prelude::*;

use crate::{games::GameKind, hud::GameClock, state::GameState, storage, table::Table};

/// How many of the fastest wins are kept for each game.
pub const BEST_TIMES: usize = 5;

/// One won game.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Win {
    pub seconds: f64,
    pub moves: u32,
}

impl Win {
    /// Does this win rank ahead of `other`? Faster wins rank first; for equal times, fewer moves.
    fn beats(&self, other: &Win) -> bool {
        (self.seconds, self.moves) < (other.seconds, other.moves)
    }
}

/// The record for one kind of game.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Stats {
    /// How many games were won.
    pub won: u32,
    /// The fastest wins, fastest first.
    pub best: Vec<Win>,
}

impl Stats {
    /// Counts `win`, and returns its place among the best times (0 for the fastest), if it made
    /// the list. A win that ties an earlier one ranks after it.
    pub fn record(&mut self, win: Win) -> Option<usize> {
        self.won += 1;
        let place = self
            .best
            .iter()
            .position(|best| win.beats(best))
            .unwrap_or(self.best.len());
        if place >= BEST_TIMES {
            return None;
        }
        self.best.insert(place, win);
        self.best.truncate(BEST_TIMES);
        Some(place)
    }

    /// The record as text: a `won=` line, then a `best=<seconds>,<moves>` line per best time.
    fn to_text(&self) -> String {
        let mut text = format!("won={}\n", self.won);
        for win in &self.best {
            text.push_str(&format!("best={:.1},{}\n", win.seconds, win.moves));
        }
        text
    }

    /// Reads a record written by [`Stats::to_text`], skipping anything it doesn't understand.
    fn from_text(text: &str) -> Self {
        let mut stats = Self::default();
        for (key, value) in text.lines().filter_map(|line| line.split_once('=')) {
            match key.trim() {
                "won" => stats.won = value.trim().parse().unwrap_or_default(),
                "best" => {
                    let win = value.split_once(',').and_then(|(seconds, moves)| {
                        let seconds = seconds.trim().parse::<f64>().ok()?;
                        let moves = moves.trim().parse().ok()?;
                        (seconds.is_finite() && seconds >= 0.0).then_some(Win { seconds, moves })
                    });
                    stats.best.extend(win);
                }
                _ => {}
            }
        }
        stats.best.sort_by(|a, b| {
            (a.seconds, a.moves)
                .partial_cmp(&(b.seconds, b.moves))
                .unwrap()
        });
        stats.best.truncate(BEST_TIMES);
        stats
    }
}

fn storage_name(kind: GameKind) -> String {
    format!("stats-{}", kind.key())
}

/// The record for `kind`.
pub fn load(kind: GameKind) -> Stats {
    storage::load(&storage_name(kind))
        .map(|text| Stats::from_text(&text))
        .unwrap_or_default()
}

/// The game just won: the updated record, and the win's place among the best times, if it made
/// the list.
#[derive(Resource, Default)]
pub struct LastWin {
    pub stats: Stats,
    pub place: Option<usize>,
}

pub struct StatsPlugin;

impl Plugin for StatsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<LastWin>()
            .add_systems(OnEnter(GameState::Win), record_win);
    }
}

/// Records the win (the clock has already stopped).
fn record_win(
    table: Res<Table>,
    time: Res<Time>,
    clock: Res<GameClock>,
    mut last: ResMut<LastWin>,
) {
    let kind = table.0.kind;
    let mut stats = load(kind);
    let win = Win {
        seconds: clock.elapsed(time.elapsed_secs_f64()),
        moves: table.0.moves,
    };
    let place = stats.record(win);
    storage::save(&storage_name(kind), &stats.to_text());
    *last = LastWin { stats, place };
}

#[cfg(test)]
mod tests {
    use super::*;

    fn win(seconds: f64, moves: u32) -> Win {
        Win { seconds, moves }
    }

    #[test]
    fn keeps_the_five_fastest() {
        let mut stats = Stats::default();
        assert_eq!(stats.record(win(300.0, 120)), Some(0));
        assert_eq!(stats.record(win(200.0, 150)), Some(0));
        assert_eq!(stats.record(win(250.0, 100)), Some(1));
        // A tie on time goes by moves; a full tie ranks after the earlier win.
        assert_eq!(stats.record(win(250.0, 90)), Some(1));
        assert_eq!(stats.record(win(250.0, 90)), Some(2));
        assert_eq!(stats.best.len(), 5);
        // Too slow to make the list, but still counted.
        assert_eq!(stats.record(win(400.0, 80)), None);
        assert_eq!(stats.won, 6);
        // A new fastest pushes the slowest off.
        assert_eq!(stats.record(win(100.0, 200)), Some(0));
        assert_eq!(stats.best.len(), BEST_TIMES);
        let times: Vec<f64> = stats.best.iter().map(|w| w.seconds).collect();
        assert_eq!(times, [100.0, 200.0, 250.0, 250.0, 250.0]);
    }

    #[test]
    fn round_trips_through_text() {
        let mut stats = Stats::default();
        for (seconds, moves) in [(95.5, 110), (300.0, 98), (61.2, 140)] {
            stats.record(win(seconds, moves));
        }
        assert_eq!(Stats::from_text(&stats.to_text()), stats);
        assert_eq!(Stats::from_text(""), Stats::default());
        // Nonsense lines are skipped, and the list is put back in order.
        let text = "won=x\nbest=10\nbest=-3,5\nbest=90,5\nbest=30,7\nwhat\n";
        let stats = Stats::from_text(text);
        assert_eq!(stats.won, 0);
        assert_eq!(stats.best, [win(30.0, 7), win(90.0, 5)]);
    }
}
