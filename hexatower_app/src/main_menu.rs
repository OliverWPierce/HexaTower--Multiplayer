use std::marker::PhantomData;
use std::net::UdpSocket;
use std::time::SystemTime;

use bevy::ecs::component::Mutable;
use bevy::prelude::*;
use bevy::text::EditableText;
use bevy::{color::palettes::tailwind::*, text::TextCursorStyle};
use bevy_renet::netcode::{
    ClientAuthentication, NetcodeClientTransport, NetcodeServerTransport, ServerAuthentication,
    ServerConfig,
};
use bevy_renet::renet::{ConnectionConfig, DefaultChannel};
use bevy_renet::{RenetClient, RenetServer};
use core_game_logic::players::{PlayerData, PlayerId};
use serde::{Deserialize, Serialize};

use crate::VERSION_NUMBER;

use crate::functional_assets::create_board;
use crate::inputs_interface::NetworkTransmission;
use crate::{
    AppState,
    inputs_interface::MultiplayerNetworkingMode,
    ui_panels::{self, UNIVERSAL_BACKGROUND, UNIVERSAL_BORDER_WIDTH},
};

pub struct MainMenuAndLobbyPluggin;

impl Plugin for MainMenuAndLobbyPluggin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::MainMenu), render_main_menu);
        app.add_systems(
            OnEnter(AppState::ParametersScreen),
            render_parameters_screen,
        );
        app.add_systems(
            OnEnter(AppState::PreGame),
            render_pregame_if_server.run_if(
                resource_exists::<RenetServer>
                    .and_then(resource_exists_and_changed::<InfoForConnectedClients>),
            ),
        );

        app.add_systems(
            OnEnter(AppState::PreGame),
            render_pre_game_if_client.run_if(resource_exists_and_equals(
                MultiplayerNetworkingMode::Client,
            )),
        );

        app.add_systems(
            Update,
            render_client_connection_status_during_pregame.run_if(
                in_state(AppState::PreGame).and_then(resource_exists_and_changed::<RenetClient>),
            ),
        );

        app.add_systems(
            Update,
            render_pregame_if_server.run_if(
                in_state(AppState::PreGame).and_then(
                    resource_exists::<RenetServer>
                        .and_then(resource_exists_and_changed::<InfoForConnectedClients>),
                ),
            ),
        );

        app.add_systems(
            Update,
            get_client_info.run_if(
                in_state(AppState::PreGame).and_then(resource_exists_and_changed::<RenetServer>),
            ),
        );
    }
}

const HEADER_SIZE: FontSize = FontSize::Vh(6.0);

const BUTTON_TEXT_SIZE: FontSize = FontSize::Vh(6.0);
const BUTTON_HEIGHT: Val = Val::Vh(10.0);
fn render_main_menu(mut commands: Commands) {
    commands.spawn(Camera2d);

    let source_node = commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::SpaceAround,
                ..default()
            },
            BackgroundColor(UNIVERSAL_BACKGROUND),
            MenusBackgroundNode,
        ))
        .id();

    commands.spawn((
        ChildOf(source_node),
        Text::new("Hexatower"),
        TextFont::from_font_size(HEADER_SIZE),
    ));

    let button_bundle = (
        ChildOf(source_node),
        Node {
            width: Val::Percent(60.0),
            height: BUTTON_HEIGHT,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            border: UiRect::all(UNIVERSAL_BORDER_WIDTH),
            border_radius: BorderRadius::all(Val::Px(6.0)),
            ..default()
        },
        ui_panels::hoverable_elements::create_hoverable_ui_bundle(
            BorderColor::all(SLATE_800),
            BackgroundColor(SLATE_700.into()),
            BorderColor::all(SLATE_500),
            BackgroundColor(SLATE_600.into()),
        ),
    );

    commands
        .spawn((
            button_bundle.clone(),
            children![(
                Text::new("Join Game"),
                TextFont::from_font_size(BUTTON_TEXT_SIZE),
            )],
        ))
        .observe(
            |_: On<Pointer<Click>>,
             mut commands: Commands,
             mut state: ResMut<NextState<AppState>>| {
                commands.insert_resource(MultiplayerNetworkingMode::Client);
                state.set(AppState::ParametersScreen);
            },
        );

    commands
        .spawn((
            button_bundle.clone(),
            children![(
                Text::new("Host and Play"),
                TextFont::from_font_size(BUTTON_TEXT_SIZE),
            )],
        ))
        .observe(
            |_: On<Pointer<Click>>,
             mut commands: Commands,
             mut state: ResMut<NextState<AppState>>| {
                commands.insert_resource(MultiplayerNetworkingMode::Host);
                state.set(AppState::ParametersScreen);
            },
        );

    commands
        .spawn((
            button_bundle.clone(),
            children![(
                Text::new("Exit"),
                TextFont::from_font_size(BUTTON_TEXT_SIZE),
            )],
        ))
        .observe(|_: On<Pointer<Click>>, mut commands: Commands| {
            commands.write_message(AppExit::Success);
        });
}

