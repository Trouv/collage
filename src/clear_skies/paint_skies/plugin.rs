use bevy::prelude::*;
use bevy_pipe_affect::prelude::*;

use crate::clear_skies::ClearSkiesState;
use crate::clear_skies::camera::PaintSkiesAction;
use crate::clear_skies::paint_skies::paint_meshes::PaintMeshesPlugin;
use crate::clear_skies::paint_skies::player::rotate_spherical_coords;
use crate::clear_skies::paint_skies::settings::PaintSkiesSettings;
use crate::clear_skies::paint_skies::spherical_coords::look_at_spherical_coords;
use crate::clear_skies::switch_gamepads::SwitchGamepadsPlugin;

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, Hash, Reflect)]
pub struct PaintSkiesPlugin;

impl Plugin for PaintSkiesPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            SwitchGamepadsPlugin::<PaintSkiesAction>::default(),
            PaintMeshesPlugin,
        ))
        .init_resource::<PaintSkiesSettings>()
        .add_systems(
            FixedUpdate,
            (
                rotate_spherical_coords.pipe(affect),
                look_at_spherical_coords.pipe(affect),
            )
                .run_if(in_state(ClearSkiesState::PaintSkies)),
        );
    }
}
