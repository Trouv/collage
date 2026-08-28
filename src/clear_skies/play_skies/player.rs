use avian3d::collision::collider::Collider;
use avian3d::dynamics::rigid_body::{LinearVelocity, LockedAxes, RigidBody};
use bevy::prelude::*;
use bevy_pipe_affect::prelude::*;
use leafwing_input_manager::prelude::*;

use crate::clear_skies::ClearSkiesState;
use crate::clear_skies::play_skies::PlaySkiesCamera;
use crate::clear_skies::switch_gamepads::SwitchGamepadsPlugin;

#[derive(Copy, Clone, PartialEq, Eq, Default, Debug)]
pub struct ClearSkiesPlayerPlugin;

impl Plugin for ClearSkiesPlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(SwitchGamepadsPlugin::<PaintSkiesPlayerAction>::default())
            .add_systems(
                OnEnter(ClearSkiesState::PlaySkies),
                spawn_player.pipe(affect),
            )
            .add_systems(
                Update,
                (
                    transition_to_paint_skies.pipe(affect),
                    move_player.pipe(affect),
                )
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
    LockedAxes::ROTATION_LOCKED,
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

fn move_player(
    camera: Single<&Camera, With<PlaySkiesCamera>>,
    player: Single<(
        Entity,
        &LinearVelocity,
        &ActionState<PaintSkiesPlayerAction>,
    )>,
) -> QueryEntityAffect<ComponentSet<LinearVelocity>> {
    let (player_entity, current_velocity, input) = *player;
    let input_xz = input
        .dual_axis_data(&PaintSkiesPlayerAction::Move)
        .map(|dual_axis_data| dual_axis_data.pair)
        .unwrap_or_default();

    let input_jump = input.pressed(&PaintSkiesPlayerAction::Jump);

    let velocity_with_movement = current_velocity.with_xz(input_xz);

    let velocity = if input_jump {
        velocity_with_movement.with_y(10.)
    } else {
        velocity_with_movement
    };

    dbg!(velocity);

    query_entity_affect(player_entity, component_set(LinearVelocity(velocity)))
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
