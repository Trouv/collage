use bevy::prelude::*;

#[derive(Debug, PartialEq, Copy, Clone, Deref, DerefMut, Reflect, Resource)]
pub struct ClearSkiesResolution(Vec2);

impl Default for ClearSkiesResolution {
    fn default() -> Self {
        ClearSkiesResolution(Vec2::new(480.0, 360.0))
    }
}
