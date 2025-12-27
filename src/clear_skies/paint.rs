use std::f32::consts::PI;

use bevy::input::gamepad::{
    GamepadAxisChangedEvent,
    GamepadButtonChangedEvent,
    GamepadConnectionEvent,
    GamepadEvent,
};
use bevy::prelude::*;
use bevy_pipe_affect::prelude::*;
use leafwing_input_manager::prelude::*;
use thiserror::Error;

use crate::clear_skies::ClearSkiesState;

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, Hash, Reflect)]
pub struct PaintSkiesPlugin;

impl Plugin for PaintSkiesPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(InputManagerPlugin::<PaintSkiesAction>::default())
            .init_resource::<PaintSkiesSettings>()
            .add_systems(
                OnEnter(ClearSkiesState::PaintSkies),
                spawn_player.pipe(affect),
            )
            .add_systems(
                FixedUpdate,
                (
                    switch_gamepads.pipe(affect),
                    rotate_spherical_coords.pipe(affect),
                    look_at_spherical_coords.pipe(affect),
                )
                    .run_if(in_state(ClearSkiesState::PaintSkies)),
            );
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Reflect, Resource)]
pub struct PaintSkiesSettings {
    rotate_sensitivity: f32,
}

impl Default for PaintSkiesSettings {
    fn default() -> Self {
        PaintSkiesSettings {
            rotate_sensitivity: 0.05,
        }
    }
}

#[derive(Debug, Default, Copy, Clone, PartialEq, Reflect, Component)]
pub struct SphericalCoordsBounds {
    max_phi: f32,
    min_phi: f32,
}

#[derive(Debug, Default, Copy, Clone, PartialEq, Reflect, Component)]
#[require(SphericalCoordsBounds, Transform)]
pub struct LookAtSphericalCoords {
    theta: f32,
    phi: f32,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Reflect, Actionlike)]
pub enum PaintSkiesAction {
    #[actionlike(DualAxis)]
    Rotate,
}

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, Hash, Reflect, Component)]
pub struct PaintSkiesPlayer;

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, Hash, Reflect, Error)]
#[error("no gamepads detected")]
pub struct NoGamepadsDetected;

pub fn switch_gamepads(
    mut gamepad_events: MessageReader<GamepadEvent>,
) -> Vec<impl Effect + use<>> {
    gamepad_events
        .read()
        .into_iter()
        .map(|event| {
            let (GamepadEvent::Connection(GamepadConnectionEvent {
                gamepad: entity, ..
            })
            | GamepadEvent::Button(GamepadButtonChangedEvent { entity, .. })
            | GamepadEvent::Axis(GamepadAxisChangedEvent { entity, .. })) = event;

            let entity = *entity;


            components_set_filtered_with::<_, _, With<PaintSkiesPlayer>>(
                move |(input_map,): (InputMap<PaintSkiesAction>,)| {
                    (input_map.with_gamepad(entity),)
                },
            )
        })
        .collect()
}

pub fn spawn_player(
    gamepads: Query<Entity, With<Gamepad>>,
) -> Result<impl Effect + use<>, NoGamepadsDetected> {
    let entity = gamepads.iter().next().ok_or(NoGamepadsDetected)?;
    let input_map = InputMap::default()
        .with_dual_axis(
            PaintSkiesAction::Rotate,
            GamepadStick::LEFT.with_deadzone_symmetric(0.2),
        )
        .with_gamepad(entity);

    Ok(command_spawn((
        input_map,
        Camera3d::default(),
        PaintSkiesPlayer,
        SphericalCoordsBounds {
            max_phi: 3.0 * PI / 8.0,
            min_phi: -3.0 * PI / 8.0,
        },
        LookAtSphericalCoords::default(),
    )))
}

pub fn rotate_spherical_coords(settings: Res<PaintSkiesSettings>) -> impl Effect + use<> {
    let rotate_sensitivity = settings.rotate_sensitivity;

    components_set_filtered_with_query_data::<
        _,
        _,
        (&ActionState<PaintSkiesAction>, &SphericalCoordsBounds),
        With<PaintSkiesPlayer>,
    >(
        move |(spherical_coords,): (LookAtSphericalCoords,), (action_state, bounds)| {
            let rotate_by = action_state.clamped_axis_pair(&PaintSkiesAction::Rotate);

            let phi = (spherical_coords.phi + (rotate_by.y * rotate_sensitivity))
                .clamp(bounds.min_phi, bounds.max_phi);
            let theta = (spherical_coords.theta - (rotate_by.x * rotate_sensitivity)) % (2.0 * PI);

            (LookAtSphericalCoords { phi, theta },)
        },
    )
}

pub fn look_at_spherical_coords() -> impl Effect + use<> {
    components_set_with_query_data::<_, _, &LookAtSphericalCoords>(
        |(transform,): (Transform,), coords| {
            let theta_unit_circle_coords = Vec2::new(coords.theta.cos(), coords.theta.sin());
            let phi_unit_circle_coords = Vec2::new(coords.phi.cos(), coords.phi.sin());

            let xy = theta_unit_circle_coords * phi_unit_circle_coords.x;
            let look_at = xy.extend(phi_unit_circle_coords.y) + transform.translation;

            (transform.looking_at(look_at, Vec3::Z),)
        },
    )
}
