use bevy::prelude::*;
use bevy_pipe_affect::prelude::*;
use leafwing_input_manager::prelude::*;

use crate::clear_skies::ClearSkiesState;
use crate::clear_skies::paint_skies::paint_meshes::PaintMeshesPlugin;
use crate::clear_skies::paint_skies::player::{
    PaintSkiesAction,
    rotate_spherical_coords,
    spawn_player,
    switch_gamepads,
};
use crate::clear_skies::paint_skies::settings::PaintSkiesSettings;
use crate::clear_skies::paint_skies::spherical_coords::look_at_spherical_coords;

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, Hash, Reflect)]
pub struct PaintSkiesPlugin;

impl Plugin for PaintSkiesPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            InputManagerPlugin::<PaintSkiesAction>::default(),
            PaintMeshesPlugin,
        ))
        .init_resource::<PaintSkiesSettings>()
        .add_systems(OnEnter(ClearSkiesState::Setup), spawn_player.pipe(affect))
        .add_systems(
            FixedUpdate,
            (
                switch_gamepads.pipe(affect),
                rotate_spherical_coords.pipe(affect),
                look_at_spherical_coords.pipe(affect),
            )
                .run_if(in_state(ClearSkiesState::PaintSkies)),
        );
    }
}
