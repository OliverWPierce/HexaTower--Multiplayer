use bevy::{asset::LoadedAsset, ecs::schedule::ScheduleLabel, prelude::*, state::state};

use crate::pregame_loading::{PackPathsToLoad, PreGameLoadingPlugin};

mod pregame_loading;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, StartGamePlugin, PreGameLoadingPlugin))
        .run();
}

struct StartGamePlugin;

impl Plugin for StartGamePlugin {
    fn build(&self, app: &mut App) {
        app.insert_state::<AppState>(AppState::MainMenu);
        app.add_systems(Startup, tmp_startup);

        #[cfg(debug_assertions)]
        app.add_systems(OnEnter(AppState::InGame), tst_enter_game);
    }
}

#[derive(Debug, Resource)]
pub struct GameSetupInstructions {
    pub packs: PackPathsToLoad,
    pub board_size: u32,
    pub ex_player_names: Vec<String>,
}

fn tmp_startup(mut commands: Commands, mut state_changer: ResMut<NextState<AppState>>) {
    commands.insert_resource(GameSetupInstructions {
        packs: PackPathsToLoad(vec![String::from("CorePack.pack.ron")]),
        board_size: 5,
        ex_player_names: vec![
            String::from("Samantha"),
            String::from("JoeDaBoss"),
            String::from("Bob :)"),
        ],
    });

    state_changer.set(AppState::LoadingFunctionalAssets);
}

#[derive(Debug, Resource)]
pub struct LogicalWorld(pub World);

#[derive(Debug, States, PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
pub enum AppState {
    MainMenu,
    LoadingFunctionalAssets,
    InGame,
}

fn tst_enter_game() {
    info!("In the in-game state.")
}
