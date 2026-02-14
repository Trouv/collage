mod plugin;
pub use plugin::PaintSkiesPlugin;

mod settings;

mod spherical_coords;
pub use spherical_coords::{LookAtSphericalCoords, SphericalCoordsBounds};

mod control_spherical_coords;

mod paint_meshes;
pub use paint_meshes::Paintable;

mod triangle_with_uvs;

mod paint_layer_history;
