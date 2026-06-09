use bevy::{color::palettes::tailwind::*, prelude::*};
use core_game_logic::{
    orders::OrderDirectory,
    pieces::{OccupiedByPiece, Orders},
    tiles::TileDirectory,
};

use crate::{
    functional_assets::{LogicalWorld, SetUpBoard, VisOrderDirectory},
    inputs_interface::{LoadedAction, Source},
    ui_panels::{LEFT_SIDE_HEADER_PARAMS, OrdersPanel, hoverable_elements, spawn_basic_ui_layout},
    vis_tiles::ActiveTile,
};

pub struct VisualOrdersPlugin;

impl Plugin for VisualOrdersPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(SetUpBoard, manage_orders_panel.after(spawn_basic_ui_layout));

        app.add_systems(
            Update,
            manage_orders_panel.run_if(
                resource_changed_or_removed::<ActiveTile>
                    .or(resource_changed_or_removed::<LoadedAction>),
            ),
        );

        app.add_observer(load_order);
    }
}

#[derive(Debug, Component)]
struct Header;

const ORDER_LIBRARY_LABEL: &str = "Piece Capabilities";

fn render_orders_of_active_piece(
    overarching_order_panel: Entity,
    logical_entity_of_active_piece: Entity,
    logical_world: &LogicalWorld,
    visual_order_data: &VisOrderDirectory,
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
                font_size: LEFT_SIDE_HEADER_PARAMS.text_size_px,
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

    Ok(())
}

#[derive(Debug, Component, Clone, Copy)]
pub struct OrderAtPieceIndex(pub u8);

fn load_order(
    click: On<Pointer<Click>>,
    mut commands: Commands,
    orders: Query<&OrderAtPieceIndex>,
    active_piece: If<Res<ActiveTile>>,
    logical_world: Res<LogicalWorld>,
) -> Result<(), BevyError> {
    let Ok(&order_index) = orders.get(click.entity) else {
        return Ok(());
    };
    commands.insert_resource(LoadedAction {
        source: Source::Order(order_index),
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
                                    .get_entity(active_piece.0.0)?,
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
            .action_cache(active_piece.0.0, &logical_world.0)?,
    });

    Ok(())
}

fn manage_orders_panel(
    overarching_order_panel: Single<Entity, With<OrdersPanel>>,
    loaded_action: Option<Res<LoadedAction>>,
    active_tile: Option<Res<ActiveTile>>,
    logical_world: Res<LogicalWorld>,
    visual_order_data: Res<VisOrderDirectory>,
    mut commands: Commands,
) -> Result<(), BevyError> {
    if let Some(tile_of_active_piece) = active_tile
        && let Some(active_piece) = logical_world.0.get::<OccupiedByPiece>(
            logical_world
                .0
                .resource::<TileDirectory>()
                .get_entity(tile_of_active_piece.0)?,
        )
    {
        if let Some(action) = loaded_action {
            match action.source {
                Source::Order(order_index) => todo!(), // render the order execution process panel,
                _ => render_orders_of_active_piece(
                    overarching_order_panel.entity(),
                    active_piece.piece(),
                    &logical_world,
                    &visual_order_data,
                    &mut commands,
                ),
            }
        } else {
            render_orders_of_active_piece(
                overarching_order_panel.entity(),
                active_piece.piece(),
                &logical_world,
                &visual_order_data,
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
