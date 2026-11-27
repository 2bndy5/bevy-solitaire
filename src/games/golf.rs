//! Golf: clear seven columns of face-up cards onto the waste, one rank up or down at a time.
use crate::{
    cards::Card,
    game::{Board, Pile, Rules, Shape, Slot, Spot, ranks_adjacent},
};

const TABLEAUS: usize = 7;
/// Cards dealt to each column.
const DEPTH: usize = 5;

/// The rules of Golf.
pub struct Golf;

impl Golf {
    /// Can the top card of `from` go on the waste?
    fn plays(board: &Board, from: Pile) -> bool {
        match (board.top(from), board.top(Pile::Waste)) {
            (Some(card), Some(waste)) => ranks_adjacent(card.card.rank, waste.card.rank, false),
            _ => false,
        }
    }
}

impl Rules for Golf {
    fn piles(&self) -> Vec<Pile> {
        [Pile::Stock, Pile::Waste]
            .into_iter()
            .chain((0..TABLEAUS).map(Pile::Tableau))
            .collect()
    }

    /// Deals seven columns of five face-up cards, turns one card onto the waste, and leaves the
    /// rest as the stock.
    fn deal(&self, mut deck: Vec<Card>, board: &mut Board) {
        for _ in 0..DEPTH {
            for column in 0..TABLEAUS {
                let card = deck.pop().expect("a full deck has enough cards to deal");
                board.pile_mut(Pile::Tableau(column)).push(Slot::up(card));
            }
        }
        let first = deck.pop().expect("a full deck has a card for the waste");
        board.pile_mut(Pile::Waste).push(Slot::up(first));
        *board.pile_mut(Pile::Stock) = deck.into_iter().map(Slot::down).collect();
    }

    fn shape(&self) -> Shape {
        Shape {
            columns: TABLEAUS,
            fan_row: 1.0,
            landscape_fan: (0, DEPTH - 1),
            portrait_fan: (0, DEPTH - 1),
        }
    }

    fn spot(&self, pile: Pile) -> Spot {
        match pile {
            Pile::Stock => Spot::stacked(2.0, 0.0),
            Pile::Waste => Spot::stacked(4.0, 0.0),
            Pile::Tableau(i) => Spot::fanned(i as f32, 1.0),
            Pile::Foundation(_) | Pile::Cell(_) => unreachable!("Golf has no {pile:?}"),
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

    fn can_pick_up(&self, board: &Board, from: Pile, index: usize) -> bool {
        matches!(from, Pile::Tableau(_)) && index + 1 == board.pile(from).len()
    }

    fn can_move(&self, board: &Board, from: Pile, _index: usize, to: Pile) -> bool {
        to == Pile::Waste && Self::plays(board, from)
    }

    fn best_move(&self, _board: &Board, _from: Pile, _index: usize) -> Option<Pile> {
        Some(Pile::Waste)
    }

    fn is_won(&self, board: &Board) -> bool {
        (0..TABLEAUS).all(|i| board.pile(Pile::Tableau(i)).is_empty())
    }

    fn is_lost(&self, board: &Board) -> bool {
        board.pile(Pile::Stock).is_empty()
            && !(0..TABLEAUS).any(|i| Self::plays(board, Pile::Tableau(i)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        cards::{Rank, Suit, new_deck, shuffle_deck},
        game::{Game, card},
        games::GameKind,
    };

    /// A game of Golf with no cards, changed by `setup`.
    fn golf(setup: impl FnOnce(&mut Board)) -> Game {
        let mut board = Board::new(Golf.piles());
        setup(&mut board);
        Game::with_board(GameKind::Golf, board)
    }

    fn up(cards: &[Card]) -> Vec<Slot> {
        cards.iter().copied().map(Slot::up).collect()
    }

    #[test]
    fn deal_layout() {
        let game = Game::deal(GameKind::Golf, shuffle_deck());
        for i in 0..TABLEAUS {
            let pile = game.pile(Pile::Tableau(i));
            assert_eq!(pile.len(), DEPTH);
            assert!(pile.iter().all(|slot| slot.face_up));
        }
        assert_eq!(game.pile(Pile::Waste).len(), 1);
        assert_eq!(game.pile(Pile::Stock).len(), 52 - 35 - 1);
    }

    #[test]
    fn plays_one_rank_up_or_down_without_wrapping() {
        let mut game = golf(|board| {
            *board.pile_mut(Pile::Waste) = up(&[card(Rank::Five, Suit::Hearts)]);
            *board.pile_mut(Pile::Tableau(0)) =
                up(&[card(Rank::Six, Suit::Clubs), card(Rank::Four, Suit::Spades)]);
            *board.pile_mut(Pile::Tableau(1)) = up(&[card(Rank::Nine, Suit::Clubs)]);
        });
        assert!(!game.can_pick_up(Pile::Tableau(0), 0), "only top cards");
        assert_eq!(game.auto_move(Pile::Tableau(0), 1), Some(Pile::Waste));
        assert_eq!(game.auto_move(Pile::Tableau(0), 0), None, "Six on Four");
        assert_eq!(game.best_move(Pile::Tableau(1), 0), None);

        let game = golf(|board| {
            *board.pile_mut(Pile::Waste) = up(&[card(Rank::King, Suit::Hearts)]);
            *board.pile_mut(Pile::Tableau(0)) = up(&[card(Rank::Ace, Suit::Clubs)]);
            *board.pile_mut(Pile::Tableau(1)) = up(&[card(Rank::Queen, Suit::Clubs)]);
        });
        assert!(
            !game.can_move(Pile::Tableau(0), 0, Pile::Waste),
            "no wrapping"
        );
        assert!(game.can_move(Pile::Tableau(1), 0, Pile::Waste));
    }

    #[test]
    fn stock_is_dealt_once() {
        let mut game = Game::deal(GameKind::Golf, new_deck());
        for _ in 0..16 {
            assert!(game.draw());
        }
        assert!(!game.draw());
        assert_eq!(game.pile(Pile::Waste).len(), 17);
    }

    #[test]
    fn won_when_columns_are_clear_and_lost_when_stuck() {
        let game = golf(|board| {
            *board.pile_mut(Pile::Waste) = up(&[card(Rank::Five, Suit::Hearts)]);
            *board.pile_mut(Pile::Stock) = vec![Slot::down(card(Rank::Two, Suit::Hearts))];
        });
        assert!(game.is_won(), "the stock needn't be used up");
        assert!(!game.is_lost());

        let stuck = |stock: Vec<Slot>| {
            golf(|board| {
                *board.pile_mut(Pile::Waste) = up(&[card(Rank::Five, Suit::Hearts)]);
                *board.pile_mut(Pile::Tableau(3)) = up(&[card(Rank::Nine, Suit::Clubs)]);
                *board.pile_mut(Pile::Stock) = stock;
            })
        };
        assert!(stuck(Vec::new()).is_lost());
        assert!(!stuck(vec![Slot::down(card(Rank::Two, Suit::Hearts))]).is_lost());
    }
}
