use std::marker::PhantomData;
use std::time::Duration;

use bevy::camera::visibility::RenderLayers;
use bevy::prelude::*;
use bevy::scene::SceneInstanceReady;
use bevy_pipe_affect::prelude::*;

use crate::clear_skies::ClearSkiesState;
use crate::clear_skies::camera::ClearSkiesRenderTarget;
use crate::clear_skies::play_skies::PlaySkiesCamera;
use crate::clear_skies::render_layers::{PAINTABLE_LAYER, PAINTED_LAYER};

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, Hash, Reflect)]
pub struct PaintMeshesPlugin;

impl Plugin for PaintMeshesPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PaintMeshesTimer>()
            .init_resource::<PaintLayerSettings>()
            .init_resource::<LayerIndex>()
            .add_systems(
                Startup,
                (|| command_spawn(Observer::new(propagate_paintable_on_scenes.pipe(affect))))
                    .pipe(affect),
            )
            .add_systems(
                Update,
                (
                    tick_paint_meshes_timer.pipe(affect),
                    paint_meshes.pipe(affect),
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
        PaintMeshesTimer(Timer::new(Duration::from_millis(200), TimerMode::Repeating))
    }
}

fn tick_paint_meshes_timer(
    time: Res<Time>,
) -> ResSetWith<impl FnOnce(PaintMeshesTimer) -> PaintMeshesTimer + use<>, PaintMeshesTimer> {
    let delta_time = time.delta();

    res_set_with(move |mut timer: PaintMeshesTimer| {
        timer.tick(delta_time);

        timer
    })
}

/// [`Effect`] that inserts a component recursively on relationship targets.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Reflect)]
pub struct EntityCommandInsertRecursive<RT, B>
where
    RT: RelationshipTarget,
    B: Bundle + Clone,
{
    pub entity: Entity,
    pub bundle: B,
    _phantom: PhantomData<RT>,
}

impl<RT, B> EntityCommandInsertRecursive<RT, B>
where
    B: Bundle + Clone,
    RT: RelationshipTarget,
{
    /// Construct a new [`EntityCommandInsertRecursive`].
    fn new(entity: Entity, bundle: B) -> Self {
        Self {
            entity,
            bundle,
            _phantom: PhantomData,
        }
    }
}

impl<RT, B> Effect for EntityCommandInsertRecursive<RT, B>
where
    RT: RelationshipTarget,
    B: Bundle + Clone,
{
    type MutParam = Commands<'static, 'static>;

    fn affect(self, param: &mut <Self::MutParam as bevy::ecs::system::SystemParam>::Item<'_, '_>) {
        param
            .entity(self.entity)
            .insert_recursive::<RT>(self.bundle);
    }
}

fn propagate_paintable_on_scenes(
    instance_ready: On<SceneInstanceReady>,
    paintables: Query<(), With<Paintable>>,
) -> Option<EntityCommandInsertRecursive<Children, Paintable>> {
    if paintables.contains(instance_ready.entity) {
        Some(EntityCommandInsertRecursive::new(
            instance_ready.entity,
            Paintable,
        ))
    } else {
        None
    }
}

/// `Effect` that adds an asset to an `Assets` resource and can produce more effects with the
/// `Handle`.
#[derive(Default, Debug, PartialEq, Eq, Copy, Clone, Hash, Reflect)]
pub struct AssetsAddAnd<A, E, F>
where
    A: Asset,
    E: Effect,
    F: FnOnce(Handle<A>) -> E,
{
    pub asset: A,
    pub f: F,
}

pub fn assets_add_and<A, E, F>(asset: A, f: F) -> AssetsAddAnd<A, E, F>
where
    A: Asset,
    E: Effect,
    F: FnOnce(Handle<A>) -> E,
{
    AssetsAddAnd { asset, f }
}

