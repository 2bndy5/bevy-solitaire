//! Spider: two decks' worth of cards in ten columns; build runs of one suit from King to Ace,
//! which leave the table when complete.
use crate::{
    cards::{Card, Rank, Suit},
    game::{Board, Pile, Rules, Shape, Slot, Spot},
};

const TABLEAUS: usize = 10;
/// One foundation for each complete run: 104 cards make eight.
const FOUNDATIONS: usize = 8;
/// Cards dealt from the stock at a time: one to each column.
const DEAL: usize = TABLEAUS;

/// The rules of Spider, played with one, two or four suits.
pub struct Spider {
    /// How many suits the 104 cards come in. Fewer suits make an easier game.
    pub suits: u8,
}

impl Spider {
    /// Is every card from `index` to the top of `pile` face up, in one suit, and going down one
    /// rank at a time?
    fn is_run(board: &Board, pile: Pile, index: usize) -> bool {
        let cards = &board.pile(pile)[index..];
        cards.iter().all(|slot| slot.face_up)
            && cards.windows(2).all(|pair| {
                pair[0].card.suit == pair[1].card.suit
                    && pair[0].card.rank.as_u8() == pair[1].card.rank.as_u8() + 1
            })
    }

    /// Moves any complete King-to-Ace run off the top of each column to a foundation, and turns
    /// up the cards that exposes.
    fn clear_complete_runs(board: &mut Board) {
        for column in (0..TABLEAUS).map(Pile::Tableau) {
            let len = board.pile(column).len();
            let Some(start) = len.checked_sub(Rank::ALL.len()) else {
                continue;
            };
            let complete = board.pile(column)[start].card.rank == Rank::King
                && Self::is_run(board, column, start);
            let foundation = (0..FOUNDATIONS)
                .map(Pile::Foundation)
                .find(|&pile| board.pile(pile).is_empty());
            if complete && let Some(foundation) = foundation {
                board.move_cards(column, start, foundation);
                board.turn_up_top(column);
            }
        }
    }
}

impl Rules for Spider {
    fn piles(&self) -> Vec<Pile> {
        std::iter::once(Pile::Stock)
            .chain((0..FOUNDATIONS).map(Pile::Foundation))
            .chain((0..TABLEAUS).map(Pile::Tableau))
            .collect()
    }

    /// 104 cards: the 13 ranks in `suits` suits, as many times over as it takes.
    fn deck(&self) -> Vec<Card> {
        let suits: &[Suit] = match self.suits {
            1 => &[Suit::Spades],
            2 => &[Suit::Spades, Suit::Hearts],
            _ => &Suit::ALL,
        };
        let copies = (FOUNDATIONS / suits.len()) as u8;
        (0..copies)
            .flat_map(|copy| {
                suits.iter().flat_map(move |&suit| {
                    Rank::ALL
                        .iter()
                        .map(move |&rank| Card::new(rank, suit).with_copy(copy))
                })
            })
            .collect()
    }

    /// Deals 54 cards to the ten columns (six to each of the first four, five to the rest), only
    /// the top card of each face up. The other 50 form the stock: five deals of ten.
    fn deal(&self, mut deck: Vec<Card>, board: &mut Board) {
        for i in 0..54 {
            let card = deck.pop().expect("a double deck has enough cards to deal");
            board
                .pile_mut(Pile::Tableau(i % TABLEAUS))
                .push(Slot::down(card));
        }
        for column in 0..TABLEAUS {
            board.turn_up_top(Pile::Tableau(column));
        }
        *board.pile_mut(Pile::Stock) = deck.into_iter().map(Slot::down).collect();
    }

    fn shape(&self) -> Shape {
        Shape {
            columns: TABLEAUS,
            fan_row: 1.0,
            landscape_fan: (4, 9),
            portrait_fan: (5, 14),
        }
    }

    fn spot(&self, pile: Pile) -> Spot {
        match pile {
            Pile::Foundation(i) => Spot::stacked(i as f32, 0.0),
            Pile::Stock => Spot::stacked(9.0, 0.0),
            Pile::Tableau(i) => Spot::fanned(i as f32, 1.0),
            Pile::Waste | Pile::Cell(_) => unreachable!("Spider has no {pile:?}"),
        }
    }

