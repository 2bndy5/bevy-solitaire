//! The game model shared by every solitaire game, independent of Bevy so it can be unit tested.
//!
//! A [`Board`] holds the cards in each pile. Each game's [`Rules`] (see [`crate::games`]) say how
//! the board is dealt and which moves are legal, and change the board when a move is made. A
//! [`Game`] ties a board to its rules, and keeps the move count and the undo history, so the rules
//! never deal with either.
use crate::{
    cards::{Card, Rank},
    games::GameKind,
};

/// Identifies a pile on the table. Which piles exist depends on the game.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Pile {
    /// Face-down cards to draw from.
    Stock,
    /// Face-up cards drawn from the stock.
    Waste,
    /// A pile that cards are played onto to win, usually built up by suit from Ace to King.
    Foundation(usize),
    /// A pile in the main playing area.
    Tableau(usize),
    /// A space that holds a single card (FreeCell's free cells).
    Cell(usize),
}

impl Pile {
    /// A short name for the pile, used in saved games (e.g. `t3` for the fourth tableau).
    pub fn key(self) -> String {
        match self {
            Pile::Stock => "stock".to_string(),
            Pile::Waste => "waste".to_string(),
            Pile::Foundation(i) => format!("f{i}"),
            Pile::Tableau(i) => format!("t{i}"),
            Pile::Cell(i) => format!("c{i}"),
        }
    }

    /// Reads a name written by [`Pile::key`].
    pub fn from_key(key: &str) -> Option<Self> {
        match key {
            "stock" => Some(Pile::Stock),
            "waste" => Some(Pile::Waste),
            _ => {
                let (kind, index) = key.split_at_checked(1)?;
                let index = index.parse().ok()?;
                match kind {
                    "f" => Some(Pile::Foundation(index)),
                    "t" => Some(Pile::Tableau(index)),
                    "c" => Some(Pile::Cell(index)),
                    _ => None,
                }
            }
        }
    }
}

/// A card at a specific spot on the table.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Slot {
    pub card: Card,
    pub face_up: bool,
}

impl Slot {
    pub fn up(card: Card) -> Self {
        Self {
            card,
            face_up: true,
        }
    }

    pub fn down(card: Card) -> Self {
        Self {
            card,
            face_up: false,
        }
    }
}

/// The cards in each pile on the table.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Board {
    piles: Vec<(Pile, Vec<Slot>)>,
}

impl Board {
    /// An empty board with the given piles.
    pub fn new(piles: impl IntoIterator<Item = Pile>) -> Self {
        Self {
            piles: piles.into_iter().map(|pile| (pile, Vec::new())).collect(),
        }
    }

    /// The board's piles, in the order they were given to [`Board::new`].
    pub fn piles(&self) -> impl Iterator<Item = Pile> + '_ {
        self.piles.iter().map(|(pile, _)| *pile)
    }

    /// The cards in `pile`, from bottom to top. Empty if the board has no such pile.
    pub fn pile(&self, pile: Pile) -> &[Slot] {
        self.piles
            .iter()
            .find(|(p, _)| *p == pile)
            .map_or(&[], |(_, cards)| cards)
    }

    /// The cards in `pile`.
    ///
    /// # Panics
    ///
    /// If the board has no such pile.
    pub fn pile_mut(&mut self, pile: Pile) -> &mut Vec<Slot> {
        self.piles
            .iter_mut()
            .find(|(p, _)| *p == pile)
            .map(|(_, cards)| cards)
            .unwrap_or_else(|| panic!("no {pile:?} pile on this board"))
    }

    /// Whether the board has `pile`.
    pub fn has(&self, pile: Pile) -> bool {
        self.piles.iter().any(|(p, _)| *p == pile)
    }

    /// The top card of `pile`, if any.
    pub fn top(&self, pile: Pile) -> Option<&Slot> {
        self.pile(pile).last()
    }

    /// Moves the cards from `index` to the top of `from` onto the top of `to`.
    pub fn move_cards(&mut self, from: Pile, index: usize, to: Pile) {
        let moved = self.pile_mut(from).split_off(index);
        self.pile_mut(to).extend(moved);
    }

    /// Turns the top card of `pile` face up, if it isn't already.
    pub fn turn_up_top(&mut self, pile: Pile) {
        if let Some(top) = self.pile_mut(pile).last_mut() {
            top.face_up = true;
        }
    }

    /// Every card on the board, pile by pile.
    pub fn cards(&self) -> impl Iterator<Item = &Slot> {
        self.piles.iter().flat_map(|(_, cards)| cards)
    }
}

