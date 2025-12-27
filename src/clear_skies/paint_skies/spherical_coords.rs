use bevy::prelude::*;
use bevy_pipe_affect::prelude::*;

#[derive(Debug, Default, Copy, Clone, PartialEq, Reflect, Component)]
pub struct SphericalCoordsBounds {
    pub max_phi: f32,
    pub min_phi: f32,
}

#[derive(Debug, Default, Copy, Clone, PartialEq, Reflect, Component)]
#[require(SphericalCoordsBounds, Transform)]
pub struct LookAtSphericalCoords {
    pub theta: f32,
    pub phi: f32,
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
