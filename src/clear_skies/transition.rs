use bevy::prelude::*;
use bevy_asset_loader::asset_collection::AssetCollection;
use bevy_pipe_affect::prelude::*;
use thiserror::Error;

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, Error)]
#[error("GLTF assets handles should be strong paths")]
pub struct GltfAssetNotStrongPath;

pub fn setup(
    assets: Res<ClearSkiesAssetCollection>,
) -> Result<impl Effect + use<>, GltfAssetNotStrongPath> {
    Ok((
        command_spawn((
            Camera3d::default(),
            Transform::from_translation(Vec3::splat(10.0)).looking_at(Vec3::ZERO, Vec3::Y),
        )),
        asset_server_load_and(
            GltfAssetLabel::Scene(0).from_asset(assets.torus.path().ok_or(GltfAssetNotStrongPath)?),
            |handle| command_spawn(SceneRoot(handle)),
        ),
        command_spawn(DirectionalLight::default()),
    ))
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Resource, AssetCollection)]
pub struct ClearSkiesAssetCollection {
    #[asset(path = "models/torus.gltf")]
    torus: Handle<Gltf>,
}
