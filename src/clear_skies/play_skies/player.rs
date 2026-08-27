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
        )
        .add_systems(
            Update,
            transition_to_paint_skies
                .pipe(affect)
                .run_if(in_state(ClearSkiesState::PlaySkies)),
        )
        .add_systems(
            OnExit(ClearSkiesState::PlaySkies),
            despawn_player.pipe(affect),
        );
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Default, Debug, Component)]
#[require(
    Name = "ClearSkiesPlayer",
    Collider::capsule(5f32, 10f32),
    RigidBody::Dynamic
)]
struct ClearSkiesPlayer;

#[derive(Copy, Clone, PartialEq, Eq, Debug, Hash, Reflect, Actionlike)]
pub enum PaintSkiesPlayerAction {
    #[actionlike(DualAxis)]
    Move,
    #[actionlike(Button)]
    Jump,
    #[actionlike(Button)]
    Transition,
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
        .with(PaintSkiesPlayerAction::Jump, GamepadButton::South)
        .with(PaintSkiesPlayerAction::Transition, GamepadButton::East)
        .with(PaintSkiesPlayerAction::Transition, KeyCode::KeyC);
    command_spawn((
        ClearSkiesPlayer,
        Transform::from_xyz(0.0, 500.0, -750.0),
        input_map,
    ))
}

fn despawn_player(player: Single<Entity, With<ClearSkiesPlayer>>) -> EntityCommandDespawn {
    entity_command_despawn(*player)
}

fn transition_to_paint_skies(
    input: Single<&ActionState<PaintSkiesPlayerAction>>,
) -> Option<ResSet<NextState<ClearSkiesState>>> {
    input
        .just_released(&PaintSkiesPlayerAction::Transition)
        .then_some(res_set(NextState::Pending(ClearSkiesState::PaintSkies)))
}
