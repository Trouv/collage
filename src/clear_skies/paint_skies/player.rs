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

use crate::clear_skies::camera::{PaintSkiesAction, PaintSkiesCamera};
use crate::clear_skies::paint_skies::settings::PaintSkiesSettings;
use crate::clear_skies::paint_skies::spherical_coords::{
    LookAtSphericalCoords,
    SphericalCoordsBounds,
};

pub fn switch_gamepads<A: Actionlike>(
    mut gamepad_events: MessageReader<GamepadEvent>,
) -> Vec<impl Effect + use<A>> {
    gamepad_events
        .read()
        .map(|event| {
            let (GamepadEvent::Connection(GamepadConnectionEvent {
                gamepad: entity, ..
            })
            | GamepadEvent::Button(GamepadButtonChangedEvent { entity, .. })
            | GamepadEvent::Axis(GamepadAxisChangedEvent { entity, .. })) = event;

            let entity = *entity;

            components_set_with::<_, _>(move |(input_map,): (InputMap<A>,)| {
                (input_map.with_gamepad(entity),)
            })
        })
        .collect()
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
