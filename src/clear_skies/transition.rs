use std::f32::consts::PI;

use bevy::prelude::*;
use bevy_asset_loader::asset_collection::AssetCollection;
use bevy_pipe_affect::prelude::*;
use thiserror::Error;

use crate::clear_skies::ClearSkiesState;
use crate::clear_skies::paint_skies::Paintable;

/// GLTF assets handles should be strong paths.
#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, Error)]
#[error("GLTF assets handles should be strong paths")]
pub struct GltfAssetNotStrongPath;

/// A dummy scene for prototyping..
pub fn spawn_scene(
    assets: Res<ClearSkiesAssetCollection>,
) -> Result<impl Effect + use<>, GltfAssetNotStrongPath> {
    Ok((asset_server_load_and(
        GltfAssetLabel::Scene(0).from_asset(assets.cube.path().ok_or(GltfAssetNotStrongPath)?),
        |handle| {
            (0..2)
                .map(|cube_num| {
                    command_spawn((
                        SceneRoot(handle.clone()),
                        Transform::from_xyz(10.0, cube_num as f32, 3.0 * (cube_num - 1) as f32)
                            .with_rotation(Quat::from_axis_angle(Vec3::Z, PI)),
                        Paintable,
                    ))
                })
                .collect::<Vec<_>>()
        },
    ),))
}

/// Asset collection for the clear skies scenes.
#[derive(Debug, Default, Clone, PartialEq, Eq, Resource, AssetCollection)]
pub struct ClearSkiesAssetCollection {
    /// Basic cube mash.
    #[asset(path = "models/cube.glb")]
    pub cube: Handle<Gltf>,
}

/// Go to clear skies state when this system runs.
pub fn proceed_to_paint_skies() -> ResSet<NextState<ClearSkiesState>> {
    res_set(NextState::Pending(ClearSkiesState::PaintSkies))
}
