use std::marker::PhantomData;
use std::time::Duration;

use bevy::camera::visibility::RenderLayers;
use bevy::prelude::*;
use bevy::scene::SceneInstanceReady;
use bevy_pipe_affect::prelude::*;

use crate::clear_skies::ClearSkiesState;
use crate::clear_skies::render_layers::PAINTABLE_LAYER;

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
                    //paint_meshes.pipe(affect),
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
        PaintMeshesTimer(Timer::new(Duration::from_millis(250), TimerMode::Repeating))
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

//fn paint_meshes(
//paint_meshes: Res<PaintMeshesTimer>,
//timer: Res<PaintMeshesTimer>,
//) -> Option<impl Effect + use<>> {
//if timer.just_finished() {
//// Spawn Some meshes in RenderLayer 1 that are based off the screen coordinates of meshes in
//// the real world (give these marker components tbh).
//// Orthographic projection (so we don't have to scale things up as we move them further
//// from the cmaera)
//} else {
//None
//}
//}
