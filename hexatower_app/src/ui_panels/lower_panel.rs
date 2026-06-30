use bevy::{color::palettes::tailwind::*, prelude::*};
use core_game_logic::markets::LogicalMarket;

use crate::{
    functional_assets::{VisCardDirectory, VisMarket},
    inputs_interface::LoadedAction,
    ui_panels::{LEFT_SIDE_HEADER_PARAMS, hoverable_elements},
    vis_tiles::ActiveTile,
};

pub struct VisualMarketUIPlugin;

impl Plugin for VisualMarketUIPlugin {
    fn build(&self, app: &mut App) {
        // app.add_systems(
        //     Update,
        //     manage_inventory_panel
        //         .before(execution_button::update_panel)
        //         .run_if(resource_changed_or_removed::<LoadedAction>),
        // );

        // app.add_systems(
        //     SetUpBoard,
        //     manage_inventory_panel.after(spawn_basic_ui_layout),
        // );

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
                max_width: LEFT_SIDE_HEADER_PARAMS.width,
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

    for (card, price) in logical_market.0 {
        let background_board = Node {
            width: Val::Percent(20.0),
            height: Val::Percent(50.0),
            flex_direction: FlexDirection::Column,
            align_content: AlignContent::Center,
            justify_content: JustifyContent::SpaceAround,
            ..default()
        };

        commands.spawn((
            ChildOf(container_for_item_boards),
            background_board,
            BackgroundColor(Color::BLACK),
            children![
                (
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
                    children![(
                        ImageNode {
                            image: visual_cards.get_card(card)?.image.clone(),
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
