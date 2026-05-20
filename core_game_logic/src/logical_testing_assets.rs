use crate::{
    cards::{CardFunction, CardId, LogicalCard},
    markets::{CardPrice, LogicalMarket, MarketId},
    tiles::TileType,
};
// only used in testing
#[allow(unused)]
pub const LOGICAL_CARDS_FOR_TESTING: [LogicalCard; 4] = [
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
        functionality: CardFunction::Ex2,
    },
    LogicalCard {
        functionality: CardFunction::SpawnMarket {
            selection_bounds: 1..3,
            market: MarketId(0),
        },
    },
];
//only used in testing
#[allow(unused)]
pub const LOGICAL_MARKETS_FOR_TESTING: [LogicalMarket; 1] = [LogicalMarket([
    (CardId(0), CardPrice(3)),
    (CardId(2), CardPrice(5)),
    (CardId(3), CardPrice(1)),
])];
//only used in testing.
#[allow(unused)]
pub const STARTING_CARDS_FOR_TESTING: [CardId; 2] = [CardId(1), CardId(3)];
