use bevy::ecs::world::World;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{
    InvalidEntityState,
    cards::{CardDirectory, CardId},
    markets::{MarketDirectory, MarketId},
    pieces::{OccupiedByPiece, PieceOwnedByPlayer},
    players::{
        self, ActivePlayer, Coins, Inventory, InventoryIndex, PlayerDirectory, PlayerId,
        PlayerState,
    },
    tile_based_actions::TileActionProcessCache,
    tile_mapping::TileId,
    tiles::{MarketTile, TileDirectory, TileType},
};

#[derive(Debug, PartialEq)]
pub enum ActionEffect {
    ConvertedTileType {
        tile: TileId,
        new_type: TileType,
    },
    SpawnedNewMarket {
        tile: TileId,
        market: MarketId,
    },
    SpawnedPiece {
        tile: TileId,
        player: PlayerId,
        archetype: crate::pieces::ArchetypeId,
    },
    AddedCardToInventory {
        player: PlayerId,
        card: CardId,
    },
    AlteredCoins {
        player: PlayerId,
        delta_coins: i32,
    },
    EndedTurn(PlayerId),
    BeganTurn(PlayerId),
    RemovedCardFromInventory {
        player: PlayerId,
        index: InventoryIndex,
    },
}
#[derive(Default)]
pub struct ChangeLog(Vec<ActionEffect>);

impl ChangeLog {
    pub fn read(&self) -> &Vec<ActionEffect> {
        &self.0
    }

    pub fn write(&mut self, effect: ActionEffect) {
        self.0.push(effect);
    }

    pub fn append(&mut self, second_log: &mut ChangeLog) {
        self.0.append(&mut second_log.0);
    }
}

impl From<Vec<ActionEffect>> for ChangeLog {
    fn from(effects: Vec<ActionEffect>) -> Self {
        Self(effects)
    }
}

pub enum ActionProcessCache {
    TileAction(TileActionProcessCache),
    Ex1,
}

impl From<TileActionProcessCache> for ActionProcessCache {
    fn from(cache: TileActionProcessCache) -> Self {
        ActionProcessCache::TileAction(cache)
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct BackendRequest {
    pub acting_player: PlayerId,
    pub request: RequestType,
}
#[derive(Debug, Deserialize, Serialize, Clone)]
pub enum RequestType {
    UseCard {
        inventory_index: InventoryIndex,
        input: InputData,
    },
    PurchaseCard {
        market_tile: TileId,
        index_of_card: u8,
    },
    EndTurn,
}
#[derive(Debug, Deserialize, Serialize, Clone)]
pub enum InputData {
    AffectedTiles(Box<[TileId]>),
    Ex1,
}
#[derive(Debug, Error)]
#[error(
    "The request contained a certain type of additional input data, but it did not match the input expected by the action."
)]
struct UnexpectedInputType;

#[derive(Debug, Error)]
pub enum PurchaseCardError {
    #[error("Tile {0:?} is not a market tile.")]
    TileIsNotMarket(TileId),
    #[error("The player does not have a piece occupying tile {0:?}.")]
    PlayerDoesNotOccupyTile(TileId),
    #[error("The player cannot afford this card.")]
    NotEnoughCoins,
    #[error("No card availible at this index at the market tile.")]
    IndexOutOfBounds,
}

pub fn try_consume_request(
    action_to_process: BackendRequest,
    world: &mut World,
) -> anyhow::Result<ChangeLog> {
    match action_to_process.request {
        RequestType::UseCard {
            inventory_index,
            input,
        } => {
            let player_ent = world
                .resource::<PlayerDirectory>()
                .get_player(action_to_process.acting_player)?;

            let card = world
                .get::<Inventory>(player_ent)
                .ok_or(InvalidEntityState)?
                .get_card(inventory_index)?;

            let action_cache = world
                .resource::<CardDirectory>()
                .get_card(card)?
                .functionality
                .action_cache(world)?;

            let mut log = match action_cache {
                ActionProcessCache::TileAction(mut tile_action_process_cache) => {
                    let InputData::AffectedTiles(tiles) = input else {
                        return Result::Err(UnexpectedInputType.into());
                    };

                    for id in tiles {
                        tile_action_process_cache
                            .try_select_tile_and_update_elligibility(id, world)?
                    }

                    tile_action_process_cache.try_execute(world)?
                }
                ActionProcessCache::Ex1 => todo!(),
            };

            world
                .get_mut::<Inventory>(player_ent)
                .unwrap()
                .remove_card(inventory_index);

            log.write(ActionEffect::RemovedCardFromInventory {
                player: action_to_process.acting_player,
                index: inventory_index,
            });

            Ok(log)
        }
        RequestType::PurchaseCard {
            market_tile,
            index_of_card,
        } => {
            let tile_entity = world.resource::<TileDirectory>().get_entity(market_tile)?;
            let player_ent = world
                .resource::<PlayerDirectory>()
                .get_player(action_to_process.acting_player)?;

            if let Some(occupying_piece) = world.get::<OccupiedByPiece>(tile_entity)
                && let Some(owner) = world.get::<PieceOwnedByPlayer>(occupying_piece.piece())
                && player_ent == owner.0
            {
                let market = world
                    .get::<MarketTile>(tile_entity)
                    .ok_or(PurchaseCardError::TileIsNotMarket(market_tile))?
                    .0;

                let (card, price) = *world
                    .resource::<MarketDirectory>()
                    .get_market(market)?
                    .0
                    .get(index_of_card as usize)
                    .ok_or(PurchaseCardError::IndexOutOfBounds)?;

                if world.get::<Coins>(player_ent).unwrap().0 >= price.0 {
                    world
                        .get_mut::<Inventory>(player_ent)
                        .unwrap()
                        .try_add_card(card)?;
                    world.get_mut::<Coins>(player_ent).unwrap().0 -= price.0;

                    let mut log = ChangeLog::default();

                    log.write(ActionEffect::AddedCardToInventory {
                        player: action_to_process.acting_player,
                        card,
                    });

                    log.write(ActionEffect::AlteredCoins {
                        player: action_to_process.acting_player,
                        delta_coins: -(price.0 as i32),
                    });

                    Ok(log)
                } else {
                    Err(PurchaseCardError::NotEnoughCoins)?
                }
            } else {
                Err(PurchaseCardError::PlayerDoesNotOccupyTile(market_tile))?
            }
        }
        RequestType::EndTurn => {
            let exiting_player = world.resource::<ActivePlayer>().0;

            let mut change_log = ChangeLog::default();

            change_log.write(ActionEffect::EndedTurn(exiting_player));

            change_log.append(&mut players::apply_end_turn_effects(world, exiting_player));

            let next_player = {
                let (preceding_players, next_players) = world
                    .resource::<PlayerDirectory>()
                    .read()
                    .split_at(exiting_player.0 as usize);

                *world.get::<PlayerId>(*next_players.iter().skip(1).chain(preceding_players).find(|player| *world.get::<PlayerState>(**player).unwrap() != PlayerState::Dead).expect("Tried to end turn, but all players were dead (except perhaps the active player.) This indicates the game is over, which should have been handled by another system. (Players cannot end their turn when the game is over).")).ok_or(InvalidEntityState)?
            };

            world.resource_mut::<ActivePlayer>().0 = next_player;

            change_log.write(ActionEffect::BeganTurn(next_player));

            change_log.append(&mut players::apply_start_turn_effects(world, next_player));

            Ok(change_log)
        }
    }
}
