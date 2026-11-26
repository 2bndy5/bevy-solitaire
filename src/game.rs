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
    /// The states before each move, most recent last, for [`Klondike::undo`].
    history: Vec<Klondike>,
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
        if self.stock.is_empty() && self.waste.is_empty() {
            return false;
        }
        self.save();
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
        self.save();
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

    /// Where a tap on the card at `index` in `from` should send it (with any cards on top of it).
    ///
    /// A lone top card goes to a foundation if it can. Otherwise the cards go to the first tableau
    /// column that accepts them, searching rightward from (and wrapping around past) the column they
    /// are in, so tapping again moves them on to the next option. A King already at the bottom of a
    /// column is never moved to an empty one, since that changes nothing.
    pub fn best_move(&self, from: Pile, index: usize) -> Option<Pile> {
        if !self.can_pick_up(from, index) {
            return None;
        }
        if index + 1 == self.pile(from).len()
            && let Some(foundation) = self.foundation_for(from)
        {
            return Some(foundation);
        }
        let start = match from {
            Pile::Tableau(column) => column + 1,
            _ => 0,
        };
        let king_at_bottom = matches!(from, Pile::Tableau(_)) && index == 0;
        (start..start + Pile::TABLEAUS)
            .map(|column| Pile::Tableau(column % Pile::TABLEAUS))
            .filter(|&to| !(king_at_bottom && self.pile(to).is_empty()))
            .find(|&to| self.can_move(from, index, to))
    }

    /// Makes the move [`Klondike::best_move`] suggests, returning where the cards went.
    pub fn auto_move(&mut self, from: Pile, index: usize) -> Option<Pile> {
        let to = self.best_move(from, index)?;
        self.try_move(from, index, to).then_some(to)
    }

    /// Have all cards been moved to the foundations?
    pub fn is_won(&self) -> bool {
        self.foundations.iter().all(|f| f.len() == Rank::ALL.len())
    }

    /// A game with only `cards` on the table, face up in the waste (the last card on top).
    #[cfg(test)]
    pub fn with_waste(cards: Vec<Card>) -> Self {
        Self {
            waste: cards.into_iter().map(Slot::up).collect(),
            ..Default::default()
        }
    }

    /// Remembers the current state so the next move can be undone.
    fn save(&mut self) {
        let snapshot = Klondike {
            stock: self.stock.clone(),
            waste: self.waste.clone(),
            foundations: self.foundations.clone(),
            tableaus: self.tableaus.clone(),
            moves: self.moves,
            history: Vec::new(),
        };
        self.history.push(snapshot);
    }

    /// Is there a move to undo?
    pub fn can_undo(&self) -> bool {
        !self.history.is_empty()
    }

    /// Takes back the last move (including turning back over any card it exposed).
    ///
    /// Returns `false` if there was nothing to undo.
    pub fn undo(&mut self) -> bool {
        let Some(previous) = self.history.pop() else {
            return false;
        };
        let history = std::mem::take(&mut self.history);
        *self = Klondike {
            history,
            ..previous
        };
        true
    }

    /// Is the game certain to be won from here?
    ///
    /// Once the stock is empty and every tableau card is face up, the cards can always be played
    /// to the foundations: each tableau pile is a descending run, so the lowest card still in
    /// play is on top of its pile, or can be reached by cycling through the waste.
    pub fn is_certainly_winnable(&self) -> bool {
        self.stock.is_empty() && self.tableaus.iter().flatten().all(|slot| slot.face_up)
    }

    /// Makes one move toward finishing a game that [is certainly winnable].
    ///
    /// Plays the lowest-ranked card that can go to a foundation; when none can, draws from (or
    /// recycles) the waste. Returns `false` once there is nothing left to do.
    ///
    /// [is certainly winnable]: Klondike::is_certainly_winnable
    pub fn auto_finish_step(&mut self) -> bool {
        let best = std::iter::once(Pile::Waste)
            .chain((0..Pile::TABLEAUS).map(Pile::Tableau))
            .filter(|&pile| self.foundation_for(pile).is_some())
            .min_by_key(|&pile| self.pile(pile).last().map(|slot| slot.card.rank));
        match best {
            Some(pile) => self.move_to_foundation(pile),
            None => !self.is_won() && self.draw(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Plays [`Klondike::auto_finish_step`] until it stops, failing if it seems stuck.
    fn auto_finish(game: &mut Klondike) {
        for _ in 0..10_000 {
            if !game.auto_finish_step() {
                return;
            }
        }
        panic!("auto-finish did not stop");
    }

    /// A game with only the given tableau columns (bottom card first, all face up).
    fn with_tableaus(columns: &[&[Card]]) -> Klondike {
        let mut game = Klondike::default();
        for (i, cards) in columns.iter().enumerate() {
            game.tableaus[i] = cards.iter().copied().map(Slot::up).collect();
        }
        game
    }

    #[test]
    fn tap_prefers_foundation_for_top_card() {
        let mut game = with_tableaus(&[
            &[card(Rank::Two, Suit::Spades), card(Rank::Ace, Suit::Hearts)],
            &[card(Rank::Two, Suit::Clubs)],
        ]);
        // The ace could go nowhere else; it goes home.
        assert_eq!(
            game.auto_move(Pile::Tableau(0), 1),
            Some(Pile::Foundation(0))
        );
        // The two of clubs can't go to a foundation or onto the two of spades.
        assert_eq!(game.best_move(Pile::Tableau(1), 0), None);
    }

    #[test]
    fn tap_moves_to_first_tableau_after_source() {
        let game = with_tableaus(&[
            &[card(Rank::Six, Suit::Spades)],
            &[card(Rank::Five, Suit::Hearts)],
            &[card(Rank::Nine, Suit::Clubs)],
            &[card(Rank::Six, Suit::Clubs)],
        ]);
        // From column 1, column 3 comes before wrapping around to column 0.
        assert_eq!(game.best_move(Pile::Tableau(1), 0), Some(Pile::Tableau(3)));
        let mut game = game;
        assert_eq!(game.auto_move(Pile::Tableau(1), 0), Some(Pile::Tableau(3)));
        // Tapping again moves it on to the next option.
        assert_eq!(game.auto_move(Pile::Tableau(3), 1), Some(Pile::Tableau(0)));
    }

    #[test]
    fn tap_moves_runs_and_waste_cards() {
        let mut game = with_tableaus(&[
            &[
                card(Rank::Four, Suit::Clubs),
                card(Rank::Nine, Suit::Spades),
                card(Rank::Eight, Suit::Hearts),
            ],
            &[card(Rank::Ten, Suit::Diamonds)],
        ]);
        game.tableaus[0][0].face_up = false;
        assert_eq!(game.best_move(Pile::Tableau(0), 0), None, "face down");
        assert_eq!(game.auto_move(Pile::Tableau(0), 1), Some(Pile::Tableau(1)));
        assert_eq!(game.pile(Pile::Tableau(1)).len(), 3);
        assert!(game.pile(Pile::Tableau(0))[0].face_up);

        game.waste = vec![Slot::up(card(Rank::Seven, Suit::Clubs))];
        assert_eq!(game.auto_move(Pile::Waste, 0), Some(Pile::Tableau(1)));
        assert_eq!(game.best_move(Pile::Stock, 0), None);
    }

    #[test]
    fn tap_does_not_shuffle_kings_between_empty_columns() {
        let game = with_tableaus(&[&[], &[card(Rank::King, Suit::Hearts)]]);
        assert_eq!(game.best_move(Pile::Tableau(1), 0), None);
        // But a King on top of other cards may move to an empty column.
        let game = with_tableaus(&[
            &[],
            &[card(Rank::Two, Suit::Clubs), card(Rank::King, Suit::Hearts)],
        ]);
        assert_eq!(game.best_move(Pile::Tableau(1), 1), Some(Pile::Tableau(2)));
    }

    #[test]
    fn undo_restores_previous_states() {
        let mut game = Klondike::deal(new_deck());
        assert!(!game.can_undo());
        assert!(!game.undo());

        let dealt = game.clone();
        let mut after_draw = None;
        for _ in 0..30 {
            // draws, then recycles the waste
            assert!(game.draw());
            after_draw.get_or_insert_with(|| game.clone());
        }
        let after_draw = after_draw.unwrap();
        for _ in 0..29 {
            assert!(game.undo());
        }
        assert_eq!(game.pile(Pile::Waste), after_draw.pile(Pile::Waste));
        assert_eq!(game.pile(Pile::Stock), after_draw.pile(Pile::Stock));
        assert_eq!(game.moves, 1);
        assert!(game.undo());
        for pile in Klondike::all_piles() {
            assert_eq!(game.pile(pile), dealt.pile(pile));
        }
        assert_eq!(game.moves, 0);
        assert!(!game.can_undo());
    }

    #[test]
    fn undo_turns_exposed_card_back_over() {
        let mut game = Klondike {
            tableaus: [
                vec![
                    Slot::down(card(Rank::Four, Suit::Clubs)),
                    Slot::up(card(Rank::Ace, Suit::Hearts)),
                ],
                vec![],
                vec![],
                vec![],
                vec![],
                vec![],
                vec![],
            ],
            ..Default::default()
        };
        assert!(game.move_to_foundation(Pile::Tableau(0)));
        assert!(game.pile(Pile::Tableau(0))[0].face_up);
        assert!(game.undo());
        assert_eq!(
            game.pile(Pile::Tableau(0)),
            &[
                Slot::down(card(Rank::Four, Suit::Clubs)),
                Slot::up(card(Rank::Ace, Suit::Hearts)),
            ]
        );
        assert!(game.pile(Pile::Foundation(0)).is_empty());
    }

    #[test]
    fn failed_moves_are_not_recorded() {
        let mut game = Klondike::default();
        assert!(!game.draw());
        assert!(!game.try_move(Pile::Waste, 0, Pile::Foundation(0)));
        assert!(!game.can_undo());
    }

    #[test]
    fn certainly_winnable() {
        let game = Klondike::deal(new_deck());
        assert!(
            !game.is_certainly_winnable(),
            "stock and face-down cards remain"
        );

        // Everything face up in the tableau, with some cards still in the waste.
        let mut deck = new_deck();
        let mut game = Klondike::default();
        for (i, card) in deck.drain(..40).enumerate() {
            game.tableaus[i % Pile::TABLEAUS].push(Slot::up(card));
        }
        game.waste = deck.into_iter().map(Slot::up).collect();
        assert!(game.is_certainly_winnable());
        game.tableaus[2][0].face_up = false;
        assert!(!game.is_certainly_winnable());
    }

    #[test]
    fn auto_finish_wins_from_any_face_up_position() {
        // Lay out shuffled decks with every card face up: tableau piles as descending,
        // alternating runs (as real play leaves them) and everything else in the waste.
        // Auto-finish must clear the table, cycling the waste as needed.
        for seed in 0..50u64 {
            let mut deck = new_deck();
            // A simple deterministic shuffle.
            let mut state = seed
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            for i in (1..deck.len()).rev() {
                state = state
                    .wrapping_mul(6364136223846793005)
                    .wrapping_add(1442695040888963407);
                deck.swap(i, (state >> 33) as usize % (i + 1));
            }
            let mut game = Klondike::default();
            let mut rest = Vec::new();
            for card in deck {
                let fits = game.tableaus.iter().position(|pile| match pile.last() {
                    None => card.rank == Rank::King,
                    Some(top) => {
                        top.card.is_red() != card.is_red()
                            && card.rank.as_u8() + 1 == top.card.rank.as_u8()
                    }
                });
                match fits {
                    Some(i) => game.tableaus[i].push(Slot::up(card)),
                    None => rest.push(Slot::up(card)),
                }
            }
            game.waste = rest;
            assert!(game.is_certainly_winnable());
            auto_finish(&mut game);
            assert!(game.is_won(), "seed {seed} not finished");
        }
    }

    #[test]
    fn auto_finish_plays_lowest_card_first() {
        let mut game = Klondike {
            waste: vec![Slot::up(card(Rank::Two, Suit::Spades))],
            tableaus: [
                vec![Slot::up(card(Rank::Ace, Suit::Spades))],
                vec![Slot::up(card(Rank::Ace, Suit::Hearts))],
                vec![],
                vec![],
                vec![],
                vec![],
                vec![],
            ],
            ..Default::default()
        };
        assert!(game.auto_finish_step());
        assert!(game.pile(Pile::Waste).len() == 1, "an ace goes first");
        auto_finish(&mut game);
        assert!(game.pile(Pile::Waste).is_empty());
        assert_eq!(
            game.pile(Pile::Foundation(0)).len() + game.pile(Pile::Foundation(1)).len(),
            3
        );
    }
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
