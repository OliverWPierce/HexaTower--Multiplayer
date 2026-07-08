use bevy::{color::palettes::tailwind::*, prelude::*};
use core_game_logic::{
    cards::{CardDirectory, CardId},
    forensic_action_descriptions::{ForensicDescribe, LinkedGamplayElement, TextSnippet},
    markets::{CardPrice, LogicalMarket, MarketDirectory, SlotInMarket},
    tiles::{MarketTile, TileDirectory},
};

use crate::{
    AppState, OperatingPlayer,
    functional_assets::{
        LogicalWorld, SetUpBoard, VisCardDirectory, VisMarket, VisMarketDirectory,
    },
    inputs_interface::{ActionInputManager, FrontendAction},
    ui_panels::{
        LEFT_SIDE_HEADER_PARAMS, MarketPanel, TextLinkToGameplayElement, UnloadActionButton,
        display_themes::{self, DEFAULT_COLOR_THEME, DESCRIPTION_FONT_SIZE, TOOLTIP_FONT_SIZE},
        execution_button::{self, ExecutionButtonPanel},
        hoverable_elements, spawn_basic_ui_layout,
    },
    vis_pieces::{self, visual_piece_archetypes_storage::VisualPieceArchetypeDirectory},
};

pub struct VisualMarketUIPlugin;

