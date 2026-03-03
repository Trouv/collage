use bevy::camera::RenderTarget;
use bevy::camera::visibility::RenderLayers;
use bevy::prelude::*;
use bevy_pipe_affect::prelude::*;

use crate::clear_skies::camera::ClearSkiesRenderTarget;
use crate::clear_skies::render_layers::PAINTED_LAYER;

#[derive(Default, Debug, PartialEq, Eq, Copy, Clone, Hash, Reflect, Component)]
#[require(Name = "PlaySkiesCamera", Camera3d, RenderLayers = PAINTED_LAYER)]
pub struct PlaySkiesCamera;

pub fn spawn_camera(
    render_target: Res<ClearSkiesRenderTarget>,
) -> CommandSpawn<(PlaySkiesCamera, Camera, RenderTarget)> {
    command_spawn((
        PlaySkiesCamera,
        Camera {
            order: 1,
            clear_color: ClearColorConfig::Custom(Color::srgb(0.0, 0.4, 1.0)),
            ..default()
        },
        RenderTarget::from((**render_target).clone()),
    ))
}
