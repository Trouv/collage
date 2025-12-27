use bevy::prelude::*;
use bevy_asset_loader::asset_collection::AssetCollection;
use bevy_pipe_affect::prelude::*;

pub fn setup() -> impl Effect + use<> {
    (
        command_spawn((
            Camera3d::default(),
            Transform::from_translation(Vec3::splat(10.0)).looking_at(Vec3::ZERO, Vec3::Y),
        )),
        asset_server_load_and(
            GltfAssetLabel::Scene(0).from_asset("models/torus.gltf"),
            |handle| command_spawn(SceneRoot(handle)),
        ),
        command_spawn(DirectionalLight::default()),
    )
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Resource, AssetCollection)]
pub struct ClearSkiesAssetCollection {
    #[asset(path = "models/torus.gltf")]
    torus: Handle<Scene>,
}
