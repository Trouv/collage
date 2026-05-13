use bevy::prelude::*;

/// Various settings for behavior of the `PaintSkiesPlugin`.
#[derive(Debug, Copy, Clone, PartialEq, Reflect, Resource)]
pub struct PaintSkiesSettings {
    /// Mouse/left stick sensitivity.
    pub rotate_sensitivity: f32,
}

impl Default for PaintSkiesSettings {
    fn default() -> Self {
        PaintSkiesSettings {
            rotate_sensitivity: 0.02,
        }
    }
}
