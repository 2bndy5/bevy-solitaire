//! Pure Klondike (draw one) rules, independent of Bevy so they can be unit tested.
use crate::cards::{Card, Rank};

/// Identifies one of the 13 piles on the table.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Pile {
    /// The face-down draw pile.
    Stock,
    /// Face-up cards drawn from the stock.
    Waste,
    /// One of the four piles built up by suit from Ace to King.
    Foundation(usize),
    /// One of the seven piles built down in alternating colors.
    Tableau(usize),
}

impl Pile {
    pub const FOUNDATIONS: usize = 4;
    pub const TABLEAUS: usize = 7;
}

/// A card at a specific spot on the table.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Slot {
    pub card: Card,
    pub face_up: bool,
}

impl Slot {
    fn up(card: Card) -> Self {
        Self {
            card,
            face_up: true,
        }
    }

    fn down(card: Card) -> Self {
        Self {
            card,
            face_up: false,
        }
    }
}

/// The complete state of a game of Klondike solitaire.
#[derive(Debug, Clone, Default)]
pub struct Klondike {
    stock: Vec<Slot>,
    waste: Vec<Slot>,
    foundations: [Vec<Slot>; Pile::FOUNDATIONS],
    tableaus: [Vec<Slot>; Pile::TABLEAUS],
    /// Number of moves made (draws, recycles and card moves).
    pub moves: u32,
}

impl Klondike {
    /// Deals a new game from `deck`.
    ///
    /// Cards are dealt from the end of `deck`. Tableau `i` gets `i + 1` cards with only the last
    /// one face up; the remaining 24 cards form the stock.
    pub fn deal(mut deck: Vec<Card>) -> Self {
        let mut game = Self::default();
        for row in 0..Pile::TABLEAUS {
            for (col, tableau) in game.tableaus.iter_mut().enumerate().skip(row) {
                let card = deck.pop().expect("a full deck has enough cards to deal");
                tableau.push(if row == col {
                    Slot::up(card)
                } else {
                    Slot::down(card)
                });
            }
        }
        game.stock = deck.into_iter().map(Slot::down).collect();
        game
    }

    /// The cards in `pile`, from bottom to top.
    pub fn pile(&self, pile: Pile) -> &[Slot] {
        match pile {
            Pile::Stock => &self.stock,
            Pile::Waste => &self.waste,
            Pile::Foundation(i) => &self.foundations[i],
            Pile::Tableau(i) => &self.tableaus[i],
        }
    }

    fn pile_mut(&mut self, pile: Pile) -> &mut Vec<Slot> {
        match pile {
            Pile::Stock => &mut self.stock,
            Pile::Waste => &mut self.waste,
            Pile::Foundation(i) => &mut self.foundations[i],
            Pile::Tableau(i) => &mut self.tableaus[i],
        }
    }

    /// Every pile on the table.
    pub fn all_piles() -> impl Iterator<Item = Pile> {
        [Pile::Stock, Pile::Waste]
            .into_iter()
            .chain((0..Pile::FOUNDATIONS).map(Pile::Foundation))
            .chain((0..Pile::TABLEAUS).map(Pile::Tableau))
    }

    /// Draws one card from the stock onto the waste.
    ///
    /// If the stock is empty, the waste is turned over to become the stock again.
    /// Returns `false` if there was nothing to draw or recycle.
    pub fn draw(&mut self) -> bool {
        if let Some(mut slot) = self.stock.pop() {
            slot.face_up = true;
            self.waste.push(slot);
        } else if !self.waste.is_empty() {
            self.stock = self
                .waste
                .drain(..)
                .rev()
                .map(|s| Slot::down(s.card))
                .collect();
        } else {
            return false;
        }
        self.moves += 1;
        true
    }

    /// Can the cards from `index` to the top of `from` be picked up?
    pub fn can_pick_up(&self, from: Pile, index: usize) -> bool {
        let cards = self.pile(from);
        let Some(slot) = cards.get(index) else {
            return false;
        };
        let is_top = index + 1 == cards.len();
        match from {
            Pile::Stock => false,
            Pile::Waste | Pile::Foundation(_) => is_top,
            // Face-up tableau cards are always a valid alternating sequence.
            Pile::Tableau(_) => slot.face_up,
        }
    }

