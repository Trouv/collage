use bevy::prelude::*;
use bevy_pipe_affect::prelude::*;
use bevy_simple_screen_boxing::CameraBox;

use crate::clear_skies::render_layers::PAINTED_LAYER;
use crate::clear_skies::resolution::ClearSkiesResolution;

#[derive(Default, Debug, PartialEq, Eq, Copy, Clone, Hash, Reflect, Component)]
#[require(Name = "PlaySkiesCamera")]
pub struct PlaySkiesCamera;

pub fn spawn_camera(resolution: Res<ClearSkiesResolution>) -> impl Effect + use<> {
    command_spawn((
        Camera3d::default(),
        PlaySkiesCamera,
        PAINTED_LAYER,
        CameraBox::from(*resolution),
        Camera {
            order: 1,
            clear_color: ClearColorConfig::Custom(Color::srgb(0.0, 0.4, 1.0)),
            ..default()
        },
    ))
}
