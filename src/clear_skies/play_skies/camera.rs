use bevy::prelude::*;
use bevy_pipe_affect::prelude::*;

use crate::clear_skies::render_layers::PAINTED_LAYER;

#[derive(Default, Debug, PartialEq, Eq, Copy, Clone, Hash, Reflect, Component)]
#[require(Name = "PlaySkiesCamera")]
pub struct PlaySkiesCamera;

pub fn spawn_camera() -> impl Effect {
    command_spawn((
        Camera3d::default(),
        PlaySkiesCamera,
        Camera {
            order: 1,
            ..default()
        },
    ))
}
