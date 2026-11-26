use bevy::prelude::*;

/// The different states of the game.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, States)]
pub enum GameState {
    /// World is being spawned.
    #[default]
    Loading,
    /// Main menu is being displayed.
    Menu,
    /// Cards are being dealt.
    Deal,
    /// Game is in progress.
    Play,
    /// Win screen is being displayed.
    Win,
}
