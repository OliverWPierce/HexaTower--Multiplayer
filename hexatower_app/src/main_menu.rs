use bevy::prelude::*;
use bevy::text::EditableText;
use bevy::{color::palettes::tailwind::*, text::TextCursorStyle};

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

    const BUTTON_TEXT_SIZE: FontSize = FontSize::Vh(6.0);
    const BUTTON_HEIGHT: Val = Val::Vh(10.0);

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
                        Text::new("IP address:"),
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
        }
        MultiplayerNetworkingMode::Client => todo!(),
    }
}

#[derive(Debug, Component)]
struct IpAdressCollectionNode;
#[derive(Debug, Component)]
struct GamertagCollectionNode;
