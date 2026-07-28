use bevy::{
    ecs::{entity::Entity, world::World},
    log::error,
};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{
    cards::{CardDirectory, CardId},
    forensic_action_descriptions::ForensicDescribe,
    markets::{MarketDirectory, MarketId, SlotInMarket},
    orders::OrderDirectory,
    pieces::{
        FacingHexDirection, IsWinCondition, OccupiedByPiece, Orders, OrdersReceivable, OwnsPieces,
        PieceOwnedByPlayer,
    },
    players::{
        self, ActivePlayer, Coins, InventoryIndex, LifeState, PlayerCardInventory, PlayerDirectory,
        PlayerId, PlayerOrdersRemaining, apply_start_turn_effects, compute_player_state,
    },
    tile_based_actions::{SelectedTile, TileActionProcessCache},
    tile_mapping::TileId,
    tiles::{MarketTile, TileDirectory, TileType},
};

#[derive(Debug, PartialEq, Clone)]
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
        facing_direction: FacingHexDirection,
    },
    AddedCardToInventory {
        player: PlayerId,
        card: CardId,
    },
    AlteredCoins {
        player: PlayerId,
        delta_coins: i32,
        from_tile: Option<TileId>,
    },
    EndedTurn(PlayerId),
    BeganTurn(PlayerId),
    RemovedCardFromInventory {
        player: PlayerId,
        index: InventoryIndex,
    },
    IncreasedRemainingOrdersOfPiece {
        tile_of_piece: TileId,
    },
    ReducedRemainingOrdersOfPiece {
        tile_of_piece: TileId,
    },
    ReducedRemaingOrdersOfPlayer(PlayerId),
    IncreasedRemainingOrdersOfPlayer {
        receipient: PlayerId,
        source: Option<TileId>,
    },
    DamagedPiece {
        on_tile: TileId,
        hp_removed: u32,
    },
    HealedPiece {
        on_tile: TileId,
        hp_added: u32,
    },
    PieceKilled {
        on_tile: TileId,
    },
    GameOver {
        winner: Option<PlayerId>,
    },
    PieceRotated {
        on_tile: TileId,
        new_rotation: FacingHexDirection,
    },
    PieceMoved {
        from_tile: TileId,
        to_tile: TileId,
    },
    PlayerDied(PlayerId),
}

#[derive(Default, Debug)]
pub struct ChangeLog(Vec<ActionEffect>);

impl ChangeLog {
    pub fn read(&self) -> &[ActionEffect] {
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
#[derive(Debug)]
pub enum RotationTileStates {
    ElligibleTiles(Box<[TileId]>),
    SelectedTile(TileId),
}

#[derive(Debug)]
pub enum ActionProcessCache {
    TileAction(TileActionProcessCache),
    Ex1,
}

impl ForensicDescribe for ActionProcessCache {
    fn forensic_description(&self) -> Box<[crate::forensic_action_descriptions::TextSnippet]> {
        match self {
            ActionProcessCache::TileAction(tile_action_process_cache) => {
                tile_action_process_cache.forensic_describe()
            }
            ActionProcessCache::Ex1 => todo!(),
        }
    }
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
        slot_in_market: SlotInMarket,
    },
    EndTurn,
    UseOrder {
        tile: TileId,
        input: InputData,
        index_of_order: u8,
    },
}
#[derive(Debug, Deserialize, Serialize, Clone)]
pub enum InputData {
    SelectedTiles(Box<[SelectedTile]>),
    Ex1,
}
#[derive(Debug, Error)]
#[error(
    "The request contained a certain type of additional input data, but it did not match the input expected by the action."
)]
struct UnexpectedInputType;

#[derive(Debug, Error)]
#[error("It is not player {0:?}'s turn.")]
struct NotPlayersTurn(PlayerId);

#[derive(Debug, Error)]
#[error("Player {0:?} is dead and cannot make requests")]
struct PlayerIsDead(PlayerId);

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

#[derive(Debug, Error)]
#[error("Tile {0:?} is vacant.")]
struct NoPieceOnTile(TileId);

#[derive(Debug, Error)]
pub enum ExecuteOrderError {
    #[error("The peice has no orders remaining for this round.")]
    PieceHasNoRemainingOrders,
    #[error("The acting player is out of orders for this roudn.")]
    PlayerHasNoRemainingOrders,
    #[error("The piece does not belong to the acting player.")]
    PieceNotOwnedByPlayer,
    #[error(
        "Tried and failed to find an order on the piece at index {0:?}. This could be because it was an option of None, or because the index was greater than the array of orders could hold."
    )]
    PieceHasNoOrderAtIndex(u8),
}

