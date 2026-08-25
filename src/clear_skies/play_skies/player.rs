use avian3d::collision::collider::Collider;
use avian3d::dynamics::rigid_body::RigidBody;
use bevy::prelude::*;
use bevy_pipe_affect::prelude::*;
use leafwing_input_manager::prelude::*;

use crate::clear_skies::ClearSkiesState;

#[derive(Copy, Clone, PartialEq, Eq, Default, Debug)]
pub struct ClearSkiesPlayerPlugin;

impl Plugin for ClearSkiesPlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(ClearSkiesState::PlaySkies),
            spawn_player.pipe(affect),
        );
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Default, Debug, Component)]
#[require(Name = "ClearSkiesPlayer", Collider::sphere(1f32.into()), RigidBody::Dynamic)]
struct ClearSkiesPlayer;

#[derive(Copy, Clone, PartialEq, Eq, Debug, Hash, Reflect, Actionlike)]
pub enum PaintSkiesPlayerAction {
    #[actionlike(DualAxis)]
    Move,
    #[actionlike(Button)]
    Jump,
}

fn spawn_player() -> CommandSpawn<(
    ClearSkiesPlayer,
    Transform,
    InputMap<PaintSkiesPlayerAction>,
)> {
    let input_map = InputMap::default()
        .with_dual_axis(
            PaintSkiesPlayerAction::Move,
            GamepadStick::LEFT.with_deadzone_symmetric(0.1),
        )
        .with_dual_axis(PaintSkiesPlayerAction::Move, VirtualDPad::wasd())
        .with(PaintSkiesPlayerAction::Jump, KeyCode::Space)
        .with(PaintSkiesPlayerAction::Jump, GamepadButton::South);
    command_spawn((ClearSkiesPlayer, Transform::default(), input_map))
}
