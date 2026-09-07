use std::marker::PhantomData;

use bevy::asset::uuid::uuid;
use bevy::prelude::*;
use bevy::render::render_resource::{AsBindGroup, ShaderType};
use bevy::render::storage::ShaderBuffer;
use bevy::shader::ShaderRef;
use bevy_pipe_affect::prelude::*;

#[derive(Copy, Clone, PartialEq, Eq, Default, Debug)]
pub struct PlatformerShadowPlugin;

impl Plugin for PlatformerShadowPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(MaterialPlugin::<PlatformerShadowMaterial>::default())
            .add_systems(
                Update,
                write_caster_info
                    .pipe(affect)
                    .after(TransformSystems::Propagate),
            );
    }
}

const PLATFORMER_SHADOW_CASTER_INFO_BUFFER_HANDLE: Handle<ShaderBuffer> =
    Handle::Uuid(uuid!("c0c74b8a-dd5a-44a8-b5cc-1b9c506d66cd"), PhantomData);

#[derive(Clone, PartialEq, Debug, AsBindGroup, Asset, TypePath)]
pub struct PlatformerShadowMaterial {
    #[storage(0, read_only)]
    casters: Handle<ShaderBuffer>,
    #[texture(1)]
    #[sampler(2)]
    color_texture: Handle<Image>,
}

impl From<Handle<Image>> for PlatformerShadowMaterial {
    fn from(color_texture: Handle<Image>) -> Self {
        PlatformerShadowMaterial {
            casters: PLATFORMER_SHADOW_CASTER_INFO_BUFFER_HANDLE,
            color_texture,
        }
    }
}

const PLATFORMER_SHADOW_SHADER_PATH: &str = "shaders/platformer_shadow.wgsl";

impl Material for PlatformerShadowMaterial {
    fn fragment_shader() -> ShaderRef {
        PLATFORMER_SHADOW_SHADER_PATH.into()
    }
}

#[derive(Copy, Clone, PartialEq, Debug, ShaderType)]
pub struct PlatformerShadowCasterInfo {
    radius: f32,
    translation_xz: Vec2,
}

impl Default for PlatformerShadowCasterInfo {
    fn default() -> Self {
        PlatformerShadowCasterInfo {
            radius: 0.1,
            translation_xz: Vec2::default(),
        }
    }
}

#[derive(Copy, Clone, PartialEq, Default, Debug, Component)]
pub struct PlatformerShadowCaster {
    pub radius: f32,
}

fn write_caster_info(
    casters: Query<(&GlobalTransform, &PlatformerShadowCaster)>,
) -> AssetInsert<ShaderBuffer> {
    let caster_info = casters
        .into_iter()
        .next()
        .map(|(transform, caster)| {
            let radius = caster.radius;
            let translation_xz = transform.translation().xz();

            PlatformerShadowCasterInfo {
                radius,
                translation_xz,
            }
        })
        .unwrap_or_default();

    let shader_buffer = {
        let mut buffer = ShaderBuffer::default();
        buffer.set_data(caster_info);
        buffer
    };

    asset_insert(&PLATFORMER_SHADOW_CASTER_INFO_BUFFER_HANDLE, shader_buffer)
}