    /// Deals a face-up card from the stock onto every column. As in the classic game, this isn't
    /// allowed while a column is empty.
    fn draw(&self, board: &mut Board) -> bool {
        let stock = board.pile(Pile::Stock).len();
        let any_empty = (0..TABLEAUS).any(|i| board.pile(Pile::Tableau(i)).is_empty());
        if stock == 0 || any_empty {
            return false;
        }
        for column in 0..DEAL.min(stock) {
            let card = board.pile_mut(Pile::Stock).pop().expect("checked above");
            board
                .pile_mut(Pile::Tableau(column))
                .push(Slot::up(card.card));
        }
        Self::clear_complete_runs(board);
        true
    }

    fn can_pick_up(&self, board: &Board, from: Pile, index: usize) -> bool {
        matches!(from, Pile::Tableau(_)) && Self::is_run(board, from, index)
    }

    /// A run goes on any card one rank higher, whatever its suit, or in an empty column.
    fn can_move(&self, board: &Board, from: Pile, index: usize, to: Pile) -> bool {
        let Some(card) = board.pile(from).get(index).map(|slot| slot.card) else {
            return false;
        };
        match to {
            Pile::Tableau(_) => board
                .top(to)
                .is_none_or(|top| top.face_up && top.card.rank.as_u8() == card.rank.as_u8() + 1),
            _ => false,
        }
    }

    /// Moves the cards, then clears any run it completes and turns up the card it exposes.
    fn apply_move(&self, board: &mut Board, from: Pile, index: usize, to: Pile) {
        board.move_cards(from, index, to);
        board.turn_up_top(from);
        Self::clear_complete_runs(board);
    }

    /// The first column (searching rightward, wrapping around) whose top card is one rank higher
    /// and the same suit; failing that, one rank higher in any suit; failing that, an empty column
    /// (unless the run is already a whole column).
    fn best_move(&self, board: &Board, from: Pile, index: usize) -> Option<Pile> {
        let card = board.pile(from).get(index)?.card;
        let start = match from {
            Pile::Tableau(column) => column + 1,
            _ => 0,
        };
        let columns = || {
            (start..start + TABLEAUS)
                .map(|column| Pile::Tableau(column % TABLEAUS))
                .filter(move |&to| to != from)
        };
        let fits = |to: &Pile| self.can_move(board, from, index, *to);
        let same_suit = |to: &Pile| board.top(*to).is_some_and(|top| top.card.suit == card.suit);
        let whole_column = index == 0;
        columns()
            .filter(same_suit)
            .find(fits)
            .or_else(|| {
                columns()
                    .filter(|&to| !board.pile(to).is_empty())
                    .find(fits)
            })
            .or_else(|| {
                columns()
                    .filter(|&to| board.pile(to).is_empty() && !whole_column)
                    .find(fits)
            })
    }

    fn is_won(&self, board: &Board) -> bool {
        (0..FOUNDATIONS).all(|i| board.pile(Pile::Foundation(i)).len() == Rank::ALL.len())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        cards::shuffled,
        game::{Game, card},
        games::GameKind,
    };

    const KINDS: [GameKind; 3] = [GameKind::Spider1, GameKind::Spider2, GameKind::Spider4];

    /// A game of one-suit Spider with no cards, changed by `setup`.
    fn spider(setup: impl FnOnce(&mut Board)) -> Game {
        let mut board = Board::new(GameKind::Spider1.rules().piles());
        setup(&mut board);
        Game::with_board(GameKind::Spider1, board)
    }

    /// Face-up spades of the given ranks (as numbers), bottom first.
    fn spades(ranks: &[u8]) -> Vec<Slot> {
        ranks
            .iter()
            .map(|&rank| {
                let rank = Rank::ALL[usize::from(rank) - 1];
                Slot::up(card(rank, Suit::Spades))
            })
            .collect()
    }

    #[test]
    fn decks_have_104_distinct_cards() {
        for kind in KINDS {
            let deck = kind.rules().deck();
            assert_eq!(deck.len(), 104);
            let unique: std::collections::HashSet<_> = deck.iter().collect();
            assert_eq!(unique.len(), 104, "{kind:?}");
        }
        let suits = |kind: GameKind| {
            let mut suits: Vec<Suit> = kind.rules().deck().iter().map(|c| c.suit).collect();
            suits.sort();
            suits.dedup();
            suits.len()
        };
        assert_eq!(KINDS.map(suits), [1, 2, 4]);
    }

