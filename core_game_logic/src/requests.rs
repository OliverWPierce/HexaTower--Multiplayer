use bevy::ecs::world::World;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{
    InvalidEntityState,
    cards::CardDirectory,
    players::{Inventory, InventoryIndex, PlayerDirectory, PlayerId},
    tile_based_actions::TileActionProcessCache,
    tile_mapping::TileId,
    tiles::TileType,
};

#[derive(Debug, PartialEq)]
pub enum ActionEffect {
    DamagedPieceOnTile(TileId),
    ConvertedTileType { tile: TileId, new_type: TileType },
}
#[derive(Default)]
pub struct ChangeLog(Vec<ActionEffect>);

impl ChangeLog {
    pub fn read(&self) -> &Vec<ActionEffect> {
        &self.0
    }

    pub fn write(&mut self, effect: ActionEffect) {
        self.0.push(effect);
    }
}

pub enum ActionProcessCache {
    TileAction(TileActionProcessCache),
    Ex1,
}

impl From<TileActionProcessCache> for ActionProcessCache {
    fn from(cache: TileActionProcessCache) -> Self {
        ActionProcessCache::TileAction(cache)
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct BackendRequest {
    pub acting_player: PlayerId,
    pub request: RequestType,
}
#[derive(Debug, Deserialize, Serialize, Clone)]
pub enum RequestType {
    UseCard {
        inventory_index: InventoryIndex,
        input: InputData,
    },
}
#[derive(Debug, Deserialize, Serialize, Clone)]
enum InputData {
    AffectedTiles(Box<[TileId]>),
    Ex1,
}
#[derive(Debug, Error)]
#[error(
    "The request contained a certain type of additional input data, but it did not match the input expected by the action."
)]
struct UnexpectedInputType;

pub fn try_consume_request(
    action_to_process: BackendRequest,
    world: &mut World,
) -> anyhow::Result<ChangeLog> {
    match action_to_process.request {
        RequestType::UseCard {
            inventory_index,
            input,
        } => {
            let player_ent = world
                .resource::<PlayerDirectory>()
                .get_player(action_to_process.acting_player)?;

            let card = world
                .get::<Inventory>(player_ent)
                .ok_or(InvalidEntityState)?
                .get_card(inventory_index)?;

            let action_cache = world
                .resource::<CardDirectory>()
                .get_card(card)?
                .functionality
                .action_cache(world)?;

            match action_cache {
                ActionProcessCache::TileAction(mut tile_action_process_cache) => {
                    let InputData::AffectedTiles(tiles) = input else {
                        return Result::Err(UnexpectedInputType.into());
                    };

                    for id in tiles {
                        tile_action_process_cache
                            .try_select_tile_and_update_elligibility(id, world)?
                    }

                    tile_action_process_cache.try_execute(world)?
                }
                ActionProcessCache::Ex1 => todo!(),
            };
        }
    };

    todo!()
}
