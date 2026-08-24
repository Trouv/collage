use avian3d::collision::collider::Collider;
use avian3d::dynamics::rigid_body::RigidBody;
use avian3d::math::Scalar;
use bevy::prelude::*;
use bevy_pipe_affect::prelude::*;

#[derive(Copy, Clone, PartialEq, Eq, Default, Debug)]
pub struct ClearSkiesPlayerPlugin;

impl Plugin for ClearSkiesPlayerPlugin {
    fn build(&self, app: &mut App) {}
}

#[derive(Copy, Clone, PartialEq, Eq, Default, Debug, Component)]
#[require(Name = "ClearSkiesPlayer", Collider::sphere(1.0.into()), RigidBody::Dynamic)]
struct ClearSkiesPlayer;

fn spawn_player() -> CommandSpawn<(ClearSkiesPlayer, Transform)> {
    command_spawn((ClearSkiesPlayer, Transform::default()))
}
