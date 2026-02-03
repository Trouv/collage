mod plugin;
pub use plugin::ClearSkiesPlugin;

mod state;
pub use state::ClearSkiesState;

mod transition;

pub mod paint_skies;

mod play_skies;

mod render_layers;

mod camera;

mod switch_gamepads;
