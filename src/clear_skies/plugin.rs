use bevy::prelude::*;
use bevy_asset_loader::prelude::*;
use bevy_pipe_affect::prelude::*;

use crate::clear_skies::state::ClearSkiesState;
use crate::clear_skies::transition::{ClearSkiesAssetCollection, setup};

/// Plugin for the Clear Skies game.
#[derive(Debug, Default, Copy, Clone, PartialEq, Eq)]
pub struct ClearSkiesPlugin;

impl Plugin for ClearSkiesPlugin {
    fn build(&self, app: &mut App) {
        app.add_sub_state::<ClearSkiesState>()
            .add_loading_state(
                LoadingState::new(ClearSkiesState::Loading)
                    .continue_to_state(ClearSkiesState::PaintSkies)
                    .load_collection::<ClearSkiesAssetCollection>(),
            )
            .add_systems(OnEnter(ClearSkiesState::PaintSkies), setup.pipe(affect));
    }
}
