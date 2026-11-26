# Bevy Solitaire

A basic game of Klondike solitaire (draw one) made with [Bevy], played with 3D cards on a gently
tilted table.

It grew out of [bevy-52-card-pickup], reusing its free
[asset pack of hand-drawn cards][card-assets-link], card model, menus and build setup.

## How to play

- Build the four foundations (top right) up by suit, from Ace to King.
- Build the seven tableau piles down in alternating colors (red on black, black on red).
  Only a King may be placed on an empty tableau pile.
- Click the stock (top left) to draw a card. When the stock is empty, click it to turn the waste
  pile back over.
- Drag a card (and any cards on top of it) to move it.
- Double-click a card to send it to a foundation.
- Press "New Game" at any time to deal again.

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
| `src/game.rs`   | Klondike rules, independent of Bevy (unit tested).                      |
| `src/layout.rs` | Where piles and cards sit on the table, and pointer hit-testing.        |
| `src/scene.rs`  | The 3D scene: tilted camera, lighting and the felt table.               |
| `src/table.rs`  | Bevy systems: dealing, card animation, dragging and the HUD.            |
| `src/menu.rs`   | Start and win screens.                                                  |

[Bevy]: https://bevy.org
[bevy-52-card-pickup]: https://github.com/2bndy5/bevy-52-card-pickup
[card-assets-link]: https://hazmat-game-studios.itch.io/hand-drawn-playing-cards
