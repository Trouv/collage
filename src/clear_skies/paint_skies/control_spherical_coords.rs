use std::f32::consts::PI;

use bevy::prelude::*;
use bevy_pipe_affect::prelude::*;
use leafwing_input_manager::prelude::*;

use crate::clear_skies::camera::{PaintSkiesAction, PaintSkiesCamera};
use crate::clear_skies::paint_skies::settings::PaintSkiesSettings;
use crate::clear_skies::paint_skies::spherical_coords::{
    LookAtSphericalCoords,
    SphericalCoordsBounds,
};

/// Updates the [`LookAtSphericalCoords`] for the [`PaintSkiesCamera`] according to
/// [`PaintSkiesAction::Rotate`] input.
pub fn control_spherical_coords(
    settings: Res<PaintSkiesSettings>,
) -> QueryMap<
    (
        &'static LookAtSphericalCoords,
        &'static ActionState<PaintSkiesAction>,
        &'static SphericalCoordsBounds,
    ),
    ComponentSet<LookAtSphericalCoords>,
    With<PaintSkiesCamera>,
> {
    let rotate_sensitivity = settings.rotate_sensitivity;

    query_map(
        move |(spherical_coords, action_state, bounds): (
            &LookAtSphericalCoords,
            &ActionState<PaintSkiesAction>,
            &SphericalCoordsBounds,
        )| {
            let rotate_by = action_state.clamped_axis_pair(&PaintSkiesAction::Rotate);

            let phi = (spherical_coords.phi + (rotate_by.y * rotate_sensitivity))
                .clamp(bounds.min_phi, bounds.max_phi);
            let theta = (spherical_coords.theta - (rotate_by.x * rotate_sensitivity)) % (2.0 * PI);

            component_set(LookAtSphericalCoords { phi, theta })
        },
    )
}
