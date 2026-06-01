use bevy::{color::palettes::tailwind::*, prelude::*};
use core_game_logic::players::{InventoryIndex, PlayerCardInventory, PlayerDirectory};

use crate::{
    DisplayPlayer,
    functional_assets::{LogicalWorld, SetUpBoard, VisCardDirectory, VisualCardId},
    ui_panels::{LEFT_SIDE_HEADER_PARAMS, hoverable_elements},
};

use super::{InventoryPanel, spawn_basic_ui_layout};

pub struct VisualInventoryPlugin;

impl Plugin for VisualInventoryPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(SetUpBoard, render_inventory.after(spawn_basic_ui_layout));
        app.add_observer(hover_slot);
        app.add_observer(unhover_slot);
    }
}
#[derive(Debug, Component)]
struct Header;

const INVENTORY_LABEL: &str = "Inventory";

fn render_inventory(
    mut commands: Commands,
    panel: Single<Entity, With<InventoryPanel>>,
    vis_cards: Res<VisCardDirectory>,
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
        Header,
        Text::new(INVENTORY_LABEL),
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
        .expect("all logical players have an inventory for their cards.");

    let players_cards = card_inventory.all_cards();

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

    for maybe_index in 0..card_inventory.max_card_capacity() {
        if let Some(card) = players_cards.get(maybe_index as usize) {
            commands.spawn((
                Node {
                    aspect_ratio: Some(1.0),
                    height: Val::Percent(30.0),
                    border_radius: BorderRadius::all(Val::Percent(10.0)),
                    border: UiRect::all(Val::Px(3.0)),
                    ..default()
                },
                hoverable_elements::create_hoverable_ui_bundle(
                    BorderColor::all(SLATE_950),
                    BackgroundColor(Color::Srgba(SKY_800)),
                    BorderColor::all(SLATE_400),
                    BackgroundColor(Color::Srgba(SKY_700)),
                ),
                ChildOf(container_for_item_icons),
                VisualCardIndex(InventoryIndex(maybe_index)),
                VisualCardId(*card),
                InventorySlot,
                children![(
                    ImageNode {
                        image: vis_cards
                            .0
                            .get(card.0 as usize)
                            .ok_or(format!("No visual found for card {card:?}"))?
                            .image
                            .clone(),
                        image_mode: NodeImageMode::Stretch,
                        ..default()
                    },
                    Node {
                        width: Val::Percent(100.0),
                        height: Val::Percent(100.0),
                        ..default()
                    }
                )],
            ));
        } else {
            commands.spawn((
                Node {
                    aspect_ratio: Some(1.0),
                    height: Val::Percent(30.0),
                    border_radius: BorderRadius::all(Val::Percent(10.0)),
                    border: UiRect::all(Val::Px(3.0)),
                    ..default()
                },
                InventorySlot,
                hoverable_elements::create_hoverable_ui_bundle(
                    BorderColor::all(SLATE_950),
                    BackgroundColor(Color::Srgba(SKY_950)),
                    BorderColor::all(SLATE_400),
                    BackgroundColor(Color::Srgba(SKY_900)),
                ),
                ChildOf(container_for_item_icons),
            ));
        }
    }

    Ok(())
}
#[derive(Debug, Component)]
struct InventorySlot;

#[derive(Debug, Component)]
struct VisualCardIndex(InventoryIndex);

fn hover_slot(
    trigger: On<Pointer<Over>>,
    mut header_text: Single<&mut Text, With<Header>>,
    vis_cards: Res<VisCardDirectory>,
    mut squares_in_inventory: Query<Option<&VisualCardId>, With<InventorySlot>>,
) -> Result<(), BevyError> {
    let Ok(slot_status) = squares_in_inventory.get_mut(trigger.entity) else {
        return Ok(());
    };

    if let Some(&VisualCardId(card)) = slot_status {
        header_text.0 = vis_cards
            .0
            .get(card.0 as usize)
            .ok_or(format!("No visual found for card {card:?}"))?
            .name
            .clone();
    } else {
        header_text.0 = "Empty Slot".into();
    }

    Ok(())
}

fn unhover_slot(
    trigger: On<Pointer<Out>>,
    mut header_text: Single<&mut Text, With<Header>>,
    squares_in_inventory: Query<(), With<InventorySlot>>,
) {
    if squares_in_inventory.get(trigger.entity).is_err() {
        return;
    }

    header_text.0 = INVENTORY_LABEL.into();
}
