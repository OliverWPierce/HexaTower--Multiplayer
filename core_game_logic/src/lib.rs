use thiserror::Error;

pub mod cards;
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
        initialize_cards(&mut logical_world);
        initialize_players(&mut logical_world, self.player_count);
        logical_world
    }
}
