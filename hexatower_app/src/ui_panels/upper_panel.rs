use bevy::{color::palettes::tailwind::*, prelude::*};
use core_game_logic::{
    cards::CardDirectory,
    players::{InventoryIndex, PlayerCardInventory, PlayerDirectory},
};

use crate::{
    DisplayPlayer,
    functional_assets::{LogicalWorld, SetUpBoard, VisCardDirectory, VisualCardId},
    inputs_interface::{LoadedAction, Source},
    ui_panels::{
        LEFT_SIDE_HEADER_PARAMS, UnloadActionButton, execution_button::ExecutionButtonPanel,
        hoverable_elements,
    },
};

use super::{InventoryPanel, spawn_basic_ui_layout};

pub struct VisualInventoryPlugin;

impl Plugin for VisualInventoryPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(SetUpBoard, render_inventory.after(spawn_basic_ui_layout));
        app.add_systems(
            Update,
            render_inventory.run_if(resource_removed::<LoadedAction>),
        );
        app.add_observer(hover_slot);
        app.add_observer(unhover_slot);
        app.add_observer(load_card_action);
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
            justify_content: JustifyContent::Center,
            ..default()
        },
        BackgroundColor(LEFT_SIDE_HEADER_PARAMS.background_color),
        BorderColor::all(LEFT_SIDE_HEADER_PARAMS.border_color),
        children![(
            Header,
            Text::new(INVENTORY_LABEL),
            TextFont {
                font_size: LEFT_SIDE_HEADER_PARAMS.text_size_px,
                ..default()
            },
            TextLayout {
                justify: Justify::Center,
                linebreak: LineBreak::WordBoundary,
            },
        )],
        ChildOf(panel.entity()),
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
                height: Val::Percent(60.0),
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
                    BackgroundColor(Color::Srgba(ZINC_800)),
                    BorderColor::all(SLATE_400),
                    BackgroundColor(Color::Srgba(ZINC_700)),
                ),
                ChildOf(container_for_item_icons),
                VisualCardIndex(InventoryIndex(maybe_index)),
                VisualCardId(*card),
                InventorySlot,
                children![(
                    ImageNode {
                        image: vis_cards.get_card(*card)?.image.clone(),
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
                    BackgroundColor(Color::Srgba(ZINC_950)),
                    BorderColor::all(SLATE_400),
                    BackgroundColor(Color::Srgba(ZINC_900)),
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
        header_text.0 = vis_cards.get_card(card)?.name.clone();
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

fn load_card_action(
    trigger: On<Pointer<Click>>,
    cards_in_inventory: Query<(&VisualCardIndex, &VisualCardId)>,
    mut commands: Commands,
    logical_world: Res<LogicalWorld>,
    parent_panel: Single<Entity, With<InventoryPanel>>,
    visual_cards: Res<VisCardDirectory>,
) -> Result<(), BevyError> {
    let Ok((VisualCardIndex(index_in_the_players_inventory), VisualCardId(card))) =
        cards_in_inventory.get(trigger.entity)
    else {
        return Ok(());
    };

    commands.insert_resource(LoadedAction {
        source: Source::Card(*index_in_the_players_inventory),
        cache: logical_world
            .0
            .resource::<CardDirectory>()
            .get_card(*card)?
            .functionality
            .action_cache(&logical_world.0)?,
    });

    {
        let card_details = visual_cards.get_card(*card)?;

        commands.entity(parent_panel.entity()).despawn_children();

        commands.spawn((
            Node {
                height: LEFT_SIDE_HEADER_PARAMS.height,
                width: Val::Percent(96.0),
                flex_shrink: 0.0,
                ..default()
            },
            ChildOf(parent_panel.entity()),
            children![
                (
                    Node {
                        height: Val::Percent(100.0),
                        width: Val::Percent(15.0),
                        border: UiRect::all(LEFT_SIDE_HEADER_PARAMS.border_thickness),
                        justify_content: JustifyContent::Center,
                        ..default()
                    },
                    BorderColor::all(STONE_900),
                    BackgroundColor(Color::Srgba(STONE_700)),
                    UnloadActionButton,
                    children![(
                        Text::new("<--"),
                        TextFont {
                            font_size: LEFT_SIDE_HEADER_PARAMS.text_size_px,
                            ..default()
                        },
                        TextLayout {
                            justify: Justify::Center,
                            linebreak: LineBreak::WordBoundary,
                        },
                    )]
                ),
                (
                    Node {
                        height: Val::Percent(100.0),
                        width: Val::Percent(85.0),
                        border: UiRect::all(LEFT_SIDE_HEADER_PARAMS.border_thickness),
                        justify_content: JustifyContent::Center,
                        ..default()
                    },
                    BackgroundColor(LEFT_SIDE_HEADER_PARAMS.background_color),
                    BorderColor::all(LEFT_SIDE_HEADER_PARAMS.border_color),
                    children![(
                        Text::new(card_details.name.clone()),
                        TextFont {
                            font_size: LEFT_SIDE_HEADER_PARAMS.text_size_px,
                            ..default()
                        },
                        TextLayout {
                            justify: Justify::Center,
                            linebreak: LineBreak::WordBoundary,
                        },
                    )]
                ),
            ],
        ));

        commands.spawn((
            Node {
                width: Val::Percent(96.0),
                height: Val::Percent(25.0),
                flex_shrink: 0.0,
                ..Default::default()
            },
            ChildOf(parent_panel.entity()),
            children![
                (
                    Node {
                        aspect_ratio: Some(1.0),
                        flex_shrink: 0.0,
                        height: Val::Percent(100.0),
                        border_radius: BorderRadius::all(Val::Percent(10.0)),
                        border: UiRect::all(Val::Px(3.0)),
                        ..default()
                    },
                    BorderColor::all(SLATE_950),
                    BackgroundColor(Color::Srgba(ZINC_800)),
                    InventorySlot,
                    children![(
                        ImageNode {
                            image: card_details.image.clone(),
                            image_mode: NodeImageMode::Stretch,
                            ..default()
                        },
                        Node {
                            width: Val::Percent(100.0),
                            height: Val::Percent(100.0),
                            ..default()
                        }
                    ),],
                ),
                (
                    Node {
                        height: Val::Percent(100.0),
                        width: Val::Percent(100.0),
                        flex_direction: FlexDirection::Column,
                        ..default()
                    },
                    ExecutionButtonPanel
                )
            ],
        ));

        commands.spawn((
            Node {
                width: Val::Percent(96.0),
                height: Val::Percent(60.0),
                border: UiRect::top(LEFT_SIDE_HEADER_PARAMS.border_thickness),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                ..default()
            },
            BorderColor::all(LEFT_SIDE_HEADER_PARAMS.border_color),
            ChildOf(parent_panel.entity()),
            children![
                (
                    Text::new("This is a purely forensic description of what this item does...."),
                    TextLayout {
                        justify: Justify::Center,
                        linebreak: LineBreak::WordBoundary,
                    },
                    TextFont {
                        font_size: 18.0,
                        ..default()
                    }
                ),
                (
                    Text::new(card_details.tooltip.clone()),
                    TextLayout {
                        justify: Justify::Center,
                        linebreak: LineBreak::WordBoundary,
                    },
                    TextFont {
                        font_size: 12.0,
                        ..default()
                    }
                )
            ],
        ));
    }

    Ok(())
}