#[derive(Debug, Component)]
struct MenusBackgroundNode;

fn render_parameters_screen(
    networking_mode: Res<MultiplayerNetworkingMode>,
    background_node: Single<Entity, With<MenusBackgroundNode>>,
    mut commands: Commands,
) {
    commands.entity(background_node.entity()).despawn_children();

    #[derive(Debug, Component)]
    struct IpAdressCollectionNode;
    #[derive(Debug, Component)]
    struct GamertagCollectionNode;

    match *networking_mode {
        MultiplayerNetworkingMode::SingleDevice => todo!(),
        MultiplayerNetworkingMode::Host => {
            commands.spawn((
                ChildOf(background_node.entity()),
                Text::new("Host Game"),
                TextFont::from_font_size(HEADER_SIZE),
            ));

            commands.spawn((
                Node {
                    width: Val::Percent(25.0),
                    height: Val::Vh(7.0),
                    flex_direction: FlexDirection::Column,
                    ..default()
                },
                ChildOf(background_node.entity()),
                children![
                    (
                        Text::new("IP address and port of game:"),
                        TextFont::from_font_size(FontSize::Vh(2.0)),
                    ),
                    (
                        Node {
                            width: Val::Percent(100.0),
                            border: px(2).all(),
                            ..Default::default()
                        },
                        IpAdressCollectionNode,
                        BorderColor::from(Color::from(SLATE_700)),
                        EditableText {
                            visible_width: Some(10.),
                            allow_newlines: false,
                            max_characters: Some(25),
                            ..Default::default()
                        },
                        TextLayout::no_wrap(),
                        TextFont {
                            font_size: FontSize::Vh(4.0),
                            ..default()
                        },
                        TextCursorStyle::default(),
                        BackgroundColor(SLATE_800.into()),
                    )
                ],
            ));

            commands.spawn((
                Node {
                    width: Val::Percent(25.0),
                    height: Val::Vh(7.0),
                    flex_direction: FlexDirection::Column,
                    ..default()
                },
                ChildOf(background_node.entity()),
                children![
                    (
                        Text::new("Display name. Enter a title fit for a king."),
                        TextFont::from_font_size(FontSize::Vh(2.0)),
                    ),
                    (
                        Node {
                            width: Val::Percent(100.0),
                            border: px(2).all(),
                            ..Default::default()
                        },
                        GamertagCollectionNode,
                        BorderColor::from(Color::from(SLATE_700)),
                        EditableText {
                            visible_width: Some(10.),
                            allow_newlines: false,
                            max_characters: Some(15),
                            ..Default::default()
                        },
                        TextLayout::no_wrap(),
                        TextFont {
                            font_size: FontSize::Vh(4.0),
                            ..default()
                        },
                        TextCursorStyle::default(),
                        BackgroundColor(SLATE_800.into()),
                    )
                ],
            ));

            let board_size_ui = display_clickthrough_selectors::<PresetBoardSizes>(&mut commands);

            commands
                .entity(background_node.entity())
                .add_child(board_size_ui);

            commands
                .spawn((
                    ChildOf(background_node.entity()),
                    Node {
                        width: Val::Percent(60.0),
                        height: BUTTON_HEIGHT,
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        border: UiRect::all(UNIVERSAL_BORDER_WIDTH),
                        border_radius: BorderRadius::all(Val::Px(6.0)),
                        ..default()
                    },
                    ui_panels::hoverable_elements::create_hoverable_ui_bundle(
                        BorderColor::all(SLATE_800),
                        BackgroundColor(SLATE_700.into()),
                        BorderColor::all(SLATE_500),
                        BackgroundColor(SLATE_600.into()),
                    ),
                    children![(
                        Text::new("Start Server"),
                        TextFont::from_font_size(BUTTON_TEXT_SIZE),
                    )],
                ))
                .observe(
                    |_: On<Pointer<Click>>,
                     mut commands: Commands,
                     ip_address: Single<&EditableText, With<IpAdressCollectionNode>>,
                     name: Single<&EditableText, With<GamertagCollectionNode>>,
                     mut state: ResMut<NextState<AppState>>| {
                        let Ok(server_addr) = ip_address.value().to_string().parse() else {
                            warn!("Invalid IP adress");
                            return;
                        };

                        if name.value().into_iter().len() > 15 {
                            warn!("Client's name is too long.");
                            return;
                        }

                        let Ok(initial_message) = postcard::to_stdvec::<NetworkTransmission>(
                            &NetworkTransmission::InitialConnectionMessage {
                                name: name.value().to_string(),
                                is_spectator: false,
                            },
                        ) else {
                            warn!("Could not serialize the clients name.");
                            return;
                        };

                        let socket = UdpSocket::bind(server_addr).unwrap();
                        let server_config = ServerConfig {
                            current_time: SystemTime::now()
                                .duration_since(SystemTime::UNIX_EPOCH)
                                .unwrap(),
                            max_clients: 64,
                            protocol_id: VERSION_NUMBER,
                            public_addresses: vec![server_addr],
                            authentication: ServerAuthentication::Unsecure,
                        };

                        let host_server = RenetServer::new(ConnectionConfig::default());
                        commands.insert_resource(host_server);
                        let server_transport =
                            NetcodeServerTransport::new(server_config, socket).unwrap();
                        commands.insert_resource(server_transport);

                        let time = SystemTime::now()
                            .duration_since(SystemTime::UNIX_EPOCH)
                            .unwrap();

                        let client_adrr = "127.0.0.1:0".to_string();
                        let socket = UdpSocket::bind(client_adrr).unwrap();
                        let current_time = SystemTime::now()
                            .duration_since(SystemTime::UNIX_EPOCH)
                            .unwrap();
                        let authentication = ClientAuthentication::Unsecure {
                            server_addr,
                            client_id: (time.as_secs_f64() * 1000.0) as u64,
                            user_data: None,
                            protocol_id: VERSION_NUMBER,
                        };

                        let client_transport =
                            NetcodeClientTransport::new(current_time, authentication, socket)
                                .unwrap();

                        commands.insert_resource(client_transport);

                        let mut client = RenetClient::new(ConnectionConfig::default());
                        client.0.send_message(
                            DefaultChannel::ReliableOrdered,
                            initial_message.into_boxed_slice(),
                        );
                        commands.insert_resource(client);

                        commands.insert_resource(InfoForConnectedClients(Vec::new()));

                        state.set(AppState::PreGame);
                    },
                );
        }
        MultiplayerNetworkingMode::Client => {
            commands.spawn((
                ChildOf(background_node.entity()),
                Text::new("Join Game"),
                TextFont::from_font_size(HEADER_SIZE),
            ));

            commands.spawn((
                Node {
                    width: Val::Percent(25.0),
                    height: Val::Vh(7.0),
                    flex_direction: FlexDirection::Column,
                    ..default()
                },
                ChildOf(background_node.entity()),
                children![
                    (
                        Text::new("IP address and port of host:"),
                        TextFont::from_font_size(FontSize::Vh(2.0)),
                    ),
                    (
                        Node {
                            width: Val::Percent(100.0),
                            border: px(2).all(),
                            ..Default::default()
                        },
                        IpAdressCollectionNode,
                        BorderColor::from(Color::from(SLATE_700)),
                        EditableText {
                            visible_width: Some(10.),
                            allow_newlines: false,
                            max_characters: Some(25),
                            ..Default::default()
                        },
                        TextLayout::no_wrap(),
                        TextFont {
                            font_size: FontSize::Vh(4.0),
                            ..default()
                        },
                        TextCursorStyle::default(),
                        BackgroundColor(SLATE_800.into()),
                    )
                ],
            ));

            commands.spawn((
                Node {
                    width: Val::Percent(25.0),
                    height: Val::Vh(7.0),
                    flex_direction: FlexDirection::Column,
                    ..default()
                },
                ChildOf(background_node.entity()),
                children![
                    (
                        Text::new("Display name. Enter a title fit for a king."),
                        TextFont::from_font_size(FontSize::Vh(2.0)),
                    ),
                    (
                        Node {
                            width: Val::Percent(100.0),
                            border: px(2).all(),
                            ..Default::default()
                        },
                        GamertagCollectionNode,
                        BorderColor::from(Color::from(SLATE_700)),
                        EditableText {
                            visible_width: Some(10.),
                            allow_newlines: false,
                            max_characters: Some(15),
                            ..Default::default()
                        },
                        TextLayout::no_wrap(),
                        TextFont {
                            font_size: FontSize::Vh(4.0),
                            ..default()
                        },
                        TextCursorStyle::default(),
                        BackgroundColor(SLATE_800.into()),
                    )
                ],
            ));

            commands
                .spawn((
                    ChildOf(background_node.entity()),
                    Node {
                        width: Val::Percent(60.0),
                        height: BUTTON_HEIGHT,
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        border: UiRect::all(UNIVERSAL_BORDER_WIDTH),
                        border_radius: BorderRadius::all(Val::Px(6.0)),
                        ..default()
                    },
                    ui_panels::hoverable_elements::create_hoverable_ui_bundle(
                        BorderColor::all(SLATE_800),
                        BackgroundColor(SLATE_700.into()),
                        BorderColor::all(SLATE_500),
                        BackgroundColor(SLATE_600.into()),
                    ),
                    children![(
                        Text::new("Join"),
                        TextFont::from_font_size(BUTTON_TEXT_SIZE),
                    )],
                ))
                .observe(
                    |_: On<Pointer<Click>>,
                     mut commands: Commands,
                     ip_address: Single<&EditableText, With<IpAdressCollectionNode>>,
                     name: Single<&EditableText, With<GamertagCollectionNode>>,
                     mut state: ResMut<NextState<AppState>>| {
                        if name.value().into_iter().len() > 15 {
                            warn!("Client's name is too long.");
                            return;
                        }

                        let Ok(initial_message) =
                            postcard::to_stdvec(&NetworkTransmission::InitialConnectionMessage {
                                name: name.value().to_string(),
                                is_spectator: false,
                            })
                        else {
                            warn!("Could not serialize the clients name.");
                            return;
                        };

                        let Ok(server_addr) = ip_address.value().to_string().parse() else {
                            warn!("Invalid IP adress");
                            return;
                        };

                        let mut client = RenetClient::new(ConnectionConfig::default());

                        let time = SystemTime::now()
                            .duration_since(SystemTime::UNIX_EPOCH)
                            .unwrap();

                        let authentication = ClientAuthentication::Unsecure {
                            server_addr,
                            client_id: (time.as_secs_f64() * 1000.0) as u64,
                            user_data: None,
                            protocol_id: VERSION_NUMBER,
                        };

                        let client_adrr = "127.0.0.1:0".to_string();
                        let socket = UdpSocket::bind(client_adrr).unwrap();
                        let current_time = time;

                        let transport =
                            NetcodeClientTransport::new(current_time, authentication, socket)
                                .unwrap();

                        commands.insert_resource(transport);

                        state.set(AppState::PreGame);

                        client.0.send_message(
                            DefaultChannel::ReliableOrdered,
                            initial_message.into_boxed_slice(),
                        );

                        commands.insert_resource(client);
                    },
                );
        }
    }
}