pub fn try_consume_request(
    request_to_process: BackendRequest,
    world: &mut World,
) -> anyhow::Result<ChangeLog> {
    if request_to_process.acting_player != world.resource::<ActivePlayer>().0 {
        return Err(NotPlayersTurn(request_to_process.acting_player).into());
    }

    if compute_player_state(
        world,
        *world
            .resource::<PlayerDirectory>()
            .get(request_to_process.acting_player),
    ) == LifeState::Dead
    {
        error!("The active player is dead and still attempting to make requests! Bad bad bad...");
        return Err(PlayerIsDead(request_to_process.acting_player).into());
    }

    let player_states_before_action = world
        .resource::<PlayerDirectory>()
        .list()
        .iter()
        .map(|&player| compute_player_state(world, player))
        .collect::<Box<_>>();

    let mut log = match request_to_process.request {
        RequestType::UseCard {
            inventory_index,
            input,
        } => {
            let player_ent = *world
                .resource::<PlayerDirectory>()
                .get(request_to_process.acting_player);

            let card = world
                .get::<PlayerCardInventory>(player_ent)
                .unwrap()
                .get_card(inventory_index)?;

            let action_cache = world
                .resource::<CardDirectory>()
                .get_card(card)?
                .functionality
                .action_cache(world, world.resource::<ActivePlayer>().0)?;

            let mut log = match action_cache {
                ActionProcessCache::TileAction(mut tile_action_process_cache) => {
                    let InputData::SelectedTiles(selections) = input else {
                        return Result::Err(UnexpectedInputType.into());
                    };

                    for tile in selections {
                        tile_action_process_cache
                            .try_select_tile_and_update_elligibility(tile, world)?
                    }

                    tile_action_process_cache.try_execute(world)?
                }
                ActionProcessCache::Ex1 => todo!(),
            };

            world
                .get_mut::<PlayerCardInventory>(player_ent)
                .unwrap()
                .remove_card(inventory_index);

            log.write(ActionEffect::RemovedCardFromInventory {
                player: request_to_process.acting_player,
                index: inventory_index,
            });

            log
        }
        RequestType::PurchaseCard {
            market_tile,
            slot_in_market,
        } => {
            let tile_entity = world.resource::<TileDirectory>().get_entity(market_tile);
            let player_ent = *world
                .resource::<PlayerDirectory>()
                .get(request_to_process.acting_player);

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
                    .get_card_and_price(slot_in_market);

                if world.get::<Coins>(player_ent).unwrap().0 >= price.0 {
                    world
                        .get_mut::<PlayerCardInventory>(player_ent)
                        .unwrap()
                        .try_add_card(card)?;
                    world.get_mut::<Coins>(player_ent).unwrap().0 -= price.0;

                    let mut log = ChangeLog::default();

                    log.write(ActionEffect::AddedCardToInventory {
                        player: request_to_process.acting_player,
                        card,
                    });

                    log.write(ActionEffect::AlteredCoins {
                        player: request_to_process.acting_player,
                        delta_coins: -(price.0 as i32),
                        from_tile: None,
                    });

                    log
                } else {
                    Err(PurchaseCardError::NotEnoughCoins)?
                }
            } else {
                Err(PurchaseCardError::PlayerDoesNotOccupyTile(market_tile))?
            }
        }
        RequestType::EndTurn => {
            let exiting_player = world.resource::<ActivePlayer>().0;

            let mut log = ChangeLog::default();

            log.write(ActionEffect::EndedTurn(exiting_player));

            log.append(&mut players::apply_end_turn_effects(world, exiting_player));

            start_next_turn(world, &mut log);

            log
        }
        RequestType::UseOrder {
            tile,
            input,
            index_of_order,
        } => {
            let piece = world
                .get::<OccupiedByPiece>(world.resource::<TileDirectory>().get_entity(tile))
                .ok_or(NoPieceOnTile(tile))?
                .piece();

            if world.get::<OrdersReceivable>(piece).unwrap().currently < 1 {
                return Err(ExecuteOrderError::PieceHasNoRemainingOrders.into());
            }

            let acting_player = *world
                .resource::<PlayerDirectory>()
                .get(request_to_process.acting_player);

            if world
                .get::<PlayerOrdersRemaining>(acting_player)
                .unwrap()
                .remaining
                < 1
            {
                return Err(ExecuteOrderError::PlayerHasNoRemainingOrders.into());
            }

            if world.get::<PieceOwnedByPlayer>(piece).unwrap().0 != acting_player {
                return Err(ExecuteOrderError::PieceNotOwnedByPlayer.into());
            }

            let desired_order = world.resource::<OrderDirectory>().get_order(
                world
                    .get::<Orders>(piece)
                    .unwrap()
                    .0
                    .get(index_of_order as usize)
                    .ok_or(ExecuteOrderError::PieceHasNoOrderAtIndex(index_of_order))?
                    .ok_or(ExecuteOrderError::PieceHasNoOrderAtIndex(index_of_order))?,
            )?;

            let action = desired_order.functionality.action_cache(tile, world)?;

            let mut change_log = ChangeLog::default();

            // We haven't actually made these changes yet, but we want them to appear to the player before the action actually fires. If the action somehow fails, the changelog isn't emitted, so this doesn't introduce a visual bug.
            change_log.write(ActionEffect::ReducedRemaingOrdersOfPlayer(
                request_to_process.acting_player,
            ));
            change_log.write(ActionEffect::ReducedRemainingOrdersOfPiece {
                tile_of_piece: tile,
            });

            match action {
                ActionProcessCache::TileAction(mut tile_action_process_cache) => {
                    let InputData::SelectedTiles(tiles_to_use_order_on) = input else {
                        return Err(UnexpectedInputType.into());
                    };

                    for id in tiles_to_use_order_on {
                        tile_action_process_cache
                            .try_select_tile_and_update_elligibility(id, world)?
                    }

                    change_log.append(&mut tile_action_process_cache.try_execute(world)?);
                }
                ActionProcessCache::Ex1 => todo!(),
            }

            if let Some(mut orders) = world.get_mut::<OrdersReceivable>(piece) {
                orders.currently -= 1
            }

            world
                .get_mut::<PlayerOrdersRemaining>(acting_player)
                .unwrap()
                .remaining -= 1;

            change_log
        }
    };

    let player_states_after_action = world
        .resource::<PlayerDirectory>()
        .list()
        .iter()
        .map(|&player| compute_player_state(world, player))
        .collect::<Box<[_]>>();

    for (index, (prior_state, new_state)) in player_states_before_action
        .iter()
        .copied()
        .zip(player_states_after_action)
        .enumerate()
    {
        if !(prior_state == LifeState::Alive && new_state == LifeState::Dead) {
            continue;
        }

        log.write(ActionEffect::PlayerDied(
            world.resource::<PlayerDirectory>().make_id(index as u8)?,
        ));

        if world.resource::<ActivePlayer>().0.id() == index as u8 {
            // the current player has died and we need to start a new turn or end the game.
            start_next_turn(world, &mut log);
        }
    }

    let surviving_players = world
        .resource::<PlayerDirectory>()
        .list()
        .iter()
        .filter(|&&player| compute_player_state(world, player) == LifeState::Alive)
        .collect::<Box<[_]>>();

    match surviving_players.len() {
        0 => log.write(ActionEffect::GameOver { winner: None }),
        1 => log.write(ActionEffect::GameOver {
            winner: Some(
                *world
                    .get::<PlayerId>(**surviving_players.first().unwrap())
                    .unwrap(),
            ),
        }),
        _ => (),
    }

    println!("Change log is as follows {:?}", log);

    Ok(log)
}
/// Note, if no players are viable to take over the turn, the turn will logically remain the current player's; this indicates that the game should end.
fn start_next_turn(world: &mut World, log: &mut ChangeLog) {
    let exiting_player = world.resource::<ActivePlayer>().0;

    let (preceding_players, next_players) = world
        .resource::<PlayerDirectory>()
        .list()
        .split_at(exiting_player.id() as usize);

    for player in next_players
        .iter()
        .skip(1)
        .chain(preceding_players)
        .copied()
        .collect::<Box<[_]>>()
    {
        if compute_player_state(world, player) == LifeState::Dead {
            continue;
        }

        let id = *world.get::<PlayerId>(player).unwrap();

        if id == exiting_player {
            // the game is over. We don't write an event for it here though, since it will be caught at the end of the request anyway.
            return;
        }

        // we've found a candidate for the next player!
        log.write(ActionEffect::BeganTurn(id));
        log.append(&mut apply_start_turn_effects(world, id));

        // now we see if they died at the start of their turn...
        if compute_player_state(world, player) != LifeState::Dead {
            world.resource_mut::<ActivePlayer>().0 = id;
            return;
        } else {
            log.write(ActionEffect::PlayerDied(id));
            log.write(ActionEffect::EndedTurn(id));
        }
    }

    println!(
        "No viable candidate found to replace the active player. The game should be ending during the changelog of this request."
    )
}
