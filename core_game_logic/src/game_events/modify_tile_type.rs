// use std::ops::Deref;

// use bevy::ecs::{entity::Entity, error::BevyError, system::IntoResult};
// use thiserror::Error;

// use crate::{
//     game_events::BoardEvent,
//     tile_mapping::TileId,
//     tiles::{self, TileDirectory, TileType},
// };

// pub struct ModifyTiles {
//     tiles: Vec<TileId>,
//     new_type: TileType,
// }

// impl BoardEvent for ModifyTiles {
//     fn validate(&self, world: &bevy::ecs::world::World) -> anyhow::Result<()> {
//         let directory = world.resource::<TileDirectory>();

//         for id in self.tiles.iter() {
//             directory.get_entity(*id)?;
//         }

//         Ok(())
//     }

//     fn consume(self, world: &mut bevy::ecs::world::World) -> anyhow::Result<()> {
//         let directory = world.resource::<TileDirectory>();

//         self.tiles.iter().try_for_each(|id| {
//             let a = directory.get_entity(*id)?;

//             let mut b = world.get_mut::<TileType>(a).ok_or(NoTileTypeErr)?;

//             Ok(())
//         })
//     }
// }
// #[derive(Debug, Error)]
// #[error("Expected an entity to have a tile type component.")]
// struct NoTileTypeErr;
