use bevy::ecs::{component::Component, entity::Entity, resource::Resource, world::World};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{
    cards::CardId,
    pieces::{OccupiesTile, OrdersPerRound, OwnsLogPieces},
    requests::{ActionEffect, ChangeLog},
    tile_mapping::TileId,
};

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Deserialize, Serialize, Component)]
pub struct PlayerId(pub u8);

#[derive(Debug, Resource)]
pub struct PlayerDirectory(Box<[Entity]>);

#[derive(Debug, Error)]
#[error("Tried to get a player with an invalid id: {:?}", self.0.0)]
pub struct InvalidIdErr(PlayerId);

impl PlayerDirectory {
    pub fn get_player(&self, id: PlayerId) -> Result<Entity, InvalidIdErr> {
        self.0.get(id.0 as usize).copied().ok_or(InvalidIdErr(id))
    }

    pub fn read(&self) -> &[Entity] {
        &self.0
    }
}

pub const STARTING_COINS: u32 = 25;
pub const INVENTORY_MAX_SIZE: u8 = 5;

pub fn initialize_players(world: &mut World, player_count: u8, starting_cards: &[CardId]) {
    let players = world
        .spawn_batch((0..player_count).map(|id| {
            (
                PlayerId(id),
                Inventory {
                    hand: starting_cards.to_vec(),
                    max_size: INVENTORY_MAX_SIZE,
                },
                Coins(STARTING_COINS),
                PlayerState::HasNoWinConditionYet,
            )
        }))
        .collect::<Vec<Entity>>()
        .into_boxed_slice();

    world.insert_resource(PlayerDirectory(players));
}

#[derive(Debug, Component)]
pub struct Inventory {
    hand: Vec<CardId>,
    max_size: u8,
}

#[derive(Debug, Error)]
pub enum InventoryError {
    #[error("Tried to add a card to a player's inventory, but their inventory was full.")]
    InventoryIsFull,
    #[error("The inventory did not have a card at that index.")]
    InvalidIndex(InventoryIndex),
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Deserialize, Serialize)]
pub struct InventoryIndex(pub u8);

impl Inventory {
    pub(crate) fn try_add_card(&mut self, card: CardId) -> Result<(), InventoryError> {
        if (self.hand.len() as u8) < self.max_size {
            self.hand.push(card);
            Ok(())
        } else {
            Err(InventoryError::InventoryIsFull)
        }
    }

    pub fn get_card(&self, index_in_inventory: InventoryIndex) -> Result<CardId, InventoryError> {
        self.hand
            .get(index_in_inventory.0 as usize)
            .copied()
            .ok_or(InventoryError::InvalidIndex(index_in_inventory))
    }

    /// Note that removing a card will invalidate any data holding an index for the inventory.
    pub(crate) fn remove_card(&mut self, index_in_inventory: InventoryIndex) {
        self.hand.remove(index_in_inventory.0 as usize);
    }
}

#[derive(Debug, Component)]
pub struct Coins(pub u32);

#[derive(Debug, Resource)]
pub struct ActivePlayer(pub PlayerId);

pub fn apply_start_turn_effects(world: &mut World, player: PlayerId) -> ChangeLog {
    ChangeLog::default()
}

pub fn apply_end_turn_effects(
    world: &mut World,
    player: PlayerId,
) -> Result<ChangeLog, InvalidIdErr> {
    let player_ent = world.resource::<PlayerDirectory>().get_player(player)?;

    let mut log = ChangeLog::default();

    if let Some(owned_pieces) = world.get::<OwnsLogPieces>(player_ent) {
        for piece in owned_pieces.list().clone() {
            let tile_id = world
                .get::<TileId>(
                    world
                        .get::<OccupiesTile>(piece)
                        .expect("all pieces should have an id.")
                        .0,
                )
                .expect("all tile entities must have a TileId")
                .clone();

            let mut order_information = world.get_mut::<OrdersPerRound>(piece).expect(
                "Pieces should always have information about how many orders they can use.",
            );

            for _ in 0..(order_information.per_round - order_information.currently) {
                log.write(ActionEffect::GaveOrderToPiece {
                    tile_of_piece: tile_id,
                });
            }

            order_information.currently = order_information.per_round;
        }
    }

    Ok(ChangeLog::default())
}
#[derive(Debug, Component, PartialEq, Eq)]
pub enum PlayerState {
    HasNoWinConditionYet,
    Alive,
    Dead,
}
