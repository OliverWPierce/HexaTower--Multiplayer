use bevy::ecs::{component::Component, entity::Entity, resource::Resource, world::World};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{
    cards::CardId,
    pieces::{GivesExtraPlayerOrder, OccupiesTile, OrdersReceivable, OwnsPieces},
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

    pub fn list(&self) -> &[Entity] {
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
                PlayerOrdersRemaining { remaining: 2 },
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

    pub fn all_cards(&self) -> &[CardId] {
        &self.hand
    }

    pub fn max_card_capacity(&self) -> u8 {
        self.max_size
    }
}

#[derive(Debug, Component)]
pub struct Coins(pub u32);

#[derive(Debug, Resource)]
pub struct ActivePlayer(pub PlayerId);

pub fn apply_start_turn_effects(
    world: &mut World,
    player: PlayerId,
) -> Result<ChangeLog, InvalidIdErr> {
    let player_ent = world.resource::<PlayerDirectory>().get_player(player)?;

    let mut orders_to_give = 2;
    let mut log = ChangeLog::default();

    log.write(ActionEffect::IncreasedRemainingOrdersOfPlayer {
        receipient: player,
        source: None,
    });
    log.write(ActionEffect::IncreasedRemainingOrdersOfPlayer {
        receipient: player,
        source: None,
    });

    if let Some(players_pieces) = world.get::<OwnsPieces>(player_ent) {
        for piece in players_pieces.list() {
            if world.get::<GivesExtraPlayerOrder>(*piece).is_some() {
                orders_to_give += 1;
                log.write(ActionEffect::IncreasedRemainingOrdersOfPlayer {
                    receipient: player,
                    source: Some(
                        *world
                            .get::<TileId>(world.get::<OccupiesTile>(*piece).unwrap().0)
                            .unwrap(),
                    ),
                });
            }
        }
    }

    world
        .get_mut::<PlayerOrdersRemaining>(player_ent)
        .unwrap()
        .remaining += orders_to_give;
    // we add instead of simply setting so that it is easy to allow other players to "gift" an order later on in development, if playtesting finds that beneficial. This also avoids visual bugs.

    Ok(ChangeLog::default())
}

pub fn apply_end_turn_effects(
    world: &mut World,
    player: PlayerId,
) -> Result<ChangeLog, InvalidIdErr> {
    let player_ent = world.resource::<PlayerDirectory>().get_player(player)?;

    let mut log = ChangeLog::default();

    {
        let mut player_orders = world.get_mut::<PlayerOrdersRemaining>(player_ent).unwrap();

        for _ in 0..player_orders.remaining {
            log.write(ActionEffect::ReducedRemaingOrdersOfPlayer(player));
        }
        player_orders.remaining = 0;
    }

    if let Some(owned_pieces) = world.get::<OwnsPieces>(player_ent) {
        for piece in owned_pieces.list().iter().copied().collect::<Box<[_]>>() {
            let tile_id = *world
                .get::<TileId>(
                    world
                        .get::<OccupiesTile>(piece)
                        .expect("all pieces should have an id.")
                        .0,
                )
                .expect("all tile entities must have a TileId");

            let mut order_information = world.get_mut::<OrdersReceivable>(piece).expect(
                "Pieces should always have information about how many orders they can use.",
            );

            let difference_in_orders =
                order_information.per_round as i8 - order_information.currently as i8;

            match difference_in_orders.signum() {
                1 => {
                    for _ in 0..difference_in_orders.abs() {
                        log.write(ActionEffect::IncreasedRemainingOrdersOfPiece {
                            tile_of_piece: tile_id,
                        });
                    }
                }
                -1 => {
                    for _ in 0..difference_in_orders.abs() {
                        log.write(ActionEffect::ReducedRemainingOrdersOfPiece {
                            tile_of_piece: tile_id,
                        });
                    }
                }
                _ => (),
            }

            order_information.currently = order_information.per_round;
        }
    }

    Ok(log)
}
#[derive(Debug, Component, PartialEq, Eq)]
pub enum PlayerState {
    HasNoWinConditionYet,
    Alive,
    Dead,
}

#[derive(Debug, Component)]
pub struct PlayerOrdersRemaining {
    pub remaining: u8,
}