impl Plugin for VisualMarketUIPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            manage_market_ui_panel
                .before(execution_button::update_panel)
                .run_if(in_state(AppState::InGame))
                .run_if(resource_exists_and_changed::<ActionInputManager>),
        );

        app.add_systems(
            SetUpBoard,
            manage_market_ui_panel.after(spawn_basic_ui_layout),
        );

        app.add_observer(load_purchase_action.run_if(in_state(AppState::InGame)));
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
                font_size: LEFT_SIDE_HEADER_PARAMS.text_size,
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
                        font_size: LEFT_SIDE_HEADER_PARAMS.text_size,
                        ..default()
                    },
                    TextLayout {
                        justify: Justify::Center,
                        linebreak: LineBreak::WordBoundary,
                    },
                    TextColor(DEFAULT_COLOR_THEME.money_color),
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
                font_size: DESCRIPTION_FONT_SIZE,
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
    logical_world: &LogicalWorld,
    vis_markets: &VisMarketDirectory,
    vis_pieces: &VisualPieceArchetypeDirectory,
    operating_player: &OperatingPlayer,
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
                children![
                    (
                        Text::new(format!("Buy {}, for ", card_details.name)),
                        TextFont {
                            font_size: LEFT_SIDE_HEADER_PARAMS.text_size,
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
                            font_size: LEFT_SIDE_HEADER_PARAMS.text_size,
                            ..default()
                        },
                        TextLayout {
                            justify: Justify::Center,
                            linebreak: LineBreak::WordBoundary,
                        },
                        TextColor(DEFAULT_COLOR_THEME.money_color),
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

    for snippet in logical_world
        .0
        .resource::<CardDirectory>()
        .get_card(*card)?
        .functionality
        .action_cache(&logical_world.0)?
        .forensic_description()
    {
        match snippet {
            TextSnippet::PlainText { text, color } => {
                if let Some(color) = color {
                    commands.spawn((ChildOf(description_block), TextFont::from_font_size(DESCRIPTION_FONT_SIZE), TextSpan::new(text), TextColor(match color {
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
            TextSnippet::Link(linked_gamplay_element) => match linked_gamplay_element {
                core_game_logic::forensic_action_descriptions::LinkedGamplayElement::Card(
                    card_id,
                ) => {
                    commands.spawn((
                        ChildOf(description_block),
                        TextColor(DEFAULT_COLOR_THEME.highlight_color),
                        TextSpan::new(visual_cards.get_card(card_id)?.name.clone()),
                        TextFont::from_font_size(DESCRIPTION_FONT_SIZE),
                        TextLinkToGameplayElement(linked_gamplay_element),
                    ));
                }
                core_game_logic::forensic_action_descriptions::LinkedGamplayElement::Piece(
                    archetype_id,
                ) => {
                    commands.spawn((
                        ChildOf(description_block),
                        TextColor(DEFAULT_COLOR_THEME.highlight_color),
                        TextSpan::new(vis_pieces.get_visual_details(archetype_id)?.name.clone()),
                        TextFont::from_font_size(DESCRIPTION_FONT_SIZE),
                        TextLinkToGameplayElement(linked_gamplay_element),
                    ));
                }
                core_game_logic::forensic_action_descriptions::LinkedGamplayElement::Market(
                    market_id,
                ) => {
                    commands.spawn((
                        ChildOf(description_block),
                        TextColor(DEFAULT_COLOR_THEME.highlight_color),
                        TextSpan::new(vis_markets.get_market(market_id)?.name.clone()),
                        TextFont::from_font_size(DESCRIPTION_FONT_SIZE),
                        TextLinkToGameplayElement(linked_gamplay_element),
                    ));
                }
                core_game_logic::forensic_action_descriptions::LinkedGamplayElement::Tile(
                    tile_type,
                ) => {
                    commands.spawn((
                        ChildOf(description_block),
                        TextColor(display_themes::color_for_tile_type_under_default_theme(
                            &tile_type,
                        )),
                        TextSpan::new(display_themes::name_for_tile_type_under_default_theme(
                            &tile_type,
                        )),
                        TextFont::from_font_size(DESCRIPTION_FONT_SIZE),
                        TextLinkToGameplayElement(LinkedGamplayElement::Tile(tile_type)),
                    ));
                }
                core_game_logic::forensic_action_descriptions::LinkedGamplayElement::Player(
                    player_id,
                ) => {
                    commands.spawn((
                        ChildOf(description_block),
                        TextColor(DEFAULT_COLOR_THEME.highlight_color),
                        TextLinkToGameplayElement(linked_gamplay_element),
                        TextSpan::new(if operating_player.0 == player_id {
                            "You"
                        } else {
                            "PLAYER NAME HERE"
                        }),
                        TextFont::from_font_size(DESCRIPTION_FONT_SIZE),
                    ));
                }
            },
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
            TextFont::from_font_size(TOOLTIP_FONT_SIZE),
            TextColor(DEFAULT_COLOR_THEME.unimportant_color)
        )],
    ));

    Ok(())
}

fn manage_market_ui_panel(
    panel: Single<Entity, With<MarketPanel>>,
    manager: Res<ActionInputManager>,
    logical_world: Res<LogicalWorld>,
    visual_markets: Res<VisMarketDirectory>,
    visual_cards: Res<VisCardDirectory>,
    vis_cards: Res<VisCardDirectory>,
    vis_markets: Res<VisMarketDirectory>,
    vis_pieces: Res<VisualPieceArchetypeDirectory>,
    operating_player: Res<OperatingPlayer>,
    mut commands: Commands,
) -> Result<(), BevyError> {
    if let Some(active_tile) = manager.active_tile()
        && let Some(&MarketTile(market)) = logical_world.0.get::<MarketTile>(
            logical_world
                .0
                .resource::<TileDirectory>()
                .get_entity(active_tile)?,
        )
    {
        if let Some(FrontendAction::PurchaseCard { slot }) = manager.loaded_action() {
            let (card, price) = logical_world
                .0
                .resource::<MarketDirectory>()
                .get_market(market)?
                .get_card_and_price(*slot);

            render_card_execution_panel(
                panel.entity(),
                &visual_cards,
                &mut commands,
                price,
                card,
                &logical_world,
                &vis_markets,
                &vis_pieces,
                &operating_player,
            )?
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

fn load_purchase_action(
    mut trigger: On<Pointer<Click>>,
    card_purchase_icons: Query<&IndicatesSlotInMarket>,
    mut action_manager: ResMut<ActionInputManager>,
) -> Result<(), BevyError> {
    let Ok(slot) = card_purchase_icons.get(trigger.entity) else {
        return Ok(());
    };

    trigger.propagate(false);

    action_manager.try_load_action(Some(FrontendAction::PurchaseCard { slot: slot.0 }))?;

    Ok(())
}
