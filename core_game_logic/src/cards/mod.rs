use std::ops::Range;

use bevy::ecs::{component::Component, world::World};
use thiserror::Error;

mod card_storage;
pub use card_storage::*;

use crate::{
    markets::MarketId,
    pieces::ArchetypeId,
    players::ActivePlayer,
    requests::ActionProcessCache,
    tile_based_actions::{
        self, TileAction, TileActionProcessCache, change_tile_type::ConvertTileTo,
        make_market_tile::MakeMarketTile, spawn_pieces::SpawnPieces,
    },
    tiles::TileType,
};

#[derive(Debug, Component, PartialEq)]
pub enum CardFunction {
    TileConversionToSingleType {
        selection_bounds: Range<usize>,
        target_type: TileType,
    },
    SpawnPiece {
        selection_bounds: Range<usize>,
        piece_archetype: ArchetypeId,
    },
    SpawnMarket {
        selection_bounds: Range<usize>,
        market: MarketId,
    },
}

pub fn initialize_cards(world: &mut World, cards: Box<[LogicalCard]>) {
    world.insert_resource::<CardDirectory>(CardDirectory::new(cards));
}

#[derive(Debug, Error)]
pub enum CardFunctionConversionError {
    #[error("Attempted to select a tile of invalid id. Id number {0}")]
    CouldNotConvertToTileAction(#[from] tile_based_actions::InvalidSelectionBounds),
}

impl CardFunction {
    pub fn action_cache(
        &self,
        world: &World,
    ) -> Result<ActionProcessCache, CardFunctionConversionError> {
        let cache = match self {
            CardFunction::TileConversionToSingleType {
                selection_bounds,
                target_type,
            } => TileActionProcessCache::initialize(
                TileAction::new(
                    ConvertTileTo {
                        target_type: target_type.clone(),
                    },
                    selection_bounds.clone(),
                )?,
                world,
            )
            .into(),
            CardFunction::SpawnMarket {
                selection_bounds,
                market,
            } => TileActionProcessCache::initialize(
                TileAction::new(MakeMarketTile { market: *market }, selection_bounds.clone())?,
                world,
            )
            .into(),
            CardFunction::SpawnPiece {
                selection_bounds,
                piece_archetype,
            } => TileActionProcessCache::initialize(
                TileAction::new(
                    SpawnPieces {
                        archetype: *piece_archetype,
                        owner: world.resource::<ActivePlayer>().0,
                    },
                    selection_bounds.clone(),
                )?,
                world,
            )
            .into(),
        };

        Ok(cache)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        CreationParameters,
        cards::CardFunction,
        tile_mapping::TileId,
        tiles::{TileDirectory, TileType},
    };

    #[test]
    fn convert_card_into_action_cache() {
        let mut world = CreationParameters::testing_default();

        let card1 = CardFunction::TileConversionToSingleType {
            selection_bounds: 1..4,
            target_type: crate::tiles::TileType::Ex1,
        };

        let crate::requests::ActionProcessCache::TileAction(mut action) =
            card1.action_cache(&world).unwrap()
        else {
            panic!()
        };

        action
            .try_select_tile_and_update_elligibility(TileId::new(1), &world)
            .unwrap();
        action
            .try_select_tile_and_update_elligibility(TileId::new(3), &world)
            .unwrap();

        action.try_execute(&mut world).unwrap();

        let tile1 = world
            .resource::<TileDirectory>()
            .get_entity(TileId::new(1))
            .unwrap();
        let tile2 = world
            .resource::<TileDirectory>()
            .get_entity(TileId::new(2))
            .unwrap();
        let tile3 = world
            .resource::<TileDirectory>()
            .get_entity(TileId::new(3))
            .unwrap();

        assert_eq!(*world.get::<TileType>(tile1).unwrap(), TileType::Ex1);
        assert_eq!(*world.get::<TileType>(tile2).unwrap(), TileType::Basic);
        assert_eq!(*world.get::<TileType>(tile3).unwrap(), TileType::Ex1);
    }
}
