use std::marker::PhantomData;
use std::time::SystemTime;

use bevy::ecs::component::Mutable;
use bevy::ecs::storage::Column;
use bevy::prelude::*;
use bevy::text::EditableText;
use bevy::{color::palettes::tailwind::*, text::TextCursorStyle};
use bevy_renet::{RenetClient, RenetServer};
use core_game_logic::players::PlayerData;
use core_game_logic::tile_mapping::tiles_on_board;
use core_game_logic::tiles::TileType;
use rand::RngExt;
use serde::{Deserialize, Serialize};

use crate::{HEXATOWER_APP_ID_FOR_STEAM, VERSION_NUMBER, functional_assets};

use crate::cameras::{MainMenuCam, spawn_main_menu_cam};
use crate::functional_assets::{LOGICAL_MARKETS, create_board};
use crate::inputs_interface::{HOST_CLIENT_ID, NetworkTransmission};
use crate::{
    AppState,
    inputs_interface::MultiplayerNetworkingMode,
    ui_panels::{self, UNIVERSAL_BACKGROUND, UNIVERSAL_BORDER_WIDTH},
};

pub struct MainMenuAndLobbyPluggin;

impl Plugin for MainMenuAndLobbyPluggin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(AppState::MainMenu),
            render_main_menu.after(spawn_main_menu_cam),
        );
    }
}

