use std::marker::PhantomData;
use std::time::Duration;

use bevy::camera::visibility::RenderLayers;
use bevy::prelude::*;
use bevy::scene::SceneInstanceReady;
use bevy_pipe_affect::prelude::*;

use crate::clear_skies::ClearSkiesState;
use crate::clear_skies::play_skies::PlaySkiesCamera;
use crate::clear_skies::render_layers::{PAINTABLE_LAYER, PAINTED_LAYER};

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, Hash, Reflect)]
pub struct PaintMeshesPlugin;

impl Plugin for PaintMeshesPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PaintMeshesTimer>()
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
        PaintMeshesTimer(Timer::new(Duration::from_millis(100), TimerMode::Repeating))
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

fn paint_meshes(
    timer: Res<PaintMeshesTimer>,
    paintable_meshes: Query<(&Mesh3d, &GlobalTransform), With<Paintable>>,
    mut mesh_assets: ResMut<Assets<Mesh>>,
    paintable_camera: Single<(&Camera, &GlobalTransform), With<Paintable>>,
    play_skies_camera: Single<(&Camera, &GlobalTransform), With<PlaySkiesCamera>>,
) -> Option<Vec<impl Effect + use<>>> {
    if timer.just_finished() {
        Some(
            paintable_meshes
                .iter()
                .flat_map(|(mesh, mesh_transform)| {
                    let mesh = mesh_assets.get(mesh)?;

                    let spawn_commands = mesh
                        .clone()
                        .triangles()
                        .ok()?
                        .flat_map(|triangle| {
                            let vertices = triangle
                                .vertices
                                .into_iter()
                                .map(|vertex| {
                                    let (paintable_camera, paintable_camera_transform) =
                                        *paintable_camera;
                                    let world_translation = mesh_transform.transform_point(vertex);
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

                                    Some(play_skies_ray.get_point(10.0))
                                })
                                .collect::<Option<Vec<_>>>()?;

                            let world_triangle =
                                Triangle3d::new(vertices[0], vertices[1], vertices[2]);

                            let centroid = world_triangle.centroid();

                            let centered_triangle = Triangle3d {
                                vertices: world_triangle.vertices.map(|vertex| vertex - centroid),
                            };

                            let mesh = Mesh::from(centered_triangle);

                            let mesh_handle = mesh_assets.add(mesh);

                            // Note: We don't need to adjust this relative to camera translation
                            // since we already calculated it in world-space
                            let transform = Transform::from_translation(centroid);

                            Some(command_spawn((
                                Mesh3d(mesh_handle),
                                transform,
                                PAINTED_LAYER,
                            )))
                        })
                        .collect::<Vec<_>>();

                    Some(spawn_commands)
                })
                .flatten()
                .collect::<Vec<_>>(),
        )
    } else {
        None
    }
}
