use bevy::color::palettes::tailwind::*;
use bevy::prelude::*;

use crate::{
    AppState,
    inputs_interface::MultiplayerNetworkingMode,
    ui_panels::{self, UNIVERSAL_BACKGROUND, UNIVERSAL_BORDER_WIDTH},
};

pub struct MainMenuAndLobbyPluggin;

impl Plugin for MainMenuAndLobbyPluggin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::MainMenu), render_main_menu);
    }
}

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
        ))
        .id();

    commands.spawn((
        ChildOf(source_node),
        Text::new("Hexatower"),
        TextFont::from_font_size(FontSize::Vh(15.0)),
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
                Text::new("Join Online Game"),
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
                Text::new("Host Online Game"),
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
