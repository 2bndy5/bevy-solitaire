//! Playing card primitives, adapted from `bevy-52-card-pickup`.
use rand::{rng, seq::SliceRandom};
use std::fmt;

/// Rendered card width (in world units).
pub const CARD_W: f32 = 84.0;
/// Rendered card height (in world units).
pub const CARD_H: f32 = 120.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Card {
    pub rank: Rank,
    pub suit: Suit,
}

impl Card {
    pub const fn new(rank: Rank, suit: Suit) -> Self {
        Self { rank, suit }
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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

/// Returns all 52 cards in a random order.
pub fn shuffle_deck() -> Vec<Card> {
    let mut deck = new_deck();
    deck.shuffle(&mut rng());
    deck
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
    fn resource_names_match_asset_layout() {
        let card = Card::new(Rank::Queen, Suit::Hearts);
        assert_eq!(card.face_resource_name(), "images/Hearts/12h.png");
        for card in new_deck() {
            let path = format!("assets/{}", card.face_resource_name());
            assert!(std::path::Path::new(&path).exists(), "missing {path}");
        }
    }
}
