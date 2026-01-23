use bevy::prelude::*;

use crate::state::GameState;

/// Substate for the *Clear Skies* game.
#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, Hash, SubStates)]
#[source(GameState = GameState::ClearSkies)]
pub enum ClearSkiesState {
    #[default]
    /// Loading state...
    Loading,
    /// Temporary state for systems that need to set up the PaintSkies/PlaySkies duology..
    Setup,
    /// The skybox is being drawn by the camera.
    PaintSkies,
}
