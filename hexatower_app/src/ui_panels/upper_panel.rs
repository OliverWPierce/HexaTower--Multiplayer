use bevy::{color::palettes::tailwind::*, prelude::*};
use core_game_logic::{
    cards::{CardDirectory, CardId},
    forensic_action_descriptions::{ForensicDescribe, TextSnippet},
    players::{Coins, InventoryIndex, PlayerCardInventory, PlayerDirectory, PlayerOrdersRemaining},
    tiles::TileDirectory,
};

use crate::{
    AppState, OperatingPlayer,
    functional_assets::{
        LogicalWorld, PlayerNames, VisCardDirectory, VisMarketDirectory, VisualCard, VisualCardId,
    },
    inputs_interface::{ActionInputManager, FrontendAction},
    ui_panels::{
        LEFT_SIDE_HEADER_PARAMS, StatBar, UNIVERSAL_BORDER_WIDTH, UnloadActionButton, display_bar,
        display_themes::{DEFAULT_COLOR_THEME, DESCRIPTION_FONT_SIZE, TOOLTIP_FONT_SIZE},
        execution_button::{self, ExecutionButtonPanel},
    },
    vis_pieces::visual_piece_archetypes_storage::VisualPieceArchetypeDirectory,
};

use super::{InventoryPanel, spawn_basic_ui_layout};

pub struct VisualInventoryPlugin;

impl Plugin for VisualInventoryPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            manage_inventory_panel
                .before(execution_button::update_panel)
                .run_if(in_state(AppState::InGame))
                .run_if(resource_exists_and_changed::<ActionInputManager>),
        );

        app.add_systems(
            OnEnter(AppState::InGame),
            manage_inventory_panel.after(spawn_basic_ui_layout),
        );

        app.add_observer(hover_slot.run_if(in_state(AppState::InGame)));
        app.add_observer(unhover_slot.run_if(in_state(AppState::InGame)));
        app.add_observer(load_card_action.run_if(in_state(AppState::InGame)));
    }
}
#[derive(Debug, Component)]
struct Header;

const INVENTORY_LABEL: &str = "Inventory";

impl StatBar for Coins {
    const STEP_SIZE: u32 = 3;

    const SEG_BACK_COLOR: Color = Color::Srgba(YELLOW_700);

    const SEG_BORDER_COLOR: Color = Color::Srgba(YELLOW_900);

    const SEG_EMPTY_BACK_COLOR: Color = Color::Srgba(GRAY_800);

    const SEG_EMPTY_BORDER_COLOR: Color = Color::Srgba(GRAY_900);

    const SUPERCHARGED_SEG_BORDER: Color = Color::Srgba(YELLOW_200);

    const ICON_PATH: &'static str = "item_images/purple_potion.png";

    const STAT_TITLE: &'static str = "Coins";

    fn teir(&self) -> Option<u8> {
        None
    }

    fn current(&self) -> u32 {
        self.0
    }

    fn max(&self) -> u32 {
        21
    }
}

impl StatBar for PlayerOrdersRemaining {
    const STEP_SIZE: u32 = 1;

    const SEG_BACK_COLOR: Color = Color::Srgba(ROSE_700);

    const SEG_BORDER_COLOR: Color = Color::Srgba(ROSE_950);

    const SEG_EMPTY_BACK_COLOR: Color = Color::Srgba(GRAY_800);

    const SEG_EMPTY_BORDER_COLOR: Color = Color::Srgba(GRAY_900);

    const SUPERCHARGED_SEG_BORDER: Color = Color::Srgba(ROSE_400);

    const ICON_PATH: &'static str = "item_images/purple_potion.png";

    const STAT_TITLE: &'static str = "Orders";
    #[inline]
    fn teir(&self) -> Option<u8> {
        None
    }

    #[inline]
    fn current(&self) -> u32 {
        self.remaining as u32
    }

    #[inline]
    fn max(&self) -> u32 {
        3
    }
}

impl StatBar for PlayerCardInventory {
    const STEP_SIZE: u32 = 1;

    const SEG_BACK_COLOR: Color = Color::Srgba(SKY_800);

    const SEG_BORDER_COLOR: Color = Color::Srgba(SKY_950);

    const SEG_EMPTY_BACK_COLOR: Color = Color::Srgba(GRAY_800);

    const SEG_EMPTY_BORDER_COLOR: Color = Color::Srgba(GRAY_900);

    const SUPERCHARGED_SEG_BORDER: Color = Color::Srgba(SKY_500);

    const ICON_PATH: &'static str = "card_images/market_1.png";

    const STAT_TITLE: &'static str = "Hand";

    #[inline]
    fn teir(&self) -> Option<u8> {
        None
    }
    #[inline]
    fn current(&self) -> u32 {
        self.cards_in_hand() as u32
    }
    #[inline]
    fn max(&self) -> u32 {
        self.max_card_capacity() as u32
    }
}

