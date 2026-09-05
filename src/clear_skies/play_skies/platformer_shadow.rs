use std::marker::PhantomData;

use bevy::asset::uuid::uuid;
use bevy::prelude::*;
use bevy::render::render_resource::{AsBindGroup, ShaderType};
use bevy::render::storage::ShaderBuffer;
use bevy_pipe_affect::prelude::*;

#[derive(Copy, Clone, PartialEq, Eq, Default, Debug)]
pub struct PlatformerShadowPlugin;

impl Plugin for PlatformerShadowPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            PostUpdate,
            write_caster_info
                .pipe(affect)
                .after(TransformSystems::Propagate),
        );
    }
}

const PLATFORMER_SHADOW_CASTER_INFO_BUFFER_HANDLE: Handle<ShaderBuffer> =
    Handle::Uuid(uuid!("c0c74b8a-dd5a-44a8-b5cc-1b9c506d66cd"), PhantomData);

#[derive(Clone, PartialEq, Default, Debug, AsBindGroup)]
pub struct PlatformerShadowMaterial {
    #[storage(64, read_only)]
    pub casters: Handle<ShaderBuffer>,
}

#[derive(Copy, Clone, PartialEq, Default, Debug, ShaderType)]
pub struct PlatformerShadowCasterInfo {
    radius: f32,
    translation_xz: Vec2,
}

#[derive(Copy, Clone, PartialEq, Default, Debug, Component)]
pub struct PlatformerShadowCaster {
    radius: f32,
}

fn write_caster_info(
    casters: Query<(&GlobalTransform, &PlatformerShadowCaster)>,
) -> AssetInsert<ShaderBuffer> {
    let caster_infos = casters
        .into_iter()
        .map(|(transform, caster)| {
            let radius = caster.radius;
            let translation_xz = transform.translation().xz();

            PlatformerShadowCasterInfo {
                radius,
                translation_xz,
            }
        })
        .collect::<Vec<_>>();

    let shader_buffer = {
        let mut buffer = ShaderBuffer::default();
        buffer.set_data(caster_infos);
        buffer
    };

    asset_insert(&PLATFORMER_SHADOW_CASTER_INFO_BUFFER_HANDLE, shader_buffer)
}
