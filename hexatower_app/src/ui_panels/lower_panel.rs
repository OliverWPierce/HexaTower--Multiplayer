use bevy::{color::palettes::tailwind::*, prelude::*};
use core_game_logic::{
    cards::CardId,
    markets::{CardPrice, LogicalMarket, MarketDirectory, SlotInMarket},
    tiles::{MarketTile, TileDirectory},
};

use crate::{
    functional_assets::{
        LogicalWorld, SetUpBoard, VisCardDirectory, VisMarket, VisMarketDirectory,
    },
    ui_panels::{
        LEFT_SIDE_HEADER_PARAMS, MarketPanel, UnloadActionButton,
        execution_button::{self, ExecutionButtonPanel},
        hoverable_elements, spawn_basic_ui_layout,
    },
};

pub struct VisualMarketUIPlugin;

impl Plugin for VisualMarketUIPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            manage_market_ui_panel
                .before(execution_button::update_panel)
                .run_if(
                    resource_changed_or_removed::<ActiveTile>
                        .or(resource_changed_or_removed::<LoadedAction>),
                ),
        );

        app.add_systems(
            SetUpBoard,
            manage_market_ui_panel.after(spawn_basic_ui_layout),
        );

        // app.add_observer(hover_slot);
        // app.add_observer(unhover_slot);
        // app.add_observer(load_card_action);
    }
}

#[derive(Debug, Component)]
struct Header;

fn display_options(
    commands: &mut Commands,
    logical_market: &LogicalMarket,
    visual_market: &VisMarket,
    visual_cards: &VisCardDirectory,
    overarching_panel: Entity,
) -> Result<(), BevyError> {
    commands.entity(overarching_panel).despawn_children();

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
            Text::new(visual_market.name.clone()),
            TextFont {
                font_size: LEFT_SIDE_HEADER_PARAMS.text_size_px,
                ..default()
            },
            TextLayout {
                justify: Justify::Center,
                linebreak: LineBreak::WordBoundary,
            },
        )],
        ChildOf(overarching_panel),
    ));

    let container_for_item_boards = commands
        .spawn((
            Node {
                min_width: LEFT_SIDE_HEADER_PARAMS.width,
                height: Val::Percent(60.0),
                flex_wrap: FlexWrap::Wrap,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::SpaceAround,
                flex_grow: 1.0,
                ..default()
            },
            ChildOf(overarching_panel),
        ))
        .id();

    for (slot, (card, price)) in logical_market.0.iter().enumerate() {
        let background_board = Node {
            aspect_ratio: Some(0.8),
            width: Val::Percent(30.0),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::SpaceAround,
            border_radius: BorderRadius::all(Val::Percent(10.0)),
            border: UiRect::all(Val::Px(3.0)),
            ..default()
        };

        commands.spawn((
            ChildOf(container_for_item_boards),
            background_board,
            hoverable_elements::create_hoverable_ui_bundle(
                BorderColor::all(SLATE_950),
                BackgroundColor(Color::Srgba(ZINC_800)),
                BorderColor::all(SLATE_400),
                BackgroundColor(Color::Srgba(ZINC_700)),
            ),
            match slot {
                0 => IndicatesSlotInMarket(SlotInMarket::First),
                1 => IndicatesSlotInMarket(SlotInMarket::Second),
                2 => IndicatesSlotInMarket(SlotInMarket::Third),
                _ => unreachable!(),
            },
            children![
                (
                    Node {
                        aspect_ratio: Some(1.0),
                        width: Val::Percent(95.0),
                        border_radius: BorderRadius::all(Val::Percent(10.0)),
                        border: UiRect::all(Val::Px(3.0)),
                        ..default()
                    },
                    BorderColor::all(SLATE_950),
                    BackgroundColor(Color::Srgba(ZINC_800)),
                    children![(
                        ImageNode {
                            image: visual_cards.get_card(*card)?.image.clone(),
                            image_mode: NodeImageMode::Stretch,
                            ..default()
                        },
                        Node {
                            width: Val::Percent(100.0),
                            height: Val::Percent(100.0),
                            ..default()
                        }
                    )],
                ),
                (
                    Text::new(format!("{}", price.0)),
                    TextFont {
                        font_size: LEFT_SIDE_HEADER_PARAMS.text_size_px,
                        ..default()
                    },
                    TextLayout {
                        justify: Justify::Center,
                        linebreak: LineBreak::WordBoundary,
                    },
                    TextColor(AMBER_300.into()),
                )
            ],
        ));
    }

    commands.spawn((
        Node {
            height: LEFT_SIDE_HEADER_PARAMS.height,
            width: LEFT_SIDE_HEADER_PARAMS.width,
            justify_content: JustifyContent::Center,
            ..default()
        },
        children![(
            Text::new(visual_market.description.clone()),
            TextFont {
                font_size: 18.0,
                ..default()
            },
            TextLayout {
                justify: Justify::Center,
                linebreak: LineBreak::WordBoundary,
            },
        )],
        ChildOf(overarching_panel),
    ));

    Ok(())
}

