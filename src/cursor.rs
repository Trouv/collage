use bevy::prelude::*;
use bevy::window::{CursorGrabMode, CursorOptions};
use bevy_pipe_affect::prelude::*;

#[derive(PartialEq, Eq, Debug, Default, Copy, Clone, Resource, Reflect)]
pub enum CursorLock {
    #[default]
    Lock,
    Unlock,
}

/// When this system runs, the cursor is locked/invisible.
pub fn lock_cursor(
    cursor_lock: Res<CursorLock>,
) -> QueryAffect<ComponentSet<CursorOptions>, With<Window>> {
    let cursor_options = match *cursor_lock {
        CursorLock::Lock => CursorOptions {
            visible: false,
            grab_mode: CursorGrabMode::Locked,
            ..default()
        },
        CursorLock::Unlock => CursorOptions::default(),
    };
    query_affect(component_set(cursor_options))
}
