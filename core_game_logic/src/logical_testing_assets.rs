use crate::{
    cards::{CardFunction, CardId, LogicalCard},
    markets::{CardPrice, LogicalMarket, MarketId},
    orders::{LogicalOrder, OrderId},
    pieces::{ArchetypeId, LogicalPieceArchetype, Orders},
    tiles::TileType,
};
// only used in testing
#[allow(unused)]
pub const LOGICAL_CARDS_FOR_TESTING: [LogicalCard; 6] = [
    LogicalCard {
        functionality: CardFunction::TileConversionToSingleType {
            selection_bounds: 1..3,
            target_type: TileType::Basic,
        },
    },
    LogicalCard {
        functionality: CardFunction::TileConversionToSingleType {
            selection_bounds: 1..1,
            target_type: TileType::Ex1,
        },
    },
    LogicalCard {
        functionality: CardFunction::SpawnPiece {
            selection_bounds: 1..2,
            piece_archetype: ArchetypeId(1),
        },
    },
    LogicalCard {
        functionality: CardFunction::SpawnMarket {
            selection_bounds: 1..3,
            market: MarketId(0),
        },
    },
    LogicalCard {
        functionality: CardFunction::SpawnPiece {
            selection_bounds: 1..1,
            piece_archetype: ArchetypeId(2),
        },
    },
    LogicalCard {
        functionality: CardFunction::SpawnPiece {
            selection_bounds: 1..1,
            piece_archetype: ArchetypeId(0),
        },
    },
];
//only used in testing
#[allow(unused)]
pub const LOGICAL_MARKETS_FOR_TESTING: [LogicalMarket; 2] = [
    LogicalMarket([
        (CardId(0), CardPrice(3)),
        (CardId(2), CardPrice(5)),
        (CardId(3), CardPrice(1)),
    ]),
    LogicalMarket([
        (CardId(1), CardPrice(3)),
        (CardId(4), CardPrice(5)),
        (CardId(5), CardPrice(1)),
    ]),
];
//only used in testing.
#[allow(unused)]
pub const STARTING_CARDS_FOR_TESTING: [CardId; 2] = [CardId(1), CardId(3)];

//only used in testing.
#[allow(unused)]
pub const LOGICAL_PIECES_FOR_TESTING: [crate::pieces::LogicalPieceArchetype; 3] = [
    LogicalPieceArchetype {
        max_health: 23,
        starting_orders_per_round: 1,
        orders: Orders([
            Some(OrderId(0)),
            Some(OrderId(2)),
            Some(OrderId(1)),
            None,
            None,
        ]),
    },
    LogicalPieceArchetype {
        max_health: 1,
        starting_orders_per_round: 2,
        orders: Orders([
            Some(OrderId(1)),
            Some(OrderId(0)),
            Some(OrderId(1)),
            None,
            None,
        ]),
    },
    LogicalPieceArchetype {
        max_health: 12,
        starting_orders_per_round: 1,
        orders: Orders([Some(OrderId(1)), None, None, None, None]),
    },
];

pub const LOGICAL_ORDERS_FOR_TESTING: [LogicalOrder; 3] = [
    LogicalOrder {
        functionality: crate::orders::OrderFunction::Ex1,
    },
    LogicalOrder {
        functionality: crate::orders::OrderFunction::ConvertAdjacentTiles {
            selection_range: 1..1,
            target_type: TileType::Basic,
        },
    },
    LogicalOrder {
        functionality: crate::orders::OrderFunction::ConvertAdjacentTiles {
            selection_range: 1..3,
            target_type: TileType::Ex1,
        },
    },
];