fn render_card_execution_panel(
    parent_panel: Entity,
    visual_cards: &VisCardDirectory,
    commands: &mut Commands,
    price: &CardPrice,
    card: &CardId,
) -> Result<(), BevyError> {
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
                children![
                    (
                        Text::new(format!("Buy {}, for ", card_details.name)),
                        TextFont {
                            font_size: LEFT_SIDE_HEADER_PARAMS.text_size_px,
                            ..default()
                        },
                        TextLayout {
                            justify: Justify::Center,
                            linebreak: LineBreak::WordBoundary,
                        },
                    ),
                    (
                        Text::new(format!("{}", price.0)),
                        TextFont {
                            font_size: LEFT_SIDE_HEADER_PARAMS.text_size_px,
                            ..default()
                        },
                        TextLayout {
                            justify: Justify::Center,
                            linebreak: LineBreak::WordBoundary,
                        },
                        TextColor(AMBER_300.into()),
                    ),
                ]
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

    commands.spawn((
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
        children![
            (
                Text::new("This is a purely forensic description of what this order does.... It is such a long description that it stretches for many lines until it is far too cumbersome for the eyes to manage in a single sitting."),
                TextLayout {
                    justify: Justify::Center,
                    linebreak: LineBreak::WordBoundary,
                },
                TextFont {
                    font_size: 16.0,
                    ..default()
                }
            ),
        ],
    ));

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

fn manage_market_ui_panel(
    active_tile: Option<Res<ActiveTile>>,
    panel: Single<Entity, With<MarketPanel>>,
    logical_world: Res<LogicalWorld>,
    loaded_action: Option<Res<LoadedAction>>,
    visual_markets: Res<VisMarketDirectory>,
    visual_cards: Res<VisCardDirectory>,
    mut commands: Commands,
) -> Result<(), BevyError> {
    if let Some(tile) = active_tile
        && let Some(&MarketTile(market)) = logical_world.0.get::<MarketTile>(
            logical_world
                .0
                .resource::<TileDirectory>()
                .get_entity(tile.0)?,
        )
    {
        if let Some(action) = loaded_action
            && let crate::inputs_interface::Source::Market(slot) = action.source
        {
            let (card, price) = logical_world
                .0
                .resource::<MarketDirectory>()
                .get_market(market)?
                .get_card_and_price(slot);

            render_card_execution_panel(panel.entity(), &visual_cards, &mut commands, price, card)?
        } else {
            display_options(
                &mut commands,
                logical_world
                    .0
                    .resource::<MarketDirectory>()
                    .get_market(market)?,
                visual_markets.get_market(market)?,
                &visual_cards,
                panel.entity(),
            )?
        }
    } else {
        commands.entity(panel.entity()).despawn_children();

        commands.spawn((
            Text::new("Activate a tile with a market to view its offers."),
            TextFont::from_font_size(24.0),
            ChildOf(panel.entity()),
            TextLayout {
                justify: Justify::Center,
                linebreak: LineBreak::WordBoundary,
            },
        ));
    }

    Ok(())
}

#[derive(Debug, Component)]
struct IndicatesSlotInMarket(SlotInMarket);

// fn load_purchase_action(
//     click: On<Pointer<Click>>,
//     mut commands: Commands,
//     orders: Query<&IndicatesSlotInMarket>,
//     active_piece: If<Res<ActiveTile>>,
//     logical_world: Res<LogicalWorld>,
// ) -> Result<(), BevyError> {
//     let Ok(&order_index) = orders.get(click.entity) else {
//         return Ok(());
//     };

//     click.propagate(false);

//     commands.insert_resource(LoadedAction {
//         source: crate::inputs_interface::Source::Market(order_index),
//         cache: logical_world
//             .0
//             .resource::<OrderDirectory>()
//             .get_order(
//                 logical_world
//                     .0
//                     .get::<Orders>(
//                         logical_world
//                             .0
//                             .get::<OccupiedByPiece>(
//                                 logical_world
//                                     .0
//                                     .resource::<TileDirectory>()
//                                     .get_entity(active_piece.0.0)?,
//                             )
//                             .ok_or("Tile was unnoccupied")?
//                             .piece(),
//                     )
//                     .expect("all pieces should store data about the orders they use")
//                     .0
//                     .get(order_index.0 as usize)
//                     .unwrap()
//                     .ok_or("No order found at this index for this piece")?,
//             )?
//             .functionality
//             .action_cache(active_piece.0.0, &logical_world.0)?,
//     });

//     Ok(())
// }
