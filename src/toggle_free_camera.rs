use bevy::camera::RenderTarget;
use bevy::camera::visibility::RenderLayers;
use bevy::camera_controller::free_camera::FreeCamera;
use bevy::prelude::*;
use bevy::render::render_resource::TextureFormat;
use bevy_pipe_affect::prelude::*;

use crate::effects::{AssetsAddAnd, assets_add_and};

/// Debug plugin that spawns or despawns a flycam.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct ToggleFreeCameraPlugin {
    /// The flycam will render layers 0..num_render_layers
    pub num_render_layers: usize,
}

impl Default for ToggleFreeCameraPlugin {
    fn default() -> Self {
        ToggleFreeCameraPlugin {
            num_render_layers: 2,
        }
    }
}

impl Plugin for ToggleFreeCameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                spawn_or_despawn_free_cam(*self).pipe(affect),
                spawn_viewport.pipe(affect),
                despawn_viewports.pipe(affect),
            ),
        );
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Debug, Component)]
#[require(Camera3d, FreeCamera, Name = "ToggleFreeCamera")]
struct ToggleFreeCamera;

#[derive(Effect)]
enum SpawnOrDespawnFreeCamera {
    Spawn(
        AssetsAddAnd<
            Image,
            CommandSpawn<(ToggleFreeCamera, RenderLayers, RenderTarget)>,
            Box<
                dyn Fn(
                    Handle<Image>,
                )
                    -> CommandSpawn<(ToggleFreeCamera, RenderLayers, RenderTarget)>,
            >,
        >,
    ),
    Despawn(EntityCommandDespawn),
    Wait,
}

fn spawn_or_despawn_free_cam(
    settings: ToggleFreeCameraPlugin,
) -> impl Fn(
    Res<ButtonInput<KeyCode>>,
    Option<Single<Entity, With<ToggleFreeCamera>>>,
) -> SpawnOrDespawnFreeCamera {
    move |input, free_cam| {
        if input.all_just_pressed([KeyCode::KeyF]) {
            match free_cam {
                None => {
                    let image =
                        Image::new_target_texture(1, 1, TextureFormat::Bgra8UnormSrgb, None);

                    SpawnOrDespawnFreeCamera::Spawn(assets_add_and(
                        image,
                        Box::new(move |handle| {
                            command_spawn((
                                ToggleFreeCamera,
                                RenderLayers::from_layers(
                                    (0..=settings.num_render_layers)
                                        .collect::<Vec<_>>()
                                        .as_slice(),
                                ),
                                RenderTarget::from(handle),
                            ))
                        }),
                    ))
                }
                Some(entity) => SpawnOrDespawnFreeCamera::Despawn(entity_command_despawn(*entity)),
            }
        } else {
            SpawnOrDespawnFreeCamera::Wait
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Debug, Component)]
#[require(Name = "ToggleFreeCameraViewportNode", ZIndex(10))]
struct ToggleFreeCameraViewportNode;

fn spawn_viewport(
    camera: Single<Entity, Added<ToggleFreeCamera>>,
) -> CommandSpawn<(ToggleFreeCameraViewportNode, Node, ViewportNode)> {
    let camera = camera.clone();
    command_spawn((
        ToggleFreeCameraViewportNode,
        Node {
            position_type: PositionType::Absolute,
            right: percent(0),
            width: percent(50),
            height: percent(50),
            ..default()
        },
        ViewportNode::new(camera),
    ))
}

fn despawn_viewports(
    removed_cameras: RemovedComponents<ToggleFreeCamera>,
    viewports: Query<Entity, With<ToggleFreeCameraViewportNode>>,
) -> Vec<EntityCommandDespawn> {
    if removed_cameras.is_empty() {
        vec![]
    } else {
        viewports.iter().map(entity_command_despawn).collect()
    }
}
