use std::ops::Range;

use bevy::ecs::{component::Component, world::World};
use thiserror::Error;

mod card_storage;
pub use card_storage::*;

use crate::{
    markets::MarketId,
    requests::ActionProcessCache,
    tile_based_actions::{
        self, TileAction, TileActionProcessCache, change_tile_type::ConvertTileTo,
    },
    tiles::TileType,
};

#[derive(Debug, Component, PartialEq)]
pub enum CardFunction {
    TileConversionToSingleType {
        selection_bounds: Range<usize>,
        target_type: TileType,
    },
    Ex2,
    SpawnMarket {
        selection_bounds: Range<usize>,
        market: MarketId,
    },
}

pub fn initialize_cards(world: &mut World) {
    let example_cards = [
        LogicalCard {
            functionality: CardFunction::Ex2,
        },
        LogicalCard {
            functionality: CardFunction::TileConversionToSingleType {
                selection_bounds: 1..2,
                target_type: TileType::Ex1,
            },
        },
    ];

    let mut constructor = CardAssetsConstructor::default();

    for card in example_cards {
        constructor.add_card(card);
    }

    world.insert_resource::<CardDirectory>(constructor.into());
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
            CardFunction::Ex2 => todo!(),
            CardFunction::SpawnMarket {
                selection_bounds,
                market,
            } => todo!(),
        };

        Ok(cache)
    }
}

// #[cfg(test)]
// mod tests {
//     use crate::{
//         CreationParameters,
//         cards::CardFunction,
//         tile_mapping::TileId,
//         tiles::{TileDirectory, TileType},
//     };

//     #[test]
//     fn convert_card_into_action_cache() {
//         let mut world = CreationParameters::new(4, 1).create_logical_world();

//         let card1 = CardFunction::TileConversionToSingleType {
//             selection_bounds: 1..4,
//             target_type: crate::tiles::TileType::Ex1,
//         };

//         let crate::requests::ActionProcessCache::TileAction(mut action) =
//             card1.action_cache(&world).unwrap()
//         else {
//             panic!()
//         };

//         action
//             .try_select_tile_and_update_elligibility(TileId::new(1), &world)
//             .unwrap();
//         action
//             .try_select_tile_and_update_elligibility(TileId::new(3), &world)
//             .unwrap();

//         action.try_execute(&mut world).unwrap();

//         let tile1 = world
//             .resource::<TileDirectory>()
//             .get_entity(TileId::new(1))
//             .unwrap();
//         let tile2 = world
//             .resource::<TileDirectory>()
//             .get_entity(TileId::new(2))
//             .unwrap();
//         let tile3 = world
//             .resource::<TileDirectory>()
//             .get_entity(TileId::new(3))
//             .unwrap();

//         assert_eq!(*world.get::<TileType>(tile1).unwrap(), TileType::Ex1);
//         assert_eq!(*world.get::<TileType>(tile2).unwrap(), TileType::Basic);
//         assert_eq!(*world.get::<TileType>(tile3).unwrap(), TileType::Ex1);
//     }
// }
