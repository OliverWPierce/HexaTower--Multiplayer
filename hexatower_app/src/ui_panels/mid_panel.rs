use bevy::{color::palettes::tailwind::*, prelude::*};
use core_game_logic::{
    forensic_action_descriptions::{ForensicDescribe, TextSnippet},
    orders::OrderDirectory,
    pieces::{OccupiedByPiece, Orders, OrdersReceivable, PieceOwnedByPlayer},
    players::{PlayerDirectory, PlayerId, PlayerOrdersRemaining},
    tiles::TileDirectory,
};

use crate::{
    AppState, OperatingPlayer,
    functional_assets::{
        LogicalWorld, PlayerNames, SetUpBoard, VisCardDirectory, VisMarketDirectory,
        VisOrderDirectory,
    },
    inputs_interface::{ActionInputManager, FrontendAction},
    ui_panels::{
        LEFT_SIDE_HEADER_PARAMS, OrdersPanel, UnloadActionButton,
        display_themes::{DEFAULT_COLOR_THEME, DESCRIPTION_FONT_SIZE, TOOLTIP_FONT_SIZE},
        execution_button::ExecutionButtonPanel,
        hoverable_elements, spawn_basic_ui_layout,
    },
    vis_pieces::visual_piece_archetypes_storage::VisualPieceArchetypeDirectory,
};

pub struct VisualOrdersPlugin;

impl Plugin for VisualOrdersPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(SetUpBoard, manage_orders_panel.after(spawn_basic_ui_layout));

        app.add_systems(
            Update,
            manage_orders_panel
                .run_if(in_state(AppState::InGame))
                .run_if(resource_exists_and_changed::<ActionInputManager>),
        );

        app.add_observer(load_order.run_if(in_state(AppState::InGame)));
    }
}

#[derive(Debug, Component)]
struct Header;

const ORDER_LIBRARY_LABEL: &str = "Piece Capabilities";

