use bevy::{ecs::schedule::ScheduleLabel, prelude::*};
use core_game_logic::{CreationParameters, cards::LogicalCard};

use crate::vis_tiles::BoardSize;

pub struct StartupPlugin;

impl Plugin for StartupPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, tmp_startup);
    }
}
#[derive(Debug, ScheduleLabel, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SetUpBoard;

fn tmp_startup(mut commands: Commands) {
    commands.insert_resource(GameCreationSettings {
        board_size: BoardSize::Standard,
    });

    commands.insert_resource(LogicalWorld(CreationParameters::testing_default()));

    commands.run_schedule(SetUpBoard);
}

#[derive(Resource, Debug)]
pub struct LogicalWorld(pub World);

#[derive(Debug, Resource)]
pub struct GameCreationSettings {
    pub board_size: BoardSize,
}

struct HolisticCard {
    logical: LogicalCard,
    name: String,
    tooltip: String,
    image_path: String,
}
