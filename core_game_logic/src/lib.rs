pub use bevy::*;

use crate::tiles::initialize_tiles;
pub mod player_actions;
pub mod tile_based_actions;
pub mod tile_mapping;
pub mod tiles;

pub struct CreationSettings {
    board_size: u32,
}

impl CreationSettings {
    pub fn new(board_size: u32) -> CreationSettings {
        CreationSettings { board_size }
    }

    pub fn create_board(self) -> bevy::ecs::world::World {
        let mut logical_world = bevy::ecs::world::World::new();

        initialize_tiles(&mut logical_world, self.board_size);

        logical_world
    }
}
