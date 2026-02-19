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

pub fn control_spherical_coords(
    settings: Res<PaintSkiesSettings>,
) -> ComponentsSetFilteredWithQueryData<
    (LookAtSphericalCoords,),
    (
        &'static ActionState<PaintSkiesAction>,
        &'static SphericalCoordsBounds,
    ),
    With<PaintSkiesCamera>,
> {
    let rotate_sensitivity = settings.rotate_sensitivity;

    components_set_filtered_with_query_data(
        move |(spherical_coords,): (LookAtSphericalCoords,),
              (action_state, bounds): (&ActionState<PaintSkiesAction>, &SphericalCoordsBounds)| {
            let rotate_by = action_state.clamped_axis_pair(&PaintSkiesAction::Rotate);

            let phi = (spherical_coords.phi + (rotate_by.y * rotate_sensitivity))
                .clamp(bounds.min_phi, bounds.max_phi);
            let theta = (spherical_coords.theta - (rotate_by.x * rotate_sensitivity)) % (2.0 * PI);

            (LookAtSphericalCoords { phi, theta },)
        },
    )
}
