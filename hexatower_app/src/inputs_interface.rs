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

mod loaded_action_invariance {
    use bevy::prelude::Resource;
    use core_game_logic::{
        markets::SlotInMarket, players::InventoryIndex, requests::ActionProcessCache,
        tile_mapping::TileId,
    };
    use thiserror::Error;

    use crate::ui_panels::OrderAtPieceIndex;

    #[derive(Debug, Resource, Default)]
    pub struct PlayerActionInputSequence {
        active_tile: Option<TileId>,
        loaded_action: Option<FrontendAction>,
    }

    #[derive(Debug, Error)]
    pub enum LoadActionError {
        #[error("The action {0:?} requires a tile to be active.")]
        ActiveTileMissing(FrontendAction),
    }

    #[derive(Debug)]
    pub enum FrontendAction {
        UseCard {
            index: InventoryIndex,
            cache: ActionProcessCache,
        },
        UseOrder {
            index_of_order_on_active_piece: OrderAtPieceIndex,
            cache: ActionProcessCache,
        },
        PurchaseCard {
            slot: SlotInMarket,
        },
    }

    impl Foo {
        pub fn active_tile(&self) -> Option<TileId> {
            self.active_tile
        }

        pub fn process_cache(&self) -> Option<&ActionProcessCache> {
            if self.loaded_action.is_none() {
                return None;
            }

            match self.loaded_action.as_ref().unwrap() {
                FrontendAction::UseCard { cache, .. } => Some(cache),
                FrontendAction::UseOrder { cache, .. } => Some(cache),
                FrontendAction::PurchaseCard { .. } => None,
            }
        }
        pub fn try_load_action(&mut self, action: FrontendAction) -> Result<(), LoadActionError> {
            match &action {
                FrontendAction::UseCard { .. } => {
                    self.loaded_action = Some(action);
                    Ok(())
                }
                _ => {
                    if self.active_tile.is_none() {
                        return Err(LoadActionError::ActiveTileMissing(action));
                    }

                    self.loaded_action = Some(action);

                    Ok(())
                }
            }
        }

        pub fn set_active_tile(&mut self, tile: Option<TileId>) {
            self.active_tile = tile;

            if self.loaded_action.is_none() {
                return;
            }

            match self.loaded_action.as_ref().unwrap() {
                FrontendAction::UseCard { .. } => (),
                _ => self.loaded_action = None,
            }
        }
    }
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
