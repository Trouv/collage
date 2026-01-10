use bevy::prelude::*;
use bevy_simple_screen_boxing::CameraBox;

#[derive(Debug, PartialEq, Copy, Clone, Deref, DerefMut, Reflect, Resource)]
pub struct ClearSkiesResolution(Vec2);

impl Default for ClearSkiesResolution {
    fn default() -> Self {
        ClearSkiesResolution(Vec2::new(480.0, 360.0))
    }
}

impl From<ClearSkiesResolution> for CameraBox {
    fn from(resolution: ClearSkiesResolution) -> Self {
        CameraBox::ResolutionIntegerScale {
            resolution: *resolution,
            allow_imperfect_downscaled_boxing: true,
        }
    }
}
