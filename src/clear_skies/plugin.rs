use bevy::prelude::*;
use bevy_asset_loader::prelude::*;
use bevy_pipe_affect::prelude::*;

use crate::clear_skies::camera::{
    ClearSkiesResolution,
    CreateClearSkiesRenderTarget,
    create_clear_skies_render_target,
    letterbox_or_pillarbox_viewport,
    spawn_viewport,
};
use crate::clear_skies::paint_skies::PaintSkiesPlugin;
use crate::clear_skies::play_skies::PlaySkiesPlugin;
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
            .insert_resource(ClearColor(Color::BLACK))
            .add_loading_state(
                LoadingState::new(ClearSkiesState::Loading)
                    .continue_to_state(ClearSkiesState::Setup)
                    .load_collection::<ClearSkiesAssetCollection>(),
            )
            .add_systems(
                OnEnter(ClearSkiesState::Setup),
                (
                    (create_clear_skies_render_target.pipe(affect), ApplyDeferred)
                        .chain()
                        .in_set(CreateClearSkiesRenderTarget),
                    spawn_scene.pipe(affect),
                ),
            )
            .add_systems(
                Startup,
                (|| command_spawn(Camera3d::default())).pipe(affect),
            )
            .add_systems(
                Update,
                (
                    letterbox_or_pillarbox_viewport.pipe(affect),
                    proceed_to_paint_skies
                        .pipe(affect)
                        .run_if(in_state(ClearSkiesState::Setup)),
                    spawn_viewport
                        .pipe(affect)
                        .run_if(in_state(ClearSkiesState::Setup)),
                ),
            );
    }
}
