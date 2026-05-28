use bevy::prelude::*;
use core_game_logic::{
    cards::{CardDirectory, CardId},
    requests::{ActionEffect, ActionProcessCache},
    tile_mapping::TileId,
};

use crate::{
    functional_assets::LogicalWorld,
    vis_tiles::{TileActionProcess, TileTypeConverted},
};

pub struct InputInterfacePlugin;

impl Plugin for InputInterfacePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, tmp_show_selection_indicators);
    }
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

fn tmp_show_selection_indicators(
    inputs: Res<ButtonInput<KeyCode>>,
    log_world: Res<LogicalWorld>,
    res: Option<ResMut<TileActionProcess>>,
    mut commands: Commands,
) {
    if inputs.just_pressed(KeyCode::KeyA) {
        let process_cache = log_world
            .0
            .resource::<CardDirectory>()
            .get_card(CardId(0))
            .unwrap()
            .functionality
            .action_cache(&log_world.0)
            .unwrap();

        match process_cache {
            ActionProcessCache::TileAction(tile_action_process_cache) => {
                commands.insert_resource(TileActionProcess(tile_action_process_cache))
            }
            ActionProcessCache::Ex1 => unimplemented!(),
        }
    } else if inputs.just_pressed(KeyCode::KeyB) {
        commands.remove_resource::<TileActionProcess>();
    } else if inputs.pressed(KeyCode::KeyC) {
        let Some(mut res) = res else { return };
        _ = res
            .0
            .try_select_tile_and_update_elligibility(TileId::new(5), &log_world.0);
    }
}
