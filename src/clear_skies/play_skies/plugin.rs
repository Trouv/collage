use bevy::prelude::*;
use bevy_pipe_affect::prelude::*;

use crate::clear_skies::ClearSkiesState;
use crate::clear_skies::play_skies::camera::spawn_camera;

#[derive(Default, Debug, PartialEq, Eq, Copy, Clone, Hash, Reflect)]
pub struct PlaySkiesPlugin;

impl Plugin for PlaySkiesPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(ClearSkiesState::Setup), spawn_camera.pipe(affect));
    }
}