/// How a pile's cards are spread out on the table.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Fan {
    /// Stacked on top of each other: only the top card shows.
    Stacked,
    /// Spread down toward the player, so every card shows a strip.
    Down,
}

/// Where a pile sits, on a grid of card-sized columns and rows.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Spot {
    /// Column, counting from 0 at the left.
    pub column: f32,
    /// Row, counting from 0 at the top.
    pub row: f32,
    pub fan: Fan,
    /// How many layers up the pile rests, for piles that overlap others (TriPeaks' rows): a pile
    /// on a higher level is drawn over one on a lower level.
    pub level: u8,
}

impl Spot {
    pub const fn stacked(column: f32, row: f32) -> Self {
        Self {
            column,
            row,
            fan: Fan::Stacked,
            level: 0,
        }
    }

    pub const fn fanned(column: f32, row: f32) -> Self {
        Self {
            column,
            row,
            fan: Fan::Down,
            level: 0,
        }
    }

    /// This spot, raised to `level`.
    pub const fn raised(self, level: u8) -> Self {
        Self { level, ..self }
    }
}

/// The overall shape of a game's table, used to size the layout.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Shape {
    /// How many columns wide the table is.
    pub columns: usize,
    /// The row of the piles that fan down. The table leaves room below it for their fans.
    pub fan_row: f32,
    /// The number of face-down and face-up cards in a fan the landscape layout makes room for.
    /// Longer fans are squeezed to fit.
    pub landscape_fan: (usize, usize),
    /// The same for the portrait layout, which has more height to spare.
    pub portrait_fan: (usize, usize),
}

/// A move to make, as suggested by [`Rules::auto_finish_step`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    /// Draw from the stock (or turn the waste back over).
    Draw,
    /// Move the cards from `index` to the top of `from` onto `to`.
    Move { from: Pile, index: usize, to: Pile },
}

/// The rules of one solitaire game. Rules are stateless: everything about a game in progress is
/// on its [`Board`].
pub trait Rules: Sync {
    /// The game's piles. Pointer hits are tested in this order.
    fn piles(&self) -> Vec<Pile>;

    /// The cards the game is played with, in a standard order.
    fn deck(&self) -> Vec<Card> {
        crate::cards::new_deck()
    }

    /// Deals `deck` (shuffled) onto `board`, which has the game's piles and no cards.
    fn deal(&self, deck: Vec<Card>, board: &mut Board);

    /// The size and shape of the table.
    fn shape(&self) -> Shape;

    /// Where `pile` sits on the table.
    fn spot(&self, pile: Pile) -> Spot;

    /// Draws from the stock, returning whether anything happened.
    fn draw(&self, _board: &mut Board) -> bool {
        false
    }

    /// Can the cards from `index` to the top of `from` be picked up?
    fn can_pick_up(&self, board: &Board, from: Pile, index: usize) -> bool;

    /// Is moving the cards from `index` upward in `from` onto `to` legal?
    fn can_move(&self, board: &Board, from: Pile, index: usize, to: Pile) -> bool;

    /// Makes a move that [`Rules::can_move`] allows.
    fn apply_move(&self, board: &mut Board, from: Pile, index: usize, to: Pile) {
        board.move_cards(from, index, to);
    }

    /// Where a tap on the card at `index` in `from` should send it, if anywhere.
    fn best_move(&self, board: &Board, from: Pile, index: usize) -> Option<Pile>;

    /// Has the game been won?
    fn is_won(&self, board: &Board) -> bool;