impl<A, E, F> Effect for AssetsAddAnd<A, E, F>
where
    A: Asset,
    E: Effect,
    F: FnOnce(Handle<A>) -> E,
{
    type MutParam = (ResMut<'static, Assets<A>>, E::MutParam);

    fn affect(self, param: &mut <Self::MutParam as bevy::ecs::system::SystemParam>::Item<'_, '_>) {
        let handle = param.0.add(self.asset);
        (self.f)(handle).affect(&mut param.1);
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
            layer_distance_collapse_rate: 0.95,
        }
    }
}

fn world_to_viewport_uv(
    camera: &Camera,
    camera_transform: &GlobalTransform,
    world_position: Vec3,
) -> Option<Vec2> {
    let device_coords = camera.world_to_ndc(camera_transform, world_position)?.xy();

    // ndc coords and uv corods are slightly different:
    // | ndc    | uv    |
    // | ------ | ----- |
    // | y+     | y-    |
    // | [-1,1] | [0,1] |
    let big_uv_coords = Vec2::new(device_coords.x, -device_coords.y);

    let uv_coords = (big_uv_coords + 1.0) / 2.0;

    Some(uv_coords)
}

fn paint_meshes(
    timer: Res<PaintMeshesTimer>,
    paintable_meshes: Query<(&Mesh3d, &GlobalTransform), With<Paintable>>,
    mesh_assets: Res<Assets<Mesh>>,
    paintable_camera: Single<(&Camera, &GlobalTransform), With<Paintable>>,
    play_skies_camera: Single<(&Camera, &GlobalTransform), With<PlaySkiesCamera>>,
    layer_index: Res<LayerIndex>,
    paint_layer_settings: Res<PaintLayerSettings>,
    clear_skies_render_target: Res<ClearSkiesRenderTarget>,
) -> Option<(Vec<impl Effect + use<>>, ResSet<LayerIndex>)> {
    if timer.just_finished() {
        Some((
            paintable_meshes
                .iter()
                .flat_map(|(mesh, mesh_transform)| {
                    let mesh = mesh_assets.get(mesh)?;

                    let spawn_commands = mesh
                        .clone()
                        .triangles()
                        .ok()?
                        .flat_map(|triangle| {
                            let vertex_uvs = triangle
                                .vertices
                                .into_iter()
                                .map(|vertex| {
                                    let (paintable_camera, paintable_camera_transform) =
                                        *paintable_camera;
                                    let world_translation = mesh_transform.transform_point(vertex);

                                    let uv = world_to_viewport_uv(
                                        paintable_camera,
                                        paintable_camera_transform,
                                        world_translation,
                                    )?;

                                    let viewport_coords = paintable_camera
                                        .world_to_viewport(
                                            paintable_camera_transform,
                                            world_translation,
                                        )
                                        .ok()?;

                                    let (play_skies_camera, play_skies_camera_transform) =
                                        *play_skies_camera;
                                    let play_skies_ray = play_skies_camera
                                        .viewport_to_world(
                                            play_skies_camera_transform,
                                            viewport_coords,
                                        )
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

                            let world_triangle =
                                Triangle3d::new(vertex_uvs[0].0, vertex_uvs[1].0, vertex_uvs[2].0);

                            let centroid = world_triangle.centroid();

                            let centered_triangle = Triangle3d {
                                vertices: world_triangle.vertices.map(|vertex| vertex - centroid),
                            };

                            let mesh = Mesh::from(centered_triangle);

                            let mesh_with_uvs = mesh.with_inserted_attribute(
                                Mesh::ATTRIBUTE_UV_0,
                                [vertex_uvs[0].1, vertex_uvs[1].1, vertex_uvs[2].1]
                                    .map(Into::<[f32; 2]>::into)
                                    .to_vec(),
                            );

                            // Note: We don't need to adjust this relative to camera translation
                            // since we already calculated it in world-space
                            let transform = Transform::from_translation(centroid);

                            let material =
                                StandardMaterial::from((**clear_skies_render_target).clone());

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
        ))
    } else {
        None
    }
}
