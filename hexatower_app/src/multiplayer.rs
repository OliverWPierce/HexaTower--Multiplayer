// This module details how the Renet Server and Renet Client are to operate. It does not bother with how the main menu establishes initial connections between the players.
// sd = SingleDevice, hp = Hostplayer, cl = client.
use bevy::prelude::*;
use bevy_renet::{RenetClient, RenetServer, renet::DefaultChannel};
use core_game_logic::{
    players::ActivePlayer,
    requests::{BackendRequest, RequestType, try_consume_request},
};
use serde::{Deserialize, Serialize};

use crate::{
    OperatingPlayer, VERSION_NUMBER,
    functional_assets::LogicalWorld,
    inputs_interface::{ActionInputManager, RequestLog},
};

pub struct MultiplayerPlugin;

impl Plugin for MultiplayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(sd_execute_action.run_if(resource_exists_and_equals(
            MultiplayerNetworkingMode::SingleDevice,
        )));
        app.add_observer(sd_end_turn.run_if(resource_exists_and_equals(
            MultiplayerNetworkingMode::SingleDevice,
        )));
        app.add_observer(hp_acts.run_if(resource_exists::<RenetServer>));
        app.add_observer(hp_ends_turn.run_if(resource_exists::<RenetServer>));
    }
}

#[derive(Debug, Resource, PartialEq, Clone, Copy)]
pub enum MultiplayerNetworkingMode {
    SingleDevice,
    HostPlayer,
    Client,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum NetworkTransmission {
    InitialConnectionMessage {
        name: String,
        prefer_spectator: bool,
        version_number: u64,
    },
    ConnectionConfirmationMessage,
    StartGame(crate::main_menu::BoardSetupInstructions),
    ActionDone(core_game_logic::requests::BackendRequest),
}

#[derive(Event, Debug)]
pub struct TryExecuteLoadedAction;

#[derive(Debug, Event)]
pub struct TryEndTurn;

fn sd_execute_action(
    _: On<TryExecuteLoadedAction>,
    mut loaded_action: ResMut<ActionInputManager>,
    actor: ResMut<OperatingPlayer>,
    mut log_world: ResMut<LogicalWorld>,
    mut effects_queue: ResMut<crate::inputs_interface::EffectsQueue>,
    mut request_log: If<ResMut<RequestLog>>,
) {
    let Some(request): Option<RequestType> = loaded_action.as_ref().into() else {
        warn!("Tried to execute an action while none was loaded.");
        return;
    };

    match try_consume_request(
        BackendRequest {
            acting_player: actor.0,
            request: request.clone(),
        },
        &mut log_world.0,
    ) {
        Ok(log) => {
            request_log.write(BackendRequest {
                acting_player: actor.0,
                request,
            });
            actor.into_inner().0 = log_world.0.resource::<ActivePlayer>().0;
            loaded_action.clear_action();
            effects_queue.append(log);
        }
        Err(err) => warn!("Action execution error: {}", err),
    }
}

fn sd_end_turn(
    _: On<TryEndTurn>,
    mut loaded_action: ResMut<ActionInputManager>,
    actor: ResMut<OperatingPlayer>,
    mut log_world: ResMut<LogicalWorld>,
    mut effects_queue: ResMut<crate::inputs_interface::EffectsQueue>,
    mut request_log: If<ResMut<RequestLog>>,
) {
    match try_consume_request(
        BackendRequest {
            acting_player: actor.0,
            request: RequestType::EndTurn,
        },
        &mut log_world.0,
    ) {
        Ok(log) => {
            request_log.write(BackendRequest {
                acting_player: actor.0,
                request: RequestType::EndTurn,
            });
            actor.into_inner().0 = log_world.0.resource::<ActivePlayer>().0;
            loaded_action.clear_action();
            effects_queue.append(log);
        }
        Err(err) => warn!("Could not end turn: {}", err),
    }
}

fn hp_acts(
    _: On<TryExecuteLoadedAction>,
    mut loaded_action: ResMut<ActionInputManager>,
    actor: Res<OperatingPlayer>,
    mut log_world: ResMut<LogicalWorld>,
    mut effects_queue: ResMut<crate::inputs_interface::EffectsQueue>,
    mut server: ResMut<RenetServer>,
    mut request_log: If<ResMut<RequestLog>>,
) -> Result<(), BevyError> {
    let Some(request): Option<RequestType> = loaded_action.as_ref().into() else {
        warn!("Tried to execute an action while none was loaded.");
        return Ok(());
    };

    match try_consume_request(
        BackendRequest {
            acting_player: actor.0,
            request: request.clone(),
        },
        &mut log_world.0,
    ) {
        Ok(log) => {
            request_log.write(BackendRequest {
                acting_player: actor.0,
                request: request.clone(),
            });
            loaded_action.clear_action();
            effects_queue.append(log);
            server.broadcast_message(
                DefaultChannel::ReliableOrdered,
                postcard::to_stdvec(&NetworkTransmission::ActionDone(BackendRequest {
                    acting_player: actor.0,
                    request,
                }))?,
            )
        }
        Err(err) => warn!(
            "Action execution error: {}. Action was not sent to clients",
            err
        ),
    }
    Ok(())
}

fn hp_ends_turn(
    _: On<TryEndTurn>,
    mut loaded_action: ResMut<ActionInputManager>,
    actor: Res<OperatingPlayer>,
    mut log_world: ResMut<LogicalWorld>,
    mut effects_queue: ResMut<crate::inputs_interface::EffectsQueue>,
    mut server: ResMut<RenetServer>,
    mut request_log: If<ResMut<RequestLog>>,
) -> Result<(), BevyError> {
    match try_consume_request(
        BackendRequest {
            acting_player: actor.0,
            request: RequestType::EndTurn,
        },
        &mut log_world.0,
    ) {
        Ok(log) => {
            loaded_action.clear_action();
            effects_queue.append(log);
            server.broadcast_message(
                DefaultChannel::ReliableOrdered,
                postcard::to_stdvec(&NetworkTransmission::ActionDone(BackendRequest {
                    acting_player: actor.0,
                    request: RequestType::EndTurn,
                }))?,
            );
            request_log.write(BackendRequest {
                acting_player: actor.0,
                request: RequestType::EndTurn,
            });
        }
        Err(err) => warn!(
            "End turn error: {}. End Turn request was not sent to clients",
            err
        ),
    }
    Ok(())
}

#[derive(Debug, Resource, Default)]
pub struct ConnectedClients(Vec<FullyConnectedClient>);

impl ConnectedClients {
    #[inline]
    pub fn slice(&self) -> &[FullyConnectedClient] {
        &self.0
    }
}

#[derive(Debug)]
struct FullyConnectedClient {
    name: String,
    client_id: u64,
    prefer_spectator: bool,
}

fn hp_server(
    mut server: ResMut<RenetServer>,
    mut connected_clients: ResMut<ConnectedClients>,
    mut request_log: Option<ResMut<RequestLog>>,
    mut effects_queue: ResMut<crate::inputs_interface::EffectsQueue>,
    mut log_world: ResMut<LogicalWorld>,
) {
    for id in server.clients_id() {
        while let Some(message) = server.receive_message(id, DefaultChannel::ReliableOrdered) {
            let Ok(transmission) = postcard::from_bytes::<NetworkTransmission>(&message) else {
                warn!("Could not deserialize message from client {:?}", id);
                continue;
            };

            match transmission {
                NetworkTransmission::InitialConnectionMessage {
                    name,
                    prefer_spectator,
                    version_number,
                } => {
                    if version_number != VERSION_NUMBER {
                        server.disconnect(id);
                        continue;
                    }
                    connected_clients.0.push(FullyConnectedClient {
                        name,
                        client_id: id,
                        prefer_spectator,
                    });

                    server.send_message(
                        id,
                        DefaultChannel::ReliableOrdered,
                        postcard::to_stdvec(&NetworkTransmission::ConnectionConfirmationMessage)
                            .unwrap(),
                    );

                    if let Some(request_log) = &request_log {
                        server.send_message(
                            id,
                            DefaultChannel::ReliableOrdered,
                            postcard::to_stdvec(&NetworkTransmission::StartGame(
                                request_log.starting_state().clone(),
                            ))
                            .unwrap(),
                        );

                        request_log.slice().iter().for_each(|backend_request| {
                            server.send_message(
                                id,
                                DefaultChannel::ReliableOrdered,
                                postcard::to_stdvec(&NetworkTransmission::ActionDone(
                                    backend_request.clone(),
                                ))
                                .unwrap(),
                            );
                        });
                    }
                }
                NetworkTransmission::ActionDone(backend_request) => {
                    match try_consume_request(backend_request.clone(), &mut log_world.0) {
                        Ok(change_log) => {
                            server.broadcast_message_except(
                                id,
                                DefaultChannel::ReliableOrdered,
                                message,
                            );

                            effects_queue.append(change_log);
                            let Some(request_log) = request_log.as_mut() else {
                                warn!(
                                    "Server executed an action on the logical world, but the request log was unavailible."
                                );
                                continue;
                            };

                            request_log.write(backend_request);
                        }
                        Err(err) => {
                            #[cfg(debug_assertions)]
                            panic!(
                                "Server/client desync. Client {} requested to accomplish the following action {:?} which the server found to be invalid according to this error {:?}.",
                                id, backend_request, err
                            );

                            #[cfg(not(debug_assertions))]
                            todo!()
                        }
                    }
                }
                _ => warn!(
                    "Client sent a message type that they were not allowed to: {:?}.",
                    transmission
                ),
            }
        }
    }
}
