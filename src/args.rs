use clap::Parser;

/// CLI arguments only available to dev builds of this game.
#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, Hash, Parser)]
pub struct DevArgs {
    /// Enable wireframes on meshes.
    #[arg(short, long, env)]
    pub wireframe: bool,
}
