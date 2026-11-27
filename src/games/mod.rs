//! The solitaire games on offer, each with its own [`Rules`].
use crate::game::Rules;

mod klondike;

pub use klondike::Klondike;

/// One of the games on offer.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GameKind {
    #[default]
    Klondike,
}

impl GameKind {
    /// Every game, in the order the main menu lists them.
    pub const ALL: [GameKind; 1] = [GameKind::Klondike];

    /// The game's rules.
    pub fn rules(self) -> &'static dyn Rules {
        match self {
            GameKind::Klondike => &Klondike,
        }
    }

    /// A short, stable name for the game, used for saved games and stats.
    pub fn key(self) -> &'static str {
        match self {
            GameKind::Klondike => "klondike",
        }
    }

    pub fn from_key(key: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|kind| kind.key() == key)
    }

    /// The game's name, as shown to the player.
    pub fn name(self) -> &'static str {
        match self {
            GameKind::Klondike => "Klondike",
        }
    }

    /// A one-line summary of how the game is played.
    pub fn summary(self) -> &'static str {
        match self {
            GameKind::Klondike => {
                "Build the foundations up by suit from Ace to King. Build the tableau down in \
                 alternating colors; draw from the stock one card at a time."
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
