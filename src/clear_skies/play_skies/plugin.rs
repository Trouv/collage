use bevy::prelude::*;
use bevy_pipe_affect::prelude::*;

use crate::clear_skies::ClearSkiesState;
use crate::clear_skies::play_skies::camera::spawn_camera;
use crate::clear_skies::render_layers::PAINTED_LAYER;
use crate::clear_skies::transition::{ClearSkiesAssetCollection, GltfAssetNotStrongPath};

#[derive(Default, Debug, PartialEq, Eq, Copy, Clone, Hash, Reflect)]
pub struct PlaySkiesPlugin;

impl Plugin for PlaySkiesPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(ClearSkiesState::Setup),
            (spawn_camera.pipe(affect), spawn_scene.pipe(affect)),
        );
    }
}

pub fn spawn_scene(
    assets: Res<ClearSkiesAssetCollection>,
) -> Result<impl Effect + use<>, GltfAssetNotStrongPath> {
    Ok((
        asset_server_load_and(
            GltfAssetLabel::Scene(0).from_asset(assets.cube.path().ok_or(GltfAssetNotStrongPath)?),
            |handle| {
                command_spawn((
                    SceneRoot(handle.clone()),
                    Transform::from_xyz(0.0, 0.0, -10.0),
                ))
            },
        ),
        command_spawn(DirectionalLight::default()),
    ))
}
