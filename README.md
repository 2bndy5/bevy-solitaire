# Bevy Solitaire

A collection of solitaire games made with [Bevy], played with 3D cards on a gently tilted table.
So far it has Klondike (draw one); more games are on the way.

It grew out of [bevy-52-card-pickup], reusing its free
[asset pack of hand-drawn cards][card-assets-link], card model, menus and build setup.

Besides those hand-drawn cards, the settings offer four more decks (Modern and Large print, each
light or dark) and four more card backs. They're drawn by [`tools/generate_decks.py`](tools/generate_decks.py)
(Python with Pillow) from vector shapes; re-run it from the repository root after changing it.

## How to play

Pick a game from the main menu. Your last game of each kind is saved as you play, so picking that
game again carries on where you left off (the menu shows its moves and time). Press the house
button in the top bar to return to the main menu at any time.

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