trait ClickThroughSelector: Resource<Mutability = Mutable> + Default {
    fn next_option(&mut self);

    fn previous_option(&mut self);

    fn display_text(&self) -> String;
}
#[derive(Debug, Resource, Default, Serialize, Deserialize, Clone, Copy)]
pub enum PresetBoardSizes {
    Small,
    #[default]
    Regular,
    Large,
}

impl ClickThroughSelector for PresetBoardSizes {
    fn next_option(&mut self) {
        *self = match self {
            PresetBoardSizes::Small => Self::Regular,
            PresetBoardSizes::Regular => Self::Large,
            PresetBoardSizes::Large => Self::Small,
        };
    }

    fn previous_option(&mut self) {
        *self = match self {
            PresetBoardSizes::Small => Self::Large,
            PresetBoardSizes::Regular => Self::Small,
            PresetBoardSizes::Large => Self::Regular,
        };
    }

    fn display_text(&self) -> String {
        String::from(match self {
            PresetBoardSizes::Small => "Small (2p)",
            PresetBoardSizes::Regular => "Regular (3-4p)",
            PresetBoardSizes::Large => "Large (5+ p)",
        })
    }
}

fn display_clickthrough_selectors<C: ClickThroughSelector>(commands: &mut Commands) -> Entity {
    commands.insert_resource(C::default());

    const CENTER_FONTSIZE: FontSize = FontSize::Vh(3.0);

    let overall_box = commands
        .spawn(Node {
            width: Val::Vw(20.0),
            justify_content: JustifyContent::SpaceBetween,
            ..default()
        })
        .id();

    let button_colors = ui_panels::hoverable_elements::create_hoverable_ui_bundle(
        BorderColor::all(SLATE_800),
        BackgroundColor(SLATE_600.into()),
        BorderColor::all(SLATE_500),
        BackgroundColor(SLATE_400.into()),
    );

    commands
        .spawn((
            ChildOf(overall_box),
            Node {
                height: Val::Percent(100.0),
                border: UiRect::all(UNIVERSAL_BORDER_WIDTH),
                border_radius: BorderRadius {
                    top_left: Val::Px(5.0),
                    bottom_left: Val::Px(5.0),
                    ..default()
                },
                ..default()
            },
            button_colors.clone(),
            children![(Text::new("<"), TextFont::from_font_size(CENTER_FONTSIZE),)],
        ))
        .observe(
            |_: On<Pointer<Click>>,
             mut resource: ResMut<C>,
             mut text: Single<&mut Text, With<EntityWithTextRepresentingResource<C>>>| {
                resource.previous_option();
                text.0 = resource.display_text();
            },
        );

    #[derive(Debug, Component)]
    struct EntityWithTextRepresentingResource<C: ClickThroughSelector>(PhantomData<C>);

    commands.spawn((
        ChildOf(overall_box),
        Node {
            flex_grow: 2.0,
            border: UiRect::top(UNIVERSAL_BORDER_WIDTH).with_bottom(UNIVERSAL_BORDER_WIDTH),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        BackgroundColor(SLATE_900.into()),
        BorderColor::all(SLATE_950),
        children![(
            Text::new(C::default().display_text()),
            TextFont::from_font_size(CENTER_FONTSIZE),
            EntityWithTextRepresentingResource::<C>(PhantomData)
        )],
    ));

    commands
        .spawn((
            ChildOf(overall_box),
            Node {
                height: Val::Percent(100.0),
                border: UiRect::all(UNIVERSAL_BORDER_WIDTH),
                border_radius: BorderRadius {
                    top_right: Val::Px(5.0),
                    bottom_right: Val::Px(5.0),
                    ..default()
                },
                ..default()
            },
            button_colors,
            children![(Text::new(">"), TextFont::from_font_size(CENTER_FONTSIZE),)],
        ))
        .observe(
            |_: On<Pointer<Click>>,
             mut resource: ResMut<C>,
             mut text: Single<&mut Text, With<EntityWithTextRepresentingResource<C>>>| {
                resource.next_option();
                text.0 = resource.display_text();
            },
        );

    overall_box
}

#[derive(Debug)]
struct FullyConnectedClient {
    name: String,
    client_id: u64,
    is_spectator: bool,
}
#[derive(Debug, Resource)]
struct InfoForConnectedClients(Vec<FullyConnectedClient>);

fn render_pregame_if_server(
    mut commands: Commands,
    client_info: Res<InfoForConnectedClients>,
    background_node: Single<Entity, With<MenusBackgroundNode>>,
) {
    commands.entity(background_node.entity()).despawn_children();

    commands.spawn((
        ChildOf(background_node.entity()),
        Text::new("Fully Connected Players"),
        TextFont::from_font_size(HEADER_SIZE),
    ));

    let player_name_displaybox = commands
        .spawn((
            Node {
                flex_direction: FlexDirection::Column,
                ..default()
            },
            ChildOf(background_node.entity()),
        ))
        .id();

    commands.spawn((
        ChildOf(background_node.entity()),
        Text::new("Fully Connected Spectators"),
        TextFont::from_font_size(HEADER_SIZE),
    ));

    let spectator_name_displaybox = commands
        .spawn((
            Node {
                flex_direction: FlexDirection::Column,
                ..default()
            },
            ChildOf(background_node.entity()),
        ))
        .id();

    let fully_connected_clients = client_info.0.iter();

    let mut player_count = 0;

    for FullyConnectedClient {
        name, is_spectator, ..
    } in fully_connected_clients
    {
        commands.spawn((
            ChildOf(if *is_spectator {
                spectator_name_displaybox
            } else {
                player_count += 1;
                player_name_displaybox
            }),
            Text::new(name.clone()),
        ));
    }

    if player_count > 1 {
        commands
            .spawn((
                ChildOf(background_node.entity()),
                Node {
                    width: Val::Percent(60.0),
                    height: BUTTON_HEIGHT,
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    border: UiRect::all(UNIVERSAL_BORDER_WIDTH),
                    border_radius: BorderRadius::all(Val::Px(6.0)),
                    ..default()
                },
                ui_panels::hoverable_elements::create_hoverable_ui_bundle(
                    BorderColor::all(SLATE_800),
                    BackgroundColor(SLATE_700.into()),
                    BorderColor::all(SLATE_500),
                    BackgroundColor(SLATE_600.into()),
                ),
                children![(
                    Text::new("Start Match"),
                    TextFont::from_font_size(BUTTON_TEXT_SIZE),
                )],
            ))
            .observe(
                |_: On<Pointer<Click>>,
                 mut server: ResMut<RenetServer>,
                 client_info: Res<InfoForConnectedClients>,
                 board_size: Res<PresetBoardSizes>,
                 mut commands: Commands| {
                    let mut player_names = Vec::new();
                    let mut player_client_ids = Vec::new();

                    for FullyConnectedClient {
                        name,
                        client_id,
                        is_spectator,
                    } in client_info.0.iter()
                    {
                        if !is_spectator {
                            player_names.push(name.clone());
                            player_client_ids.push(*client_id);
                        }
                    }

                    let player_names = player_names.into_boxed_slice();
                    let player_client_ids = player_client_ids.into_boxed_slice();

                    for client in server.clients_id() {
                        let start_game_transmission =
                            NetworkTransmission::StartGame(BoardSetupInstructions {
                                board_size: *board_size,
                                player_names: player_names.clone(),
                                you_are_player: player_client_ids
                                    .iter()
                                    .enumerate()
                                    .find(|(_, client_id)| client == **client_id)
                                    .map(|(index, _)| index as u8),
                            });

                        server.send_message(
                            client,
                            DefaultChannel::ReliableOrdered,
                            postcard::to_stdvec(&start_game_transmission).unwrap(),
                        );
                    }
                },
            );
    } else {
        commands.spawn((
            ChildOf(background_node.entity()),
            Text::new("Register at least two players to start the game."),
            TextFont::from_font_size(HEADER_SIZE),
        ));
    }
}

fn get_client_info(
    mut server: ResMut<RenetServer>,
    mut fully_connected_clients: ResMut<InfoForConnectedClients>,
) {
    for client_id in server.clients_id() {
        if fully_connected_clients
            .0
            .iter()
            .find(
                |FullyConnectedClient {
                     client_id: connected_client_id,
                     ..
                 }| *connected_client_id == client_id,
            )
            .is_none()
        {
            while let Some(message) =
                server.receive_message(client_id, DefaultChannel::ReliableOrdered)
            {
                let Ok(NetworkTransmission::InitialConnectionMessage { name, is_spectator }) =
                    postcard::from_bytes::<NetworkTransmission>(&message)
                else {
                    warn!(
                        "Received unexpected message from client with id {}. Either it could not be deserialized, or it was of an unexpected discriminant for this stage of the app's life.)",
                        client_id
                    );
                    continue;
                };

                fully_connected_clients.0.push(FullyConnectedClient {
                    name,
                    client_id,
                    is_spectator,
                });

                let confirmation_message =
                    postcard::to_stdvec(&NetworkTransmission::ConnectionConfirmationMessage)
                        .unwrap();

                server.send_message(
                    client_id,
                    DefaultChannel::ReliableOrdered,
                    confirmation_message,
                );
            }
        }
    }
}

fn render_pre_game_if_client(
    mut commands: Commands,
    background_node: Single<Entity, With<MenusBackgroundNode>>,
) {
    commands.entity(background_node.entity()).despawn_children();

    commands.spawn((
        ChildOf(background_node.entity()),
        Text::new("Connecting to server..."),
        TextFont::from_font_size(HEADER_SIZE),
    ));
}

fn render_client_connection_status_during_pregame(
    mut commands: Commands,
    background_node: Single<Entity, With<MenusBackgroundNode>>,
    networking_mode: Res<MultiplayerNetworkingMode>,
    mut client: ResMut<RenetClient>,
    mut asset_server: ResMut<AssetServer>,
    mut state: ResMut<NextState<AppState>>,
) {
    while let Some(message) = client.receive_message(DefaultChannel::ReliableOrdered) {
        if let Ok(transmission) = postcard::from_bytes::<NetworkTransmission>(&message) {
            match transmission {
                NetworkTransmission::ConnectionConfirmationMessage => {
                    if *networking_mode == MultiplayerNetworkingMode::Host {
                        println!("Host player/spectator is connected.");
                        continue;
                    }

                    commands.entity(background_node.entity()).despawn_children();

                    commands.spawn((
                        ChildOf(background_node.entity()),
                        Text::new("Waiting on host to start the game."),
                        TextFont::from_font_size(HEADER_SIZE),
                    ));
                }
                NetworkTransmission::StartGame(instructions) => {
                    state.set(AppState::InGame);
                    commands.entity(background_node.entity()).despawn_children();
                    create_board(&mut commands, &mut asset_server, instructions).unwrap()
                }
                NetworkTransmission::GameplayRequest(..) => unreachable!(),
                NetworkTransmission::InitialConnectionMessage { .. } => {
                    unreachable!()
                }
            }
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BoardSetupInstructions {
    pub board_size: PresetBoardSizes,
    pub player_names: Box<[String]>,
    pub you_are_player: Option<u8>,
}
