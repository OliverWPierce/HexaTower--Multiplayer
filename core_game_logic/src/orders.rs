use std::ops::Range;

use bevy::ecs::{resource::Resource, world::World};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{
    requests::ActionProcessCache,
    tile_based_actions::{
        TileAction, TileActionProcessCache,
        change_tile_type::{AdjecentRestriction, ConvertTileTo},
    },
    tile_mapping::TileId,
    tiles::TileType,
};
#[derive(Debug)]
pub enum OrderFunction {
    ConvertAdjacentTiles {
        selection_range: Range<usize>,
        target_type: TileType,
    },
    Ex1,
}

#[derive(Debug, Error)]
pub enum OrderFunctionConversionError {
    #[error("Order function could not be converted to a TileAction; Invalid selection bounds.")]
    CouldNotConvertToTileAction(#[from] crate::tile_based_actions::InvalidSelectionBounds),
}

impl OrderFunction {
    pub fn action_cache(
        &self,
        piece_occupies_tile: TileId,
        world: &World,
    ) -> Result<ActionProcessCache, OrderFunctionConversionError> {
        match self {
            OrderFunction::ConvertAdjacentTiles {
                selection_range,
                target_type,
            } => Ok(TileActionProcessCache::initialize(
                TileAction::new(
                    ConvertTileTo {
                        target_type: target_type.clone(),
                        restrictions: Some(AdjecentRestriction {
                            adjacent_to: piece_occupies_tile,
                        }),
                    },
                    selection_range.clone(),
                )?,
                world,
            )
            .into()),
            OrderFunction::Ex1 => todo!(),
        }
    }
}

#[derive(Debug)]
pub struct LogicalOrder {
    pub functionality: OrderFunction,
}
/// The index of an order in the order directory.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Deserialize, Serialize)]
pub struct OrderId(pub u32);

#[derive(Debug, Resource)]
pub struct OrderDirectory(Box<[LogicalOrder]>);

#[derive(Debug, Error)]
#[error{"Tried to retrieve order data using an invalid OrderId {0:?}."}]
pub struct InvaildIDErr(pub OrderId);

impl OrderDirectory {
    pub fn get_card(&self, id: OrderId) -> Result<&LogicalOrder, InvaildIDErr> {
        self.0.get(id.0 as usize).ok_or(InvaildIDErr(id))
    }
    pub fn new(orders: Box<[LogicalOrder]>) -> Self {
        OrderDirectory(orders)
    }
}

pub fn initialize_orders(world: &mut World, orders: Box<[LogicalOrder]>) {
    world.insert_resource(OrderDirectory(orders));
}
