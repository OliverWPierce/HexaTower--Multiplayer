use bevy::{color::palettes::tailwind, prelude::*};

use crate::functional_assets::SetUpBoard;

pub struct UiPanelsPlugin;

impl Plugin for UiPanelsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(SetUpBoard, spawn_basic_ui_layout);
    }
}

#[derive(Debug, Resource)]
struct UpperRowNode(Entity);
#[derive(Debug, Resource)]
struct MidRowNode(Entity);
#[derive(Debug, Resource)]
struct LowerRowNode(Entity);

#[derive(Debug, Resource)]
struct ExecutableDisplayPanelNode(Entity);

// these must add to 100.0 percent of the window's height.
const HEIGHT_OF_UPPER_ROW: Val = Val::Percent(20.0);
const HEIGHT_OF_MIDDLE_ROW: Val = Val::Percent(60.0);
const HEIGHT_OF_LOWER_ROW: Val = Val::Percent(20.0);

// these must add to 100.0 percent of the window's width.
const WIDTH_OF_EXECUTABLE_PANEL: Val = Val::Percent(20.0);
const WIDTH_OF_TRIGGER_BUTTON_PANEL: Val = Val::Percent(40.0);
const WIDTH_OF_INSPECTOR_PANEL: Val = Val::Percent(40.0);

const UNIVERSAL_BACKGROUND_COLOR: Color = Color::Srgba(tailwind::SLATE_700);
const UNIVERSAL_BORDER_COLOR: Color = Color::Srgba(tailwind::SLATE_900);

fn spawn_basic_ui_layout(mut commands: Commands) {
    let overall_parent = commands
        .spawn(Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            ..default()
        })
        .id();

    let upper_row = commands
        .spawn((
            Node {
                height: HEIGHT_OF_LOWER_ROW,
                width: Val::Percent(100.0),
                justify_content: JustifyContent::SpaceBetween,
                ..default()
            },
            ChildOf(overall_parent),
            children![
                (
                    Node {
                        height: Val::Percent(100.0),
                        width: Val::Percent(20.0),
                        border: UiRect::all(Val::Percent(0.5)),
                        border_radius: BorderRadius::bottom_right(Val::Percent(100.0)),
                        ..default()
                    },
                    BackgroundColor(UNIVERSAL_BACKGROUND_COLOR),
                    BorderColor::all(UNIVERSAL_BORDER_COLOR),
                ),
                (
                    Node {
                        height: Val::Percent(100.0),
                        width: Val::Percent(20.0),
                        border: UiRect::all(Val::Percent(0.5)),
                        border_radius: BorderRadius::bottom_left(Val::Percent(100.0)),
                        ..default()
                    },
                    BackgroundColor(UNIVERSAL_BACKGROUND_COLOR),
                    BorderColor::all(UNIVERSAL_BORDER_COLOR),
                )
            ],
        ))
        .id();

    let mid_row = commands
        .spawn((
            Node {
                height: HEIGHT_OF_MIDDLE_ROW,
                width: Val::Percent(100.0),
                ..default()
            },
            ChildOf(overall_parent),
        ))
        .id();

    let lower_row = commands
        .spawn((
            Node {
                height: HEIGHT_OF_LOWER_ROW,
                width: Val::Percent(100.0),
                border: UiRect::all(Val::Percent(0.5)),
                ..default()
            },
            BackgroundColor(UNIVERSAL_BACKGROUND_COLOR),
            BorderColor::all(UNIVERSAL_BORDER_COLOR),
            ChildOf(overall_parent),
        ))
        .id();

    commands.insert_resource(UpperRowNode(upper_row));
    commands.insert_resource(MidRowNode(mid_row));
    commands.insert_resource(LowerRowNode(lower_row));
}
