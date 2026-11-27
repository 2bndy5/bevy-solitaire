//! Klondike (draw one): the classic solitaire.
use crate::{
    cards::{Card, Rank},
    game::{
        Action, Board, Pile, Rules, Shape, Slot, Spot, builds_down_alternating, builds_up_by_suit,
        foundations_full,
    },
};

const FOUNDATIONS: usize = 4;
const TABLEAUS: usize = 7;

/// The rules of Klondike.
pub struct Klondike;

impl Klondike {
    /// A foundation that takes the top card of `from`, if any.
    fn foundation_for(board: &Board, from: Pile) -> Option<Pile> {
        let index = board.pile(from).len().checked_sub(1)?;
        (0..FOUNDATIONS).map(Pile::Foundation).find(|&to| {
            Klondike.can_pick_up(board, from, index) && Klondike.can_move(board, from, index, to)
        })
    }
}

impl Rules for Klondike {
    fn piles(&self) -> Vec<Pile> {
        [Pile::Stock, Pile::Waste]
            .into_iter()
            .chain((0..FOUNDATIONS).map(Pile::Foundation))
            .chain((0..TABLEAUS).map(Pile::Tableau))
            .collect()
    }

    /// Cards are dealt from the end of `deck`. Tableau `i` gets `i + 1` cards with only the last
    /// one face up; the remaining 24 cards form the stock.
    fn deal(&self, mut deck: Vec<Card>, board: &mut Board) {
        for row in 0..TABLEAUS {
            for column in row..TABLEAUS {
                let card = deck.pop().expect("a full deck has enough cards to deal");
                board
                    .pile_mut(Pile::Tableau(column))
                    .push(if row == column {
                        Slot::up(card)
                    } else {
                        Slot::down(card)
                    });
            }
        }
        *board.pile_mut(Pile::Stock) = deck.into_iter().map(Slot::down).collect();
    }

    fn shape(&self) -> Shape {
        Shape {
            columns: TABLEAUS,
            fan_row: 1.0,
            // A typical deep pile: 6 face-down and 7 face-up cards.
            landscape_fan: (6, 6),
            // The worst case: 6 face-down cards and a full King-to-Ace run.
            portrait_fan: (6, 12),
        }
    }

    fn spot(&self, pile: Pile) -> Spot {
        match pile {
            Pile::Stock => Spot::stacked(0.0, 0.0),
            Pile::Waste => Spot::stacked(1.0, 0.0),
            Pile::Foundation(i) => Spot::stacked(3.0 + i as f32, 0.0),
            Pile::Tableau(i) => Spot::fanned(i as f32, 1.0),
            Pile::Cell(_) => unreachable!("Klondike has no cells"),
        }
    }

