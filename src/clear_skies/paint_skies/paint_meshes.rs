use std::time::Duration;

use bevy::camera::visibility::RenderLayers;
use bevy::prelude::*;
use bevy::render::render_resource::TextureFormat;
use bevy::render::view::screenshot::{Screenshot, ScreenshotCaptured};
use bevy::scene::SceneInstanceReady;
use bevy_pipe_affect::prelude::*;
use leafwing_input_manager::prelude::*;

use crate::clear_skies::ClearSkiesState;
use crate::clear_skies::camera::{ClearSkiesRenderTarget, ClearSkiesResolution, PaintSkiesAction};
use crate::clear_skies::play_skies::PlaySkiesCamera;
use crate::clear_skies::render_layers::{PAINTABLE_LAYER, PAINTED_LAYER};
use crate::effects::{
    AssetsInsert,
    EntityCommandInsertRecursive,
    assets_add_and,
    assets_insert,
    entity_command_insert_recursive,
};

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, Hash, Reflect)]
pub struct PaintMeshesPlugin;

impl Plugin for PaintMeshesPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PaintMeshesTimer>()
            .init_resource::<PaintLayerSettings>()
            .init_resource::<LayerIndex>()
            .add_systems(
                OnEnter(ClearSkiesState::Setup),
                create_paint_skies_canvas.pipe(affect),
            )
            .register_type::<PaintSkiesCanvas>()
            .add_systems(
                Startup,
                (|| command_spawn(Observer::new(propagate_paintable_on_scenes.pipe(affect))))
                    .pipe(affect),
            )
            .add_systems(
                Last,
                (
                    tick_paint_meshes_timer.pipe(affect),
                    paint_canvas.pipe(affect),
                )
                    .chain()
                    .run_if(in_state(ClearSkiesState::PaintSkies)),
            );
    }
}

/// Marker component for paintable meshes.
#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, Hash, Reflect, Component)]
#[require(Mesh3d, RenderLayers = PAINTABLE_LAYER)]
pub struct Paintable;

#[derive(Debug, Clone, PartialEq, Eq, Reflect, Deref, DerefMut, Resource)]
struct PaintMeshesTimer(Timer);

impl Default for PaintMeshesTimer {
    fn default() -> Self {
        PaintMeshesTimer(Timer::new(Duration::from_millis(100), TimerMode::Repeating))
    }
}

fn tick_paint_meshes_timer(
    time: Res<Time>,
    paint: Single<&ActionState<PaintSkiesAction>>,
) -> ResSetWith<impl FnOnce(PaintMeshesTimer) -> PaintMeshesTimer + use<>, PaintMeshesTimer> {
    let delta_time = time.delta();

    let painting = paint.pressed(&PaintSkiesAction::Paint);

    res_set_with(move |mut timer: PaintMeshesTimer| {
        if painting {
            timer.tick(delta_time);
        }

        timer
    })
}

fn propagate_paintable_on_scenes(
    instance_ready: On<SceneInstanceReady>,
    paintables: Query<(), With<Paintable>>,
) -> Option<EntityCommandInsertRecursive<Children, Paintable>> {
    if paintables.contains(instance_ready.entity) {
        Some(entity_command_insert_recursive(
            instance_ready.entity,
            Paintable,
        ))
    } else {
        None
    }
}

/// Index for the paint mesh layer.
#[derive(
    Default, Debug, PartialEq, Eq, Copy, Clone, Hash, Reflect, Deref, DerefMut, Resource, Component,
)]
pub struct LayerIndex(u32);

/// Settings for the logic of painting layers.
#[derive(Debug, PartialEq, Copy, Clone, Reflect, Resource)]
pub struct PaintLayerSettings {
    pub zero_layer_distance: f32,
    pub layer_distance_collapse_rate: f32,
}

impl Default for PaintLayerSettings {
    fn default() -> Self {
        PaintLayerSettings {
            zero_layer_distance: 1000.0,
            layer_distance_collapse_rate: 0.99,
        }
    }
}

#[derive(Default, Debug, PartialEq, Clone, Deref, DerefMut, Reflect, Resource)]
#[reflect(Resource)]
pub struct PaintSkiesCanvas(Handle<Image>);

/// System that creates [`ClearSkiesRenderTarget`].
pub fn create_paint_skies_canvas(resolution: Res<ClearSkiesResolution>) -> impl Effect + use<> {
    let image = Image::new_target_texture(
        resolution.x,
        resolution.y,
        TextureFormat::bevy_default(),
        None,
    );

    assets_add_and(image, |handle| {
        command_insert_resource(PaintSkiesCanvas(handle))
    })
}

fn world_to_viewport_uv(
    camera: &Camera,
    camera_transform: &GlobalTransform,
    world_position: Vec3,
) -> Option<Vec2> {
    let device_coords = camera.world_to_ndc(camera_transform, world_position)?.xy();

    let small_device_coords = (device_coords + 1.0) / 2.0;

    // ndc coords and uv corods are slightly different:
    // | ndc    | uv    |
    // | ------ | ----- |
    // | y+     | y-    |
    // | [-1,1] | [0,1] |
    let uv_coords = Vec2::new(small_device_coords.x, 1.0 - small_device_coords.y);

    Some(uv_coords)
}

