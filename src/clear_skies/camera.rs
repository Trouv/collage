use bevy::prelude::*;
use bevy::render::render_resource::TextureFormat;
use bevy_pipe_affect::prelude::{command_insert_resource, *};

use crate::clear_skies::ClearSkiesState;
use crate::clear_skies::paint_skies::assets_add_and;

#[derive(Default, Debug, PartialEq, Eq, Copy, Clone, Hash, Reflect)]
pub struct ClearSkiesCameraPlugin;

impl Plugin for ClearSkiesCameraPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ClearSkiesResolution>()
            .insert_resource(ClearColor(Color::BLACK))
            .add_systems(
                OnEnter(ClearSkiesState::Setup),
                (
                    (create_clear_skies_render_target.pipe(affect), ApplyDeferred)
                        .chain()
                        .in_set(CreateClearSkiesRenderTarget),
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
                    spawn_viewport
                        .pipe(affect)
                        .run_if(in_state(ClearSkiesState::Setup)),
                ),
            );
    }
}

#[derive(Debug, PartialEq, Copy, Clone, Deref, DerefMut, Reflect, Resource)]
pub struct ClearSkiesResolution(UVec2);

impl Default for ClearSkiesResolution {
    fn default() -> Self {
        ClearSkiesResolution(UVec2::new(480, 360))
    }
}

/// The render target that will be created with a resolution of [`ClearSkiesResolution`].
#[derive(Default, Debug, PartialEq, Eq, Clone, Hash, Resource, Deref, DerefMut, Reflect)]
pub struct ClearSkiesRenderTarget(Handle<Image>);

#[derive(Default, Debug, PartialEq, Eq, Copy, Clone, Hash, Reflect, SystemSet)]
pub struct CreateClearSkiesRenderTarget;

pub fn create_clear_skies_render_target(
    resolution: Res<ClearSkiesResolution>,
) -> impl Effect + use<> {
    let image =
        Image::new_target_texture(resolution.x, resolution.y, TextureFormat::bevy_default());

    assets_add_and(image, |handle| {
        command_insert_resource(ClearSkiesRenderTarget(handle))
    })
}

#[derive(Default, Debug, PartialEq, Eq, Copy, Clone, Hash, Reflect, Component)]
struct ClearSkiesViewport;

pub fn spawn_viewport(
    resolution: Res<ClearSkiesResolution>,
    texture: Res<ClearSkiesRenderTarget>,
) -> impl Effect + use<> {
    command_spawn((
        ImageNode::new((**texture).clone()),
        ClearSkiesViewport,
        Node {
            aspect_ratio: Some(resolution.x as f32 / resolution.y as f32),
            align_self: AlignSelf::Center,
            justify_self: JustifySelf::Center,
            ..default()
        },
    ))
}

pub fn letterbox_or_pillarbox_viewport(
    window: Single<&Window>,
    resolution: Res<ClearSkiesResolution>,
) -> impl Effect + use<> {
    let window_aspect_ratio = window.width() / window.height();
    let target_aspect_ratio = resolution.x as f32 / resolution.y as f32;

    components_set_filtered_with::<_, _, With<ClearSkiesViewport>>(move |(node,): (Node,)| {
        if window_aspect_ratio > target_aspect_ratio {
            (Node {
                width: Val::Auto,
                height: percent(100),
                ..node
            },)
        } else {
            (Node {
                width: percent(100),
                height: Val::Auto,
                ..node
            },)
        }
    })
}