    /// Draws one card from the stock onto the waste. If the stock is empty, the waste is turned
    /// over to become the stock again.
    fn draw(&self, board: &mut Board) -> bool {
        if let Some(mut slot) = board.pile_mut(Pile::Stock).pop() {
            slot.face_up = true;
            board.pile_mut(Pile::Waste).push(slot);
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
        let cards = board.pile(from);
        let Some(slot) = cards.get(index) else {
            return false;
        };
        let is_top = index + 1 == cards.len();
        match from {
            Pile::Waste | Pile::Foundation(_) => is_top,
            // Face-up tableau cards are always a valid alternating sequence.
            Pile::Tableau(_) => slot.face_up,
            Pile::Stock | Pile::Cell(_) => false,
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
            Pile::Foundation(_) => count == 1 && builds_up_by_suit(top, card),
            Pile::Tableau(_) => match top {
                None => card.rank == Rank::King,
                Some(top) => builds_down_alternating(top, card),
            },
            Pile::Stock | Pile::Waste | Pile::Cell(_) => false,
        }
    }

    /// Moves the cards, turning face up any face-down card exposed on a tableau.
    fn apply_move(&self, board: &mut Board, from: Pile, index: usize, to: Pile) {
        board.move_cards(from, index, to);
        if let Pile::Tableau(_) = from {
            board.turn_up_top(from);
        }
    }

    /// A lone top card goes to a foundation if it can. Otherwise the cards go to the first tableau
    /// column that accepts them, searching rightward from (and wrapping around past) the column they
    /// are in, so tapping again moves them on to the next option. A King already at the bottom of a
    /// column is never moved to an empty one, since that changes nothing.
    fn best_move(&self, board: &Board, from: Pile, index: usize) -> Option<Pile> {
        if index + 1 == board.pile(from).len()
            && let Some(foundation) = Self::foundation_for(board, from)
        {
            return Some(foundation);
        }
        let start = match from {
            Pile::Tableau(column) => column + 1,
            _ => 0,
        };
        let king_at_bottom = matches!(from, Pile::Tableau(_)) && index == 0;
        (start..start + TABLEAUS)
            .map(|column| Pile::Tableau(column % TABLEAUS))
            .filter(|&to| !(king_at_bottom && board.pile(to).is_empty()))
            .find(|&to| to != from && self.can_move(board, from, index, to))
    }

    fn is_won(&self, board: &Board) -> bool {
        foundations_full(board, FOUNDATIONS)
    }

    /// Once the stock is empty and every tableau card is face up, the cards can always be played
    /// to the foundations: each tableau pile is a descending run, so the lowest card still in
    /// play is on top of its pile, or can be reached by cycling through the waste.
    fn is_certainly_winnable(&self, board: &Board) -> bool {
        board.pile(Pile::Stock).is_empty()
            && (0..TABLEAUS)
                .flat_map(|i| board.pile(Pile::Tableau(i)))
                .all(|slot| slot.face_up)
    }

    /// Plays the lowest-ranked card that can go to a foundation; when none can, draws from (or
    /// recycles) the waste.
    fn auto_finish_step(&self, board: &Board) -> Option<Action> {
        let best = std::iter::once(Pile::Waste)
            .chain((0..TABLEAUS).map(Pile::Tableau))
            .filter_map(|from| Some((from, Self::foundation_for(board, from)?)))
            .min_by_key(|&(from, _)| board.top(from).map(|slot| slot.card.rank));
        match best {
            Some((from, to)) => Some(Action::Move {
                from,
                index: board.pile(from).len() - 1,
                to,
            }),
            None if !self.is_won(board) => Some(Action::Draw),
            None => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        cards::{Suit, new_deck},
        game::{Game, card},
        games::GameKind,
    };

    /// A game of Klondike with no cards, changed by `setup`.
    fn klondike(setup: impl FnOnce(&mut Board)) -> Game {
        let mut board = Board::new(Klondike.piles());
        setup(&mut board);
        Game::with_board(GameKind::Klondike, board)
    }

    /// A game with only the given tableau columns (bottom card first, all face up).
    fn with_tableaus(columns: &[&[Card]]) -> Game {
        klondike(|board| {
            for (i, cards) in columns.iter().enumerate() {
                *board.pile_mut(Pile::Tableau(i)) = cards.iter().copied().map(Slot::up).collect();
            }
        })
    }

    fn up(cards: &[Card]) -> Vec<Slot> {
        cards.iter().copied().map(Slot::up).collect()
    }

    /// Plays [`Game::auto_finish_step`] until it stops, failing if it seems stuck.
    fn auto_finish(game: &mut Game) {
        for _ in 0..10_000 {
            if !game.auto_finish_step() {
                return;
            }
        }
        panic!("auto-finish did not stop");
    }

    fn foundation_for(game: &Game, from: Pile) -> Option<Pile> {
        Klondike::foundation_for(game.board(), from)
    }

    fn move_to_foundation(game: &mut Game, from: Pile) -> bool {
        match foundation_for(game, from) {
            Some(to) => game.try_move(from, game.pile(from).len() - 1, to),
            None => false,
        }
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
        let mut game = with_tableaus(&[
            &[card(Rank::Six, Suit::Spades)],
            &[card(Rank::Five, Suit::Hearts)],
            &[card(Rank::Nine, Suit::Clubs)],
            &[card(Rank::Six, Suit::Clubs)],
        ]);
        // From column 1, column 3 comes before wrapping around to column 0.
        assert_eq!(game.best_move(Pile::Tableau(1), 0), Some(Pile::Tableau(3)));
        assert_eq!(game.auto_move(Pile::Tableau(1), 0), Some(Pile::Tableau(3)));
        // Tapping again moves it on to the next option.
        assert_eq!(game.auto_move(Pile::Tableau(3), 1), Some(Pile::Tableau(0)));
    }

    #[test]
    fn tap_moves_runs_and_waste_cards() {
        let mut game = klondike(|board| {
            *board.pile_mut(Pile::Tableau(0)) = vec![
                Slot::down(card(Rank::Four, Suit::Clubs)),
                Slot::up(card(Rank::Nine, Suit::Spades)),
                Slot::up(card(Rank::Eight, Suit::Hearts)),
            ];
            *board.pile_mut(Pile::Tableau(1)) = up(&[card(Rank::Ten, Suit::Diamonds)]);
            *board.pile_mut(Pile::Waste) = up(&[card(Rank::Seven, Suit::Clubs)]);
        });
        assert_eq!(game.best_move(Pile::Tableau(0), 0), None, "face down");
        assert_eq!(game.auto_move(Pile::Tableau(0), 1), Some(Pile::Tableau(1)));
        assert_eq!(game.pile(Pile::Tableau(1)).len(), 3);
        assert!(game.pile(Pile::Tableau(0))[0].face_up);

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
        let mut game = Game::deal(GameKind::Klondike, new_deck());
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
        assert_eq!(game.board(), after_draw.board());
        assert_eq!(game.moves, 1);
        assert!(game.undo());
        assert_eq!(game.board(), dealt.board());
        assert_eq!(game.moves, 0);
        assert!(!game.can_undo());
    }

    #[test]
    fn undo_turns_exposed_card_back_over() {
        let mut game = klondike(|board| {
            *board.pile_mut(Pile::Tableau(0)) = vec![
                Slot::down(card(Rank::Four, Suit::Clubs)),
                Slot::up(card(Rank::Ace, Suit::Hearts)),
            ];
        });
        assert!(move_to_foundation(&mut game, Pile::Tableau(0)));
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
    fn certainly_winnable() {
        let game = Game::deal(GameKind::Klondike, new_deck());
        assert!(
            !game.is_certainly_winnable(),
            "stock and face-down cards remain"
        );

        // Everything face up in the tableau, with some cards still in the waste.
        let mut deck = new_deck();
        let mut game = klondike(|board| {
            for (i, card) in deck.drain(..40).enumerate() {
                board
                    .pile_mut(Pile::Tableau(i % TABLEAUS))
                    .push(Slot::up(card));
            }
            *board.pile_mut(Pile::Waste) = up(&deck);
        });
        assert!(game.is_certainly_winnable());
        let mut board = game.board().clone();
        board.pile_mut(Pile::Tableau(2))[0].face_up = false;
        game = Game::with_board(GameKind::Klondike, board);
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
            let mut game = klondike(|board| {
                let mut rest = Vec::new();
                for card in deck {
                    let fits = (0..TABLEAUS).find(|&i| match board.top(Pile::Tableau(i)) {
                        None => card.rank == Rank::King,
                        Some(top) => builds_down_alternating(top, card),
                    });
                    match fits {
                        Some(i) => board.pile_mut(Pile::Tableau(i)).push(Slot::up(card)),
                        None => rest.push(Slot::up(card)),
                    }
                }
                *board.pile_mut(Pile::Waste) = rest;
            });
            assert!(game.is_certainly_winnable());
            auto_finish(&mut game);
            assert!(game.is_won(), "seed {seed} not finished");
        }
    }

    #[test]
    fn auto_finish_plays_lowest_card_first() {
        let mut game = klondike(|board| {
            *board.pile_mut(Pile::Waste) = up(&[card(Rank::Two, Suit::Spades)]);
            *board.pile_mut(Pile::Tableau(0)) = up(&[card(Rank::Ace, Suit::Spades)]);
            *board.pile_mut(Pile::Tableau(1)) = up(&[card(Rank::Ace, Suit::Hearts)]);
        });
        assert!(game.auto_finish_step());
        assert!(game.pile(Pile::Waste).len() == 1, "an ace goes first");
        auto_finish(&mut game);
        assert!(game.pile(Pile::Waste).is_empty());
        assert_eq!(
            game.pile(Pile::Foundation(0)).len() + game.pile(Pile::Foundation(1)).len(),
            3
        );
    }

    #[test]
    fn deal_layout() {
        let game = Game::deal(GameKind::Klondike, new_deck());
        for i in 0..TABLEAUS {
            let pile = game.pile(Pile::Tableau(i));
            assert_eq!(pile.len(), i + 1);
            assert!(pile.last().unwrap().face_up);
            assert!(pile[..i].iter().all(|s| !s.face_up));
        }
        assert_eq!(game.pile(Pile::Stock).len(), 24);
        assert!(game.pile(Pile::Waste).is_empty());
        assert_eq!(game.board().cards().count(), 52);
    }

    #[test]
    fn draw_and_recycle() {
        let mut game = Game::deal(GameKind::Klondike, new_deck());
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
    fn foundation_rules() {
        let mut game = klondike(|board| {
            *board.pile_mut(Pile::Waste) =
                up(&[card(Rank::Two, Suit::Hearts), card(Rank::Ace, Suit::Hearts)]);
        });
        assert!(
            !game.can_move(Pile::Waste, 0, Pile::Foundation(0)),
            "not top card"
        );
        assert!(game.try_move(Pile::Waste, 1, Pile::Foundation(2)));
        assert!(
            !game.can_move(Pile::Waste, 0, Pile::Foundation(0)),
            "2 on empty"
        );
        assert_eq!(
            foundation_for(&game, Pile::Waste),
            Some(Pile::Foundation(2))
        );
        assert!(move_to_foundation(&mut game, Pile::Waste));
        assert_eq!(game.pile(Pile::Foundation(2)).len(), 2);

        let mut board = game.board().clone();
        *board.pile_mut(Pile::Waste) = up(&[card(Rank::Three, Suit::Diamonds)]);
        let game = Game::with_board(GameKind::Klondike, board);
        assert!(
            !game.can_move(Pile::Waste, 0, Pile::Foundation(2)),
            "wrong suit"
        );
    }

    #[test]
    fn tableau_rules() {
        let mut game = klondike(|board| {
            *board.pile_mut(Pile::Tableau(0)) = vec![
                Slot::down(card(Rank::Four, Suit::Clubs)),
                Slot::up(card(Rank::Nine, Suit::Spades)),
                Slot::up(card(Rank::Eight, Suit::Hearts)),
            ];
            *board.pile_mut(Pile::Tableau(1)) = up(&[card(Rank::Ten, Suit::Diamonds)]);
            *board.pile_mut(Pile::Tableau(2)) = up(&[card(Rank::Ten, Suit::Clubs)]);
        });

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
        let game = klondike(|board| {
            *board.pile_mut(Pile::Waste) = up(&[
                card(Rank::King, Suit::Spades),
                card(Rank::Queen, Suit::Hearts),
            ]);
        });
        assert!(!game.can_move(Pile::Waste, 1, Pile::Tableau(3)));
        let mut board = game.board().clone();
        board.pile_mut(Pile::Waste).pop();
        let mut game = Game::with_board(GameKind::Klondike, board);
        assert!(game.try_move(Pile::Waste, 0, Pile::Tableau(3)));
    }

    #[test]
    fn only_single_cards_to_foundation() {
        let game = klondike(|board| {
            *board.pile_mut(Pile::Tableau(0)) =
                up(&[card(Rank::Two, Suit::Spades), card(Rank::Ace, Suit::Hearts)]);
            *board.pile_mut(Pile::Foundation(0)) = up(&[card(Rank::Ace, Suit::Spades)]);
        });
        assert!(!game.can_move(Pile::Tableau(0), 0, Pile::Foundation(0)));
    }

    #[test]
    fn can_win() {
        let mut deck = new_deck(); // grouped by suit, Ace to King
        deck.reverse();
        let mut game = klondike(|board| *board.pile_mut(Pile::Waste) = up(&deck));
        while move_to_foundation(&mut game, Pile::Waste) {}
        assert!(game.pile(Pile::Waste).is_empty());
        assert!(game.is_won());
    }
}
