use std::{collections::VecDeque, f32, time::Duration};

use bevy::prelude::*;
use bevy_renet::{RenetClient, RenetServer, renet::DefaultChannel};
use core_game_logic::requests::{
    ActionEffect, ActionProcessCache, BackendRequest, ChangeLog, InputData, RequestType,
    try_consume_request,
};
pub use loaded_action_invariance::*;
use serde::{Deserialize, Serialize};

use crate::{
    AppState, OperatingPlayer, functional_assets::LogicalWorld, main_menu::BoardSetupInstructions,
};

pub struct InputInterfacePlugin;

impl Plugin for InputInterfacePlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(try_execute_loaded_action.run_if(in_state(AppState::InGame)));
        app.add_observer(end_turn.run_if(in_state(AppState::InGame)));

        app.add_systems(
            Update,
            (
                client_receive_in_game_message,
                server_receive_in_game_messages,
                maybe_display_effect,
            )
                .run_if(in_state(AppState::InGame)),
        );
    }
}

#[derive(Debug, Resource, PartialEq, Clone, Copy)]
pub enum MultiplayerNetworkingMode {
    SingleDevice,
    Host,
    Client,
}
#[derive(Debug, Event)]
pub struct TryEndTurn;

fn end_turn(
    _trigger: On<TryEndTurn>,
    mut logical_world: ResMut<LogicalWorld>,
    mut action_input_manager: ResMut<ActionInputManager>,
    acting_player: Res<OperatingPlayer>,
    client: Option<ResMut<RenetClient>>,
    mut effects_queue: ResMut<EffectsQueue>,
) -> Result<(), BevyError> {
    let request = BackendRequest {
        acting_player: acting_player.0,
        request: RequestType::EndTurn,
    };

    let change_log = try_consume_request(
        BackendRequest {
            acting_player: acting_player.0,
            request: RequestType::EndTurn,
        },
        &mut logical_world.0,
    )?;

    if let Some(mut client) = client {
        client.send_message(
            DefaultChannel::ReliableOrdered,
            postcard::to_stdvec(&NetworkTransmission::ActionDone(request)).unwrap(),
        );
    }

    effects_queue
        .0
        .append(&mut VecDeque::from(change_log.inner()));

    *action_input_manager = ActionInputManager::default();

    Ok(())
}

mod loaded_action_invariance {
    use bevy::prelude::Resource;
    use core_game_logic::{
        markets::SlotInMarket, players::InventoryIndex, requests::ActionProcessCache,
        tile_mapping::TileId,
    };
    use thiserror::Error;

    use crate::ui_panels::OrderAtPieceIndex;

    #[derive(Debug, Resource, Default)]
    pub struct ActionInputManager {
        active_tile: Option<TileId>,
        loaded_action: Option<FrontendAction>,
    }

    #[derive(Debug, Error)]
    pub enum LoadActionError {
        #[error("The action {0:?} requires a tile to be active.")]
        ActiveTileMissing(FrontendAction),
    }

    #[derive(Debug)]
    pub enum FrontendAction {
        UseCard {
            index: InventoryIndex,
            cache: ActionProcessCache,
        },
        UseOrder {
            index_of_order_on_active_piece: OrderAtPieceIndex,
            cache: ActionProcessCache,
        },
        PurchaseCard {
            slot: SlotInMarket,
        },
    }

    impl ActionInputManager {
        pub fn active_tile(&self) -> Option<TileId> {
            self.active_tile
        }

        pub fn process_cache(&self) -> Option<&ActionProcessCache> {
            self.loaded_action()?;

            match self.loaded_action.as_ref().unwrap() {
                FrontendAction::UseCard { cache, .. } => Some(cache),
                FrontendAction::UseOrder { cache, .. } => Some(cache),
                FrontendAction::PurchaseCard { .. } => None,
            }
        }