fn save_screenshot_to_canvas(
    screenshot: On<ScreenshotCaptured>,
    canvas: Res<PaintSkiesCanvas>,
) -> AssetsInsert<Image> {
    assets_insert((**canvas).clone(), screenshot.image.clone())
}

fn paint_canvas(
    timer: Res<PaintMeshesTimer>,
    render_target: Res<ClearSkiesRenderTarget>,
) -> Option<impl Effect + use<>> {
    if timer.just_finished() {
        let effect = command_spawn_and(Screenshot::image((**render_target).clone()), |entity| {
            (
                command_spawn(
                    Observer::new(save_screenshot_to_canvas.pipe(affect)).with_entity(entity),
                ),
                command_spawn(Observer::new(paint_meshes.pipe(affect)).with_entity(entity)),
            )
        });

        Some(effect)
    } else {
        None
    }
}

struct TriangleWithUvs {
    triangle: Triangle3d,
    uvs: [Vec2; 3],
}

fn paint_triangle(
    paint_layer_settings: &PaintLayerSettings,
    layer_index: &LayerIndex,
    mesh_transform: &GlobalTransform,
    paintable_camera: &Camera,
    paintable_camera_transform: &GlobalTransform,
    play_skies_camera: &Camera,
    play_skies_camera_transform: &GlobalTransform,
) -> impl Fn(Triangle3d) -> Option<TriangleWithUvs> {
    |triangle| {
        let vertex_uvs = triangle
            .vertices
            .into_iter()
            .map(|vertex| {
                let world_translation = mesh_transform.transform_point(vertex);

                let uv = world_to_viewport_uv(
                    paintable_camera,
                    paintable_camera_transform,
                    world_translation,
                )?;

                let viewport_coords = paintable_camera
                    .world_to_viewport(paintable_camera_transform, world_translation)
                    .ok()?;

                let play_skies_ray = play_skies_camera
                    .viewport_to_world(play_skies_camera_transform, viewport_coords)
                    .ok()?;

                let vertex = play_skies_ray.get_point(
                    paint_layer_settings.zero_layer_distance
                        * paint_layer_settings
                            .layer_distance_collapse_rate
                            .powf(**layer_index as f32),
                );

                Some((vertex, uv))
            })
            .collect::<Option<Vec<_>>>()?;

        let world_triangle = Triangle3d::new(vertex_uvs[0].0, vertex_uvs[1].0, vertex_uvs[2].0);

        Some(TriangleWithUvs {
            triangle: world_triangle,
            uvs: [vertex_uvs[0].1, vertex_uvs[1].1, vertex_uvs[2].1],
        })
    }
}

fn paint_meshes(
    _: On<ScreenshotCaptured>,
    paintable_meshes: Query<(&Mesh3d, &GlobalTransform), With<Paintable>>,
    mesh_assets: Res<Assets<Mesh>>,
    paintable_camera: Single<(&Camera, &GlobalTransform), With<Paintable>>,
    play_skies_camera: Single<(&Camera, &GlobalTransform), With<PlaySkiesCamera>>,
    layer_index: Res<LayerIndex>,
    paint_layer_settings: Res<PaintLayerSettings>,
    paint_skies_canvas: Res<PaintSkiesCanvas>,
) -> (Vec<impl Effect + use<>>, ResSet<LayerIndex>) {
    (
        paintable_meshes
            .iter()
            .flat_map(|(mesh, mesh_transform)| {
                let mesh = mesh_assets.get(mesh)?;

                let (paintable_camera, paintable_camera_transform) = *paintable_camera;

                let (play_skies_camera, play_skies_camera_transform) = *play_skies_camera;

                let spawn_commands = mesh
                    .clone()
                    .triangles()
                    .ok()?
                    .flat_map(paint_triangle(
                        &paint_layer_settings,
                        &layer_index,
                        mesh_transform,
                        paintable_camera,
                        paintable_camera_transform,
                        play_skies_camera,
                        play_skies_camera_transform,
                    ))
                    .map(|triangle_with_uvs| {
                        let world_triangle = triangle_with_uvs.triangle;

                        let centroid = world_triangle.centroid();

                        let centered_triangle = Triangle3d {
                            vertices: world_triangle.vertices.map(|vertex| vertex - centroid),
                        };

                        let mesh = Mesh::from(centered_triangle);

                        let mesh_with_uvs = mesh.with_inserted_attribute(
                            Mesh::ATTRIBUTE_UV_0,
                            triangle_with_uvs.uvs.map(Into::<[f32; 2]>::into).to_vec(),
                        );

                        // Note: We don't need to adjust this relative to camera translation
                        // since we already calculated it in world-space
                        let transform = Transform::from_translation(centroid);

                        let material = StandardMaterial {
                            unlit: true,
                            ..StandardMaterial::from((**paint_skies_canvas).clone())
                        };

                        Some(assets_add_and(mesh_with_uvs, move |mesh_handle| {
                            assets_add_and(material, move |material_handle| {
                                command_spawn((
                                    Mesh3d(mesh_handle),
                                    MeshMaterial3d(material_handle),
                                    transform,
                                    PAINTED_LAYER,
                                ))
                            })
                        }))
                    })
                    .collect::<Vec<_>>();

                Some(spawn_commands)
            })
            .flatten()
            .collect::<Vec<_>>(),
        res_set(LayerIndex(**layer_index + 1)),
    )
}
