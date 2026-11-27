//! Pyramid: clear a pyramid of cards by removing pairs that add up to 13, and Kings on their own.
use crate::{
    cards::{Card, Rank},
    game::{Board, Pile, Rules, Shape, Slot, Spot},
    games::rows::{RowSpots, all_cleared, is_exposed, piles_front_first},
};

/// Where each of the 28 pyramid cards sits: seven rows, from one card at the top to seven.
const SPOTS: [(f32, usize); 28] = [
    (3.0, 0),
    (2.5, 1),
    (3.5, 1),
    (2.0, 2),
    (3.0, 2),
    (4.0, 2),
    (1.5, 3),
    (2.5, 3),
    (3.5, 3),
    (4.5, 3),
    (1.0, 4),
    (2.0, 4),
    (3.0, 4),
    (4.0, 4),
    (5.0, 4),
    (0.5, 5),
    (1.5, 5),
    (2.5, 5),
    (3.5, 5),
    (4.5, 5),
    (5.5, 5),
    (0.0, 6),
    (1.0, 6),
    (2.0, 6),
    (3.0, 6),
    (4.0, 6),
    (5.0, 6),
    (6.0, 6),
];
/// How far apart the rows are, in table rows: each row covers the bottom of the one behind it.
const ROW_STEP: f32 = 0.5;
/// Where removed cards go.
const DISCARD: Pile = Pile::Foundation(0);
/// The total of a pair that can be removed.
const PAIR_TOTAL: u8 = 13;

/// The rules of Pyramid.
pub struct Pyramid;

impl Pyramid {
    fn spots() -> &'static RowSpots {
        &SPOTS
    }

    /// The card at the top of `pile`, if it's free to be played: an uncovered pyramid card, or
    /// the top of the waste.
    fn free_card(board: &Board, pile: Pile) -> Option<Card> {
        let free = match pile {
            Pile::Tableau(i) => is_exposed(board, Self::spots(), i),
            Pile::Waste => true,
            _ => false,
        };
        free.then(|| board.top(pile).map(|slot| slot.card))
            .flatten()
    }

    /// The piles whose top cards are free to be played.
    fn free_piles(board: &Board) -> impl Iterator<Item = Pile> + '_ {
        piles_front_first(Self::spots())
            .chain([Pile::Waste])
            .filter(|&pile| Self::free_card(board, pile).is_some())
    }

    fn pairs(a: Card, b: Card) -> bool {
        a.rank.as_u8() + b.rank.as_u8() == PAIR_TOTAL
    }
}

impl Rules for Pyramid {
    fn piles(&self) -> Vec<Pile> {
        [Pile::Stock, Pile::Waste, DISCARD]
            .into_iter()
            .chain(piles_front_first(Self::spots()))
            .collect()
    }

    /// Deals the pyramid face up, and leaves the rest as the stock.
    fn deal(&self, mut deck: Vec<Card>, board: &mut Board) {
        for index in 0..SPOTS.len() {
            let card = deck.pop().expect("a full deck has enough cards to deal");
            board.pile_mut(Pile::Tableau(index)).push(Slot::up(card));
        }
        *board.pile_mut(Pile::Stock) = deck.into_iter().map(Slot::down).collect();
    }

    fn shape(&self) -> Shape {
        Shape {
            columns: 7,
            fan_row: 6.0 * ROW_STEP,
            landscape_fan: (0, 0),
            portrait_fan: (0, 0),
        }
    }

    fn spot(&self, pile: Pile) -> Spot {
        match pile {
            // The top corners are free beside the narrow top of the pyramid.
            Pile::Stock => Spot::stacked(0.0, 0.0),
            Pile::Waste => Spot::stacked(1.0, 0.0),
            DISCARD => Spot::stacked(6.0, 0.0),
            Pile::Tableau(i) => {
                let (column, row) = SPOTS[i];
                // Rows in front rest on top of the rows behind.
                Spot::stacked(column, row as f32 * ROW_STEP).raised(row as u8)
            }
            Pile::Foundation(_) | Pile::Cell(_) => unreachable!("Pyramid has no {pile:?}"),
        }
    }

    /// Turns the next card of the stock onto the waste. Once the stock runs out, the waste is
    /// turned back over to become the stock again.
    fn draw(&self, board: &mut Board) -> bool {
        if let Some(card) = board.pile_mut(Pile::Stock).pop() {
            board.pile_mut(Pile::Waste).push(Slot::up(card.card));
            true
        } else if !board.pile(Pile::Waste).is_empty() {
            let recycled = board
                .pile_mut(Pile::Waste)
                .drain(..)
                .rev()
                .map(|slot| Slot::down(slot.card))
                .collect();
            *board.pile_mut(Pile::Stock) = recycled;
            true
        } else {
            false
        }
    }

    fn can_pick_up(&self, board: &Board, from: Pile, index: usize) -> bool {
        index + 1 == board.pile(from).len() && Self::free_card(board, from).is_some()
    }

    /// A King goes to the discard pile on its own; any other card goes onto a free card that it
    /// adds up to 13 with, and both are removed.
    fn can_move(&self, board: &Board, from: Pile, _index: usize, to: Pile) -> bool {
        let Some(card) = Self::free_card(board, from) else {
            return false;
        };
        if to == DISCARD {
            return card.rank == Rank::King;
        }
        Self::free_card(board, to).is_some_and(|other| Self::pairs(card, other))
    }

    /// Discards the card, and the card it was paired with.
    fn apply_move(&self, board: &mut Board, from: Pile, index: usize, to: Pile) {
        board.move_cards(from, index, DISCARD);
        if to != DISCARD {
            let top = board.pile(to).len() - 1;
            board.move_cards(to, top, DISCARD);
        }
    }

