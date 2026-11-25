// This module details how the Renet Server and Renet Client are to operate. It does not bother with how the main menu establishes initial connections between the players.

use bevy::prelude::*;
use bevy_renet::{RenetClient, RenetServer};
use core_game_logic::{
    players::ActivePlayer,
    requests::{BackendRequest, RequestType, try_consume_request},
};
use serde::{Deserialize, Serialize};

use crate::{
    OperatingPlayer, functional_assets::LogicalWorld, inputs_interface::ActionInputManager,
};

pub struct MultiplayerPlugin;

impl Plugin for MultiplayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(sd_execute_action.run_if(resource_exists_and_equals(
            MultiplayerNetworkingMode::SingleDevice,
        )));
    }
}

#[derive(Debug, Resource, PartialEq, Clone, Copy)]
pub enum MultiplayerNetworkingMode {
    SingleDevice,
    Host,
    Client,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum NetworkTransmission {
    InitialConnectionMessage {
        name: String,
        is_spectator: bool,
        version_number: u64,
    },
    ConnectionConfirmationMessage,
    StartGame(crate::main_menu::BoardSetupInstructions),
    ActionDone(core_game_logic::requests::BackendRequest),
}

#[derive(Event, Debug)]
pub struct TryExecuteLoadedAction;

fn sd_execute_action(
    _: On<TryExecuteLoadedAction>,
    mut loaded_action: ResMut<ActionInputManager>,
    actor: ResMut<OperatingPlayer>,
    mut log_world: ResMut<LogicalWorld>,
    mut effects_queue: ResMut<crate::inputs_interface::EffectsQueue>,
) {
    let Some(request): Option<RequestType> = loaded_action.as_ref().into() else {
        warn!("Tried to execute an action while none was loaded.");
        return;
    };

    match try_consume_request(
        BackendRequest {
            acting_player: actor.0,
            request,
        },
        &mut log_world.0,
    ) {
        Ok(log) => {
            actor.into_inner().0 = log_world.0.resource::<ActivePlayer>().0;
            loaded_action.clear_action();
            effects_queue.append(log);
        }
        Err(err) => warn!("Action execution error: {}", err),
    }
}
