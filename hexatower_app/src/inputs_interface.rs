use bevy::prelude::*;
use core_game_logic::{
    markets::SlotInMarket,
    players::InventoryIndex,
    requests::{
        ActionEffect, ActionProcessCache, BackendRequest, InputData, RequestType,
        try_consume_request,
    },
};

use crate::{
    OperatingPlayer,
    functional_assets::LogicalWorld,
    ui_panels::OrderAtPieceIndex,
    vis_markets::MarketSpawned,
    vis_pieces::{PieceSpawned, RotatePieceMessage},
    vis_tiles::{ActiveTile, TileTypeConverted},
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
            facing_direction,
        } => {
            commands.write_message(PieceSpawned {
                tile,
                owner: player,
                archetype,
                direction: facing_direction,
            });
        }
        ActionEffect::PieceRotated {
            on_tile,
            new_rotation,
        } => {
            commands.write_message(RotatePieceMessage {
                on_tile,
                in_direction: new_rotation,
            });
        }
        ActionEffect::SpawnedNewMarket { tile, market } => {
            commands.write_message(MarketSpawned { tile, market });
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
    Order(OrderAtPieceIndex),
    Market(SlotInMarket),
}

#[derive(Event, Debug)]
pub struct TryExecuteLoadedAction;

fn try_execute_loaded_action(
    _trigger: On<TryExecuteLoadedAction>,
    loaded_action: Res<LoadedAction>,
    acting_player: Res<OperatingPlayer>,
    active_tile: Option<Res<ActiveTile>>,
    mut logical_world: ResMut<LogicalWorld>,
    mut commands: Commands,
) -> Result<(), BevyError> {
    let request = {
        match &loaded_action.cache {
            ActionProcessCache::TileAction(tile_action_process_cache) => {
                match loaded_action.source {
                    Source::Card(inventory_index) => RequestType::UseCard {
                                                inventory_index,
                                                input: InputData::SelectedTiles(
                                                    tile_action_process_cache.selected_tiles().into(),
                                                ),
                                            },
                    Source::Order(order_index) => RequestType::UseOrder {
                                                tile: active_tile.ok_or("Tried to execute an order while there was no active tile. An active tile is needed to tell which piece the order is being used on.")?.0,
                                                input: InputData::SelectedTiles(
                                                    tile_action_process_cache.selected_tiles().into(),
                                                ),
                                                index_of_order: order_index.0,
                                            },
                    Source::Market(_) => todo!(),
                }
            }
            ActionProcessCache::Ex1 => todo!(),
        }
    };

    let change_log = try_consume_request(
        BackendRequest {
            acting_player: acting_player.0,
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