        pub fn process_cache_mut(&mut self) -> Option<&mut ActionProcessCache> {
            if let Some(action) = &mut self.loaded_action {
                match action {
                    FrontendAction::UseCard { cache, .. } => Some(cache),
                    FrontendAction::UseOrder { cache, .. } => Some(cache),
                    FrontendAction::PurchaseCard { .. } => None,
                }
            } else {
                None
            }
        }

        pub fn try_load_action(
            &mut self,
            action: Option<FrontendAction>,
        ) -> Result<(), LoadActionError> {
            if let Some(action) = action {
                match &action {
                    FrontendAction::UseCard { .. } => {
                        self.loaded_action = Some(action);
                    }
                    _ => {
                        if self.active_tile.is_none() {
                            return Err(LoadActionError::ActiveTileMissing(action));
                        }

                        self.loaded_action = Some(action);
                    }
                }
            } else {
                self.loaded_action = None;
            }

            Ok(())
        }

        pub fn set_active_tile(&mut self, tile: Option<TileId>) {
            self.active_tile = tile;

            if self.loaded_action.is_none() {
                return;
            }

            match self.loaded_action.as_ref().unwrap() {
                FrontendAction::UseCard { .. } => (),
                _ => self.loaded_action = None,
            }
        }

        pub fn loaded_action(&self) -> Option<&FrontendAction> {
            self.loaded_action.as_ref()
        }
    }
}

#[derive(Event, Debug)]
pub struct TryExecuteLoadedAction;

fn try_execute_loaded_action(
    _trigger: On<TryExecuteLoadedAction>,
    mut action_manager: ResMut<ActionInputManager>,
    acting_player: Res<OperatingPlayer>,
    mut logical_world: ResMut<LogicalWorld>,
    mut effects_queue: ResMut<EffectsQueue>,
    client: Option<ResMut<RenetClient>>,
) -> Result<(), BevyError> {
    let Some(action) = action_manager.loaded_action() else {
        warn!("tried to execute an action, but there was no action loaded.");
        return Ok(());
    };

    let request = {
        match action {
            FrontendAction::UseCard { index, cache } => {
                RequestType::UseCard { inventory_index: *index, input: match cache {
                    ActionProcessCache::TileAction(tile_action_process_cache) => InputData::SelectedTiles(tile_action_process_cache.selected_tiles().into()),
                    ActionProcessCache::Ex1 => todo!(),
                },  }
            },
            FrontendAction::UseOrder { index_of_order_on_active_piece, cache } => {
                RequestType::UseOrder { tile: action_manager.active_tile().ok_or("Invalid data! An action was loaded to purchase a card from a market, without an active tile.")?, index_of_order: index_of_order_on_active_piece.0, input: match cache {
                    ActionProcessCache::TileAction(tile_action_process_cache) => InputData::SelectedTiles(tile_action_process_cache.selected_tiles().into()),
                    ActionProcessCache::Ex1 => todo!(),
                }, }
            },
            FrontendAction::PurchaseCard { slot } => {
                RequestType::PurchaseCard { market_tile: action_manager.active_tile().ok_or("Invalid data! An action was loaded to purchase a card from a market, without an active tile.")?, slot_in_market: *slot }
            },
        }
    };

    if let Ok(change_log) = try_consume_request(
        BackendRequest {
            acting_player: acting_player.0,
            request: request.clone(),
        },
        &mut logical_world.0,
    ) {
        action_manager.try_load_action(None)?;

        if let Some(mut client) = client {
            client.send_message(
                DefaultChannel::ReliableOrdered,
                postcard::to_stdvec(&NetworkTransmission::ActionDone(BackendRequest {
                    acting_player: acting_player.0,
                    request: request.clone(),
                }))
                .unwrap(),
            );
        }

        effects_queue
            .0
            .append(&mut VecDeque::from(change_log.inner()));
    }

    Ok(())
}

