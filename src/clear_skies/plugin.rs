use bevy::prelude::*;

/// Plugin for the Clear Skies game.
#[derive(Debug, Default, Copy, Clone, PartialEq, Eq)]
pub struct ClearSkiesPlugin;

impl Plugin for ClearSkiesPlugin {
    fn build(&self, app: &mut App) {}
}
