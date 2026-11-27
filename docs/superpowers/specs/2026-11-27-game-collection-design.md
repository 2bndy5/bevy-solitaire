# A collection of solitaire games

## Intent

Grow Bevy Solitaire from one game (Klondike) into a collection of classic solitaire games, with
the polish people expect from a casual card games app: a main menu to pick a game, progress
saved per game, best times, and an About screen. The collection keeps this project's own look
(3D cards on a tilted table, generated decks); it is not a copy of any existing app's name, art or
menus.

## Goals

- A main menu listing the games, each with a one-line summary; picking one resumes its saved game,
  if there is one.
- Saved progress for the last game of each type, restored on the next visit (browser local storage
  on the web, a file in the config folder on desktop).
- A Home button on the win screen (and in the HUD), returning to the main menu.
- An About screen: app version, link to the source (https://github.com/2bndy5/bevy-solitaire),
  funding links (GitHub Sponsors https://github.com/sponsors/2bndy5 and PayPal
  https://www.paypal.me/Brendan884, as in `.github/FUNDING.yml`), and credits to the Bevy engine
  and to Brendan (2bndy5).
- Best times: the top 5 per game, shown on the win screen and on a Stats screen off the main menu.
- Games: Klondike (exists), FreeCell, Spider, Golf, TriPeaks, Pyramid.

## Non-goals

- Online features (accounts, leaderboards, daily challenges).
- Saving undo history across sessions (a resumed game starts with an empty undo history).
- Scoring systems beyond moves and time.

## Approaches

- **A. A shared table driven by per-game rules (picked).** Each game is a set of rules over a
  common board of piles: which piles exist and where they sit, what may be picked up, where it may
  be dropped, what a tap does, and when the game is won. Drawing, dragging, animation, undo, the
  shake, auto-finish, saving and the celebration are shared. More refactoring up front; each new
  game afterward is mostly its rules and layout.
- B. One module per game with its own table code. Quickest start, but input, animation and undo
  would be copied per game.
- C. Games described in a data file of rules. Flexible, but far more machinery than six well-known
  games need.

## Design

- `game.rs`: `Pile` (stock, waste, foundation, tableau, cell), `Slot`, a generic `Board` (the piles
  and their cards), `Game` (which game, its board, the move count and undo history) and the
  `Rules` trait. Rules are stateless: they read and change a `Board`. `Game` wraps every move with
  the undo snapshot and move count, so rules never deal with either.
- `games/<name>.rs`: one `Rules` implementation per game, with its unit tests.
- `GameKind`: the list of games, with names, summaries and their rules.
- Layout: each game places its piles on a grid of columns and rows; the shared layout code turns
  that into table positions for landscape and portrait windows, fanning piles that fan.
- Saves: one text entry per game kind, written after every move and when leaving a game, removed
  when the game is won. Stored with the same storage code as the settings.
- Best times: one text entry per game kind holding up to five (seconds, moves) results.
- Screens are new `GameState`s or overlays in `menu.rs`; buttons act on `PointerPress`, as the
  existing ones do.

## Delivery

Stacked pull requests, each targeting the branch it builds on:

1. Foundation: rules interface (Klondike moved onto it), main menu with the game list, Home
   buttons, saved progress per game.
2. About screen and best times (win screen and Stats screen).
3. One pull request per new game: FreeCell, Golf, TriPeaks, Pyramid, Spider.

## Left to the builder

Exact screen layouts and wording, how each game's piles are arranged in portrait and landscape,
and Spider's suit-count option (1, 2 or 4 suits; 1 suit by default).
