use bevy::asset::RenderAssetUsages;
use bevy::camera::RenderTarget;
use bevy::camera::visibility::RenderLayers;
use bevy::camera_controller::free_camera::FreeCamera;
use bevy::prelude::*;
use bevy::render::render_resource::{TextureDimension, TextureFormat, TextureUsages};
use bevy_pipe_affect::prelude::*;

use crate::clear_skies::ClearSkiesViewport;
use crate::clear_skies::paint_skies::{AssetsAddAnd, assets_add_and};

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct ToggleFreeCameraPlugin {
    pub num_render_layers: usize,
    pub order: isize,
}

impl Default for ToggleFreeCameraPlugin {
    fn default() -> Self {
        ToggleFreeCameraPlugin {
            num_render_layers: 2,
            order: 100,
        }
    }
}

impl Plugin for ToggleFreeCameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                spawn_or_despawn_free_cam(*self).pipe(affect),
                spawn_viewport(self.order).pipe(affect),
                despawn_viewports.pipe(affect),
            ),
        );
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Debug, Component)]
struct ToggleFreeCamera;

#[derive(Effect)]
enum SpawnOrDespawnFreeCamera {
    Spawn(
        AssetsAddAnd<
            Image,
            CommandSpawn<(
                ToggleFreeCamera,
                Camera3d,
                FreeCamera,
                Camera,
                RenderLayers,
                RenderTarget,
            )>,
            Box<
                dyn Fn(
                    Handle<Image>,
                ) -> CommandSpawn<(
                    ToggleFreeCamera,
                    Camera3d,
                    FreeCamera,
                    Camera,
                    RenderLayers,
                    RenderTarget,
                )>,
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
                    let mut image = Image::new_uninit(
                        default(),
                        TextureDimension::D2,
                        TextureFormat::Bgra8UnormSrgb,
                        RenderAssetUsages::all(),
                    );
                    image.texture_descriptor.usage = TextureUsages::TEXTURE_BINDING
                        | TextureUsages::COPY_DST
                        | TextureUsages::RENDER_ATTACHMENT;
                    SpawnOrDespawnFreeCamera::Spawn(assets_add_and(
                        image,
                        Box::new(move |handle| {
                            command_spawn((
                                ToggleFreeCamera,
                                Camera3d::default(),
                                FreeCamera::default(),
                                Camera {
                                    order: settings.order,
                                    ..default()
                                },
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
#[require(Name = "ToggleFreeCameraViewportNode")]
struct ToggleFreeCameraViewportNode;
fn spawn_viewport(
    order: isize,
) -> impl Fn(
    Single<Entity, Added<ToggleFreeCamera>>,
    Single<Entity, With<ClearSkiesViewport>>,
) -> CommandSpawn<(ToggleFreeCameraViewportNode, Node, ZIndex, ViewportNode)> {
    move |camera, viewport| {
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
            ZIndex(order as i32),
            ViewportNode::new(camera),
        ))
    }
}

//fn spawn_viewport(
//    order: isize,
//) -> impl Fn(
//    Single<Entity, Added<ToggleFreeCamera>>,
//    Single<Entity, With<ClearSkiesViewport>>,
//) -> CommandSpawnAnd<
//    (ToggleFreeCameraViewportNode, Node, ZIndex),
//    Box<dyn Fn(Entity) -> CommandSpawn<(ViewportNode, ChildOf)>>,
//    CommandSpawn<(ViewportNode, ChildOf)>,
//> {
//    move |camera, viewport| {
//        let camera = camera.clone();
//        command_spawn_and(
//            (
//                ToggleFreeCameraViewportNode,
//                Node::default(),
//                ZIndex(order as i32),
//            ),
//            Box::new(move |parent| command_spawn((ViewportNode::new(camera), ChildOf(parent)))),
//        )
//    }
//}

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
