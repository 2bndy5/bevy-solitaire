//! The solitaire games on offer, each with its own [`Rules`].
use crate::game::Rules;

mod freecell;
mod golf;
mod klondike;
mod pyramid;
mod rows;
mod spider;
mod tripeaks;

pub use freecell::FreeCell;
pub use golf::Golf;
pub use klondike::Klondike;
pub use pyramid::Pyramid;
pub use spider::Spider;
pub use tripeaks::TriPeaks;

/// One of the games on offer.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GameKind {
    #[default]
    Klondike,
    FreeCell,
    Golf,
    TriPeaks,
    Pyramid,
    /// Spider with one suit.
    Spider1,
    /// Spider with two suits.
    Spider2,
    /// Spider with four suits.
    Spider4,
}

impl GameKind {
    /// Every game, in the order the main menu lists them.
    pub const ALL: [GameKind; 8] = [
        GameKind::Klondike,
        GameKind::FreeCell,
        GameKind::Golf,
        GameKind::TriPeaks,
        GameKind::Pyramid,
        GameKind::Spider1,
        GameKind::Spider2,
        GameKind::Spider4,
    ];

    /// The game's rules.
    pub fn rules(self) -> &'static dyn Rules {
        match self {
            GameKind::Klondike => &Klondike,
            GameKind::FreeCell => &FreeCell,
            GameKind::Golf => &Golf,
            GameKind::TriPeaks => &TriPeaks,
            GameKind::Pyramid => &Pyramid,
            GameKind::Spider1 => &Spider { suits: 1 },
            GameKind::Spider2 => &Spider { suits: 2 },
            GameKind::Spider4 => &Spider { suits: 4 },
        }
    }

    /// A short, stable name for the game, used for saved games and stats.
    pub fn key(self) -> &'static str {
        match self {
            GameKind::Klondike => "klondike",
            GameKind::FreeCell => "freecell",
            GameKind::Golf => "golf",
            GameKind::TriPeaks => "tripeaks",
            GameKind::Pyramid => "pyramid",
            GameKind::Spider1 => "spider1",
            GameKind::Spider2 => "spider2",
            GameKind::Spider4 => "spider4",
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
            GameKind::Pyramid => "Pyramid",
            GameKind::Spider1 => "Spider (1 suit)",
            GameKind::Spider2 => "Spider (2 suits)",
            GameKind::Spider4 => "Spider (4 suits)",
        }
    }

    /// How the game is played, one point per line, the goal first.
    pub fn how_to_play(self) -> &'static [&'static str] {
        match self {
            GameKind::Klondike => &[
                "Move every card to the four foundations, building each up by suit from Ace to \
                 King.",
                "Build the seven columns down in alternating colors. Only a King goes in an empty \
                 column.",
                "Tap the stock to draw a card. When it runs out, tap it to turn the waste back \
                 over.",
                "Once the stock is empty and every card is face up, the game finishes itself.",
            ],
            GameKind::FreeCell => &[
                "Move every card to the four foundations (top right), building each up by suit \
                 from Ace to King.",
                "Every card is dealt face up. Build the columns down in alternating colors; any \
                 card goes in an empty column.",
                "Each of the four free cells (top left) holds one card.",
                "How many cards you can move at once depends on the free cells and empty columns \
                 there are to shuffle them through.",
            ],
            GameKind::Golf => &[
                "Clear all seven columns onto the waste.",
                "A column's top card goes on the waste if it is one rank above or below the \
                 waste's top card, in any suit. Kings and Aces don't connect.",
                "Tap the stock to turn up a new waste card. You only get one pass through it.",
            ],
            GameKind::TriPeaks => &[
                "Clear all three peaks onto the waste.",
                "An uncovered card goes on the waste if it is one rank above or below the \
                 waste's top card, in any suit. Kings and Aces connect.",
                "Cards turn face up once nothing covers them.",
                "Tap the stock to turn up a new waste card. You only get one pass through it.",
            ],
            GameKind::Pyramid => &[
                "Clear the pyramid by removing pairs of uncovered cards that add up to 13 \
                 (Ace = 1, Jack = 11, Queen = 12).",
                "Tap a card to select it, then tap its partner to remove both (or drag one onto \
                 the other). Tapping a card that doesn't match selects it instead.",
                "The top card of the waste counts as uncovered.",
                "Kings are removed on their own: tap them.",
                "Tap the stock to turn up a new waste card; tap it when empty to turn the waste \
                 back over.",
            ],
            GameKind::Spider1 | GameKind::Spider2 | GameKind::Spider4 => &[
                "Build runs from King down to Ace in one suit. A complete run leaves the table.",
                "Any card goes on a card one rank higher, whatever its suit, and any card goes \
                 in an empty column.",
                "Only runs of one suit move together.",
                "Tap the stock to deal a card onto every column (not while a column is empty).",
                "Once the stock is empty and every column is one run of one suit, the game \
                 finishes itself.",
            ],
        }
    }

    /// The games' entries in the main menu: one tile per game, except that the Spider variants
    /// share a tile that opens a menu of their own.
    pub const MENU: [GameKind; 6] = [
        GameKind::Klondike,
        GameKind::FreeCell,
        GameKind::Golf,
        GameKind::TriPeaks,
        GameKind::Pyramid,
        GameKind::Spider1,
    ];

    /// The other games played the same way with different options, if any (Spider's suit
    /// counts).
    pub fn variants(self) -> &'static [GameKind] {
        match self {
            GameKind::Spider1 | GameKind::Spider2 | GameKind::Spider4 => {
                &[GameKind::Spider1, GameKind::Spider2, GameKind::Spider4]
            }
            _ => &[],
        }
    }

    /// The name of the family of games this belongs to (e.g. "Spider" for every suit count).
    pub fn family_name(self) -> &'static str {
        match self {
            GameKind::Spider1 | GameKind::Spider2 | GameKind::Spider4 => "Spider",
            kind => kind.name(),
        }
    }

    /// What tells this variant apart from the others in its family (e.g. "2 suits").
    pub fn variant_name(self) -> &'static str {
        match self {
            GameKind::Spider1 => "1 suit (easy)",
            GameKind::Spider2 => "2 suits (medium)",
            GameKind::Spider4 => "4 suits (hard)",
            kind => kind.name(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_game_is_on_the_menu() {
        for kind in GameKind::ALL {
            assert!(
                GameKind::MENU
                    .iter()
                    .any(|&entry| entry == kind || entry.variants().contains(&kind)),
                "{kind:?}"
            );
        }
    }

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
