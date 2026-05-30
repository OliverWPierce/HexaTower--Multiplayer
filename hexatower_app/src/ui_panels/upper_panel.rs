use bevy::{color::palettes::tailwind, prelude::*};
use core_game_logic::players::{PlayerCardInventory, PlayerDirectory};

use crate::{
    DisplayPlayer,
    functional_assets::{LogicalWorld, SetUpBoard},
    ui_panels::LEFT_SIDE_HEADER_PARAMS,
};

use super::{InventoryPanel, spawn_basic_ui_layout};

pub struct VisualInventoryPlugin;

impl Plugin for VisualInventoryPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(SetUpBoard, render_inventory.after(spawn_basic_ui_layout));
    }
}
#[derive(Debug, Component)]
struct Header;

/// This is not a mathematical constant, its just shows how many squares to draw, even when there is nothing in the inventory.
pub const SLOTS_IN_INVENTORY: u8 = 6;

fn render_inventory(
    mut commands: Commands,
    panel: Single<Entity, With<InventoryPanel>>,
    log_world: Res<LogicalWorld>,
    display_player: Res<DisplayPlayer>,
) -> Result<(), BevyError> {
    commands.entity(panel.entity()).despawn_children();

    commands.spawn((
        Node {
            height: LEFT_SIDE_HEADER_PARAMS.height,
            width: LEFT_SIDE_HEADER_PARAMS.width,
            border: UiRect::all(LEFT_SIDE_HEADER_PARAMS.border_thickness),
            ..default()
        },
        BackgroundColor(LEFT_SIDE_HEADER_PARAMS.background_color),
        BorderColor::all(LEFT_SIDE_HEADER_PARAMS.border_color),
        Text::new("Inventory"),
        TextFont {
            font_size: LEFT_SIDE_HEADER_PARAMS.text_size_px,
            ..default()
        },
        ChildOf(panel.entity()),
        TextLayout {
            justify: Justify::Center,
            linebreak: LineBreak::WordBoundary,
        },
    ));

    let log_player = log_world
        .0
        .resource::<PlayerDirectory>()
        .get_player(display_player.0)?;

    let card_inventory = log_world
        .0
        .get::<PlayerCardInventory>(log_player)
        .expect("all logical players have an inventory for their cards.")
        .all_cards();

    let container_for_item_icons = commands
        .spawn((
            Node {
                max_width: LEFT_SIDE_HEADER_PARAMS.width,
                height: Val::Percent(80.0),
                flex_wrap: FlexWrap::Wrap,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::SpaceAround,
                flex_grow: 1.0,
                ..default()
            },
            ChildOf(panel.entity()),
        ))
        .id();

    let squares_to_draw = if card_inventory.len() as u8 > SLOTS_IN_INVENTORY {
        card_inventory.len() as u8
    } else {
        SLOTS_IN_INVENTORY
    };

    for maybe_index in 0..squares_to_draw {
        if let Some(card) = card_inventory.get(maybe_index as usize) {
            commands.spawn((
                Node {
                    aspect_ratio: Some(1.0),
                    height: Val::Percent(40.0),
                    ..default()
                },
                BackgroundColor(tailwind::BLUE_600.into()),
                ChildOf(container_for_item_icons),
            ));
        } else {
            commands.spawn((
                Node {
                    aspect_ratio: Some(1.0),
                    height: Val::Percent(40.0),
                    ..default()
                },
                BackgroundColor(tailwind::AMBER_700.into()),
                ChildOf(container_for_item_icons),
            ));
        }
    }

    Ok(())
}
