use bevy::prelude::*;

#[derive(Debug, Copy, Clone, PartialEq, Reflect, Resource)]
pub struct PaintSkiesSettings {
    pub rotate_sensitivity: f32,
}

impl Default for PaintSkiesSettings {
    fn default() -> Self {
        PaintSkiesSettings {
            rotate_sensitivity: 0.05,
        }
    }
}
