use avian3d::PhysicsPlugins;
use avian3d::collision::collider::Collider;
use avian3d::dynamics::integrator::Gravity;
use avian3d::dynamics::rigid_body::{LinearVelocity, LockedAxes, RigidBody};
use bevy::camera::visibility::RenderLayers;
use bevy::prelude::*;
use bevy_pipe_affect::prelude::*;
use leafwing_input_manager::prelude::*;

use crate::clear_skies::ClearSkiesState;
use crate::clear_skies::play_skies::PlaySkiesCamera;
use crate::clear_skies::render_layers::PAINTED_LAYER;
use crate::clear_skies::switch_gamepads::SwitchGamepadsPlugin;

#[derive(Copy, Clone, PartialEq, Eq, Default, Debug)]
pub struct ClearSkiesPlayerPlugin;

impl Plugin for ClearSkiesPlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            SwitchGamepadsPlugin::<ClearSkiesPlayerAction>::default(),
            PhysicsPlugins::default(),
        ))
        .insert_resource(Gravity(Vec3::NEG_Y * 100.0))
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
    ClearSkiesPlayerSettings,
    Collider::capsule(5f32, 10f32),
    LockedAxes::ROTATION_LOCKED,
    RigidBody::Dynamic,
    RenderLayers = PAINTED_LAYER
)]
struct ClearSkiesPlayer;

#[derive(Copy, Clone, PartialEq, Debug, Component)]
struct ClearSkiesPlayerSettings {
    speed: f32,
    jump: f32,
}

impl Default for ClearSkiesPlayerSettings {
    fn default() -> Self {
        ClearSkiesPlayerSettings {
            speed: 80.0,
            jump: 80.0,
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Debug, Hash, Reflect, Actionlike)]
pub enum ClearSkiesPlayerAction {
    #[actionlike(DualAxis)]
    Move,
    #[actionlike(Button)]
    Jump,
    #[actionlike(Button)]
    Transition,
}

fn spawn_player() -> AssetAddAnd<
    StandardMaterial,
    AssetAddAnd<
        Mesh,
        CommandSpawn<(
            ClearSkiesPlayer,
            Transform,
            InputMap<ClearSkiesPlayerAction>,
            Mesh3d,
            MeshMaterial3d<StandardMaterial>,
        )>,
    >,
> {
    let input_map = InputMap::default()
        .with_dual_axis(
            ClearSkiesPlayerAction::Move,
            GamepadStick::LEFT.with_deadzone_symmetric(0.1),
        )
        .with_dual_axis(ClearSkiesPlayerAction::Move, VirtualDPad::wasd())
        .with(ClearSkiesPlayerAction::Jump, KeyCode::Space)
        .with(ClearSkiesPlayerAction::Jump, GamepadButton::South)
        .with(ClearSkiesPlayerAction::Transition, GamepadButton::East)
        .with(ClearSkiesPlayerAction::Transition, KeyCode::KeyC);

    asset_add_and(StandardMaterial::default(), |material| {
        asset_add_and(Capsule3d::new(5.0, 10.0).into(), |handle| {
            command_spawn((
                ClearSkiesPlayer,
                Transform::from_xyz(0.0, 500.0, -750.0),
                input_map,
                Mesh3d(handle),
                MeshMaterial3d(material),
            ))
        })
    })
}

fn move_player(
    camera: Single<&Transform, With<PlaySkiesCamera>>,
    player: Single<(
        Entity,
        &LinearVelocity,
        &ClearSkiesPlayerSettings,
        &ActionState<ClearSkiesPlayerAction>,
    )>,
) -> QueryEntityAffect<ComponentSet<LinearVelocity>> {
    let (player_entity, current_velocity, player_settings, input) = *player;

    let input_xz = input
        .dual_axis_data(&ClearSkiesPlayerAction::Move)
        .map(|dual_axis_data| dual_axis_data.pair)
        .unwrap_or_default();

    let input_jump = input.pressed(&ClearSkiesPlayerAction::Jump);

    let direction = (input_xz.x * camera.right().xz().normalize())
        + (input_xz.y * camera.forward().xz().normalize());

    let velocity_with_movement = current_velocity.with_xz(direction * player_settings.speed);

    let velocity = if input_jump {
        velocity_with_movement.with_y(player_settings.jump)
    } else {
        velocity_with_movement
    };
    query_entity_affect(player_entity, component_set(LinearVelocity(velocity)))
}

fn despawn_player(player: Single<Entity, With<ClearSkiesPlayer>>) -> EntityCommandDespawn {
    entity_command_despawn(*player)
}

fn transition_to_paint_skies(
    input: Single<&ActionState<ClearSkiesPlayerAction>>,
) -> Option<ResSet<NextState<ClearSkiesState>>> {
    input
        .just_released(&ClearSkiesPlayerAction::Transition)
        .then_some(res_set(NextState::Pending(ClearSkiesState::PaintSkies)))
}
