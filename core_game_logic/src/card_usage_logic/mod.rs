use std::ops::Range;

use bevy::ecs::world::World;
use thiserror::Error;

use crate::{
    player_actions::ActionProcessCache,
    tile_based_actions::{
        self, TileAction, TileActionProcessCache, change_tile_type::ConvertTileTo,
    },
    tiles::{TileDirectory, TileType},
};

enum CardFunction {
    TileConversionToSingleType {
        selection_bounds: Range<usize>,
        target_type: TileType,
    },
    Ex2,
}

#[derive(Debug, Error)]
pub enum CardFunctionConversionError {
    #[error("Attempted to select a tile of invalid id. Id number {0}")]
    CouldNotConvertToTileAction(#[from] tile_based_actions::InvalidSelectionBounds),
}

impl CardFunction {
    fn action_cache(
        &self,
        world: &World,
    ) -> Result<ActionProcessCache, CardFunctionConversionError> {
        let cache = match self {
            CardFunction::TileConversionToSingleType {
                selection_bounds,
                target_type,
            } => {
                let tiles_on_board = world.resource::<TileDirectory>().tile_count();

                ActionProcessCache::TileAction(TileActionProcessCache::initialize(
                    TileAction::new(
                        ConvertTileTo {
                            target_type: target_type.clone(),
                        },
                        selection_bounds.clone(),
                    )?,
                    tiles_on_board,
                    world,
                ))
            }
            CardFunction::Ex2 => todo!(),
        };

        Ok(cache)
    }
}
