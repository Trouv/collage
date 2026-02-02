use std::f32::consts::PI;

use bevy::asset::RenderAssetUsages;
use bevy::camera::{CameraMainTextureUsages, RenderTarget};
use bevy::prelude::*;
use bevy::render::render_resource::{TextureFormat, TextureUsages};
use bevy_pipe_affect::prelude::{command_insert_resource, *};
use leafwing_input_manager::prelude::*;

use crate::clear_skies::ClearSkiesState;
use crate::clear_skies::paint_skies::{
    LookAtSphericalCoords,
    Paintable,
    SphericalCoordsBounds,
    assets_add_and,
};

/// Plugin defining camera setup and logic for clear skies.
#[derive(Default, Debug, PartialEq, Eq, Copy, Clone, Hash, Reflect)]
pub struct ClearSkiesCameraPlugin;

impl Plugin for ClearSkiesCameraPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ClearSkiesResolution>()
            .insert_resource(ClearColor(Color::BLACK))
            .register_type::<ClearSkiesRenderTarget>()
            .add_systems(
                OnEnter(ClearSkiesState::Setup),
                (
                    create_clear_skies_render_target.pipe(affect),
                    ApplyDeferred,
                    spawn_paint_skies_camera.pipe(affect),
                )
                    .chain()
                    .in_set(CreateClearSkiesRenderTarget),
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

/// Resource defining the actual pixel resolution of clear skies.
#[derive(Debug, PartialEq, Copy, Clone, Deref, DerefMut, Reflect, Resource)]
pub struct ClearSkiesResolution(UVec2);

impl Default for ClearSkiesResolution {
    fn default() -> Self {
        ClearSkiesResolution(UVec2::new(480, 360))
    }
}

/// The render target that will be created with a resolution of [`ClearSkiesResolution`].
#[derive(Default, Debug, PartialEq, Eq, Clone, Hash, Resource, Deref, DerefMut, Reflect)]
#[reflect(Resource)]
pub struct ClearSkiesRenderTarget(pub Handle<Image>);

/// System set that creates the texture in [`ClearSkiesRenderTarget`].
#[derive(Default, Debug, PartialEq, Eq, Copy, Clone, Hash, Reflect, SystemSet)]
pub struct CreateClearSkiesRenderTarget;

/// System that creates [`ClearSkiesRenderTarget`].
pub fn create_clear_skies_render_target(
    resolution: Res<ClearSkiesResolution>,
) -> impl Effect + use<> {
    let mut image = Image::new_target_texture(
        resolution.x,
        resolution.y,
        TextureFormat::bevy_default(),
        None,
    );

    image.asset_usage = RenderAssetUsages::all();

    image.texture_descriptor.usage |= TextureUsages::COPY_SRC
        | TextureUsages::COPY_DST
        | TextureUsages::TEXTURE_BINDING
        | TextureUsages::RENDER_ATTACHMENT;

    assets_add_and(image, |handle| {
        command_insert_resource(ClearSkiesRenderTarget(handle))
    })
}

/// Actions for controlling the paint skies camera.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Reflect, Actionlike)]
pub enum PaintSkiesAction {
    /// Dual axis input for rotating the camera.
    #[actionlike(DualAxis)]
    Rotate,
}

/// Defines the paint skies camera.
pub fn spawn_paint_skies_camera(render_target: Res<ClearSkiesRenderTarget>) -> impl Effect + use<> {
    let input_map = InputMap::default()
        .with_dual_axis(
            PaintSkiesAction::Rotate,
            GamepadStick::LEFT.with_deadzone_symmetric(0.1),
        )
        .with_dual_axis(
            PaintSkiesAction::Rotate,
            MouseMove::default().sensitivity(0.15).inverted_y(),
        );

    command_spawn((
        input_map,
        Camera3d::default(),
        PaintSkiesCamera,
        SphericalCoordsBounds {
            max_phi: 3.0 * PI / 8.0,
            min_phi: -3.0 * PI / 8.0,
        },
        LookAtSphericalCoords::default(),
        Paintable,
        Camera {
            order: 2,
            clear_color: ClearColorConfig::None,
            ..default()
        },
        RenderTarget::from((**render_target).clone()),
        CameraMainTextureUsages::default(),
    ))
}

/// The camera controlled in the paint skies state whose subjects get painted.
#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, Hash, Reflect, Component)]
#[require(Name = "PaintSkiesCamera")]
pub struct PaintSkiesCamera;

/// Marker component for the viewport UI node displaying the [`ClearSkiesRenderTarget`].
#[derive(Default, Debug, PartialEq, Eq, Copy, Clone, Hash, Reflect, Component)]
pub struct ClearSkiesViewport;

/// Defines the viewport UI node displaying the [`ClearSkiesRenderTarget`].
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
        ZIndex(0),
    ))
}

/// The [`ClearSkiesViewport`] will always be at the center of the screen with the correct aspect
/// ratio.
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
