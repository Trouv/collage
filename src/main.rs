use bevy::asset::AssetMetaCheck;
use bevy::pbr::wireframe::{WireframeConfig, WireframePlugin};
use bevy::prelude::*;
#[cfg(feature = "dev")]
use bevy_inspector_egui::bevy_egui::EguiPlugin;
#[cfg(feature = "dev")]
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use clap::Parser;

use crate::args::DevArgs;
use crate::clear_skies::ClearSkiesPlugin;

mod state;

mod clear_skies;

mod args;

fn main() {
    let args = if cfg!(feature = "dev") {
        let args = DevArgs::parse();
        dbg!(&args);
        args
    } else {
        default()
    };

    let mut app = App::new();

    app.add_plugins((
        DefaultPlugins
            .set(AssetPlugin {
                // Wasm builds will check for meta files (that don't exist) if this isn't set.
                // This causes errors and even panics in web builds on itch.
                // See https://github.com/bevyengine/bevy_github_ci_template/issues/48.
                meta_check: AssetMetaCheck::Never,
                ..default()
            })
            .set(ImagePlugin::default_nearest()),
        ClearSkiesPlugin,
    ))
    .init_state::<state::GameState>();

    if args.wireframe {
        app.add_plugins(WireframePlugin::default())
            .insert_resource(WireframeConfig {
                global: true,
                ..default()
            });
    }

    #[cfg(feature = "dev")]
    if args.inspector {
        app.add_plugins(EguiPlugin::default())
            .add_plugins(WorldInspectorPlugin::new());
    }

    app.run();
}