    /// Should tapping the card at `index` in `from` select it, to be matched with the next card
    /// tapped (Pyramid's pairs), rather than send it somewhere?
    fn selects_on_tap(&self, _board: &Board, _from: Pile, _index: usize) -> bool {
        false
    }

    /// Is the game over without a win: no move left to make? Games where this is hard to tell
    /// (or never happens) say no.
    fn is_lost(&self, _board: &Board) -> bool {
        false
    }

    /// Is the game certain to be won from here, so it can finish itself?
    fn is_certainly_winnable(&self, _board: &Board) -> bool {
        false
    }

    /// The next move toward finishing a game that is certainly winnable, if any.
    fn auto_finish_step(&self, _board: &Board) -> Option<Action> {
        None
    }
}

/// A game in progress: which game it is, its board, its move count and its undo history.
#[derive(Debug, Clone)]
pub struct Game {
    pub kind: GameKind,
    board: Board,
    /// Number of moves made (draws and card moves).
    pub moves: u32,
    /// The boards and move counts before each move, most recent last, for [`Game::undo`].
    history: Vec<(Board, u32)>,
    /// The board as it was dealt, for [`Game::restart`], if known.
    dealt: Option<Board>,
}

impl Default for Game {
    fn default() -> Self {
        Self::new(GameKind::default())
    }
}

impl Game {
    /// A game of `kind` with no cards on the table.
    pub fn new(kind: GameKind) -> Self {
        Self::with_board(kind, Board::new(kind.rules().piles()))
    }

    /// A game of `kind` with `board` as it is, and no moves made.
    pub fn with_board(kind: GameKind, board: Board) -> Self {
        Self {
            kind,
            board,
            moves: 0,
            history: Vec::new(),
            dealt: None,
        }
    }

    /// Deals a new game of `kind` from `deck` (already shuffled).
    pub fn deal(kind: GameKind, deck: Vec<Card>) -> Self {
        let mut game = Self::new(kind);
        kind.rules().deal(deck, &mut game.board);
        game.dealt = Some(game.board.clone());
        game
    }

