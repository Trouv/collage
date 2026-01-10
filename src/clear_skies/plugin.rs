use bevy::prelude::*;
use bevy_asset_loader::prelude::*;
use bevy_pipe_affect::prelude::*;

use crate::clear_skies::paint_skies::PaintSkiesPlugin;
use crate::clear_skies::play_skies::PlaySkiesPlugin;
use crate::clear_skies::resolution::ClearSkiesResolution;
use crate::clear_skies::state::ClearSkiesState;
use crate::clear_skies::transition::{
    ClearSkiesAssetCollection,
    proceed_to_paint_skies,
    spawn_scene,
};

/// Plugin for the Clear Skies game.
#[derive(Debug, Default, Copy, Clone, PartialEq, Eq)]
pub struct ClearSkiesPlugin;

impl Plugin for ClearSkiesPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((PaintSkiesPlugin, PlaySkiesPlugin))
            .add_sub_state::<ClearSkiesState>()
            .init_resource::<ClearSkiesResolution>()
            .add_loading_state(
                LoadingState::new(ClearSkiesState::Loading)
                    .continue_to_state(ClearSkiesState::Setup)
                    .load_collection::<ClearSkiesAssetCollection>(),
            )
            .add_systems(OnEnter(ClearSkiesState::Setup), spawn_scene.pipe(affect))
            .add_systems(
                Update,
                proceed_to_paint_skies
                    .pipe(affect)
                    .run_if(in_state(ClearSkiesState::Setup)),
            );
    }
}