pub trait ClickThroughSelector: Resource<Mutability = Mutable> + Default {
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

impl PresetBoardSizes {
    #[inline]
    pub fn ring_count(&self) -> u32 {
        match self {
            PresetBoardSizes::Small => 3,
            PresetBoardSizes::Regular => 4,
            PresetBoardSizes::Large => 5,
        }
    }
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

pub fn display_clickthrough_selectors<C: ClickThroughSelector>(commands: &mut Commands) -> Entity {
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

const HEADER_SIZE: FontSize = FontSize::Vh(6.0);
const BUTTON_TEXT_SIZE: FontSize = FontSize::Vh(6.0);
const BUTTON_HEIGHT: Val = Val::Vh(10.0);

fn render_main_menu(mut commands: Commands, camera: Single<Entity, With<MainMenuCam>>) {
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
            UiTargetCamera(camera.entity()),
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
                Text::new("Party Mode"),
                TextFont::from_font_size(BUTTON_TEXT_SIZE),
            )],
        ))
        .observe(
            |_: On<Pointer<Click>>,
             mut commands: Commands,
             mut state: ResMut<NextState<AppState>>| {
                commands.insert_resource(MultiplayerNetworkingMode::SingleDevice);
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

#[derive(Resource)]
struct SteamPlayerInfo(renet_steam::steamworks::Client);

fn render_parameters(
    networking_mode: Res<MultiplayerNetworkingMode>,
    mut commands: Commands,
    menu_background: Single<Entity, With<MenusBackgroundNode>>,
) {
    commands.spawn((
        ChildOf(menu_background.entity()),
        Text::new(match *networking_mode {
            MultiplayerNetworkingMode::SingleDevice => "Party Mode",
            MultiplayerNetworkingMode::Host => "Local Host",
            MultiplayerNetworkingMode::Client => "Join Game",
        }),
        TextFont::from_font_size(HEADER_SIZE),
    ));

    match *networking_mode {
        MultiplayerNetworkingMode::Client => {
            let steam_client =
                renet_steam::steamworks::Client::init_app(HEXATOWER_APP_ID_FOR_STEAM).unwrap();

            commands.insert_resource(SteamPlayerInfo(
                renet_steam::steamworks::Client::init_app(HEXATOWER_APP_ID_FOR_STEAM).unwrap(),
            ));

            commands.spawn((
                ChildOf(menu_background.entity()),
                Text::new("Availible Friends"),
                TextFont::from_font_size(FontSize::Vh(3.0)),
            ));

            let container_for_name_boxes = commands
                .spawn((
                    ChildOf(menu_background.entity()),
                    Node {
                        width: Val::Percent(100.0),
                        height: BUTTON_HEIGHT,
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        border: UiRect::all(UNIVERSAL_BORDER_WIDTH),
                        border_radius: BorderRadius::all(Val::Px(6.0)),
                        overflow: Overflow::scroll_y(),
                        ..default()
                    },
                    BorderColor::all(SLATE_800),
                ))
                .id();

            for friend in steam_client
                .friends()
                .get_friends(renet_steam::steamworks::FriendFlags::IMMEDIATE)
            {
                let Some(game) = friend.game_played() else {
                    continue;
                };

                if game.game.app_id().0 != HEXATOWER_APP_ID_FOR_STEAM {
                    continue;
                }

                commands
                    .spawn((
                        ChildOf(container_for_name_boxes),
                        Node {
                            width: Val::Percent(100.0),
                            height: Val::Vh(4.0),
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
                            Text::new(friend.name()),
                            TextFont::from_font_size(FontSize::Vh(2.0)),
                        )],
                    ))
                    .observe(
                        |_: On<Pointer<Click>>,
                         mut commands: Commands,
                         steam_client: Res<SteamPlayerInfo>| {
                            steam_client
                                .0
                                .networking_utils()
                                .init_relay_network_access();
                            let connection_config = bevy_renet::renet::ConnectionConfig::default();
                            let mut client = RenetClient::new(connection_config);

                            // Create steam transport
                            let server_steam_id = renet_steam::steamworks::SteamId::from_raw(0); // Here goes the steam id of the host
                            let mut steam_transport = renet_steam::SteamClientTransport::new(
                                steam_client.0.clone(),
                                &server_steam_id,
                            )
                            .unwrap();
                        },
                    );
            }
        }
        _ => todo!(), // render actual parameters,
    }
}

#[derive(Debug, Component)]
struct MenusBackgroundNode;

#[derive(Debug)]
struct FullyConnectedClient {
    name: String,
    client_id: u64,
    is_spectator: bool,
}
#[derive(Debug, Resource)]
struct InfoForConnectedClients(Vec<FullyConnectedClient>);

type PlayerClientIds = PlayerData<u64>;

#[derive(Debug, Serialize, Deserialize)]
pub struct BoardSetupInstructions {
    pub board_size: PresetBoardSizes,
    pub player_names: Box<[String]>,
    pub you_are_player: Option<u8>,
    pub initial_market_tiles: [Box<[u32]>; functional_assets::LOGICAL_MARKETS.len()],
    pub initial_special_tiles: Box<[(Box<[u32]>, TileType)]>,
}

struct RandomBoardFeatures {
    pub initial_market_tiles: [Box<[u32]>; functional_assets::LOGICAL_MARKETS.len()],
    pub initial_special_tiles: Box<[(Box<[u32]>, TileType)]>,
}

impl RandomBoardFeatures {
    #[inline]
    fn new(board_size: PresetBoardSizes) -> Self {
        let mut availible_tiles = (0..tiles_on_board(board_size.ring_count())).collect::<Vec<_>>();
        let mut rng = rand::rng();

        let instances_of_each_market = match board_size {
            PresetBoardSizes::Small => 1,
            PresetBoardSizes::Regular => 2,
            PresetBoardSizes::Large => 4,
        };

        let market_tiles = LOGICAL_MARKETS.map(|_| {
            (0..instances_of_each_market)
                .map(|_| {
                    let index = rng.random_range(0..availible_tiles.len());
                    availible_tiles.swap_remove(index)
                })
                .collect::<Box<[_]>>()
        });

        let gold_tiles = (
            (0..match board_size {
                PresetBoardSizes::Small => 3,
                PresetBoardSizes::Regular => 5,
                PresetBoardSizes::Large => 10,
            })
                .map(|_| {
                    let index = rng.random_range(0..availible_tiles.len());
                    availible_tiles.swap_remove(index)
                })
                .collect::<Box<[_]>>(),
            TileType::Gold,
        );

        RandomBoardFeatures {
            initial_market_tiles: market_tiles,
            initial_special_tiles: Box::new([gold_tiles]),
        }
    }
}
