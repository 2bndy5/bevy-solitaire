//! FreeCell: every card is dealt face up, and four free cells hold single cards.
use crate::{
    cards::Card,
    game::{
        Action, Board, Pile, Rules, Shape, Slot, Spot, builds_down_alternating, builds_up_by_suit,
        foundations_full,
    },
};

const CELLS: usize = 4;
const FOUNDATIONS: usize = 4;
const TABLEAUS: usize = 8;

/// The rules of FreeCell.
pub struct FreeCell;

impl FreeCell {
    /// Is every card from `index` to the top of `pile` a run built down in alternating colors?
    fn is_run(board: &Board, pile: Pile, index: usize) -> bool {
        board.pile(pile)[index..]
            .windows(2)
            .all(|pair| builds_down_alternating(&pair[0], pair[1].card))
    }

    /// How many cards can be moved at once onto `to`, given the free cells and empty columns.
    ///
    /// Only single cards really move, but a run can go one card at a time through the free cells
    /// and empty columns: each free cell adds one card, and each empty column (other than the
    /// destination) doubles what can be moved.
    fn max_run(board: &Board, to: Pile) -> usize {
        let free_cells = (0..CELLS)
            .filter(|&i| board.pile(Pile::Cell(i)).is_empty())
            .count();
        let empty_columns = (0..TABLEAUS)
            .map(Pile::Tableau)
            .filter(|&pile| pile != to && board.pile(pile).is_empty())
            .count();
        (free_cells + 1) << empty_columns
    }

    /// A foundation that takes the top card of `from`, if any.
    fn foundation_for(board: &Board, from: Pile) -> Option<Pile> {
        let top = board.top(from)?;
        (0..FOUNDATIONS)
            .map(Pile::Foundation)
            .find(|&to| builds_up_by_suit(board.top(to), top.card))
    }
}

impl Rules for FreeCell {
    fn piles(&self) -> Vec<Pile> {
        (0..CELLS)
            .map(Pile::Cell)
            .chain((0..FOUNDATIONS).map(Pile::Foundation))
            .chain((0..TABLEAUS).map(Pile::Tableau))
            .collect()
    }

    /// Deals every card face up, across the eight columns in turn: the first four get seven cards
    /// and the rest six.
    fn deal(&self, deck: Vec<Card>, board: &mut Board) {
        for (i, card) in deck.into_iter().rev().enumerate() {
            board
                .pile_mut(Pile::Tableau(i % TABLEAUS))
                .push(Slot::up(card));
        }
    }

    fn shape(&self) -> Shape {
        Shape {
            columns: TABLEAUS,
            fan_row: 1.0,
            landscape_fan: (0, 9),
            portrait_fan: (0, 14),
        }
    }

    fn spot(&self, pile: Pile) -> Spot {
        match pile {
            Pile::Cell(i) => Spot::stacked(i as f32, 0.0),
            Pile::Foundation(i) => Spot::stacked((CELLS + i) as f32, 0.0),
            Pile::Tableau(i) => Spot::fanned(i as f32, 1.0),
            Pile::Stock | Pile::Waste => unreachable!("FreeCell has no stock"),
        }
    }

    fn can_pick_up(&self, board: &Board, from: Pile, index: usize) -> bool {
        let is_top = index + 1 == board.pile(from).len();
        match from {
            Pile::Cell(_) => is_top,
            Pile::Tableau(_) => Self::is_run(board, from, index),
            // Cards on the foundations stay there.
            Pile::Foundation(_) | Pile::Stock | Pile::Waste => false,
        }
    }

    fn can_move(&self, board: &Board, from: Pile, index: usize, to: Pile) -> bool {
        let cards = board.pile(from);
        let Some(card) = cards.get(index).map(|slot| slot.card) else {
            return false;
        };
        let count = cards.len() - index;
        let top = board.top(to);
        match to {
            Pile::Cell(_) => count == 1 && top.is_none(),
            Pile::Foundation(_) => count == 1 && builds_up_by_suit(top, card),
            Pile::Tableau(_) => {
                count <= Self::max_run(board, to)
                    && top.is_none_or(|top| builds_down_alternating(top, card))
            }
            Pile::Stock | Pile::Waste => false,
        }
    }

