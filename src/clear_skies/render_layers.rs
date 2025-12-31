use bevy::camera::visibility::RenderLayers;

/// For meshes that are painted from the paintable meshes.
pub const PAINTED_LAYER: RenderLayers = RenderLayers::layer(1);

/// For meshes that will be painted onto the painted layer.
pub const PAINTABLE_LAYER: RenderLayers = RenderLayers::layer(2);
