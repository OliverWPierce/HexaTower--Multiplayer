use bevy::prelude::*;

use crate::{functional_assets::StartupPlugin, vis_tiles::VisTilesPlugin};

mod functional_assets;
mod vis_tiles;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, StartupPlugin, VisTilesPlugin))
        .run();
}
