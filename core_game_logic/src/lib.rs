pub mod cards;
pub mod forensic_action_descriptions;
pub mod logical_testing_assets;
pub mod markets;
pub mod orders;
pub mod pieces;
pub mod players;
pub mod requests;
pub mod tile_based_actions;
pub mod tile_mapping;
pub mod tiles;

use std::fmt::Debug;

use crate::{
    cards::{CardId, LogicalCard, initialize_cards},
    markets::{LogicalMarket, MarketDirectory, MarketId},
    orders::{LogicalOrder, initialize_orders},
    pieces::{LogicalPieceArchetype, initialize_pieces},
    players::{ActivePlayer, PlayerDirectory, initialize_players},
    requests::{ActionEffect, ChangeLog},
    tile_mapping::TileIdServer,
    tiles::{MarketTile, TileDirectory, TileType, initialize_tiles},
};

pub struct CreationParameters {
    pub board_size: u32,
    pub player_count: u8,
    pub all_cards: Box<[LogicalCard]>,
    pub all_markets: Box<[(LogicalMarket, Box<[u32]>)]>,
    pub piece_archetypes: Box<[LogicalPieceArchetype]>,
    pub starting_cards: Box<[CardId]>,
    pub orders: Box<[LogicalOrder]>,
    pub initial_special_tiles: Box<[(Box<[u32]>, TileType)]>,
}

impl CreationParameters {
    pub fn create_logical_world(self) -> (bevy::ecs::world::World, ChangeLog) {
        let mut logical_world = bevy::ecs::world::World::new();

        initialize_tiles(&mut logical_world, self.board_size);
        initialize_cards(&mut logical_world, self.all_cards);
        initialize_players(&mut logical_world, self.player_count, &self.starting_cards);
        initialize_orders(&mut logical_world, self.orders);
        initialize_pieces(&mut logical_world, self.piece_archetypes);

        logical_world.insert_resource(MarketDirectory::new(
            self.all_markets.iter().map(|tuple| tuple.0).collect(),
        ));

        let mut log = ChangeLog::default();

        self.all_markets
            .iter()
            .enumerate()
            .for_each(|(id, (_, tiles))| {
                let market_id = MarketId(id as u32);
                for tile in tiles {
                    let tile_server = logical_world.resource::<TileIdServer>();

                    let tile = tile_server
                        .construct_tile_id(*tile)
                        .expect("Tile should exist.");

                    logical_world
                        .entity_mut(logical_world.resource::<TileDirectory>().get_entity(tile))
                        .insert(MarketTile(market_id));
                    log.write(ActionEffect::SpawnedNewMarket {
                        tile,
                        market: market_id,
                    });
                }
            });

        self.initial_special_tiles
            .iter()
            .for_each(|(tiles, tile_type)| {
                for tile in tiles {
                    let tile_server = logical_world.resource::<TileIdServer>();

                    let tile = tile_server
                        .construct_tile_id(*tile)
                        .expect("Tile should exist.");

                    *logical_world
                        .get_mut::<TileType>(
                            logical_world.resource::<TileDirectory>().get_entity(tile),
                        )
                        .expect("All tiles have a type.") = *tile_type;

                    log.write(ActionEffect::ConvertedTileType {
                        tile,
                        new_type: *tile_type,
                    })
                }
            });

        let starting_player = logical_world
            .resource::<PlayerDirectory>()
            .make_id(0)
            .unwrap();

        logical_world.insert_resource(ActivePlayer(starting_player));
        log.append(&mut players::apply_start_turn_effects(
            &mut logical_world,
            starting_player,
        ));

        (logical_world, log)
    }

    // this is only used in testing; nevertheless, I don't want to duplicate this code everywhere so I'm putting it here.

    pub fn testing_default() -> bevy::ecs::world::World {
        // let parameters = CreationParameters {
        //     board_size: 4,
        //     player_count: 3,
        //     all_cards: logical_testing_assets::LOGICAL_CARDS_FOR_TESTING.into(),
        //     all_markets: logical_testing_assets::LOGICAL_MARKETS_FOR_TESTING.into(),
        //     starting_cards: logical_testing_assets::STARTING_CARDS_FOR_TESTING.into(),
        //     piece_archetypes: logical_testing_assets::LOGICAL_PIECES_FOR_TESTING.into(),
        //     orders: logical_testing_assets::LOGICAL_ORDERS_FOR_TESTING.into(),
        // };
        // parameters.create_logical_world().0
        todo!()
    }
}

pub trait IndexingId: Debug {}

#[derive(Debug, thiserror::Error)]
#[error("Tried to create an indexing id, but it was invalid. Attempted to create id: {0:?}")]
pub struct InvalidIdErr<I: IndexingId>(I);

impl<I: IndexingId> InvalidIdErr<I> {
    fn new(id: I) -> Self {
        Self(id)
    }
}