fn render_piece_overview(
    overarching_order_panel: Entity,
    logical_entity_of_active_piece: Entity,
    logical_world: &LogicalWorld,
    visual_order_data: &VisOrderDirectory,
    operating_player: &OperatingPlayer,
    commands: &mut Commands,
) -> Result<(), BevyError> {
    commands
        .entity(overarching_order_panel.entity())
        .despawn_children();

    let orders_to_display = {
        logical_world
            .0
            .get::<Orders>(logical_entity_of_active_piece)
            .expect("All pieces should have an orders component")
            .0
    };

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
            Text::new(ORDER_LIBRARY_LABEL),
            TextFont {
                font_size: LEFT_SIDE_HEADER_PARAMS.text_size,
                ..default()
            },
            TextLayout {
                justify: Justify::Center,
                linebreak: LineBreak::WordBoundary,
            },
        )],
        ChildOf(overarching_order_panel),
    ));

    let container_for_order_icons = commands
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
            ChildOf(overarching_order_panel),
        ))
        .id();

    for (index, maybe_order) in orders_to_display.iter().enumerate() {
        if let Some(order) = maybe_order {
            let visual_details = visual_order_data.get_order(*order)?;

            commands.spawn((
                Node {
                    aspect_ratio: Some(1.0),
                    height: Val::Percent(30.0),
                    border_radius: BorderRadius::all(Val::Percent(100.0)),
                    border: UiRect::all(Val::Px(3.0)),
                    ..default()
                },
                hoverable_elements::create_hoverable_ui_bundle(
                    BorderColor::all(SLATE_950),
                    BackgroundColor(Color::Srgba(ZINC_800)),
                    BorderColor::all(SLATE_400),
                    BackgroundColor(Color::Srgba(ZINC_700)),
                ),
                ChildOf(container_for_order_icons),
                OrderAtPieceIndex(index as u8),
                children![(
                    ImageNode {
                        image: visual_details.image.clone(),
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
                    border_radius: BorderRadius::all(Val::Percent(100.0)),
                    border: UiRect::all(Val::Px(3.0)),
                    ..default()
                },
                hoverable_elements::create_hoverable_ui_bundle(
                    BorderColor::all(SLATE_950),
                    BackgroundColor(Color::Srgba(ZINC_950)),
                    BorderColor::all(SLATE_400),
                    BackgroundColor(Color::Srgba(ZINC_900)),
                ),
                ChildOf(container_for_order_icons),
            ));
        }
    }

    {
        let piece_owner = logical_world
            .0
            .get::<PieceOwnedByPlayer>(logical_entity_of_active_piece);

        let piece_orders = logical_world
            .0
            .get::<OrdersReceivable>(logical_entity_of_active_piece)
            .ok_or(
                "all piece's should contain information about how many orders they can receive",
            )?;

        let big_container_bar = commands
            .spawn((
                ChildOf(overarching_order_panel),
                Node {
                    width: LEFT_SIDE_HEADER_PARAMS.width,
                    height: Val::Px(24.0),
                    border: UiRect::all(Val::Px(2.0)),
                    justify_content: JustifyContent::SpaceBetween,
                    padding: UiRect::left(Val::Px(4.0)).with_right(Val::Px(4.0)),
                    ..default()
                },
                BackgroundColor(LEFT_SIDE_HEADER_PARAMS.background_color),
                BorderColor::all(LEFT_SIDE_HEADER_PARAMS.border_color),
            ))
            .id();

        // the piece's orders.
        commands.spawn((
            ChildOf(big_container_bar),
            Node {
                height: Val::Percent(100.0),
                ..default()
            },
            children![
                (Text::new("Piece orders: "), TextFont::from_font_size(16.0),),
                (
                    Text::new(format!("{}", piece_orders.currently)),
                    TextFont::from_font_size(16.0),
                    TextColor(match piece_orders.currently {
                        0 => ROSE_600.into(),
                        1 => ROSE_300.into(),
                        2 => AMBER_300.into(),
                        3 => EMERALD_300.into(),
                        _ => TEAL_300.into(),
                    })
                ),
                (
                    Text::new(format!("/{}", piece_orders.per_round)),
                    TextFont::from_font_size(16.0),
                ),
            ],
        ));
        if let Some(owner) = piece_owner {
            let owner_id = *logical_world
                .0
                .get::<PlayerId>(owner.0)
                .ok_or("A player did not have a player Id")?;

            if owner_id == operating_player.0 {
                commands.spawn((
                    ChildOf(big_container_bar),
                    Node {
                        height: Val::Percent(100.0),
                        ..default()
                    },
                    children![(
                        Text::new(String::from("Commander: You")),
                        TextFont::from_font_size(16.0),
                    ),],
                ));
            } else {
                commands.spawn((
                    ChildOf(big_container_bar),
                    Node {
                        height: Val::Percent(100.0),
                        ..default()
                    },
                    children![(
                        Text::new(format!("Commander: {:?}", owner_id)),
                        TextFont::from_font_size(16.0),
                    )],
                ));
            }
        } else {
            commands.spawn((
                ChildOf(big_container_bar),
                Node {
                    height: Val::Percent(100.0),
                    ..default()
                },
                children![(
                    Text::new(String::from("For Sale")),
                    TextFont::from_font_size(16.0),
                )],
            ));
        }
    }

    Ok(())
}

#[derive(Debug, Component, Clone, Copy)]
pub struct OrderAtPieceIndex(pub u8);

fn load_order(
    click: On<Pointer<Click>>,
    orders: Query<&OrderAtPieceIndex>,
    logical_world: Res<LogicalWorld>,
    mut input_manager: ResMut<ActionInputManager>,
) -> Result<(), BevyError> {
    let Ok(&order_index) = orders.get(click.entity) else {
        return Ok(());
    };

    let Some(active_piece) = input_manager.active_tile() else {
        warn!("Player clicked an icon to load an order, but there was no active tile");
        return Ok(());
    };

    input_manager.try_load_action(Some(FrontendAction::UseOrder {
        index_of_order_on_active_piece: order_index,
        cache: logical_world
            .0
            .resource::<OrderDirectory>()
            .get_order(
                logical_world
                    .0
                    .get::<Orders>(
                        logical_world
                            .0
                            .get::<OccupiedByPiece>(
                                logical_world
                                    .0
                                    .resource::<TileDirectory>()
                                    .get_entity(active_piece),
                            )
                            .ok_or("Tile was unnoccupied")?
                            .piece(),
                    )
                    .expect("all pieces should store data about the orders they use")
                    .0
                    .get(order_index.0 as usize)
                    .unwrap()
                    .ok_or("No order found at this index for this piece")?,
            )?
            .functionality
            .action_cache(active_piece, &logical_world.0)?,
    }))?;

    Ok(())
}

fn manage_orders_panel(
    overarching_order_panel: Single<Entity, With<OrdersPanel>>,
    input_manager: Res<ActionInputManager>,
    logical_world: Res<LogicalWorld>,
    visual_order_data: Res<VisOrderDirectory>,
    operating_player: Res<OperatingPlayer>,
    vis_cards: Res<VisCardDirectory>,
    vis_markets: Res<VisMarketDirectory>,
    vis_pieces: Res<VisualPieceArchetypeDirectory>,
    mut commands: Commands,
    player_names: Res<PlayerNames>,
) -> Result<(), BevyError> {
    if let Some(tile_of_active_piece) = input_manager.active_tile()
        && let Some(active_piece_log_entity) = logical_world.0.get::<OccupiedByPiece>(
            logical_world
                .0
                .resource::<TileDirectory>()
                .get_entity(tile_of_active_piece),
        )
    {
        if let Some(FrontendAction::UseOrder {
            index_of_order_on_active_piece,
            cache,
        }) = input_manager.loaded_action()
        {
            render_order_execution_process(
                &mut commands,
                &logical_world,
                overarching_order_panel.entity(),
                active_piece_log_entity.piece(),
                index_of_order_on_active_piece,
                &visual_order_data,
                &operating_player,
                cache.forensic_description(),
                &vis_cards,
                &vis_markets,
                &vis_pieces,
                &player_names,
            )
        } else {
            render_piece_overview(
                overarching_order_panel.entity(),
                active_piece_log_entity.piece(),
                &logical_world,
                &visual_order_data,
                &operating_player,
                &mut commands,
            )
        }
    } else {
        display_when_no_active_piece(&mut commands, overarching_order_panel.entity());
        Ok(())
    }
}

fn display_when_no_active_piece(commands: &mut Commands, parent_panel: Entity) {
    commands.entity(parent_panel).despawn_children();

    commands.spawn((
        Text::new("Activate a tile with a piece to view its orders."),
        TextFont::from_font_size(24.0),
        ChildOf(parent_panel),
        TextLayout {
            justify: Justify::Center,
            linebreak: LineBreak::WordBoundary,
        },
    ));
}

fn render_order_execution_process(
    commands: &mut Commands,
    logical_world: &LogicalWorld,
    parent_panel: Entity,
    logical_piece: Entity,
    order_index: &OrderAtPieceIndex,
    visual_order_data: &VisOrderDirectory,
    operating_player: &OperatingPlayer,
    description: Box<[TextSnippet]>,
    visual_cards: &VisCardDirectory,
    vis_markets: &VisMarketDirectory,
    vis_pieces: &VisualPieceArchetypeDirectory,
    player_names: &PlayerNames,
) -> Result<(), BevyError> {
    let order_details = {
        let order = logical_world
            .0
            .get::<Orders>(logical_piece)
            .expect("all pieces should have an order")
            .0
            .get(order_index.0 as usize)
            .ok_or("Order index is out of bounds")?
            .ok_or("No order in this slot")?;
        visual_order_data.get_order(order)?
    };

    commands.entity(parent_panel).despawn_children();

    commands.spawn((
        Node {
            height: LEFT_SIDE_HEADER_PARAMS.height,
            width: LEFT_SIDE_HEADER_PARAMS.width,
            flex_shrink: 0.0,
            ..default()
        },
        ChildOf(parent_panel),
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
                        font_size: LEFT_SIDE_HEADER_PARAMS.text_size,
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
                    Text::new(order_details.name.clone()),
                    TextFont {
                        font_size: LEFT_SIDE_HEADER_PARAMS.text_size,
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
            width: LEFT_SIDE_HEADER_PARAMS.width,
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
                    border_radius: BorderRadius::all(Val::Percent(100.0)),
                    border: UiRect::all(Val::Px(3.0)),
                    ..default()
                },
                BorderColor::all(SLATE_950),
                BackgroundColor(Color::Srgba(ZINC_800)),
                children![(
                    ImageNode {
                        image: order_details.image.clone(),
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

    // display remaining orders, and/or owner.
    {
        let player_orders = logical_world
            .0
            .get::<PlayerOrdersRemaining>(
                *logical_world
                    .0
                    .resource::<PlayerDirectory>()
                    .get(operating_player.0),
            )
            .ok_or("All players should have information about their remaining orders")?
            .remaining;

        let piece_owner = logical_world.0.get::<PieceOwnedByPlayer>(logical_piece);

        let piece_orders = logical_world
            .0
            .get::<OrdersReceivable>(logical_piece)
            .ok_or(
                "all piece's should contain information about how many orders they can receive",
            )?;

        let big_container_bar = commands
            .spawn((
                ChildOf(parent_panel),
                Node {
                    width: LEFT_SIDE_HEADER_PARAMS.width,
                    height: Val::Px(24.0),
                    border: UiRect::all(Val::Px(2.0)),
                    justify_content: JustifyContent::SpaceBetween,
                    padding: UiRect::left(Val::Px(4.0)).with_right(Val::Px(4.0)),
                    ..default()
                },
                BackgroundColor(LEFT_SIDE_HEADER_PARAMS.background_color),
                BorderColor::all(LEFT_SIDE_HEADER_PARAMS.border_color),
            ))
            .id();

        commands.spawn((
            ChildOf(big_container_bar),
            Node {
                height: Val::Percent(100.0),
                ..default()
            },
            children![
                (Text::new("Piece orders: "), TextFont::from_font_size(16.0),),
                (
                    Text::new(format!("{}", piece_orders.currently)),
                    TextFont::from_font_size(16.0),
                    TextColor(match piece_orders.currently {
                        0 => ROSE_600.into(),
                        1 => ROSE_300.into(),
                        2 => AMBER_300.into(),
                        3 => EMERALD_300.into(),
                        _ => TEAL_300.into(),
                    })
                ),
                (
                    Text::new(format!("/{}", piece_orders.per_round)),
                    TextFont::from_font_size(16.0),
                ),
            ],
        ));
        if let Some(owner) = piece_owner {
            if *logical_world
                .0
                .get::<PlayerId>(owner.0)
                .ok_or("A player did not have a player Id")?
                == operating_player.0
            {
                commands.spawn((
                    ChildOf(big_container_bar),
                    Node {
                        height: Val::Percent(100.0),
                        ..default()
                    },
                    children![
                        (Text::new("Your orders: "), TextFont::from_font_size(16.0),),
                        (
                            Text::new(format!("{player_orders}")),
                            TextFont::from_font_size(16.0),
                            TextColor(match player_orders {
                                0 => ROSE_600.into(),
                                1 => ROSE_300.into(),
                                2 => AMBER_300.into(),
                                3 => EMERALD_300.into(),
                                _ => TEAL_300.into(),
                            })
                        )
                    ],
                ));
            } else {
                commands.spawn((
                    ChildOf(big_container_bar),
                    Node {
                        height: Val::Percent(100.0),
                        ..default()
                    },
                    children![(
                        Text::new(format!("Commander: {:?}", owner.0)),
                        TextFont::from_font_size(16.0),
                    )],
                ));
            }
        } else {
            commands.spawn((
                ChildOf(big_container_bar),
                Node {
                    height: Val::Percent(100.0),
                    ..default()
                },
                children![(
                    Text::new(String::from("For Sale")),
                    TextFont::from_font_size(16.0),
                )],
            ));
        }
    }

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

    super::add_description(
        description_block,
        &description,
        commands,
        DESCRIPTION_FONT_SIZE,
        visual_cards,
        vis_markets,
        vis_pieces,
        operating_player,
        player_names,
    )?;

    commands.spawn((
        Node {
            min_height: Val::Px(20.0),
            ..default()
        },
        ChildOf(parent_panel),
        children![(
            Text::new(format!("\"{}\"", order_details.tooltip)),
            TextFont::from_font_size(TOOLTIP_FONT_SIZE),
            TextColor(DEFAULT_COLOR_THEME.unimportant_color)
        )],
    ));

    Ok(())
}