#[derive(Debug, Serialize, Deserialize)]
pub enum NetworkTransmission {
    InitialConnectionMessage { name: String, is_spectator: bool },
    ConnectionConfirmationMessage,
    StartGame(BoardSetupInstructions),
    ActionDone(BackendRequest),
}

pub const HOST_CLIENT_ID: u64 = 2144552;

fn server_receive_in_game_messages(
    mut server: If<ResMut<RenetServer>>,
    mut logical_world: ResMut<LogicalWorld>,
    mut effects_queue: ResMut<EffectsQueue>,
) {
    for id in server.clients_id() {
        while let Some(message) = server.receive_message(id, DefaultChannel::ReliableOrdered) {
            if let NetworkTransmission::ActionDone(action) =
                postcard::from_bytes::<NetworkTransmission>(&message).unwrap()
            {
                if id == HOST_CLIENT_ID {
                    server.broadcast_message(DefaultChannel::ReliableOrdered, message);
                } else {
                    if let Ok(change_log) = try_consume_request(action, &mut logical_world.0) {
                        effects_queue
                            .0
                            .append(&mut VecDeque::from(change_log.inner()));

                        server.broadcast_message_except(
                            id,
                            DefaultChannel::ReliableOrdered,
                            message,
                        );
                    } else {
                        panic!("Desync occured between client {} and the server.", id)
                    }
                }
            }
        }
    }
}

fn client_receive_in_game_message(
    mut client: If<ResMut<RenetClient>>,
    networking_mode: Res<MultiplayerNetworkingMode>,
    mut logical_world: ResMut<LogicalWorld>,
    mut effects_queue: ResMut<EffectsQueue>,
) {
    while let Some(message) = client.0.receive_message(DefaultChannel::ReliableOrdered) {
        if let NetworkTransmission::ActionDone(action) =
            postcard::from_bytes::<NetworkTransmission>(&message).unwrap()
        {
            if *networking_mode == MultiplayerNetworkingMode::Host {
                continue;
            }

            if let Ok(change_log) = try_consume_request(action, &mut logical_world.0) {
                effects_queue
                    .0
                    .append(&mut VecDeque::from(change_log.inner()));
            }
        }
    }
}

#[derive(Debug, Resource, Default)]
pub struct EffectsQueue(VecDeque<ActionEffect>);
impl EffectsQueue {
    pub fn new(log: ChangeLog) -> Self {
        Self(VecDeque::from(log.inner()))
    }
}

#[derive(Debug, Resource)]
pub struct EffectToDisplay(pub ActionEffect);
#[derive(Debug, Resource)]
pub struct NextEffectStartsIn(pub Timer);

fn maybe_display_effect(
    mut commands: Commands,
    mut timer: ResMut<NextEffectStartsIn>,
    mut effects_queue: ResMut<EffectsQueue>,
    delta: Res<Time>,
) {
    /// Each consecutive effect will take this many seconds less than the previous turn.
    const ACCELLERATION_SPEED_SECS: f32 = 0.2;
    /// default duration at the start of a effect queue.
    const STARTING_DURATION: f32 = 3.0;
    const MINIMUM_DURATION: f32 = 0.2;

    if timer.0.is_finished() {
        if let Some(effect) = effects_queue.0.pop_front() {
            #[cfg(debug_assertions)]
            println!("Displaying effect {effect:?}");

            commands.insert_resource(EffectToDisplay(effect));

            let new_duration = (timer.0.duration().as_secs_f32() - ACCELLERATION_SPEED_SECS)
                .clamp(MINIMUM_DURATION, f32::MAX);
            timer.0.set_duration(Duration::from_secs_f32(new_duration));
            timer.0.reset();
        } else {
            commands.remove_resource::<EffectToDisplay>();
            timer
                .0
                .set_duration(Duration::from_secs_f32(STARTING_DURATION));
            return;
        }
    }

    timer.0.tick(delta.delta());
}
