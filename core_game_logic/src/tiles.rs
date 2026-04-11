use bevy::ecs::{
    component::{Component, Immutable, Mutable, StorageType},
    entity::Entity,
    resource::Resource,
    world::World,
};

use crate::tile_mapping::TileId;

pub fn initialize_tiles(world: &mut World, ring_count: u32) {
    let ordered_tiles = world
        .spawn_batch(
            (0..(3 * (ring_count + 1) * ring_count + 1))
                .map(|id| (TileId::new(id), TileType::Basic)),
        )
        .collect::<Vec<Entity>>();

    world.insert_resource(TileDirectory(ordered_tiles));
}

#[derive(Debug, Resource)]
pub struct TileDirectory(Vec<Entity>);

impl Component for TileId {
    const STORAGE_TYPE: bevy::ecs::component::StorageType = StorageType::Table;

    type Mutability = Immutable;
}

pub enum TileType {
    Basic,
}

impl Component for TileType {
    const STORAGE_TYPE: bevy::ecs::component::StorageType = StorageType::Table;

    type Mutability = Mutable;
}
