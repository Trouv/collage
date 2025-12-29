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
        asset_server_load_and(
            GltfAssetLabel::Scene(0).from_asset(assets.cube.path().ok_or(GltfAssetNotStrongPath)?),
            |handle| {
                (0..10)
                    .map(|cube_num| {
                        command_spawn((
                            SceneRoot(handle.clone()),
                            Transform::from_xyz(10.0, cube_num as f32, 3.0 * (cube_num - 5) as f32),
                        ))
                    })
                    .collect::<Vec<_>>()
            },
        ),
        command_spawn(DirectionalLight::default()),
    ))
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Resource, AssetCollection)]
pub struct ClearSkiesAssetCollection {
    #[asset(path = "models/cube.glb")]
    cube: Handle<Gltf>,
}
