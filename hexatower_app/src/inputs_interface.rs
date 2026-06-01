use bevy::prelude::*;
use core_game_logic::{
    players::InventoryIndex,
    requests::{ActionEffect, ActionProcessCache},
};

use crate::vis_tiles::TileTypeConverted;

pub struct InputInterfacePlugin;

impl Plugin for InputInterfacePlugin {
    fn build(&self, app: &mut App) {}
}

fn write_message(effect: ActionEffect, commands: &mut Commands) {
    match effect {
        ActionEffect::ConvertedTileType { tile, new_type } => {
            commands.write_message(TileTypeConverted { tile, new_type });
        }
        _ => warn!(
            "Received an action effect from the logical world, but did not have a way to to display it to the player."
        ),
    }
}
#[derive(Debug, Resource)]
pub struct LoadedAction {
    pub source: Source,
    pub cache: ActionProcessCache,
}

#[derive(Debug)]
pub enum Source {
    Card(InventoryIndex),
}