    /// Can `card` (with `count` cards in total being moved) be placed on `to`?
    fn accepts(&self, to: Pile, card: Card, count: usize) -> bool {
        let top = self.pile(to).last();
        match to {
            Pile::Stock | Pile::Waste => false,
            Pile::Foundation(_) => {
                count == 1
                    && match top {
                        None => card.rank == Rank::Ace,
                        Some(top) => {
                            top.card.suit == card.suit
                                && top.card.rank.as_u8() + 1 == card.rank.as_u8()
                        }
                    }
            }
            Pile::Tableau(_) => match top {
                None => card.rank == Rank::King,
                Some(top) => {
                    top.face_up
                        && top.card.is_red() != card.is_red()
                        && card.rank.as_u8() + 1 == top.card.rank.as_u8()
                }
            },
        }
    }

    /// Is moving the cards from `index` upward in `from` onto `to` legal?
    pub fn can_move(&self, from: Pile, index: usize, to: Pile) -> bool {
        from != to
            && self.can_pick_up(from, index)
            && self.accepts(
                to,
                self.pile(from)[index].card,
                self.pile(from).len() - index,
            )
    }

    /// Moves the cards from `index` upward in `from` onto `to`, if legal.
    ///
    /// Any face-down card exposed on a tableau is turned face up. Returns whether the move happened.
    pub fn try_move(&mut self, from: Pile, index: usize, to: Pile) -> bool {
        if !self.can_move(from, index, to) {
            return false;
        }
        let moved = self.pile_mut(from).split_off(index);
        self.pile_mut(to).extend(moved);
        if let Pile::Tableau(_) = from
            && let Some(top) = self.pile_mut(from).last_mut()
        {
            top.face_up = true;
        }
        self.moves += 1;
        true
    }

    /// Finds a foundation that accepts the top card of `from`, if any.
    pub fn foundation_for(&self, from: Pile) -> Option<Pile> {
        let index = self.pile(from).len().checked_sub(1)?;
        (0..Pile::FOUNDATIONS)
            .map(Pile::Foundation)
            .find(|&to| self.can_move(from, index, to))
    }

    /// Moves the top card of `from` to a foundation, if possible.
    pub fn move_to_foundation(&mut self, from: Pile) -> bool {
        match self.foundation_for(from) {
            Some(to) => self.try_move(from, self.pile(from).len() - 1, to),
            None => false,
        }
    }

