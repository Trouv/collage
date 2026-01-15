use std::f32::consts::PI;

use bevy::camera::RenderTarget;
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

use crate::clear_skies::camera::ClearSkiesRenderTarget;
use crate::clear_skies::paint_skies::Paintable;
use crate::clear_skies::paint_skies::settings::PaintSkiesSettings;
use crate::clear_skies::paint_skies::spherical_coords::{
    LookAtSphericalCoords,
    SphericalCoordsBounds,
};

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Reflect, Actionlike)]
pub enum PaintSkiesAction {
    #[actionlike(DualAxis)]
    Rotate,
}

/// The camera controlled in the paint skies state whose subjects get painted.
#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, Hash, Reflect, Component)]
#[require(Name = "PaintSkiesCamera")]
pub struct PaintSkiesCamera;

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, Hash, Reflect, Error)]
#[error("no gamepads detected")]
pub struct NoGamepadsDetected;

pub fn switch_gamepads(
    mut gamepad_events: MessageReader<GamepadEvent>,
) -> Vec<impl Effect + use<>> {
    gamepad_events
        .read()
        .map(|event| {
            let (GamepadEvent::Connection(GamepadConnectionEvent {
                gamepad: entity, ..
            })
            | GamepadEvent::Button(GamepadButtonChangedEvent { entity, .. })
            | GamepadEvent::Axis(GamepadAxisChangedEvent { entity, .. })) = event;

            let entity = *entity;


            components_set_filtered_with::<_, _, With<PaintSkiesCamera>>(
                move |(input_map,): (InputMap<PaintSkiesAction>,)| {
                    (input_map.with_gamepad(entity),)
                },
            )
        })
        .collect()
}

pub fn spawn_paint_skies_camera(render_target: Res<ClearSkiesRenderTarget>) -> impl Effect + use<> {
    let input_map = InputMap::default()
        .with_dual_axis(
            PaintSkiesAction::Rotate,
            GamepadStick::LEFT.with_deadzone_symmetric(0.1),
        )
        .with_dual_axis(
            PaintSkiesAction::Rotate,
            MouseMove::default().sensitivity(0.15).inverted_y(),
        );

    command_spawn((
        input_map,
        Camera3d::default(),
        PaintSkiesCamera,
        SphericalCoordsBounds {
            max_phi: 3.0 * PI / 8.0,
            min_phi: -3.0 * PI / 8.0,
        },
        LookAtSphericalCoords::default(),
        Paintable,
        Camera {
            order: 2,
            clear_color: ClearColorConfig::None,
            target: RenderTarget::from((**render_target).clone()),
            ..default()
        },
    ))
}

pub fn rotate_spherical_coords(settings: Res<PaintSkiesSettings>) -> impl Effect + use<> {
    let rotate_sensitivity = settings.rotate_sensitivity;

    components_set_filtered_with_query_data::<
        _,
        _,
        (&ActionState<PaintSkiesAction>, &SphericalCoordsBounds),
        With<PaintSkiesCamera>,
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