    pub fn rules(&self) -> &'static dyn Rules {
        self.kind.rules()
    }

    pub fn board(&self) -> &Board {
        &self.board
    }

    /// The game's piles, in hit-testing order.
    pub fn piles(&self) -> impl Iterator<Item = Pile> + '_ {
        self.board.piles()
    }

    /// The cards in `pile`, from bottom to top.
    pub fn pile(&self, pile: Pile) -> &[Slot] {
        self.board.pile(pile)
    }

    /// Makes a change to the board through `change`, keeping it (as one move that can be undone)
    /// only if `change` says something happened.
    fn record(&mut self, change: impl FnOnce(&Self, &mut Board) -> bool) -> bool {
        let mut board = self.board.clone();
        if !change(self, &mut board) {
            return false;
        }
        let before = std::mem::replace(&mut self.board, board);
        self.history.push((before, self.moves));
        self.moves += 1;
        true
    }

    /// Draws from the stock. Returns `false` if nothing happened.
    pub fn draw(&mut self) -> bool {
        self.record(|game, board| game.rules().draw(board))
    }

    pub fn can_pick_up(&self, from: Pile, index: usize) -> bool {
        index < self.pile(from).len() && self.rules().can_pick_up(&self.board, from, index)
    }

    pub fn can_move(&self, from: Pile, index: usize, to: Pile) -> bool {
        from != to
            && self.board.has(to)
            && self.can_pick_up(from, index)
            && self.rules().can_move(&self.board, from, index, to)
    }

    /// Moves the cards from `index` upward in `from` onto `to`, if legal. Returns whether the move
    /// happened.
    pub fn try_move(&mut self, from: Pile, index: usize, to: Pile) -> bool {
        if !self.can_move(from, index, to) {
            return false;
        }
        self.record(|game, board| {
            game.rules().apply_move(board, from, index, to);
            true
        })
    }

    /// Where a tap on the card at `index` in `from` should send it, if anywhere.
    pub fn best_move(&self, from: Pile, index: usize) -> Option<Pile> {
        if !self.can_pick_up(from, index) {
            return None;
        }
        self.rules()
            .best_move(&self.board, from, index)
            .filter(|&to| self.can_move(from, index, to))
    }

    /// Makes the move [`Game::best_move`] suggests, returning where the cards went.
    pub fn auto_move(&mut self, from: Pile, index: usize) -> Option<Pile> {
        let to = self.best_move(from, index)?;
        self.try_move(from, index, to).then_some(to)
    }

    pub fn is_won(&self) -> bool {
        self.rules().is_won(&self.board)
    }

    /// Is the game over without a win? See [`Rules::is_lost`].
    pub fn is_lost(&self) -> bool {
        !self.is_won() && self.rules().is_lost(&self.board)
    }

    pub fn is_certainly_winnable(&self) -> bool {
        self.rules().is_certainly_winnable(&self.board)
    }

    /// Makes one move toward finishing a game that is certainly winnable. Returns `false` once
    /// there is nothing left to do.
    pub fn auto_finish_step(&mut self) -> bool {
        match self.rules().auto_finish_step(&self.board) {
            Some(Action::Draw) => self.draw(),
            Some(Action::Move { from, index, to }) => self.try_move(from, index, to),
            None => false,
        }
    }

    /// Where `card` is: its pile and its index in the pile.
    pub fn find(&self, card: Card) -> Option<(Pile, usize)> {
        self.piles().find_map(|pile| {
            let index = self.pile(pile).iter().position(|slot| slot.card == card)?;
            Some((pile, index))
        })
    }

    /// Should a tap on this card select it? See [`Rules::selects_on_tap`].
    pub fn selects_on_tap(&self, from: Pile, index: usize) -> bool {
        self.rules().selects_on_tap(&self.board, from, index)
    }

    /// Is there a move to undo?
    pub fn can_undo(&self) -> bool {
        !self.history.is_empty()
    }

    /// Takes back the last move (including turning back over any card it exposed).
    ///
    /// Returns `false` if there was nothing to undo.
    pub fn undo(&mut self) -> bool {
        let Some((board, moves)) = self.history.pop() else {
            return false;
        };
        self.board = board;
        self.moves = moves;
        true
    }

    /// Puts the cards back as they were dealt ("undo all"), as one move that can itself be undone.
    /// The move count starts again from zero.
    ///
    /// Returns `false` if the cards are already as dealt, or the deal isn't known.
    pub fn restart(&mut self) -> bool {
        let Some(start) = self
            .dealt
            .clone()
            .or_else(|| self.history.first().map(|(board, _)| board.clone()))
        else {
            return false;
        };
        if start == self.board {
            return false;
        }
        let before = std::mem::replace(&mut self.board, start);
        self.history.push((before, self.moves));
        self.moves = 0;
        true
    }

    /// The game as text, for saving. See [`Game::from_text`].
    ///
    /// One `key=value` line each for the game, the move count and every pile. A pile's cards are
    /// listed bottom first, each as its [key](Card::key) followed by `+` if it is
    /// face up or `-` if face down.
    pub fn to_text(&self) -> String {
        let mut text = format!("game={}\nmoves={}\n", self.kind.key(), self.moves);
        let mut write_board = |prefix: &str, board: &Board| {
            for pile in board.piles() {
                let cards: Vec<String> = board
                    .pile(pile)
                    .iter()
                    .map(|slot| {
                        let side = if slot.face_up { '+' } else { '-' };
                        format!("{}{side}", slot.card.key())
                    })
                    .collect();
                text.push_str(&format!("{prefix}.{}={}\n", pile.key(), cards.join(" ")));
            }
        };
        write_board("pile", &self.board);
        if let Some(dealt) = &self.dealt {
            write_board("dealt", dealt);
        }
        text
    }

    /// Reads a game saved by [`Game::to_text`], or `None` if it isn't a complete, valid game.
    pub fn from_text(text: &str) -> Option<Self> {
        let values = || {
            text.lines()
                .filter_map(|line| line.split_once('='))
                .map(|(key, value)| (key.trim(), value.trim()))
        };
        let kind = values()
            .find(|(key, _)| *key == "game")
            .and_then(|(_, value)| GameKind::from_key(value))?;
        let mut game = Self::new(kind);
        let mut dealt = Board::new(kind.rules().piles());
        let mut has_dealt = false;
        for (key, value) in values() {
            if key == "moves" {
                game.moves = value.parse().ok()?;
            } else if let Some(pile) = key.strip_prefix("pile.") {
                read_pile(&mut game.board, pile, value)?;
            } else if let Some(pile) = key.strip_prefix("dealt.") {
                // An unreadable deal only loses "undo all", not the game.
                has_dealt |= read_pile(&mut dealt, pile, value).is_some();
            }
        }
        let deck = game.rules().deck();
        game.dealt = (has_dealt && holds_deck(&dealt, &deck)).then_some(dealt);
        holds_deck(&game.board, &deck).then_some(game)
    }
}

