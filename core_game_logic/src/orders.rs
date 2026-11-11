use std::ops::Range;

use bevy::ecs::{resource::Resource, world::World};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{
    requests::ActionProcessCache,
    tile_based_actions::{
        TileAction, TileActionProcessCache,
        change_tile_type::{AdjecentRestriction, ConvertTileTo},
        damage_forward::PieceAttacksForward,
        damage_piece::PieceAttacksAdjacent,
        give_self_coins::GiveOwnerCoins,
        move_piece::{MovePiece, MovementMethod},
        rotate_a_piece::RotatePiece,
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
    BasicMovement(MovementMethod),
    DamageAdjacent {
        depth: u8,
        selection_range: Range<usize>,
        damage: u32,
    },
    GiveOwnedCurrency {
        ammount: u32,
    },
    Rotate,
    DamageForward {
        depth: u8,
        selection_range: Range<usize>,
        damage: u32,
    },
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
            OrderFunction::BasicMovement(method) => Ok(TileActionProcessCache::initialize(
                TileAction::new(
                    MovePiece {
                        piece_on_tile: piece_occupies_tile,
                        method: *method,
                    },
                    1..1,
                )?,
                world,
            )
            .into()),
            OrderFunction::DamageAdjacent {
                depth,
                selection_range,
                damage,
            } => Ok(ActionProcessCache::from(
                TileActionProcessCache::initialize(
                    TileAction::new(
                        PieceAttacksAdjacent {
                            attacker_occupies_tile: piece_occupies_tile,
                            damage: *damage,
                            adjacency_depth: *depth,
                        },
                        selection_range.clone(),
                    )?,
                    world,
                ),
            )),
            OrderFunction::GiveOwnedCurrency { ammount } => Ok(ActionProcessCache::TileAction(
                TileActionProcessCache::initialize(
                    TileAction::new(
                        GiveOwnerCoins {
                            tile_of_piece: piece_occupies_tile,
                            ammount: *ammount,
                        },
                        1..1,
                    )?,
                    world,
                ),
            )),
            OrderFunction::Rotate => Ok(ActionProcessCache::TileAction(
                TileActionProcessCache::initialize(
                    TileAction::new(
                        RotatePiece {
                            piece_on_tile: piece_occupies_tile,
                        },
                        1..1,
                    )?,
                    world,
                ),
            )),
            OrderFunction::DamageForward {
                depth,
                selection_range,
                damage,
            } => Ok(ActionProcessCache::from(
                TileActionProcessCache::initialize(
                    TileAction::new(
                        PieceAttacksForward {
                            attacker_occupies_tile: piece_occupies_tile,
                            damage: *damage,
                            depth: *depth,
                        },
                        selection_range.clone(),
                    )?,
                    world,
                ),
            )),
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
    pub fn get_order(&self, id: OrderId) -> Result<&LogicalOrder, InvaildIDErr> {
        self.0.get(id.0 as usize).ok_or(InvaildIDErr(id))
    }
    pub fn new(orders: Box<[LogicalOrder]>) -> Self {
        OrderDirectory(orders)
    }
}

pub fn initialize_orders(world: &mut World, orders: Box<[LogicalOrder]>) {
    world.insert_resource(OrderDirectory(orders));
}
