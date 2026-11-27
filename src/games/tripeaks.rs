//! TriPeaks: clear three overlapping peaks onto the waste, one rank up or down at a time.
use crate::{
    cards::Card,
    game::{Board, Pile, Rules, Shape, Slot, Spot, ranks_adjacent},
    games::rows::{RowSpots, all_cleared, is_exposed, piles_front_first},
};

/// Where each of the 28 tableau cards sits: three peaks of three rows, on a bottom row of ten.
const SPOTS: [(f32, usize); 28] = [
    (1.5, 0),
    (4.5, 0),
    (7.5, 0),
    (1.0, 1),
    (2.0, 1),
    (4.0, 1),
    (5.0, 1),
    (7.0, 1),
    (8.0, 1),
    (0.5, 2),
    (1.5, 2),
    (2.5, 2),
    (3.5, 2),
    (4.5, 2),
    (5.5, 2),
    (6.5, 2),
    (7.5, 2),
    (8.5, 2),
    (0.0, 3),
    (1.0, 3),
    (2.0, 3),
    (3.0, 3),
    (4.0, 3),
    (5.0, 3),
    (6.0, 3),
    (7.0, 3),
    (8.0, 3),
    (9.0, 3),
];
/// How far apart the rows are, in table rows: each row covers the top of the one behind it.
const ROW_STEP: f32 = 0.5;
/// The row of the stock and waste, below the peaks.
const STOCK_ROW: f32 = 3.0 * ROW_STEP + 1.0;

/// The rules of TriPeaks.
pub struct TriPeaks;

impl TriPeaks {
    fn spots() -> &'static RowSpots {
        &SPOTS
    }

    /// Can tableau card `index` go on the waste?
    fn plays(board: &Board, index: usize) -> bool {
        let pile = Pile::Tableau(index);
        is_exposed(board, Self::spots(), index)
            && match (board.top(pile), board.top(Pile::Waste)) {
                (Some(card), Some(waste)) => ranks_adjacent(card.card.rank, waste.card.rank, true),
                _ => false,
            }
    }

    /// Turns face up every card that nothing covers any more.
    fn turn_up_exposed(board: &mut Board) {
        for index in 0..SPOTS.len() {
            if is_exposed(board, Self::spots(), index) {
                board.turn_up_top(Pile::Tableau(index));
            }
        }
    }
}

impl Rules for TriPeaks {
    fn piles(&self) -> Vec<Pile> {
        [Pile::Stock, Pile::Waste]
            .into_iter()
            .chain(piles_front_first(Self::spots()))
            .collect()
    }

    /// Deals the peaks face down with the bottom row face up, turns one card onto the waste, and
    /// leaves the rest as the stock.
    fn deal(&self, mut deck: Vec<Card>, board: &mut Board) {
        for index in 0..SPOTS.len() {
            let card = deck.pop().expect("a full deck has enough cards to deal");
            board.pile_mut(Pile::Tableau(index)).push(Slot::down(card));
        }
        Self::turn_up_exposed(board);
        let first = deck.pop().expect("a full deck has a card for the waste");
        board.pile_mut(Pile::Waste).push(Slot::up(first));
        *board.pile_mut(Pile::Stock) = deck.into_iter().map(Slot::down).collect();
    }

    fn shape(&self) -> Shape {
        Shape {
            columns: 10,
            fan_row: STOCK_ROW,
            landscape_fan: (0, 0),
            portrait_fan: (0, 0),
        }
    }

    fn spot(&self, pile: Pile) -> Spot {
        match pile {
            Pile::Stock => Spot::stacked(3.5, STOCK_ROW),
            Pile::Waste => Spot::stacked(5.5, STOCK_ROW),
            Pile::Tableau(i) => {
                let (column, row) = SPOTS[i];
                // Rows in front rest on top of the rows behind.
                Spot::stacked(column, row as f32 * ROW_STEP).raised(row as u8)
            }
            Pile::Foundation(_) | Pile::Cell(_) => unreachable!("TriPeaks has no {pile:?}"),
        }
    }

    /// Turns the next card of the stock onto the waste. The stock is only gone through once.
    fn draw(&self, board: &mut Board) -> bool {
        let Some(card) = board.pile_mut(Pile::Stock).pop() else {
            return false;
        };
        board.pile_mut(Pile::Waste).push(Slot::up(card.card));
        true
    }