fn render_inventory(
    commands: &mut Commands,
    panel: Entity,
    vis_cards: &VisCardDirectory,
    log_world: &LogicalWorld,
    display_player: &OperatingPlayer,
    asset_server: &mut AssetServer,
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
                font_size: LEFT_SIDE_HEADER_PARAMS.text_size,
                ..default()
            },
            TextLayout {
                justify: Justify::Center,
                linebreak: LineBreak::WordBoundary,
            },
        )],
        ChildOf(panel),
    ));

    let log_player = *log_world
        .0
        .resource::<PlayerDirectory>()
        .get(display_player.0);

    let card_inventory = log_world
        .0
        .get::<PlayerCardInventory>(log_player)
        .expect("all logical players have an inventory for their cards.");

    let players_cards = card_inventory.all_cards();

    let container_for_item_icons = commands
        .spawn((
            Node {
                max_width: LEFT_SIDE_HEADER_PARAMS.width,
                height: Val::Percent(35.0),
                flex_wrap: FlexWrap::Wrap,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::SpaceAround,
                border: UiRect::all(UNIVERSAL_BORDER_WIDTH),
                border_radius: BorderRadius::all(Val::Percent(100.0)),
                flex_grow: 1.0,
                ..default()
            },
            ChildOf(panel),
        ))
        .id();

    for (index, &card) in players_cards.iter().enumerate() {
        let Ok(VisualCard { image, .. }) = vis_cards.get_card(card) else {
            continue;
        };

        commands
            .spawn((
                Node {
                    aspect_ratio: Some(3.0 / 5.0),
                    ..default()
                },
                ChildOf(container_for_item_icons),
                children![(ImageNode::from(image.clone()))],
            ))
            .observe(
                move |_: On<Pointer<Click>>,
                      mut action: ResMut<ActionInputManager>,
                      operator: Res<OperatingPlayer>,
                      logical_world: Res<LogicalWorld>|
                      -> Result<(), BevyError> {
                    action.try_load_action(Some(FrontendAction::UseCard {
                        index: InventoryIndex(index as u8),
                        cache: logical_world
                            .0
                            .resource::<CardDirectory>()
                            .get_card(card)?
                            .functionality
                            .action_cache(&logical_world.0, operator.0)?,
                    }))?;

                    println!(
                        "{:?}",
                        logical_world.0.resource::<TileDirectory>().tile_entities()[0]
                    );

                    Ok(())
                },
            );
    }

    display_bar(card_inventory, commands, asset_server, panel);

    if let Some(coins) = log_world.0.get::<Coins>(log_player) {
        display_bar(coins, commands, asset_server, panel);
    }

    if let Some(orders) = log_world.0.get::<PlayerOrdersRemaining>(log_player) {
        display_bar(orders, commands, asset_server, panel);
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
    operating_player: Res<OperatingPlayer>,
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
            .action_cache(&logical_world.0, operating_player.0)?,
    }))?;

    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn render_card_execution_panel(
    parent_panel: Entity,
    visual_cards: &VisCardDirectory,
    vis_markets: &VisMarketDirectory,
    vis_pieces: &VisualPieceArchetypeDirectory,
    operating_player: &OperatingPlayer,
    commands: &mut Commands,
    description: Box<[TextSnippet]>,
    card: CardId,
    player_names: &PlayerNames,
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
                    Text::new(card_details.name.clone()),
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
            Text::new(format!("\"{}\"", card_details.tooltip)),
            TextFont::from_font_size(TOOLTIP_FONT_SIZE),
            TextColor(DEFAULT_COLOR_THEME.unimportant_color)
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
    vis_markets: Res<VisMarketDirectory>,
    vis_pieces: Res<VisualPieceArchetypeDirectory>,
    player_names: Res<PlayerNames>,
    asset_server: ResMut<AssetServer>,
) -> Result<(), BevyError> {
    if let Some(FrontendAction::UseCard { index, cache }) = loaded_action.loaded_action() {
        let card = log_world
            .0
            .get::<PlayerCardInventory>(
                *log_world
                    .0
                    .resource::<PlayerDirectory>()
                    .get(display_player.0),
            )
            .expect("all players should have an inventory")
            .get_card(*index)?;
        render_card_execution_panel(
            parent_panel.entity(),
            &vis_cards,
            &vis_markets,
            &vis_pieces,
            &display_player,
            &mut commands,
            cache.forensic_description(),
            card,
            &player_names,
        )
    } else {
        render_inventory(
            &mut commands,
            parent_panel.entity(),
            &vis_cards,
            &log_world,
            &display_player,
            asset_server.into_inner(),
        )
    }
}
