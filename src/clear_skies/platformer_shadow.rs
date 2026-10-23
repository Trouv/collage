use std::f32::INFINITY;
use std::marker::PhantomData;

use avian3d::spatial_query::RayHits;
use bevy::asset::uuid::uuid;
use bevy::prelude::*;
use bevy::render::render_resource::{AsBindGroup, ShaderType};
use bevy::render::storage::ShaderBuffer;
use bevy::shader::ShaderRef;
use bevy_pipe_affect::prelude::*;
use bytemuck::NoUninit;

/// Plugin that adds [`PlatformerShadowMaterial`] and requisite systems.
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

/// Material that should be placed on entities that platformer shadows can be cast onto.
#[derive(Clone, PartialEq, Debug, AsBindGroup, Asset, TypePath)]
pub struct PlatformerShadowMaterial {
    #[storage(0, read_only)]
    caster: Handle<ShaderBuffer>,
    #[texture(1)]
    #[sampler(2)]
    color_texture: Handle<Image>,
}

impl From<Handle<Image>> for PlatformerShadowMaterial {
    fn from(color_texture: Handle<Image>) -> Self {
        PlatformerShadowMaterial {
            caster: PLATFORMER_SHADOW_CASTER_INFO_BUFFER_HANDLE,
            color_texture,
        }
    }
}

const PLATFORMER_SHADOW_SHADER_PATH: &str = "shaders/platformer_shadow.wesl";

impl Material for PlatformerShadowMaterial {
    fn fragment_shader() -> ShaderRef {
        PLATFORMER_SHADOW_SHADER_PATH.into()
    }
}

#[derive(Copy, Clone, PartialEq, Debug, ShaderType, NoUninit)]
#[repr(C)]
struct PlatformerShadowCasterInfo {
    translation: Vec3,
    radius: f32,
    normal: Vec3,
    ground_distance: f32,
}

impl Default for PlatformerShadowCasterInfo {
    fn default() -> Self {
        PlatformerShadowCasterInfo {
            translation: Vec3::default(),
            radius: 0.1,
            normal: Vec3::Y,
            ground_distance: 0.0,
        }
    }
}

/// Component for entity that should cast platformer shadows.
#[derive(Copy, Clone, PartialEq, Default, Debug, Component)]
pub struct PlatformerShadowCaster {
    /// Radius of the shadow.
    pub radius: f32,
}

fn write_caster_info(
    casters: Query<(&GlobalTransform, &PlatformerShadowCaster, &RayHits)>,
) -> AssetInsert<ShaderBuffer> {
    let caster_info = casters
        .into_iter()
        .next()
        .map(|(transform, caster, ray_hits)| {
            let radius = caster.radius;
            let translation = transform.translation();
            let (ground_distance, normal) = ray_hits
                .iter_sorted()
                .next()
                .map(|ray_hit| (ray_hit.distance, ray_hit.normal))
                .unwrap_or((INFINITY, Vec3::Y));

            PlatformerShadowCasterInfo {
                translation,
                radius,
                normal,
                ground_distance,
            }
        })
        .unwrap_or_default();

    let shader_buffer = ShaderBuffer::from(vec![caster_info]);

    asset_insert(&PLATFORMER_SHADOW_CASTER_INFO_BUFFER_HANDLE, shader_buffer)
}
