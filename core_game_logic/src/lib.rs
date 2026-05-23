pub mod cards;
mod logical_testing_assets;
pub mod markets;
pub mod orders;
pub mod pieces;
pub mod players;
pub mod requests;
pub mod tile_based_actions;
pub mod tile_mapping;
pub mod tiles;

use crate::{
    cards::{CardId, LogicalCard, initialize_cards},
    markets::LogicalMarket,
    orders::{LogicalOrder, initialize_orders},
    pieces::{LogicalPieceArchetype, initialize_pieces},
    players::{ActivePlayer, PlayerId, initialize_players},
    requests::ChangeLog,
    tiles::initialize_tiles,
};

pub struct CreationParameters {
    pub board_size: u32,
    pub player_count: u8,
    pub all_cards: Box<[LogicalCard]>,
    pub all_markets: Box<[LogicalMarket]>,
    pub piece_archetypes: Box<[LogicalPieceArchetype]>,
    pub starting_cards: Box<[CardId]>,
    pub orders: Box<[LogicalOrder]>,
}

impl CreationParameters {
    pub fn create_logical_world(self) -> (bevy::ecs::world::World, ChangeLog) {
        let mut logical_world = bevy::ecs::world::World::new();

        initialize_tiles(&mut logical_world, self.board_size);
        initialize_cards(&mut logical_world, self.all_cards);
        initialize_players(&mut logical_world, self.player_count, &self.starting_cards);
        initialize_orders(&mut logical_world, self.orders);
        initialize_pieces(&mut logical_world, self.piece_archetypes);

        logical_world.insert_resource(ActivePlayer(PlayerId(0)));
        let log = players::apply_start_turn_effects(&mut logical_world, PlayerId(0)).unwrap();

        (logical_world, log)
    }

    // this is only used in testing; nevertheless, I don't want to duplicate this code everywhere so I'm putting it here.
    #[allow(unused)]
    fn testing_default() -> bevy::ecs::world::World {
        let parameters = CreationParameters {
            board_size: 4,
            player_count: 3,
            all_cards: logical_testing_assets::LOGICAL_CARDS_FOR_TESTING.into(),
            all_markets: logical_testing_assets::LOGICAL_MARKETS_FOR_TESTING.into(),
            starting_cards: logical_testing_assets::STARTING_CARDS_FOR_TESTING.into(),
            piece_archetypes: logical_testing_assets::LOGICAL_PIECES_FOR_TESTING.into(),
            orders: logical_testing_assets::LOGICAL_ORDERS_FOR_TESTING.into(),
        };
        parameters.create_logical_world().0
    }
}
