use bevy::prelude::*;
use core_game_logic::{
    players::InventoryIndex,
    requests::{
        ActionEffect, ActionProcessCache, BackendRequest, InputData, RequestType,
        try_consume_request,
    },
};

use crate::{
    OperatingPlayer, functional_assets::LogicalWorld, vis_pieces::PieceSpawned,
    vis_tiles::TileTypeConverted,
};

pub struct InputInterfacePlugin;

impl Plugin for InputInterfacePlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(try_execute_loaded_action);
    }
}

fn write_message(effect: ActionEffect, commands: &mut Commands) {
    match effect {
        ActionEffect::ConvertedTileType { tile, new_type } => {
            commands.write_message(TileTypeConverted { tile, new_type });
        }
        ActionEffect::SpawnedPiece {
            tile,
            player,
            archetype,
        } => {
            commands.write_message(PieceSpawned {
                tile,
                owner: player,
                archetype,
            });
        }
        _ => warn!("Display method not yet implemented..."),
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

#[derive(Event, Debug)]
pub struct TryExecuteLoadedAction;

fn try_execute_loaded_action(
    _trigger: On<TryExecuteLoadedAction>,
    loaded_action: Res<LoadedAction>,
    acting_player: Res<OperatingPlayer>,
    mut logical_world: ResMut<LogicalWorld>,
    mut commands: Commands,
) -> Result<(), BevyError> {
    let request = {
        match &loaded_action.cache {
            ActionProcessCache::TileAction(tile_action_process_cache) => {
                match loaded_action.source {
                    Source::Card(inventory_index) => RequestType::UseCard {
                        inventory_index,
                        input: InputData::AffectedTiles(
                            tile_action_process_cache.selected_tiles().into(),
                        ),
                    },
                }
            }
            ActionProcessCache::Ex1 => todo!(),
        }
    };

    let change_log = try_consume_request(
        BackendRequest {
            acting_player: acting_player.0.ok_or("User is not operating as a player")?,
            request,
        },
        &mut logical_world.0,
    )?;

    commands.remove_resource::<LoadedAction>();

    println!("Changelog is as follows: {change_log:?}");

    for item in change_log.read() {
        write_message(item.clone(), &mut commands);
    }

    Ok(())
}
