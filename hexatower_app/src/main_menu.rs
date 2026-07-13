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
use bevy_renet::renet::ConnectionConfig;
use bevy_renet::{RenetClient, RenetServer};

use crate::VERSION_NUMBER;
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
                .observe(|_: On<Pointer<Click>>, mut commands: Commands, ip_address: Single<&EditableText, With<IpAdressCollectionNode>>,| {
                    let Ok(server_addr) = ip_address.value().to_string().parse() else {warn!("Invalid IP adress");return};
                        let socket = UdpSocket::bind(server_addr).unwrap();
                        let server_config = ServerConfig {
                            current_time: SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap(),
                            max_clients: 64,
                            protocol_id: VERSION_NUMBER,
                            public_addresses: vec![server_addr],
                            authentication: ServerAuthentication::Unsecure,
                        };

                    let client = RenetClient::new(ConnectionConfig::default());
                        commands.insert_resource(client);
                    let host_server = RenetServer::new(ConnectionConfig::default());
                    commands.insert_resource(host_server);
                    let transport = NetcodeServerTransport::new(server_config, socket).unwrap();
                    commands.insert_resource(transport);
                });
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
                    .observe(|_: On<Pointer<Click>>, mut commands: Commands, ip_address: Single<&EditableText, With<IpAdressCollectionNode>>,| {
                        let Ok(server_addr) = ip_address.value().to_string().parse() else {warn!("Invalid IP adress");return};

                            let client = RenetClient::new(ConnectionConfig::default());
                                commands.insert_resource(client);

                                let authentication = ClientAuthentication::Unsecure {
                                    server_addr,
                                    client_id: 0,
                                    user_data: None,
                                    protocol_id: 0,
                                };
                                let socket = UdpSocket::bind(server_addr).unwrap();
                                let current_time = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap();
                                let transport = NetcodeClientTransport::new(current_time, authentication, socket).unwrap();

                                commands.insert_resource(transport);
                    });
        }
    }
}

trait ClickThroughSelector: Resource<Mutability = Mutable> + Default {
    fn next_option(&mut self);

    fn previous_option(&mut self);

    fn display_text(&self) -> String;
}
#[derive(Debug, Resource, Default)]
enum PresetBoardSizes {
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
            PresetBoardSizes::Large => "Large (5+ p",
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