    /// A lone top card goes to a foundation if it can. Otherwise the cards go to the first column
    /// with cards that takes them (searching rightward from the column they're in, wrapping
    /// around), then to an empty column, and a lone card finally to a free cell.
    fn best_move(&self, board: &Board, from: Pile, index: usize) -> Option<Pile> {
        let len = board.pile(from).len();
        if index + 1 == len
            && let Some(foundation) = Self::foundation_for(board, from)
        {
            return Some(foundation);
        }
        let start = match from {
            Pile::Tableau(column) => column + 1,
            _ => 0,
        };
        let columns = (start..start + TABLEAUS).map(|column| Pile::Tableau(column % TABLEAUS));
        let fits = |to: &Pile| *to != from && self.can_move(board, from, index, *to);
        // Moving a whole column to an empty one changes nothing.
        let whole_column = matches!(from, Pile::Tableau(_)) && index == 0;
        columns
            .clone()
            .filter(|&to| !board.pile(to).is_empty())
            .find(fits)
            .or_else(|| {
                columns
                    .filter(|&to| board.pile(to).is_empty() && !whole_column)
                    .find(fits)
            })
            .or_else(|| {
                let to_cell = !matches!(from, Pile::Cell(_));
                (0..CELLS).map(Pile::Cell).filter(|_| to_cell).find(fits)
            })
    }

    fn is_won(&self, board: &Board) -> bool {
        foundations_full(board, FOUNDATIONS)
    }

    /// Once every column runs downward in rank (each card no higher than the one below it), the
    /// lowest card left is always on top of a column or in a cell, and can go to its foundation.
    fn is_certainly_winnable(&self, board: &Board) -> bool {
        (0..TABLEAUS).all(|i| {
            board
                .pile(Pile::Tableau(i))
                .windows(2)
                .all(|pair| pair[0].card.rank >= pair[1].card.rank)
        })
    }

