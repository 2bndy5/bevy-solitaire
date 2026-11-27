# Bevy Solitaire

A collection of solitaire games made with [Bevy], played with 3D cards on a gently tilted table.
It has Klondike (draw one), FreeCell, Golf, TriPeaks, Pyramid and Spider (with one, two or four
suits).

It grew out of [bevy-52-card-pickup], reusing its free
[asset pack of hand-drawn cards][card-assets-link], card model, menus and build setup.

Besides those hand-drawn cards, the settings offer four more decks (Modern and Large print, each
light or dark) and four more card backs. They're drawn by [`tools/generate_decks.py`](tools/generate_decks.py)
(Python with Pillow) from vector shapes; re-run it from the repository root after changing it.

## How to play

Pick a game from the main menu. Your last game of each kind is saved as you play, so picking that
game again carries on where you left off (the menu shows its moves and time). Press the house
button in the top bar to return to the main menu at any time.

Press the question mark button in the top bar for how to play the current game; it also pops up
the first time you play each game. In games that can get stuck (like Golf), a banner at the bottom
tells you when no moves are left. Spider's tile on the main menu opens a choice of one, two or four
suits.

Winning a game shows your five best times for that game; the main menu's "Best Times" button shows
them for every game, with how many games you've won. "About" has the version, credits, a link to
the source code and ways to support the project.

### Klondike

- Build the four foundations (top right) up by suit, from Ace to King.
- Build the seven tableau piles down in alternating colors (red on black, black on red).
  Only a King may be placed on an empty tableau pile.
- Click the stock (top left) to draw a card. When the stock is empty, click it to turn the waste
  pile back over.
- Drag a card (and any cards on top of it) to move it.
- Click (or tap) a card to move it, and any cards on top of it, to the first place it fits: a
  foundation if it can go there, otherwise the next tableau column (to the right, wrapping around)
  that takes it. Click again to move it on to the next option. If it has nowhere to go, it shakes.
- Press "Undo" (or Ctrl+Z, Cmd+Z or U) to take back moves, one at a time.
- Once the stock is empty and every tableau card is face up, the game finishes itself.
- The clock starts on your first move.
- Press "New Game" at any time to deal again (this abandons the game in play).
- Press the cog to choose the card art, table color and lighting.

### FreeCell

- Every card is dealt face up into eight columns.
- Build the four foundations (top right) up by suit, from Ace to King.
- Build the columns down in alternating colors. Any card may go in an empty column.
- The four free cells (top left) each hold one card.
- Only one card really moves at a time, so how long a run you can move at once depends on the room
  to shuffle it through: one card per empty free cell, plus one, doubled for each empty column.
- Tap a card to send it to a foundation, else the next column that takes it, else an empty column,
  else a free cell.
- Once every column runs downward in rank, the game finishes itself.

### Golf

- Seven columns of five face-up cards, a stock, and one card face up on the waste.
- Tap (or drag) the top card of a column onto the waste if it is one rank above or below the waste's
  top card, regardless of suit. Kings and Aces don't wrap around.
- Tap the stock to turn its next card onto the waste. The stock is only gone through once.
- Clear all seven columns to win; cards left in the stock don't matter.

### TriPeaks

- Three peaks of overlapping cards, face down except the bottom row, with a stock and one card face
  up on the waste.
- Tap (or drag) an uncovered card onto the waste if it is one rank above or below the waste's top
  card, regardless of suit. Kings and Aces connect.
- Cards turn face up once nothing covers them.
- Tap the stock to turn its next card onto the waste. The stock is only gone through once.
- Clear all three peaks to win.

### Pyramid

- A pyramid of 28 face-up cards, each row covering the bottom of the row behind it, and a stock.
- Remove pairs of uncovered cards that add up to 13 (Jack = 11, Queen = 12, Ace = 1) by dragging
  one onto the other. The top card of the waste counts as uncovered.
- Kings are removed on their own.
- Tap a card to remove it: a King on its own, any other card with the first uncovered card that makes
  13 with it.
- Tap the stock to turn its next card onto the waste; once it's empty, tap it again to turn the
  waste back over.
- Clear the pyramid to win.

### Spider

- Played with 104 cards (two decks' worth) in one, two or four suits; fewer suits is easier. Each
  suit count has its own saved game and best times.
- Ten columns: the first four get six cards, the rest five, with only the top card face up. The
  other 50 cards form the stock.
- Any card goes on a card one rank higher, whatever its suit; any card goes in an empty column.
- Only runs of one suit, going down a rank at a time, can be moved together.
- A complete run from King to Ace in one suit leaves the table for a foundation (top left).
- Tap the stock (top right) to deal a face-up card onto every column. As in the classic game, you
  can't deal while a column is empty.
- Tap a run to move it onto the next column with a card one rank higher, preferring one of the same
  suit; failing that, into an empty column.
- Clear all eight runs to win.

Your settings and saved games are kept in the browser's local storage on the web, or in a
`bevy-solitaire` folder in your config folder on desktop.

## Try it

Build it from source:

1. Clone this repo.

   ```shell
   git clone https://github.com/2bndy5/bevy-solitaire
   ```

2. Navigate to the repo folder.

   ```shell
   cd bevy-solitaire
   ```

3. Build and run the game.

   ```shell
   cargo run
   ```

Run the tests for the game rules with `cargo test`.

## Code layout

| File            | Purpose                                                                 |
|-----------------|-------------------------------------------------------------------------|
| `src/cards.rs`  | Cards, ranks, suits and decks.                                          |
| `src/game.rs`   | The game model shared by every game: piles, the board, moves, undo and saving (unit tested). |
| `src/games/`    | Each game's rules (unit tested), and the list of games.                 |
| `src/layout.rs` | Where piles and cards sit on the table, and pointer hit-testing.        |
| `src/scene.rs`  | The 3D scene: tilted camera, lighting and the felt table.               |
| `src/table.rs`  | Bevy systems: dealing, card animation, dragging and the HUD.            |
| `src/hud.rs`    | Move count, game clock, and the Home, Settings, Undo and New Game buttons. |
| `src/celebrate.rs` | The win celebration.                                                 |
| `src/menu.rs`   | The main menu, and the win, best times and about screens.              |
| `src/progress.rs` | Saving and resuming the last game of each kind.                       |
| `src/stats.rs`  | Games won and the five best times for each game.                        |
| `src/links.rs`  | Buttons that open web pages.                                            |
| `src/storage.rs` | Saving text in local storage (web) or the config folder (desktop).     |
| `src/settings.rs` | Settings (card art, table color, lighting), saving them, and their panel. |

[Bevy]: https://bevy.org
[bevy-52-card-pickup]: https://github.com/2bndy5/bevy-52-card-pickup
[card-assets-link]: https://hazmat-game-studios.itch.io/hand-drawn-playing-cards
