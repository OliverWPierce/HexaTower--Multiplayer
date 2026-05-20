use bevy::ecs::{component::Component, entity::Entity, resource::Resource, world::World};

use thiserror::Error;

use crate::{markets::MarketId, tile_mapping::TileId};

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

#[derive(Debug, Error)]
#[error{"Tried to get a tile which was out of bounds for this board size."}]
pub struct InvaildIDErr(pub TileId);

impl TileDirectory {
    pub fn get_entity(&self, tile_id: TileId) -> Result<Entity, InvaildIDErr> {
        self.0
            .get(tile_id.id() as usize)
            .copied()
            .ok_or(InvaildIDErr(tile_id))
    }

    pub fn tile_entities(&self) -> &[Entity] {
        &self.0
    }

    pub fn id_entity_pairs(&self) -> impl Iterator<Item = (TileId, Entity)> {
        self.0
            .iter()
            .enumerate()
            .map(|(id, ent)| (TileId::new(id as u32), *ent))
    }

    // This is the total number of tiles on the board, as you would count them.
    pub fn tile_count(&self) -> usize {
        self.0.len()
    }
}

#[derive(Debug, Component, Clone, PartialEq)]
pub enum TileType {
    Basic,
    Ex1,
}

#[derive(Debug, Component)]
#[component(storage = "SparseSet")]
pub struct MarketTile(pub MarketId);
