use bevy::{ecs::schedule::ScheduleLabel, prelude::*};

use crate::logical_asset_loading::{LogicalAssetLoadingPlugin, PackPathsToLoad};

mod logical_asset_loading;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, LogicalAssetLoadingPlugin))
        .run();
}

struct StartGamePlugin;

impl Plugin for StartGamePlugin {
    fn build(&self, app: &mut App) {
        app.init_schedule(PreSetUpBoard);

        app.add_systems(Startup, tmp_startup);
    }
}

#[derive(Debug, Resource)]
pub struct GameSetupInstructions {
    pub packs: PackPathsToLoad,
}

#[derive(Debug, ScheduleLabel, Hash, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct PreSetUpBoard;

fn tmp_startup(mut commands: Commands) {
    commands.insert_resource(GameSetupInstructions { packs: todo!() });
    commands.run_schedule(PreSetUpBoard);
}
