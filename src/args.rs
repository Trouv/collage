use clap::Parser;

/// CLI arguments only available to dev builds of this game.
#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, Hash, Parser)]
pub struct DevArgs {
    /// Enable wireframes on meshes.
    #[arg(short, long, env)]
    pub wireframe: bool,
    /// Enable bevy_inspector_egui.
    #[arg(short, long, env)]
    pub inspector: bool,
    /// Add the ability to spawn a free camera with Ctrl+f.
    #[arg(short, long, env)]
    pub free_cam: bool,
}
