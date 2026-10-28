use bevy::prelude::*;
use bevy_obj::ObjPlugin;
use bevy_renet::{
    RenetClientPlugin, RenetServerPlugin,
    netcode::{NetcodeClientPlugin, NetcodeServerPlugin},
};
use core_game_logic::players::PlayerId;

use crate::{
    cameras::CameraManagementPlugin, inputs_interface::InputInterfacePlugin,
    main_menu::MainMenuAndLobbyPluggin, ui_panels::UiPanelsPlugin,
    vis_effect_reactions::VisEffectReactions, vis_markets::VisMarketsPlugin,
    vis_pieces::VisPiecesPlugin, vis_tiles::VisTilesPlugin,
};

const VERSION_NUMBER: u64 = 0;

mod cameras;
mod functional_assets;
mod inputs_interface;
mod main_menu;
mod ui_panels;
mod vis_effect_reactions;
mod vis_markets;
mod vis_pieces;
mod vis_tiles;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            MeshPickingPlugin,
            VisTilesPlugin,
            InputInterfacePlugin,
            UiPanelsPlugin,
            VisPiecesPlugin,
            VisMarketsPlugin,
            ObjPlugin,
            VisEffectReactions,
            MainMenuAndLobbyPluggin,
            RenetClientPlugin,
            RenetServerPlugin,
            NetcodeClientPlugin,
            NetcodeServerPlugin,
            CameraManagementPlugin,
        ))
        .init_state::<AppState>()
        .run();
}

/// The player that the operator of the device is representing. A spectator of a match would be Option::None, since they are not acting as a player, just a spectator. However, they will have a display player, so that the game can know which player's inventory and stats to display to the spectator.
#[derive(Debug, Resource)]
pub struct OperatingPlayer(PlayerId);

#[derive(Debug, States, Clone, Copy, Default, Eq, Hash, PartialEq)]
pub enum AppState {
    #[default]
    MainMenu,
    ParametersScreen,
    PreGame,
    InGame,
}