    /// Plays the lowest-ranked card that can go to a foundation.
    fn auto_finish_step(&self, board: &Board) -> Option<Action> {
        (0..CELLS)
            .map(Pile::Cell)
            .chain((0..TABLEAUS).map(Pile::Tableau))
            .filter_map(|from| Some((from, Self::foundation_for(board, from)?)))
            .min_by_key(|&(from, _)| board.top(from).map(|slot| slot.card.rank))
            .map(|(from, to)| Action::Move {
                from,
                index: board.pile(from).len() - 1,
                to,
            })
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

    /// A game of FreeCell with no cards, changed by `setup`.
    fn freecell(setup: impl FnOnce(&mut Board)) -> Game {
        let mut board = Board::new(FreeCell.piles());
        setup(&mut board);
        Game::with_board(GameKind::FreeCell, board)
    }

    fn up(cards: &[Card]) -> Vec<Slot> {
        cards.iter().copied().map(Slot::up).collect()
    }

    /// Fills all but `free_cells` of the cells, and columns 2 onward except the last
    /// `empty_columns`, with a King of clubs, so they're neither free nor empty.
    fn crowd(board: &mut Board, free_cells: usize, empty_columns: usize) {
        let filler = Slot::up(card(Rank::King, Suit::Clubs));
        for i in free_cells..CELLS {
            board.pile_mut(Pile::Cell(i)).push(filler);
        }
        for i in 2..TABLEAUS - empty_columns {
            board.pile_mut(Pile::Tableau(i)).push(filler);
        }
    }

    /// A run from the Nine of spades down to the Five, alternating colors.
    fn run() -> Vec<Slot> {
        up(&[
            card(Rank::Nine, Suit::Spades),
            card(Rank::Eight, Suit::Hearts),
            card(Rank::Seven, Suit::Clubs),
            card(Rank::Six, Suit::Diamonds),
            card(Rank::Five, Suit::Spades),
        ])
    }

    #[test]
    fn deal_layout() {
        let game = Game::deal(GameKind::FreeCell, shuffle_deck());
        for i in 0..TABLEAUS {
            let pile = game.pile(Pile::Tableau(i));
            assert_eq!(pile.len(), if i < 4 { 7 } else { 6 });
            assert!(pile.iter().all(|slot| slot.face_up));
        }
        assert!((0..CELLS).all(|i| game.pile(Pile::Cell(i)).is_empty()));
    }

    #[test]
    fn only_runs_can_be_picked_up() {
        let game = freecell(|board| {
            let mut cards = run();
            cards.insert(2, Slot::up(card(Rank::Two, Suit::Hearts)));
            *board.pile_mut(Pile::Tableau(0)) = cards;
        });
        assert!(!game.can_pick_up(Pile::Tableau(0), 0), "broken run");
        assert!(game.can_pick_up(Pile::Tableau(0), 3));
        assert!(game.can_pick_up(Pile::Tableau(0), 5));
    }

    #[test]
    fn runs_are_limited_by_free_cells_and_empty_columns() {
        // A Ten of hearts to move the run onto.
        let with = |free_cells, empty_columns| {
            freecell(|board| {
                *board.pile_mut(Pile::Tableau(0)) = run();
                *board.pile_mut(Pile::Tableau(1)) = up(&[card(Rank::Ten, Suit::Hearts)]);
                crowd(board, free_cells, empty_columns);
            })
        };
        // No help at all: one card at a time.
        let game = with(0, 0);
        assert!(!game.can_move(Pile::Tableau(0), 3, Pile::Tableau(1)));
        // 4 free cells: up to five cards.
        let game = with(4, 0);
        assert!(game.can_move(Pile::Tableau(0), 0, Pile::Tableau(1)));
        // 1 free cell and 1 empty column: (1 + 1) * 2 = four cards, not five.
        let game = with(1, 1);
        assert_eq!(FreeCell::max_run(game.board(), Pile::Tableau(1)), 4);
        assert!(!game.can_move(Pile::Tableau(0), 0, Pile::Tableau(1)));
        // Moving into the empty column itself doesn't count it: two cards.
        let game = with(1, 1);
        let empty = Pile::Tableau(TABLEAUS - 1);
        assert_eq!(FreeCell::max_run(game.board(), empty), 2);
        assert!(game.can_move(Pile::Tableau(0), 3, empty));
        assert!(!game.can_move(Pile::Tableau(0), 2, empty));
    }

    #[test]
    fn cells_hold_one_card() {
        let mut game = freecell(|board| {
            *board.pile_mut(Pile::Tableau(0)) = run();
        });
        assert!(
            !game.can_move(Pile::Tableau(0), 3, Pile::Cell(0)),
            "two cards"
        );
        assert!(game.try_move(Pile::Tableau(0), 4, Pile::Cell(0)));
        assert!(!game.can_move(Pile::Tableau(0), 3, Pile::Cell(0)), "full");
        assert!(
            game.try_move(Pile::Cell(0), 0, Pile::Tableau(1)),
            "any card on empty"
        );
    }

    #[test]
    fn foundation_cards_stay_put() {
        let game = freecell(|board| {
            *board.pile_mut(Pile::Foundation(0)) = up(&[card(Rank::Ace, Suit::Hearts)]);
        });
        assert!(!game.can_pick_up(Pile::Foundation(0), 0));
    }

    #[test]
    fn tap_prefers_foundation_then_columns_then_cells() {
        let game = freecell(|board| {
            *board.pile_mut(Pile::Tableau(0)) =
                up(&[card(Rank::Two, Suit::Clubs), card(Rank::Ace, Suit::Hearts)]);
            *board.pile_mut(Pile::Tableau(1)) = up(&[card(Rank::Seven, Suit::Spades)]);
            *board.pile_mut(Pile::Tableau(2)) = up(&[
                card(Rank::Four, Suit::Diamonds),
                card(Rank::Six, Suit::Hearts),
            ]);
            *board.pile_mut(Pile::Tableau(3)) = up(&[card(Rank::Nine, Suit::Clubs)]);
            // No empty columns.
            for i in 4..TABLEAUS {
                *board.pile_mut(Pile::Tableau(i)) = up(&[card(Rank::King, Suit::Clubs)]);
            }
        });
        assert_eq!(
            game.best_move(Pile::Tableau(0), 1),
            Some(Pile::Foundation(0))
        );
        // The Six of hearts goes onto the Seven of spades.
        assert_eq!(game.best_move(Pile::Tableau(2), 1), Some(Pile::Tableau(1)));
        // The Nine has no column to go to, so it goes to a free cell.
        assert_eq!(game.best_move(Pile::Tableau(3), 0), Some(Pile::Cell(0)));
    }

    #[test]
    fn tap_uses_empty_columns_but_not_for_whole_columns() {
        let game = freecell(|board| {
            *board.pile_mut(Pile::Tableau(0)) = up(&[card(Rank::Nine, Suit::Clubs)]);
            *board.pile_mut(Pile::Tableau(1)) = up(&[
                card(Rank::Two, Suit::Hearts),
                card(Rank::Eight, Suit::Clubs),
            ]);
        });
        assert_eq!(game.best_move(Pile::Tableau(1), 1), Some(Pile::Tableau(2)));
        // A lone card in a column goes to a free cell, not another empty column.
        assert_eq!(game.best_move(Pile::Tableau(0), 0), Some(Pile::Cell(0)));
    }

    #[test]
    fn auto_finishes_sorted_columns() {
        // Deal a deck sorted so that every column runs downward, Kings at the bottom.
        let mut deck = new_deck();
        deck.sort_by_key(|card| card.rank);
        let mut game = freecell(|board| {
            for (i, card) in deck.into_iter().rev().enumerate() {
                board
                    .pile_mut(Pile::Tableau(i % TABLEAUS))
                    .push(Slot::up(card));
            }
        });
        assert!(game.is_certainly_winnable());
        for _ in 0..52 {
            assert!(game.auto_finish_step());
        }
        assert!(game.is_won());
        assert!(!game.auto_finish_step());

        let dealt = Game::deal(GameKind::FreeCell, shuffle_deck());
        assert!(!dealt.is_certainly_winnable());
    }
}