    #[test]
    fn deal_layout() {
        for kind in KINDS {
            let game = Game::deal(kind, shuffled(kind.rules().deck()));
            for i in 0..TABLEAUS {
                let pile = game.pile(Pile::Tableau(i));
                assert_eq!(pile.len(), if i < 4 { 6 } else { 5 });
                assert!(pile.last().unwrap().face_up);
                assert!(pile[..pile.len() - 1].iter().all(|slot| !slot.face_up));
            }
            assert_eq!(game.pile(Pile::Stock).len(), 50);
            // Saves keep the copies apart.
            let restored = Game::from_text(&game.to_text()).unwrap();
            assert_eq!(restored.board(), game.board());
        }
    }

    #[test]
    fn runs_must_share_a_suit() {
        let game = spider(|board| {
            let mut cards = spades(&[9, 8]);
            cards.push(Slot::up(card(Rank::Seven, Suit::Hearts)));
            *board.pile_mut(Pile::Tableau(0)) = cards;
            *board.pile_mut(Pile::Tableau(1)) = spades(&[10]);
        });
        assert!(!game.can_pick_up(Pile::Tableau(0), 0), "mixed suits");
        assert!(game.can_pick_up(Pile::Tableau(0), 2), "a lone card");
        // But anything one rank lower goes on a card, whatever its suit.
        let game = spider(|board| {
            *board.pile_mut(Pile::Tableau(0)) = vec![Slot::up(card(Rank::Nine, Suit::Hearts))];
            *board.pile_mut(Pile::Tableau(1)) = spades(&[10]);
        });
        assert!(game.can_move(Pile::Tableau(0), 0, Pile::Tableau(1)));
    }

    #[test]
    fn tap_prefers_the_same_suit() {
        let game = spider(|board| {
            *board.pile_mut(Pile::Tableau(0)) = vec![Slot::up(card(Rank::Ten, Suit::Hearts))];
            *board.pile_mut(Pile::Tableau(1)) = spades(&[9]);
            *board.pile_mut(Pile::Tableau(2)) = spades(&[5, 3]);
            *board.pile_mut(Pile::Tableau(3)) = spades(&[10]);
        });
        assert_eq!(game.best_move(Pile::Tableau(1), 0), Some(Pile::Tableau(3)));
        // Nothing one rank higher for the Three: an empty column.
        assert_eq!(game.best_move(Pile::Tableau(2), 1), Some(Pile::Tableau(4)));
        // A lone card never moves to another empty column.
        assert_eq!(game.best_move(Pile::Tableau(0), 0), None);
    }

    #[test]
    fn complete_runs_leave_the_table() {
        let mut game = spider(|board| {
            let mut column = vec![Slot::down(card(Rank::Four, Suit::Hearts))];
            column.extend(spades(&[13, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2]));
            *board.pile_mut(Pile::Tableau(0)) = column;
            *board.pile_mut(Pile::Tableau(1)) = spades(&[1]);
        });
        assert!(game.try_move(Pile::Tableau(1), 0, Pile::Tableau(0)));
        assert_eq!(game.pile(Pile::Foundation(0)).len(), 13);
        assert_eq!(game.pile(Pile::Tableau(0)).len(), 1);
        assert!(
            game.pile(Pile::Tableau(0))[0].face_up,
            "exposed card turned up"
        );
        assert!(game.undo());
        assert!(game.pile(Pile::Foundation(0)).is_empty());
        assert_eq!(game.pile(Pile::Tableau(0)).len(), 13);
    }

    #[test]
    fn stock_deals_a_row_unless_a_column_is_empty() {
        let mut game = Game::deal(GameKind::Spider2, GameKind::Spider2.rules().deck());
        assert!(game.draw());
        assert_eq!(game.pile(Pile::Stock).len(), 40);
        for i in 0..TABLEAUS {
            assert!(game.pile(Pile::Tableau(i)).last().unwrap().face_up);
        }
        let mut board = game.board().clone();
        board.pile_mut(Pile::Tableau(5)).clear();
        let mut game = Game::with_board(GameKind::Spider2, board);
        assert!(!game.draw(), "an empty column");
    }
}
