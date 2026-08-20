use bevy::prelude::*;
use bevy::window::{CursorGrabMode, CursorOptions};
use bevy_pipe_affect::prelude::*;

/// When this system runs, the cursor is locked/invisible.
pub fn lock_cursor() -> QueryAffect<ComponentSet<CursorOptions>, With<Window>> {
    query_affect(component_set(CursorOptions {
        visible: false,
        grab_mode: CursorGrabMode::Locked,
        ..default()
    }))
}
