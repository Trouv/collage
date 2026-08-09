use bevy::prelude::*;
use clap::ValueEnum;

/// The main state enum of this game.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Hash, States, ValueEnum)]
pub enum GameState {
    /// The *Clear Skies* game.
    #[default]
    ClearSkies,
}
