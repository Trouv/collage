use bevy::camera::RenderTarget;
use bevy::prelude::*;
use bevy_pipe_affect::prelude::*;

use crate::clear_skies::camera::ClearSkiesRenderTarget;
use crate::clear_skies::render_layers::PAINTED_LAYER;

#[derive(Default, Debug, PartialEq, Eq, Copy, Clone, Hash, Reflect, Component)]
#[require(Name = "PlaySkiesCamera")]
pub struct PlaySkiesCamera;

pub fn spawn_camera(render_target: Res<ClearSkiesRenderTarget>) -> impl Effect + use<> {
    command_spawn((
        Camera3d::default(),
        PlaySkiesCamera,
        PAINTED_LAYER,
        Camera {
            order: 1,
            clear_color: ClearColorConfig::Custom(Color::srgb(0.0, 0.4, 1.0)),
            ..default()
        },
        Msaa::Off,
        RenderTarget::from((**render_target).clone()),
    ))
}
