use thiserror::Error;

pub mod cards;
mod logical_testing_assets;
pub mod markets;
pub mod players;
pub mod requests;

pub mod tile_based_actions;
pub mod tile_mapping;
pub mod tiles;

use crate::{
    cards::{CardId, LogicalCard, initialize_cards},
    markets::LogicalMarket,
    players::initialize_players,
    tiles::initialize_tiles,
};

pub struct CreationParameters {
    pub board_size: u32,
    pub player_count: u8,
    pub all_cards: Box<[LogicalCard]>,
    pub all_markets: Box<[LogicalMarket]>,
    pub starting_cards: Box<[CardId]>,
}

#[derive(Debug, Error)]
#[error(
    "An entity was in a supposedly unreachable state. For example, this could be when a player lacks an inventory component."
)]
pub struct InvalidEntityState;

impl CreationParameters {
    pub fn create_logical_world(self) -> bevy::ecs::world::World {
        let mut logical_world = bevy::ecs::world::World::new();

        initialize_tiles(&mut logical_world, self.board_size);
        initialize_cards(&mut logical_world, self.all_cards);
        initialize_players(&mut logical_world, self.player_count, &self.starting_cards);
        logical_world
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
        };
        parameters.create_logical_world()
    }
}
