use bevy::prelude::*;
use bevy_asset_loader::asset_collection::AssetCollection;
use bevy_pipe_affect::prelude::*;
use thiserror::Error;

use crate::clear_skies::ClearSkiesState;

/// GLTF assets handles should be strong paths.
#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, Error)]
#[error("GLTF assets handles should be strong paths")]
pub struct GltfAssetNotStrongPath;

/// A dummy scene for prototyping..
pub fn spawn_scene(
    assets: Res<ClearSkiesAssetCollection>,
) -> Result<AssetServerLoadAnd<'static, Scene, CommandSpawn<SceneRoot>>, GltfAssetNotStrongPath> {
    Ok(asset_server_load_and(
        GltfAssetLabel::Scene(0).from_asset(assets.cube.path().ok_or(GltfAssetNotStrongPath)?),
        |handle| command_spawn(SceneRoot(handle.clone())),
    ))
}

/// Asset collection for the clear skies scenes.
#[derive(Debug, Default, Clone, PartialEq, Eq, Resource, AssetCollection)]
pub struct ClearSkiesAssetCollection {
    /// Basic cube mash.
    #[asset(path = "models/clear-skies.glb")]
    pub cube: Handle<Gltf>,
}

/// Go to clear skies state when this system runs.
pub fn proceed_to_paint_skies() -> ResSet<NextState<ClearSkiesState>> {
    res_set(NextState::Pending(ClearSkiesState::PaintSkies))
}
