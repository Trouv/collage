use bevy::prelude::*;

/// The main state enum of this game.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Hash, States)]
pub enum GameState {
    /// The *Clear Skies* game.
    #[default]
    ClearSkies,
}