/// Reads a saved pile's cards (see [`Game::to_text`]) into `board`.
fn read_pile(board: &mut Board, pile: &str, value: &str) -> Option<()> {
    let pile = Pile::from_key(pile).filter(|&pile| board.has(pile))?;
    let cards = value
        .split_whitespace()
        .map(|word| {
            let (stem, side) = word.split_at_checked(word.len().checked_sub(1)?)?;
            let card = Card::from_key(stem)?;
            match side {
                "+" => Some(Slot::up(card)),
                "-" => Some(Slot::down(card)),
                _ => None,
            }
        })
        .collect::<Option<Vec<_>>>()?;
    *board.pile_mut(pile) = cards;
    Some(())
}

/// Is every card of `deck` somewhere on `board`, exactly once?
fn holds_deck(board: &Board, deck: &[Card]) -> bool {
    let mut cards: Vec<Card> = board.cards().map(|slot| slot.card).collect();
    let mut deck = deck.to_vec();
    cards.sort();
    deck.sort();
    cards == deck
}

/// Does `card` go directly on `top` when building up by suit (an Ace on nothing)?
pub fn builds_up_by_suit(top: Option<&Slot>, card: Card) -> bool {
    match top {
        None => card.rank == Rank::Ace,
        Some(top) => top.card.suit == card.suit && top.card.rank.as_u8() + 1 == card.rank.as_u8(),
    }
}

/// Does `card` go directly on `top` when building down in alternating colors?
pub fn builds_down_alternating(top: &Slot, card: Card) -> bool {
    top.face_up
        && top.card.is_red() != card.is_red()
        && card.rank.as_u8() + 1 == top.card.rank.as_u8()
}

/// Are `a` and `b` one rank apart (either way)? With `wrap`, Kings and Aces count as one apart.
pub fn ranks_adjacent(a: Rank, b: Rank, wrap: bool) -> bool {
    let difference = a.as_u8().abs_diff(b.as_u8());
    difference == 1 || (wrap && difference == 12)
}

/// Are four foundations each full, from Ace to King?
pub fn foundations_full(board: &Board, foundations: usize) -> bool {
    (0..foundations).all(|i| board.pile(Pile::Foundation(i)).len() == Rank::ALL.len())
}

