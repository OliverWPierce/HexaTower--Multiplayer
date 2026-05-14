use thiserror::Error;

pub mod cards;
pub mod players;
pub mod requests;

pub mod tile_based_actions;
pub mod tile_mapping;
pub mod tiles;

use crate::{
    cards::{CardAsset, initialize_cards},
    players::initialize_players,
    tiles::initialize_tiles,
};

pub struct CreationSettings {
    board_size: u32,
    player_count: u8,
}
#[derive(Debug, Error)]
#[error(
    "An entity was in a supposedly unreachable state. For example, this could be when a player lacks an inventory component."
)]
pub struct InvalidEntityState;

impl CreationSettings {
    pub fn new(board_size: u32, player_count: u8) -> CreationSettings {
        CreationSettings {
            board_size,
            player_count,
        }
    }

    pub fn create_logical_world(self) -> bevy::ecs::world::World {
        let mut logical_world = bevy::ecs::world::World::new();

        initialize_tiles(&mut logical_world, self.board_size);
        initialize_cards(&mut logical_world);
        initialize_players(&mut logical_world, self.player_count);
        logical_world
    }
}
