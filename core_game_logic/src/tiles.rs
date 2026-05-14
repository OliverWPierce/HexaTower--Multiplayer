use bevy::ecs::{
    component::{Component, Immutable, Mutable, StorageType},
    entity::Entity,
    resource::Resource,
    world::World,
};

use thiserror::Error;

use crate::tile_mapping::TileId;

pub fn initialize_tiles(world: &mut World, ring_count: u32) {
    let ordered_tiles = world
        .spawn_batch(
            (0..(3 * (ring_count + 1) * ring_count + 1))
                .map(|id| (TileId::new(id), TileType::Basic)),
        )
        .collect::<Vec<Entity>>()
        .into_boxed_slice();

    world.insert_resource(TileDirectory(ordered_tiles));
}

#[derive(Debug, Resource)]
pub struct TileDirectory(Box<[Entity]>);

impl crate::directories::Directory for TileDirectory {
    type Id = TileId;

    type Contains = Entity;

    fn get(
        &self,
        id: Self::Id,
    ) -> Result<&Self::Contains, crate::directories::InvalidIdErr<Self::Id>> {
        self.0
            .get(id.id() as usize)
            .ok_or(crate::directories::InvalidIdErr(id))
    }
}

impl TileDirectory {
    // This is the total number of tiles on the board, as you would count them.
    pub fn tile_count(&self) -> usize {
        self.0.len()
    }
}

impl Component for TileId {
    const STORAGE_TYPE: bevy::ecs::component::StorageType = StorageType::Table;

    type Mutability = Immutable;
}
#[derive(Debug, Clone, PartialEq)]
pub enum TileType {
    Basic,
    Ex1,
}

impl Component for TileType {
    const STORAGE_TYPE: bevy::ecs::component::StorageType = StorageType::Table;

    type Mutability = Mutable;
}