    fn can_pick_up(&self, board: &Board, from: Pile, _index: usize) -> bool {
        matches!(from, Pile::Tableau(i) if is_exposed(board, Self::spots(), i))
    }

    fn can_move(&self, board: &Board, from: Pile, _index: usize, to: Pile) -> bool {
        to == Pile::Waste && matches!(from, Pile::Tableau(i) if Self::plays(board, i))
    }

    /// Moves the card to the waste, and turns up any card it uncovers.
    fn apply_move(&self, board: &mut Board, from: Pile, index: usize, to: Pile) {
        board.move_cards(from, index, to);
        Self::turn_up_exposed(board);
    }

    fn best_move(&self, _board: &Board, _from: Pile, _index: usize) -> Option<Pile> {
        Some(Pile::Waste)
    }

    fn is_won(&self, board: &Board) -> bool {
        all_cleared(board, Self::spots())
    }

    fn is_lost(&self, board: &Board) -> bool {
        board.pile(Pile::Stock).is_empty() && !(0..SPOTS.len()).any(|i| Self::plays(board, i))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        cards::{Rank, Suit, shuffle_deck},
        game::{Game, card},
        games::GameKind,
    };

    #[test]
    fn deal_layout() {
        let game = Game::deal(GameKind::TriPeaks, shuffle_deck());
        for (index, &(_, row)) in SPOTS.iter().enumerate() {
            let pile = game.pile(Pile::Tableau(index));
            assert_eq!(pile.len(), 1);
            assert_eq!(pile[0].face_up, row == 3, "only the bottom row is face up");
        }
        assert_eq!(game.pile(Pile::Waste).len(), 1);
        assert_eq!(game.pile(Pile::Stock).len(), 52 - 28 - 1);
    }

    #[test]
    fn peaks_cover_their_rows() {
        // The top of the first peak is covered by the two cards below it.
        assert_eq!(
            crate::games::rows::covering(&SPOTS, 0).collect::<Vec<_>>(),
            [3, 4]
        );
        // The bottom row covers the row behind it, two cards each.
        assert_eq!(
            crate::games::rows::covering(&SPOTS, 9).collect::<Vec<_>>(),
            [18, 19]
        );
        // Every card but the bottom row's is covered by exactly two.
        for index in 0..18 {
            assert_eq!(crate::games::rows::covering(&SPOTS, index).count(), 2);
        }
    }

    #[test]
    fn plays_wrap_and_uncover_cards() {
        // Only the first peak's three cards, and a lone bottom card under its left side.
        let mut board = Board::new(TriPeaks.piles());
        let place = |board: &mut Board, index: usize, slot: Slot| {
            *board.pile_mut(Pile::Tableau(index)) = vec![slot];
        };
        place(&mut board, 3, Slot::down(card(Rank::Two, Suit::Clubs)));
        place(&mut board, 9, Slot::down(card(Rank::Queen, Suit::Hearts)));
        place(&mut board, 18, Slot::up(card(Rank::Ace, Suit::Spades)));
        *board.pile_mut(Pile::Waste) = vec![Slot::up(card(Rank::King, Suit::Diamonds))];
        let mut game = Game::with_board(GameKind::TriPeaks, board);

        assert!(!game.can_pick_up(Pile::Tableau(9), 0), "covered");
        // Ace on King wraps around.
        assert_eq!(game.auto_move(Pile::Tableau(18), 0), Some(Pile::Waste));
        // That uncovers the Queen, which turns face up, but it doesn't go on an Ace.
        assert!(game.pile(Pile::Tableau(9))[0].face_up);
        assert_eq!(game.best_move(Pile::Tableau(9), 0), None);
        assert!(game.is_lost(), "the stock is empty and nothing plays");
        // The Two still lies under the Queen.
        assert!(!game.pile(Pile::Tableau(3))[0].face_up);
        assert!(game.undo());
        assert!(!game.pile(Pile::Tableau(9))[0].face_up);
    }

    #[test]
    fn won_when_the_peaks_are_cleared() {
        let mut board = Board::new(TriPeaks.piles());
        *board.pile_mut(Pile::Stock) = vec![Slot::down(card(Rank::Two, Suit::Hearts))];
        let game = Game::with_board(GameKind::TriPeaks, board);
        assert!(game.is_won());
    }
}
