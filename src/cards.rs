//! Playing card primitives, adapted from `bevy-52-card-pickup`.
use rand::{rng, seq::SliceRandom};
use std::fmt;

/// Rendered card width (in world units).
pub const CARD_W: f32 = 84.0;
/// Rendered card height (in world units).
pub const CARD_H: f32 = 120.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Card {
    pub rank: Rank,
    pub suit: Suit,
    /// Which copy of this rank and suit the card is, for games played with more than one deck
    /// (Spider). 0 for the first (or only) copy.
    pub copy: u8,
}

impl Card {
    pub const fn new(rank: Rank, suit: Suit) -> Self {
        Self {
            rank,
            suit,
            copy: 0,
        }
    }

    /// Copy number `copy` of this card's rank and suit.
    pub const fn with_copy(self, copy: u8) -> Self {
        Self { copy, ..self }
    }

    /// The card as its first copy: what its face looks like.
    pub const fn face(self) -> Self {
        self.with_copy(0)
    }

    /// A name for this exact card, used in saved games: its [file stem](Card::file_stem),
    /// followed by `.` and its copy number for copies after the first (e.g. `12h.1`).
    pub fn key(&self) -> String {
        match self.copy {
            0 => self.file_stem(),
            copy => format!("{}.{copy}", self.file_stem()),
        }
    }

    /// Reads a name written by [`Card::key`].
    pub fn from_key(key: &str) -> Option<Self> {
        let (stem, copy) = match key.split_once('.') {
            Some((stem, copy)) => (stem, copy.parse().ok().filter(|&copy| copy > 0)?),
            None => (key, 0),
        };
        Some(Self::from_file_stem(stem)?.with_copy(copy))
    }

    /// The card's image file name without its extension, as used by every deck: its rank's
    /// number and its suit's initial, e.g. `12h` for the Queen of Hearts.
    pub fn file_stem(&self) -> String {
        format!("{}{}", self.rank.as_u8(), self.suit.short_name())
    }

    /// Reads a name written by [`Card::file_stem`].
    pub fn from_file_stem(stem: &str) -> Option<Self> {
        let (rank, suit) = stem.split_at_checked(stem.len().checked_sub(1)?)?;
        let rank = rank.parse::<u8>().ok()?;
        let rank = *Rank::ALL.iter().find(|r| r.as_u8() == rank)?;
        let suit = *Suit::ALL.iter().find(|s| s.short_name() == suit)?;
        Some(Self::new(rank, suit))
    }

    /// Path (relative to the `assets` folder) of this card's face image.
    pub fn face_resource_name(&self) -> String {
        format!(
            "images/{}/{}{}.png",
            self.suit.folder_name(),
            self.rank.as_u8(),
            self.suit.short_name()
        )
    }

    pub fn is_red(&self) -> bool {
        self.suit.is_red()
    }
}

impl fmt::Display for Card {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}-{}", self.rank, self.suit)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Rank {
    Ace = 1,
    Two,
    Three,
    Four,
    Five,
    Six,
    Seven,
    Eight,
    Nine,
    Ten,
    Jack,
    Queen,
    King,
}

impl Rank {
    pub const ALL: [Rank; 13] = [
        Rank::Ace,
        Rank::Two,
        Rank::Three,
        Rank::Four,
        Rank::Five,
        Rank::Six,
        Rank::Seven,
        Rank::Eight,
        Rank::Nine,
        Rank::Ten,
        Rank::Jack,
        Rank::Queen,
        Rank::King,
    ];

    pub fn as_u8(&self) -> u8 {
        *self as u8
    }
}

impl fmt::Display for Rank {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Rank::Ace => write!(f, "A"),
            Rank::Jack => write!(f, "J"),
            Rank::Queen => write!(f, "Q"),
            Rank::King => write!(f, "K"),
            other => write!(f, "{}", other.as_u8()),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Suit {
    Clubs,
    Diamonds,
    Hearts,
    Spades,
}

impl Suit {
    pub const ALL: [Suit; 4] = [Suit::Clubs, Suit::Diamonds, Suit::Hearts, Suit::Spades];

    pub fn is_red(&self) -> bool {
        matches!(self, Suit::Diamonds | Suit::Hearts)
    }

    fn folder_name(&self) -> &'static str {
        match self {
            Suit::Clubs => "Clubs",
            Suit::Diamonds => "Diamonds",
            Suit::Hearts => "Hearts",
            Suit::Spades => "Spades",
        }
    }

    fn short_name(&self) -> &'static str {
        match self {
            Suit::Clubs => "c",
            Suit::Diamonds => "d",
            Suit::Hearts => "h",
            Suit::Spades => "s",
        }
    }
}

impl fmt::Display for Suit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.folder_name().to_lowercase())
    }
}

/// Returns all 52 cards in a standard (unshuffled) order.
pub fn new_deck() -> Vec<Card> {
    Suit::ALL
        .iter()
        .flat_map(|&suit| Rank::ALL.iter().map(move |&rank| Card::new(rank, suit)))
        .collect()
}

/// Returns `deck` in a random order.
pub fn shuffled(mut deck: Vec<Card>) -> Vec<Card> {
    deck.shuffle(&mut rng());
    deck
}

/// Returns all 52 cards in a random order.
#[cfg(test)]
pub fn shuffle_deck() -> Vec<Card> {
    shuffled(new_deck())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deck_has_52_unique_cards() {
        let deck = shuffle_deck();
        let unique: std::collections::HashSet<_> = deck.iter().collect();
        assert_eq!(deck.len(), 52);
        assert_eq!(unique.len(), 52);
    }

    #[test]
    fn keys_round_trip() {
        for card in new_deck() {
            assert_eq!(Card::from_key(&card.key()), Some(card));
            let copy = card.with_copy(3);
            assert_eq!(copy.key(), format!("{}.3", card.file_stem()));
            assert_eq!(Card::from_key(&copy.key()), Some(copy));
            assert_eq!(copy.face(), card);
        }
        for bad in ["12h.", "12h.0", "12h.x", ".1", "12h.1.2"] {
            assert_eq!(Card::from_key(bad), None, "{bad}");
        }
    }

    #[test]
    fn file_stems_round_trip() {
        for card in new_deck() {
            assert_eq!(Card::from_file_stem(&card.file_stem()), Some(card));
        }
        for bad in ["", "h", "0h", "14s", "12x", "Qh"] {
            assert_eq!(Card::from_file_stem(bad), None, "{bad}");
        }
    }

    #[test]
    fn resource_names_match_asset_layout() {
        let card = Card::new(Rank::Queen, Suit::Hearts);
        assert_eq!(card.face_resource_name(), "images/Hearts/12h.png");
        for card in new_deck() {
            let path = format!("assets/{}", card.face_resource_name());
            assert!(std::path::Path::new(&path).exists(), "missing {path}");
        }
    }
}
