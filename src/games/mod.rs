//! The solitaire games on offer, each with its own [`Rules`].
use crate::game::Rules;

mod freecell;
mod golf;
mod klondike;
mod rows;
mod tripeaks;

pub use freecell::FreeCell;
pub use golf::Golf;
pub use klondike::Klondike;
pub use tripeaks::TriPeaks;

/// One of the games on offer.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GameKind {
    #[default]
    Klondike,
    FreeCell,
    Golf,
    TriPeaks,
}

impl GameKind {
    /// Every game, in the order the main menu lists them.
    pub const ALL: [GameKind; 4] = [
        GameKind::Klondike,
        GameKind::FreeCell,
        GameKind::Golf,
        GameKind::TriPeaks,
    ];

    /// The game's rules.
    pub fn rules(self) -> &'static dyn Rules {
        match self {
            GameKind::Klondike => &Klondike,
            GameKind::FreeCell => &FreeCell,
            GameKind::Golf => &Golf,
            GameKind::TriPeaks => &TriPeaks,
        }
    }

    /// A short, stable name for the game, used for saved games and stats.
    pub fn key(self) -> &'static str {
        match self {
            GameKind::Klondike => "klondike",
            GameKind::FreeCell => "freecell",
            GameKind::Golf => "golf",
            GameKind::TriPeaks => "tripeaks",
        }
    }

    pub fn from_key(key: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|kind| kind.key() == key)
    }

    /// The game's name, as shown to the player.
    pub fn name(self) -> &'static str {
        match self {
            GameKind::Klondike => "Klondike",
            GameKind::FreeCell => "FreeCell",
            GameKind::Golf => "Golf",
            GameKind::TriPeaks => "TriPeaks",
        }
    }

    /// A one-line summary of how the game is played.
    pub fn summary(self) -> &'static str {
        match self {
            GameKind::Klondike => {
                "Build the foundations up by suit from Ace to King. Build the tableau down in \
                 alternating colors; draw from the stock one card at a time."
            }
            GameKind::FreeCell => {
                "Every card is dealt face up. Build the tableau down in alternating colors, using \
                 four free cells to hold single cards."
            }
            GameKind::Golf => {
                "Clear the seven columns: play a top card onto the waste if it is one rank above \
                 or below the waste's top card. Tap the stock for a new waste card."
            }
            GameKind::TriPeaks => {
                "Clear the three peaks: play an uncovered card onto the waste if it is one rank \
                 above or below the waste's top card (King and Ace connect)."
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keys_round_trip() {
        for kind in GameKind::ALL {
            assert_eq!(GameKind::from_key(kind.key()), Some(kind));
        }
        assert_eq!(GameKind::from_key("chess"), None);
    }

    #[test]
    fn every_game_deals_its_whole_deck() {
        for kind in GameKind::ALL {
            let rules = kind.rules();
            let game = crate::game::Game::deal(kind, rules.deck());
            assert_eq!(game.board().cards().count(), rules.deck().len(), "{kind:?}");
            assert!(!game.is_won());
            // Every pile has a spot within the table's columns.
            for pile in game.piles() {
                let spot = rules.spot(pile);
                assert!(
                    (0.0..rules.shape().columns as f32).contains(&spot.column),
                    "{kind:?} {pile:?}"
                );
            }
        }
    }
}
