use bevy::prelude::*;

use crate::state::GameState;

/// Substate for the *Clear Skies* game.
#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, Hash, SubStates)]
#[source(GameState = GameState::ClearSkies)]
pub enum ClearSkiesState {
    /// Loading state...
    Loading,
    #[default]
    /// The skybox is being drawn by the camera.
    PaintSkies,
    /// The skybox is being played.
    PlaySkies,
}
