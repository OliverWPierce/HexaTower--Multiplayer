use bevy::ecs::{component::Component, entity::Entity, resource::Resource, world::World};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::cards::CardId;

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
}

pub fn initialize_players(world: &mut World, player_count: u8, starting_cards: &[CardId]) {
    let players = world
        .spawn_batch((0..player_count).map(|id| {
            (
                PlayerId(id),
                Inventory {
                    hand: starting_cards.to_vec(),
                    max_size: 5,
                },
                Coins(25),
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