    /// A King is discarded; any other card is paired with the first free card it adds up to 13
    /// with (front rows first, then the waste).
    fn best_move(&self, board: &Board, from: Pile, index: usize) -> Option<Pile> {
        if self.can_move(board, from, index, DISCARD) {
            return Some(DISCARD);
        }
        Self::free_piles(board).find(|&to| to != from && self.can_move(board, from, index, to))
    }

    fn is_won(&self, board: &Board) -> bool {
        all_cleared(board, Self::spots())
    }

    /// With the waste turned over as often as you like, every stock and waste card comes up
    /// eventually, so the game is lost only when no free pyramid card is a King or pairs with
    /// another free pyramid card, and no stock or waste card is a King or pairs with one.
    fn is_lost(&self, board: &Board) -> bool {
        let free: Vec<Card> = piles_front_first(Self::spots())
            .filter_map(|pile| Self::free_card(board, pile))
            .collect();
        let deck = board
            .pile(Pile::Stock)
            .iter()
            .chain(board.pile(Pile::Waste));
        let playable = |card: Card| {
            card.rank == Rank::King || free.iter().any(|&other| Self::pairs(card, other))
        };
        !free.iter().any(|&card| playable(card)) && !deck.map(|slot| slot.card).any(playable)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        cards::{Suit, shuffle_deck},
        game::{Game, card},
        games::GameKind,
    };

    /// A game of Pyramid with only the given pyramid cards (by index) and waste.
    fn pyramid(cards: &[(usize, Card)], waste: &[Card]) -> Game {
        let mut board = Board::new(Pyramid.piles());
        for &(index, card) in cards {
            *board.pile_mut(Pile::Tableau(index)) = vec![Slot::up(card)];
        }
        *board.pile_mut(Pile::Waste) = waste.iter().copied().map(Slot::up).collect();
        Game::with_board(GameKind::Pyramid, board)
    }

    #[test]
    fn deal_layout() {
        let game = Game::deal(GameKind::Pyramid, shuffle_deck());
        for index in 0..SPOTS.len() {
            let pile = game.pile(Pile::Tableau(index));
            assert_eq!(pile.len(), 1);
            assert!(pile[0].face_up);
        }
        assert_eq!(game.pile(Pile::Stock).len(), 24);
        assert!(game.pile(Pile::Waste).is_empty());
    }

    #[test]
    fn kings_go_alone_and_pairs_make_13() {
        // Two cards of the bottom row, and the one they cover.
        let mut game = pyramid(
            &[
                (15, card(Rank::Two, Suit::Clubs)),
                (21, card(Rank::King, Suit::Hearts)),
                (22, card(Rank::Jack, Suit::Spades)),
            ],
            &[card(Rank::Six, Suit::Diamonds)],
        );
        assert!(!game.can_pick_up(Pile::Tableau(15), 0), "covered");
        assert_eq!(game.auto_move(Pile::Tableau(21), 0), Some(DISCARD));
        // The Two is still covered by the Jack.
        assert_eq!(game.best_move(Pile::Tableau(22), 0), None, "Jack and Six");
        assert!(!game.can_move(Pile::Tableau(22), 0, Pile::Tableau(15)));

        // A Seven from the stock pairs with the Six on the waste.
        let mut board = game.board().clone();
        board
            .pile_mut(Pile::Waste)
            .push(Slot::up(card(Rank::Seven, Suit::Hearts)));
        let mut game = Game::with_board(GameKind::Pyramid, board);
        assert!(!game.can_pick_up(Pile::Waste, 0), "under the Seven");
        assert_eq!(
            game.best_move(Pile::Waste, 1),
            None,
            "nothing free adds to 13"
        );

        // The Jack and a Two pair up, which frees the Two behind them.
        let mut board = game.board().clone();
        *board.pile_mut(Pile::Waste) = vec![Slot::up(card(Rank::Two, Suit::Hearts))];
        game = Game::with_board(GameKind::Pyramid, board);
        assert_eq!(game.auto_move(Pile::Tableau(22), 0), Some(Pile::Waste));
        assert!(game.pile(Pile::Tableau(22)).is_empty());
        assert!(game.pile(Pile::Waste).is_empty());
        assert_eq!(game.pile(DISCARD).len(), 3);
        assert!(game.can_pick_up(Pile::Tableau(15), 0));
        // Undo puts both cards back.
        assert!(game.undo());
        assert_eq!(game.pile(Pile::Waste).len(), 1);
        assert_eq!(game.pile(Pile::Tableau(22)).len(), 1);
    }

    #[test]
    fn stock_recycles() {
        let mut game = Game::deal(GameKind::Pyramid, shuffle_deck());
        for _ in 0..24 {
            assert!(game.draw());
        }
        assert!(game.pile(Pile::Stock).is_empty());
        assert!(game.draw());
        assert_eq!(game.pile(Pile::Stock).len(), 24);
        assert!(game.pile(Pile::Waste).is_empty());
    }

    #[test]
    fn lost_only_when_nothing_can_ever_pair() {
        let five = card(Rank::Five, Suit::Clubs);
        let nine = card(Rank::Nine, Suit::Hearts);
        // A Five and a Nine never make 13...
        assert!(pyramid(&[(27, five)], &[nine]).is_lost());
        // ...but an Eight in the stock or waste would pair with the Five, eventually.
        let eight = card(Rank::Eight, Suit::Spades);
        assert!(!pyramid(&[(27, five)], &[eight, nine]).is_lost());
        // And a King can always go.
        assert!(!pyramid(&[(27, five)], &[card(Rank::King, Suit::Spades)]).is_lost());
        // Two free pyramid cards that pair.
        assert!(!pyramid(&[(26, five), (27, eight)], &[]).is_lost());
        assert!(pyramid(&[], &[nine]).is_won());
    }
}
