use avian3d::collision::collider::Collider;
use avian3d::dynamics::rigid_body::RigidBody;
use bevy::prelude::*;
use bevy_pipe_affect::prelude::*;
use leafwing_input_manager::prelude::*;

#[derive(Copy, Clone, PartialEq, Eq, Default, Debug)]
pub struct ClearSkiesPlayerPlugin;

impl Plugin for ClearSkiesPlayerPlugin {
    fn build(&self, app: &mut App) {}
}

#[derive(Copy, Clone, PartialEq, Eq, Default, Debug, Component)]
#[require(Name = "ClearSkiesPlayer", Collider::sphere(1f32.into()), RigidBody::Dynamic, ActionState<PaintSkiesPlayerAction>)]
struct ClearSkiesPlayer;

#[derive(Copy, Clone, PartialEq, Eq, Debug, Hash, Reflect, Actionlike)]
pub enum PaintSkiesPlayerAction {
    #[actionlike(DualAxis)]
    Move,
    #[actionlike(Button)]
    Jump,
}

fn spawn_player() -> CommandSpawn<(ClearSkiesPlayer, Transform)> {
    command_spawn((ClearSkiesPlayer, Transform::default()))
}
