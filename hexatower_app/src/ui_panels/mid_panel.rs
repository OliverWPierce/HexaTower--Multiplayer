use bevy::{color::palettes::tailwind::*, prelude::*};
use core_game_logic::{
    orders::{OrderDirectory, OrderFunction},
    pieces::{OccupiedByPiece, Orders},
    tiles::TileDirectory,
};

use crate::{
    functional_assets::{LogicalWorld, VisOrderDirectory},
    inputs_interface::{LoadedAction, Source},
    ui_panels::{LEFT_SIDE_HEADER_PARAMS, OrdersPanel, hoverable_elements},
    vis_pieces::VisOccupies,
    vis_tiles::ActiveTile,
};

pub struct VisualOrdersPlugin;

impl Plugin for VisualOrdersPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            render_orders_of_active_piece.run_if(resource_exists_and_changed::<ActiveTile>),
        );
    }
}

#[derive(Debug, Component)]
struct Header;

const ORDER_LIBRARY_LABEL: &str = "Piece Capabilities";

fn render_orders_of_active_piece(
    overarching_order_panel: Single<Entity, With<OrdersPanel>>,
    active_piece: Res<ActiveTile>,
    logical_world: Res<LogicalWorld>,
    visual_order_data: Res<VisOrderDirectory>,
    mut commands: Commands,
) -> Result<(), BevyError> {
    commands
        .entity(overarching_order_panel.entity())
        .despawn_children();

    let orders_to_display = {
        let logical_tile_occupied = logical_world
            .0
            .resource::<TileDirectory>()
            .get_entity(active_piece.0)?;
        let Some(logical_piece) = logical_world
            .0
            .get::<OccupiedByPiece>(logical_tile_occupied)
        else {
            return Ok(());
        };

        logical_world
            .0
            .get::<Orders>(logical_piece.piece())
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
        ChildOf(overarching_order_panel.entity()),
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
            ChildOf(overarching_order_panel.entity()),
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
    active_piece: Res<ActiveTile>,
    logical_world: Res<LogicalWorld>,
) -> Result<(), BevyError> {
    let Ok(&order_index) = orders.get(click.entity) else {
        return Ok(());
    };
    commands.insert_resource(LoadedAction {
        source: Source::Order {
            tile_of_piece: active_piece.0,
            order_index,
        },
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
                                    .get_entity(active_piece.0)?,
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
            .action_cache(active_piece.0, &logical_world.0)?,
    });

    Ok(())
}
