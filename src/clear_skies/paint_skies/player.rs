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

use crate::clear_skies::paint_skies::settings::PaintSkiesSettings;
use crate::clear_skies::paint_skies::spherical_coords::{
    LookAtSphericalCoords,
    SphericalCoordsBounds,
};
use crate::clear_skies::render_layers::PAINTABLE_LAYER;

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
        PAINTABLE_LAYER,
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
