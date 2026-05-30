use bevy::{color::palettes::tailwind::*, prelude::*};

use crate::{functional_assets::SetUpBoard, ui_panels::upper_panel::VisualInventoryPlugin};

mod upper_panel;

pub struct UiPanelsPlugin;

impl Plugin for UiPanelsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(SetUpBoard, spawn_basic_ui_layout);
        app.add_plugins(VisualInventoryPlugin);
    }
}

pub const UNIVERSAL_BACKGROUND: Color = Color::Srgba(ZINC_800);
pub const UNIVERSAL_BORDER: Color = Color::Srgba(ZINC_900);
pub const UNIVERSAL_BORDER_WIDTH: Val = Val::Px(6.0);

pub fn spawn_basic_ui_layout(mut commands: Commands) {
    pub const SIDE_PANELS_WIDTH_AS_A_PERCENT: f32 = 25.0;

    let overall_parent = commands
        .spawn(Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            justify_content: JustifyContent::SpaceBetween,
            ..default()
        })
        .id();

    // the inventory, orders, and market panels
    {
        const OVERALL_PANEL_HEIGHTS: Val = Val::Percent(32.0);
        const OVERALL_PANEL_WIDTHS: Val = Val::Percent(96.0);
        const PANEL_BACKGROUNDS: Color = Color::Srgba(STONE_700);
        const PANEL_BORDERS: Color = Color::Srgba(STONE_800);

        commands.spawn((
            Node {
                width: Val::Percent(SIDE_PANELS_WIDTH_AS_A_PERCENT),
                height: Val::Percent(100.0),
                border: UiRect::all(UNIVERSAL_BORDER_WIDTH),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::SpaceAround,
                align_content: AlignContent::Center,
                ..default()
            },
            BorderColor::all(UNIVERSAL_BORDER),
            BackgroundColor(UNIVERSAL_BACKGROUND),
            ChildOf(overall_parent),
            children![
                (
                    InventoryPanel,
                    Node {
                        width: OVERALL_PANEL_WIDTHS,
                        height: OVERALL_PANEL_HEIGHTS,
                        border: UiRect::all(UNIVERSAL_BORDER_WIDTH),
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        align_self: AlignSelf::Center,
                        justify_content: JustifyContent::SpaceAround,
                        ..default()
                    },
                    BorderColor::all(PANEL_BORDERS),
                    BackgroundColor(PANEL_BACKGROUNDS)
                ),
                (
                    OrdersPanel,
                    Node {
                        width: OVERALL_PANEL_WIDTHS,
                        height: OVERALL_PANEL_HEIGHTS,
                        align_self: AlignSelf::Center,
                        border: UiRect::all(UNIVERSAL_BORDER_WIDTH),
                        ..default()
                    },
                    BorderColor::all(PANEL_BORDERS),
                    BackgroundColor(PANEL_BACKGROUNDS)
                ),
                (
                    MarketPanel,
                    Node {
                        width: OVERALL_PANEL_WIDTHS,
                        align_self: AlignSelf::Center,
                        height: OVERALL_PANEL_HEIGHTS,
                        border: UiRect::all(UNIVERSAL_BORDER_WIDTH),
                        ..default()
                    },
                    BorderColor::all(PANEL_BORDERS),
                    BackgroundColor(PANEL_BACKGROUNDS)
                )
            ],
        ));
    }

    commands.spawn((
        Node {
            width: Val::Percent(SIDE_PANELS_WIDTH_AS_A_PERCENT),
            height: Val::Percent(100.0),
            border: UiRect::all(UNIVERSAL_BORDER_WIDTH),
            ..default()
        },
        BorderColor::all(UNIVERSAL_BORDER),
        BackgroundColor(UNIVERSAL_BACKGROUND),
        ChildOf(overall_parent),
    ));
}

#[derive(Debug, Component)]
struct InventoryPanel;
#[derive(Debug, Component)]
struct OrdersPanel;
#[derive(Debug, Component)]
struct MarketPanel;

pub struct HeaderParameters {
    pub border_color: Color,
    pub background_color: Color,
    pub border_thickness: Val,
    pub height: Val,
    pub width: Val,
    pub text_size_px: f32,
}

pub const LEFT_SIDE_HEADER_PARAMS: HeaderParameters = HeaderParameters {
    border_color: Color::Srgba(ZINC_900),
    background_color: Color::Srgba(ZINC_600),
    border_thickness: Val::Px(3.0),
    height: Val::Px(30.0),
    width: Val::Percent(96.0),
    text_size_px: 24.0,
};
