use bevy::{ecs::world, prelude::*};
use core_game_logic::{
    pieces::{GetsFreeRotation, OccupiesTile},
    players::InventoryIndex,
    requests::{
        ActionEffect, ActionProcessCache, BackendRequest, InputData, RequestType,
        try_consume_request,
    },
    tile_mapping::TileId,
};

use crate::{
    OperatingPlayer,
    functional_assets::LogicalWorld,
    ui_panels::OrderAtPieceIndex,
    vis_pieces::{
        EndFreeRotationAnimation, PieceSpawned, RotatePieceMessage, StartFreeRotationAnimation,
    },
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
        ActionEffect::GaveFreeRotationComponent { to_piece_on_tile } => {
            commands.write_message(StartFreeRotationAnimation {
                on_tile: to_piece_on_tile,
            });
        }
        ActionEffect::RemovedFreeRotationComponent { from_piece_on_tile } => {
            commands.write_message(EndFreeRotationAnimation {
                on_tile: from_piece_on_tile,
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
        _ => warn!("Display method not yet implemented..."),
    }
}
#[derive(Debug, Resource)]
pub struct LoadedAction {
    pub source: Source,
    pub cache: ActionProcessCache,
    pub is_immediately_mandatory: bool,
}

#[derive(Debug)]
pub enum Source {
    Card(InventoryIndex),
    Order(OrderAtPieceIndex),
    FreePieceRotation,
}

#[derive(Event, Debug)]
pub struct TryExecuteLoadedAction;

fn try_execute_loaded_action(
    _trigger: On<TryExecuteLoadedAction>,
    mut loaded_action: ResMut<LoadedAction>,
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
                                        input: InputData::AffectedTiles(
                                            tile_action_process_cache.selected_tiles().into(),
                                        ),
                                    },
                    Source::Order(order_index) => RequestType::UseOrder {
                                        tile: active_tile.ok_or("Tried to execute an order while there was no active tile. An active tile is needed to tell which piece the order is being used on.")?.0,
                                        input: InputData::AffectedTiles(
                                            tile_action_process_cache.selected_tiles().into(),
                                        ),
                                        index_of_order: order_index.0,
                                    },
                    Source::FreePieceRotation => return Err("It never makes sense for a free rotation to contain tile_action data".into()),
                }
            }
            ActionProcessCache::RotationAction {
                tile_data,
                selected_direction,
            } => {
                let piece_to_rotate_occupies = match tile_data {
                core_game_logic::requests::RotationTileStates::ElligibleTiles(..) => return Err("attempted to request a rotation action of the backend, but the rotation cache did not contain a selected tile. The request was not sent".into()),
                core_game_logic::requests::RotationTileStates::SelectedTile(tile_id) => *tile_id,
            };
                let direction = selected_direction.ok_or("attempted to request a rotation action of the backend, but the rotation cache did not contain a selected direction. The request was not sent.")?;

                match loaded_action.source {
                Source::Card(inventory_index) => RequestType::UseCard {
                                inventory_index,
                                input: InputData::RotatePiece { on_tile: piece_to_rotate_occupies,
                                towards_direction: direction}},

                Source::Order(order_index) => RequestType::UseOrder {
                                tile: active_tile.ok_or("Tried to execute an order while there was no active tile. An active tile is needed to tell which piece the order is being used on.")?.0,
                                input:InputData::RotatePiece { on_tile: piece_to_rotate_occupies, towards_direction: direction},
                                index_of_order: order_index.0,
                            },
                Source::FreePieceRotation => RequestType::FreePieceRotation { on_tile: piece_to_rotate_occupies, direction, },
            }
            }
        }
    };

    let change_log = try_consume_request(
        BackendRequest {
            acting_player: acting_player.0,
            request,
        },
        &mut logical_world.0,
    )?;

    println!("Changelog is as follows: {change_log:?}");

    for item in change_log.read() {
        write_message(item.clone(), &mut commands);
    }

    if let Some(OccupiesTile(logical_tile_of_a_piece_needing_rotation)) = logical_world.0.try_query_filtered::<&OccupiesTile, With<GetsFreeRotation>>().expect("components for which tile a piece occupies and for specifying whether a piece gets a free rotation should have already been registered.").iter(&logical_world.0).next(){
        let tile_id = *logical_world.0.get::<TileId>(*logical_tile_of_a_piece_needing_rotation).ok_or("logical tile had no component storing its tile id.")?;

        *loaded_action = LoadedAction{ source: Source::FreePieceRotation, cache: ActionProcessCache::RotationAction { tile_data: core_game_logic::requests::RotationTileStates::SelectedTile(tile_id), selected_direction: None }, is_immediately_mandatory: true };

        commands.insert_resource(ActiveTile(tile_id));
        return Ok(())
    }

    commands.remove_resource::<LoadedAction>();

    Ok(())
}
