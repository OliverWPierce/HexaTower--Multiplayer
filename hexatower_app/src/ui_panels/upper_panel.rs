use bevy::{color::palettes::tailwind::*, prelude::*};
use core_game_logic::{
    cards::{CardDirectory, CardId},
    forensic_action_descriptions::{ForensicDescribe, TextSnippet},
    players::{InventoryIndex, PlayerCardInventory, PlayerDirectory},
};

use crate::{
    OperatingPlayer,
    functional_assets::{LogicalWorld, SetUpBoard, VisCardDirectory, VisualCardId},
    inputs_interface::{ActionInputManager, FrontendAction},
    ui_panels::{
        LEFT_SIDE_HEADER_PARAMS, UnloadActionButton,
        display_themes::DEFAULT_COLOR_THEME,
        execution_button::{self, ExecutionButtonPanel},
        hoverable_elements,
    },
};

use super::{InventoryPanel, spawn_basic_ui_layout};

pub struct VisualInventoryPlugin;

impl Plugin for VisualInventoryPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            manage_inventory_panel
                .before(execution_button::update_panel)
                .run_if(resource_changed::<ActionInputManager>),
        );

        app.add_systems(
            SetUpBoard,
            manage_inventory_panel.after(spawn_basic_ui_layout),
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
    commands: &mut Commands,
    panel: Entity,
    vis_cards: &VisCardDirectory,
    log_world: &LogicalWorld,
    display_player: &OperatingPlayer,
) -> Result<(), BevyError> {
    commands.entity(panel).despawn_children();

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
        ChildOf(panel),
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
            ChildOf(panel),
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
    mut loaded_action: ResMut<ActionInputManager>,
    logical_world: Res<LogicalWorld>,
) -> Result<(), BevyError> {
    let Ok((VisualCardIndex(index_in_the_players_inventory), VisualCardId(card))) =
        cards_in_inventory.get(trigger.entity)
    else {
        return Ok(());
    };

    loaded_action.try_load_action(Some(FrontendAction::UseCard {
        index: *index_in_the_players_inventory,
        cache: logical_world
            .0
            .resource::<CardDirectory>()
            .get_card(*card)?
            .functionality
            .action_cache(&logical_world.0)?,
    }))?;

    Ok(())
}

fn render_card_execution_panel(
    parent_panel: Entity,
    visual_cards: &VisCardDirectory,
    commands: &mut Commands,
    description: Box<[TextSnippet]>,
    card: CardId,
) -> Result<(), BevyError> {
    let card_details = visual_cards.get_card(card)?;

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

    const DESCRIPTION_FONT_SIZE: f32 = 16.0;

    let description_block = commands
        .spawn((
            Node {
                max_width: LEFT_SIDE_HEADER_PARAMS.width,
                width: LEFT_SIDE_HEADER_PARAMS.width,
                border: UiRect::top(LEFT_SIDE_HEADER_PARAMS.border_thickness),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::Center,
                ..default()
            },
            BorderColor::all(LEFT_SIDE_HEADER_PARAMS.border_color),
            ChildOf(parent_panel.entity()),
            Text::new(""),
        ))
        .id();

    for snippet in description {
        match snippet {
            TextSnippet::PlainText { text, color } => {
                if let Some(color) = color {
                    commands.spawn((ChildOf(description_block), TextFont::from_font_size(DESCRIPTION_FONT_SIZE + 2.0), TextSpan::new(text), TextColor(match color {
                    core_game_logic::forensic_action_descriptions::ColorIndicators::Money => DEFAULT_COLOR_THEME.money_color,
                    core_game_logic::forensic_action_descriptions::ColorIndicators::Damage => DEFAULT_COLOR_THEME.negative_color,
                    core_game_logic::forensic_action_descriptions::ColorIndicators::Health => DEFAULT_COLOR_THEME.positive_color,
                    core_game_logic::forensic_action_descriptions::ColorIndicators::GeneralHighlight => DEFAULT_COLOR_THEME.highlight_color,
                    core_game_logic::forensic_action_descriptions::ColorIndicators::Unimportant => DEFAULT_COLOR_THEME.unimportant_color,
                })));
                } else {
                    commands.spawn((
                        ChildOf(description_block),
                        TextFont::from_font_size(DESCRIPTION_FONT_SIZE),
                        TextSpan::new(text),
                    ));
                }
            }
            TextSnippet::Link(linked_gamplay_element) => {
                commands.spawn((
                    ChildOf(description_block),
                    TextSpan::new("LINK"),
                    TextFont::from_font_size(DESCRIPTION_FONT_SIZE + 2.0),
                ));
            }
        }
    }

    commands.spawn((
        Node {
            min_height: Val::Px(20.0),
            ..default()
        },
        ChildOf(parent_panel),
        children![(
            Text::new(format!("\"{}\"", card_details.tooltip)),
            TextFont::from_font_size(16.0),
            TextColor(Color::Hsva(Hsva {
                hue: 0.0,
                saturation: 0.0,
                value: 0.7,
                alpha: 1.0,
            }))
        )],
    ));

    Ok(())
}

fn manage_inventory_panel(
    parent_panel: Single<Entity, With<InventoryPanel>>,
    display_player: Res<OperatingPlayer>,
    loaded_action: Res<ActionInputManager>,
    log_world: Res<LogicalWorld>,
    mut commands: Commands,
    vis_cards: Res<VisCardDirectory>,
) -> Result<(), BevyError> {
    debug!("managing");

    if let Some(FrontendAction::UseCard { index, cache }) = loaded_action.loaded_action() {
        let card = log_world
            .0
            .get::<PlayerCardInventory>(
                log_world
                    .0
                    .resource::<PlayerDirectory>()
                    .get_player(display_player.0)?,
            )
            .expect("all players should have an inventory")
            .get_card(*index)?;
        render_card_execution_panel(
            parent_panel.entity(),
            &vis_cards,
            &mut commands,
            cache.forensic_description(),
            card,
        )
    } else {
        render_inventory(
            &mut commands,
            parent_panel.entity(),
            &vis_cards,
            &log_world,
            &display_player,
        )
    }
}