    /// Have all cards been moved to the foundations?
    pub fn is_won(&self) -> bool {
        self.foundations.iter().all(|f| f.len() == Rank::ALL.len())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cards::{Suit, new_deck};

    fn card(rank: Rank, suit: Suit) -> Card {
        Card::new(rank, suit)
    }

    #[test]
    fn deal_layout() {
        let game = Klondike::deal(new_deck());
        for i in 0..Pile::TABLEAUS {
            let pile = game.pile(Pile::Tableau(i));
            assert_eq!(pile.len(), i + 1);
            assert!(pile.last().unwrap().face_up);
            assert!(pile[..i].iter().all(|s| !s.face_up));
        }
        assert_eq!(game.pile(Pile::Stock).len(), 24);
        assert!(game.pile(Pile::Waste).is_empty());
        let total: usize = Klondike::all_piles().map(|p| game.pile(p).len()).sum();
        assert_eq!(total, 52);
    }

    #[test]
    fn draw_and_recycle() {
        let mut game = Klondike::deal(new_deck());
        let first = game.pile(Pile::Stock).last().unwrap().card;
        assert!(game.draw());
        assert_eq!(game.pile(Pile::Waste).last().unwrap(), &Slot::up(first));
        for _ in 0..23 {
            assert!(game.draw());
        }
        assert!(game.pile(Pile::Stock).is_empty());
        assert!(game.draw()); // recycle
        assert!(game.pile(Pile::Waste).is_empty());
        assert_eq!(game.pile(Pile::Stock).len(), 24);
        assert!(game.pile(Pile::Stock).iter().all(|s| !s.face_up));
        // the first card drawn is drawn first again
        assert_eq!(game.pile(Pile::Stock).last().unwrap().card, first);
        assert_eq!(game.moves, 25);
    }

    #[test]
    fn draw_from_empty_table_does_nothing() {
        let mut game = Klondike::default();
        assert!(!game.draw());
        assert_eq!(game.moves, 0);
    }

    #[test]
    fn foundation_rules() {
        let mut game = Klondike {
            waste: vec![
                Slot::up(card(Rank::Two, Suit::Hearts)),
                Slot::up(card(Rank::Ace, Suit::Hearts)),
            ],
            ..Default::default()
        };
        assert!(
            !game.can_move(Pile::Waste, 0, Pile::Foundation(0)),
            "not top card"
        );
        assert!(game.try_move(Pile::Waste, 1, Pile::Foundation(2)));
        assert!(
            !game.can_move(Pile::Waste, 0, Pile::Foundation(0)),
            "2 on empty"
        );
        assert_eq!(game.foundation_for(Pile::Waste), Some(Pile::Foundation(2)));
        assert!(game.move_to_foundation(Pile::Waste));
        assert_eq!(game.pile(Pile::Foundation(2)).len(), 2);

        game.waste = vec![Slot::up(card(Rank::Three, Suit::Diamonds))];
        assert!(
            !game.can_move(Pile::Waste, 0, Pile::Foundation(2)),
            "wrong suit"
        );
    }

    #[test]
    fn tableau_rules() {
        let mut game = Klondike::default();
        game.tableaus[0] = vec![
            Slot::down(card(Rank::Four, Suit::Clubs)),
            Slot::up(card(Rank::Nine, Suit::Spades)),
            Slot::up(card(Rank::Eight, Suit::Hearts)),
        ];
        game.tableaus[1] = vec![Slot::up(card(Rank::Ten, Suit::Diamonds))];
        game.tableaus[2] = vec![Slot::up(card(Rank::Ten, Suit::Clubs))];

        assert!(!game.can_pick_up(Pile::Tableau(0), 0), "face down");
        assert!(
            !game.can_move(Pile::Tableau(0), 1, Pile::Tableau(2)),
            "same color"
        );
        assert!(
            !game.can_move(Pile::Tableau(0), 2, Pile::Tableau(1)),
            "wrong rank"
        );
        assert!(
            !game.can_move(Pile::Tableau(0), 1, Pile::Tableau(0)),
            "same pile"
        );
        assert!(game.try_move(Pile::Tableau(0), 1, Pile::Tableau(1)));
        assert_eq!(game.pile(Pile::Tableau(1)).len(), 3);
        // the exposed card is flipped
        assert_eq!(
            game.pile(Pile::Tableau(0)),
            &[Slot::up(card(Rank::Four, Suit::Clubs))]
        );
    }

    #[test]
    fn only_kings_on_empty_tableau() {
        let mut game = Klondike {
            waste: vec![
                Slot::up(card(Rank::King, Suit::Spades)),
                Slot::up(card(Rank::Queen, Suit::Hearts)),
            ],
            ..Default::default()
        };
        assert!(!game.can_move(Pile::Waste, 1, Pile::Tableau(3)));
        game.waste.pop();
        assert!(game.try_move(Pile::Waste, 0, Pile::Tableau(3)));
    }

    #[test]
    fn only_single_cards_to_foundation() {
        let mut game = Klondike::default();
        game.tableaus[0] = vec![
            Slot::up(card(Rank::Two, Suit::Spades)),
            Slot::up(card(Rank::Ace, Suit::Hearts)),
        ];
        game.foundations[0] = vec![Slot::up(card(Rank::Ace, Suit::Spades))];
        assert!(!game.can_move(Pile::Tableau(0), 0, Pile::Foundation(0)));
    }

    #[test]
    fn can_win() {
        let mut game = Klondike::default();
        let mut deck = new_deck(); // grouped by suit, Ace to King
        deck.reverse();
        game.waste = deck.into_iter().map(Slot::up).collect();
        while game.move_to_foundation(Pile::Waste) {}
        assert!(game.waste.is_empty());
        assert!(game.is_won());
    }
}