#[cfg(test)]
pub fn card(rank: Rank, suit: crate::cards::Suit) -> Card {
    Card::new(rank, suit)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cards::{Suit, new_deck, shuffled};

    #[test]
    fn adjacent_ranks() {
        assert!(ranks_adjacent(Rank::Five, Rank::Six, false));
        assert!(ranks_adjacent(Rank::Six, Rank::Five, false));
        assert!(!ranks_adjacent(Rank::Six, Rank::Six, false));
        assert!(!ranks_adjacent(Rank::King, Rank::Ace, false));
        assert!(ranks_adjacent(Rank::King, Rank::Ace, true));
        assert!(ranks_adjacent(Rank::Ace, Rank::King, true));
        assert!(!ranks_adjacent(Rank::Queen, Rank::Ace, true));
    }

    #[test]
    fn pile_keys_round_trip() {
        for pile in [
            Pile::Stock,
            Pile::Waste,
            Pile::Foundation(3),
            Pile::Tableau(12),
            Pile::Cell(0),
        ] {
            assert_eq!(Pile::from_key(&pile.key()), Some(pile));
        }
        assert_eq!(Pile::from_key("x1"), None);
        assert_eq!(Pile::from_key("t"), None);
        assert_eq!(Pile::from_key(""), None);
    }

    #[test]
    fn board_moves_cards() {
        let mut board = Board::new([Pile::Tableau(0), Pile::Tableau(1)]);
        *board.pile_mut(Pile::Tableau(0)) = vec![
            Slot::down(card(Rank::Four, Suit::Clubs)),
            Slot::up(card(Rank::Nine, Suit::Spades)),
            Slot::up(card(Rank::Eight, Suit::Hearts)),
        ];
        board.move_cards(Pile::Tableau(0), 1, Pile::Tableau(1));
        assert_eq!(board.pile(Pile::Tableau(0)).len(), 1);
        assert_eq!(board.pile(Pile::Tableau(1)).len(), 2);
        board.turn_up_top(Pile::Tableau(0));
        assert!(board.top(Pile::Tableau(0)).unwrap().face_up);
        assert!(
            board.pile(Pile::Stock).is_empty(),
            "missing piles are empty"
        );
        assert!(!board.has(Pile::Stock));
    }

    #[test]
    fn saved_games_round_trip() {
        for kind in GameKind::ALL {
            let mut game = Game::deal(kind, shuffled(kind.rules().deck()));
            game.draw();
            let restored = Game::from_text(&game.to_text()).expect("a valid save");
            assert_eq!(restored.kind, kind);
            assert_eq!(restored.moves, game.moves);
            assert_eq!(restored.board, game.board);
            assert!(!restored.can_undo(), "undo history isn't saved");
        }
    }

    #[test]
    fn broken_saves_are_rejected() {
        let game = Game::deal(GameKind::Klondike, new_deck());
        let text = game.to_text();
        assert!(Game::from_text("").is_none());
        assert!(Game::from_text(&text.replace("game=klondike", "game=nope")).is_none());
        // A card missing.
        assert!(Game::from_text(&text.replacen("13s+", "", 1)).is_none());
        // A card twice.
        assert!(Game::from_text(&text.replacen("13s+", "13s+ 13s+", 1)).is_none());
        // A pile the game doesn't have.
        assert!(Game::from_text(&format!("{text}pile.c0=\n")).is_none());
        assert!(Game::from_text(&text.replacen("moves=0", "moves=lots", 1)).is_none());
    }

    #[test]
    fn restart_goes_back_to_the_deal_and_can_be_undone() {
        let mut game = Game::deal(GameKind::Klondike, new_deck());
        let dealt = game.board.clone();
        assert!(!game.restart(), "nothing to undo yet");
        for _ in 0..5 {
            game.draw();
        }
        let played = game.board.clone();
        assert!(game.restart());
        assert_eq!(game.board, dealt);
        assert_eq!(game.moves, 0);
        assert!(game.undo());
        assert_eq!(game.board, played);
        assert_eq!(game.moves, 5);

        // A saved game remembers its deal.
        let mut restored = Game::from_text(&game.to_text()).unwrap();
        assert!(!restored.can_undo());
        assert!(restored.restart());
        assert_eq!(restored.board, dealt);
        // An old save, without the deal, can't restart without history.
        let old: String = game
            .to_text()
            .lines()
            .filter(|line| !line.starts_with("dealt."))
            .map(|line| format!("{line}\n"))
            .collect();
        assert!(!Game::from_text(&old).unwrap().restart());
    }

    #[test]
    fn failed_moves_are_not_recorded() {
        let mut game = Game::new(GameKind::Klondike);
        assert!(!game.draw());
        assert!(!game.try_move(Pile::Waste, 0, Pile::Foundation(0)));
        assert!(!game.can_undo());
        assert_eq!(game.moves, 0);
    }
}
