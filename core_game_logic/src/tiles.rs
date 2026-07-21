use bevy::ecs::{component::Component, entity::Entity, resource::Resource, world::World};

use crate::{
    IndexingId, InvalidIdErr,
    markets::MarketId,
    tile_mapping::{TileId, TileIdServer},
};

pub fn initialize_tiles(world: &mut World, ring_count: u32) {
    let _ = world.register_component::<TileId>();
    let _ = world.register_component::<MarketTile>();

    let tile_server = TileIdServer {
        total_tiles_on_board: (3 * (ring_count + 1) * ring_count + 1),
    };

    let ordered_tiles = world
        .spawn_batch(
            (0..tile_server.total_tiles_on_board)
                .map(|id| (tile_server.construct_tile_id(id).unwrap(), TileType::Basic)),
        )
        .collect::<Vec<Entity>>()
        .into_boxed_slice();

    world.insert_resource(TileDirectory(ordered_tiles));
    world.insert_resource(tile_server);
}

impl IndexingId for TileId {}

#[derive(Debug, Resource)]
pub struct TileDirectory(Box<[Entity]>);

impl TileDirectory {
    pub fn get_entity(&self, tile: TileId) -> Entity {
        self.0[tile.id() as usize]
    }

    pub fn tile_entities(&self) -> &[Entity] {
        &self.0
    }

    pub fn id_entity_pairs(&self) -> impl Iterator<Item = (TileId, Entity)> {
        let server = TileIdServer {
            total_tiles_on_board: self.0.len() as u32,
        };

        self.0
            .iter()
            .enumerate()
            .map(move |(index, ent)| (server.construct_tile_id(index as u32).unwrap(), *ent))
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
