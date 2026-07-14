use bevy::{color::palettes::tailwind::*, ecs::component::Immutable, prelude::*};
use core_game_logic::forensic_action_descriptions::{LinkedGamplayElement, TextSnippet};

use crate::{
    AppState,
    functional_assets::SetUpBoard,
    inputs_interface::{ActionInputManager, TryEndTurn},
    ui_panels::{
        lower_panel::VisualMarketUIPlugin, mid_panel::VisualOrdersPlugin,
        upper_panel::VisualInventoryPlugin,
    },
};

mod lower_panel;
mod mid_panel;
mod upper_panel;

pub use mid_panel::OrderAtPieceIndex;

pub struct UiPanelsPlugin;

impl Plugin for UiPanelsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(SetUpBoard, spawn_basic_ui_layout);
        app.add_observer(hoverable_elements::hover_colors);
        app.add_observer(hoverable_elements::un_hover_colors);
        app.add_observer(unload_action_button.run_if(in_state(AppState::InGame)));

        app.add_systems(
            Update,
            execution_button::update_panel
                .run_if(in_state(AppState::InGame))
                .run_if(resource_exists_and_changed::<ActionInputManager>),
        );

        app.add_plugins(VisualInventoryPlugin);
        app.add_plugins(VisualOrdersPlugin);
        app.add_plugins(VisualMarketUIPlugin);
    }
}

pub const UNIVERSAL_BACKGROUND: Color = Color::Srgba(ZINC_800);
pub const UNIVERSAL_BORDER: Color = Color::Srgba(ZINC_900);
pub const UNIVERSAL_BORDER_WIDTH: Val = Val::Px(6.0);

pub fn spawn_basic_ui_layout(mut commands: Commands) {
    pub const SIDE_PANELS_WIDTH_AS_A_PERCENT: f32 = 25.0;
    const SUB_PANEL_WIDTHS: Val = Val::Percent(96.0);

    let overall_parent = commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::SpaceBetween,
                ..default()
            },
            Pickable {
                should_block_lower: false,
                is_hoverable: false,
            },
        ))
        .id();

    // the inventory, orders, and market panels
    {
        const OVERALL_PANEL_HEIGHTS: Val = Val::Percent(32.0);
        const PANEL_BACKGROUNDS: Color = Color::Srgba(STONE_700);
        const PANEL_BORDERS: Color = Color::Srgba(STONE_800);

        commands.spawn((
            Node {
                width: Val::Percent(SIDE_PANELS_WIDTH_AS_A_PERCENT),
                height: Val::Percent(100.0),
                border: UiRect::all(UNIVERSAL_BORDER_WIDTH),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::SpaceAround,
                align_content: AlignContent::Center,
                ..default()
            },
            BorderColor::all(UNIVERSAL_BORDER),
            BackgroundColor(UNIVERSAL_BACKGROUND),
            ChildOf(overall_parent),
            children![
                (
                    InventoryPanel,
                    Node {
                        width: SUB_PANEL_WIDTHS,
                        height: OVERALL_PANEL_HEIGHTS,
                        border: UiRect::all(UNIVERSAL_BORDER_WIDTH),
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        align_self: AlignSelf::Center,
                        row_gap: Val::Px(8.0),
                        ..default()
                    },
                    BorderColor::all(PANEL_BORDERS),
                    BackgroundColor(PANEL_BACKGROUNDS)
                ),
                (
                    OrdersPanel,
                    Node {
                        width: SUB_PANEL_WIDTHS,
                        height: OVERALL_PANEL_HEIGHTS,
                        border: UiRect::all(UNIVERSAL_BORDER_WIDTH),
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        align_self: AlignSelf::Center,
                        row_gap: Val::Px(8.0),
                        ..default()
                    },
                    BorderColor::all(PANEL_BORDERS),
                    BackgroundColor(PANEL_BACKGROUNDS)
                ),
                (
                    MarketPanel,
                    Node {
                        width: SUB_PANEL_WIDTHS,
                        height: OVERALL_PANEL_HEIGHTS,
                        border: UiRect::all(UNIVERSAL_BORDER_WIDTH),
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        align_self: AlignSelf::Center,
                        row_gap: Val::Px(8.0),
                        ..default()
                    },
                    BorderColor::all(PANEL_BORDERS),
                    BackgroundColor(PANEL_BACKGROUNDS)
                )
            ],
        ));
    }

    commands
        .spawn((
            Node {
                width: Val::Percent(SIDE_PANELS_WIDTH_AS_A_PERCENT),
                height: Val::Percent(100.0),
                border: UiRect::all(UNIVERSAL_BORDER_WIDTH),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::SpaceAround,
                ..default()
            },
            BorderColor::all(UNIVERSAL_BORDER),
            BackgroundColor(UNIVERSAL_BACKGROUND),
            ChildOf(overall_parent),
            children![(
                Node {
                    width: SUB_PANEL_WIDTHS,
                    height: Val::Px(65.0),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    border: UiRect::all(UNIVERSAL_BORDER_WIDTH),
                    ..Default::default()
                },
                BorderColor::all(SLATE_900),
                BackgroundColor(SLATE_600.into()),
                EndTurnButton,
                children![(
                    Text::new("End Turn"),
                    TextFont {
                        font_size: FontSize::Px(24.0),
                        ..default()
                    },
                )]
            )],
        ))
        .observe(|_: On<Pointer<Click>>, mut commands: Commands| commands.trigger(TryEndTurn));
}

#[derive(Debug, Component)]
pub struct EndTurnButton;

#[derive(Debug, Component)]
struct InventoryPanel;
#[derive(Debug, Component)]
struct OrdersPanel;
#[derive(Debug, Component)]
struct MarketPanel;

pub struct HeaderParameters {
    pub border_color: Color,
    pub background_color: Color,
    pub border_thickness: Val,
    pub height: Val,
    pub width: Val,
    pub text_size: FontSize,
}

pub const LEFT_SIDE_HEADER_PARAMS: HeaderParameters = HeaderParameters {
    border_color: Color::Srgba(ZINC_900),
    background_color: Color::Srgba(ZINC_600),
    border_thickness: Val::Px(3.0),
    height: Val::Vh(4.0),
    width: Val::Percent(96.0),
    text_size: FontSize::Vh(3.0),
};

#[derive(Debug, Component)]
pub struct UnloadActionButton;

fn unload_action_button(
    trigger: On<Pointer<Click>>,
    buttons: Query<(), With<UnloadActionButton>>,
    mut manager: ResMut<ActionInputManager>,
) {
    if buttons.get(trigger.entity).is_ok() {
        manager
            .try_load_action(None)
            .expect("This function cannot error when given an input of None.")
    }
}
#[derive(Debug, Component)]
pub struct TextLinkToGameplayElement(pub LinkedGamplayElement);

mod display_themes {
    use bevy::{
        color::{Color, palettes::tailwind::*},
        text::FontSize,
    };
    use core_game_logic::tiles::TileType;

    pub struct ColorTheme {
        pub money_color: Color,
        pub negative_color: Color,
        pub positive_color: Color,
        pub highlight_color: Color,
        pub unimportant_color: Color,
    }
    pub const DEFAULT_COLOR_THEME: ColorTheme = ColorTheme {
        money_color: Color::Srgba(AMBER_300),
        negative_color: Color::Srgba(RED_300),
        positive_color: Color::Srgba(GREEN_300),
        highlight_color: Color::Srgba(CYAN_300),
        unimportant_color: Color::Srgba(GRAY_300),
    };

    pub const DESCRIPTION_FONT_SIZE: FontSize = FontSize::Vh(2.25);
    pub const TOOLTIP_FONT_SIZE: FontSize = FontSize::Vh(2.0);

    pub fn color_for_tile_type_under_default_theme(tile_type: &TileType) -> Color {
        match tile_type {
            TileType::Basic => SLATE_500.into(),
            TileType::Ex1 => PURPLE_400.into(),
        }
    }

    pub fn name_for_tile_type_under_default_theme(tile_type: &TileType) -> String {
        match tile_type {
            TileType::Basic => "Basic".into(),
            TileType::Ex1 => "Corrupted".into(),
        }
    }
}

pub mod hoverable_elements {
    use bevy::prelude::*;

    /// This module provides an easy way to make UI elements hoverable. Simply use the "create_hoverable_ui_bundle" function when spawning an entity, and the rest is handled.

    #[derive(Debug, Component, Default, Clone)]
    #[require(BackgroundColor, BorderColor, ColorsForDormantUI)]
    pub struct ColorsForHoveredUI {
        border: BorderColor,
        background: BackgroundColor,
    }

    #[derive(Debug, Component, Default, Clone)]
    #[require(BackgroundColor, BorderColor)]
    pub struct ColorsForDormantUI {
        border: BorderColor,
        background: BackgroundColor,
    }

    pub fn create_hoverable_ui_bundle(
        dormant_border: BorderColor,
        dormant_background: BackgroundColor,
        hovered_border: BorderColor,
        hovered_background: BackgroundColor,
    ) -> (
        ColorsForDormantUI,
        ColorsForHoveredUI,
        BackgroundColor,
        BorderColor,
    ) {
        (
            ColorsForDormantUI {
                border: dormant_border,
                background: dormant_background,
            },
            ColorsForHoveredUI {
                border: hovered_border,
                background: hovered_background,
            },
            dormant_background,
            dormant_border,
        )
    }

    pub(super) fn hover_colors(
        trigger: On<Pointer<Over>>,
        mut hoverable_elements: Query<(
            &mut BorderColor,
            &mut BackgroundColor,
            &ColorsForHoveredUI,
        )>,
    ) {
        let Ok((mut border, mut background, presets)) = hoverable_elements.get_mut(trigger.entity)
        else {
            return;
        };

        *border = presets.border;
        *background = presets.background;
    }

    pub(super) fn un_hover_colors(
        trigger: On<Pointer<Out>>,
        mut hoverable_elements: Query<(
            &mut BorderColor,
            &mut BackgroundColor,
            &ColorsForDormantUI,
        )>,
    ) {
        let Ok((mut border, mut background, presets)) = hoverable_elements.get_mut(trigger.entity)
        else {
            return;
        };

        *border = presets.border;
        *background = presets.background;
    }
}

mod execution_button {
    use bevy::{color::palettes::tailwind::*, prelude::*};
    use core_game_logic::{
        markets::MarketDirectory,
        pieces::{OccupiedByPiece, OrdersReceivable, PieceOwnedByPlayer},
        players::{
            ActivePlayer, Coins, PlayerCardInventory, PlayerDirectory, PlayerId,
            PlayerOrdersRemaining,
        },
        tiles::{MarketTile, TileDirectory},
    };

    use crate::{
        OperatingPlayer,
        functional_assets::LogicalWorld,
        inputs_interface::{ActionInputManager, FrontendAction, TryExecuteLoadedAction},
        ui_panels::LEFT_SIDE_HEADER_PARAMS,
    };

    #[derive(Debug, Component)]
    pub struct ExecutionButtonPanel;

    pub fn update_panel(
        mut commands: Commands,
        action_manager: Res<ActionInputManager>,
        panel: Single<Entity, With<ExecutionButtonPanel>>,
        operating_player: Res<OperatingPlayer>,
        logical_world: Res<LogicalWorld>,
    ) -> Result<(), BevyError> {
        let Some(action) = action_manager.loaded_action() else {
            warn!("Tried to update the execution button, but there was no loaded action.");
            return Ok(());
        };

        let blockers_for_action_execution =
            action_blockers(&action_manager, &logical_world, &operating_player)?;

        struct ColorScheme {
            unready_background: BackgroundColor,
            unready_border: BorderColor,
            ready_background: BackgroundColor,
            ready_border: BorderColor,
        }

        let colors = match action {
            FrontendAction::UseCard { .. } => ColorScheme {
                unready_background: BackgroundColor(BLUE_900.into()),
                unready_border: BorderColor::all(BLUE_950),
                ready_background: BackgroundColor(BLUE_500.into()),
                ready_border: BorderColor::all(BLUE_600),
            },
            FrontendAction::UseOrder { .. } => ColorScheme {
                unready_background: BackgroundColor(RED_900.into()),
                unready_border: BorderColor::all(RED_950),
                ready_background: BackgroundColor(RED_500.into()),
                ready_border: BorderColor::all(RED_600),
            },
            FrontendAction::PurchaseCard { .. } => ColorScheme {
                unready_background: BackgroundColor(EMERALD_900.into()),
                unready_border: BorderColor::all(EMERALD_950),
                ready_background: BackgroundColor(EMERALD_500.into()),
                ready_border: BorderColor::all(EMERALD_600),
            },
        };

        commands.entity(panel.entity()).despawn_children();

        let button = commands
            .spawn((
                Node {
                    min_height: Val::Px(38.0),
                    height: Val::Percent(100.0),
                    width: Val::Percent(100.0),
                    border: UiRect::all(Val::Px(3.0)),
                    border_radius: BorderRadius::all(Val::Px(5.0)),
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                ChildOf(panel.entity()),
                {
                    if blockers_for_action_execution.is_none() {
                        (colors.ready_background, colors.ready_border)
                    } else {
                        (colors.unready_background, colors.unready_border)
                    }
                },
            ))
            .observe(try_start_execution_request)
            .id();

        commands.spawn((
            Text::new(match action {
                FrontendAction::UseCard { .. } => String::from("USE ITEM!"),
                FrontendAction::UseOrder { .. } => String::from("ORDER!"),
                FrontendAction::PurchaseCard { .. } => String::from("Purchase!"),
            }),
            TextFont {
                font_size: LEFT_SIDE_HEADER_PARAMS.text_size,
                ..default()
            },
            ChildOf(button),
            TextColor({
                if blockers_for_action_execution.is_none() {
                    Color::Hsva(Hsva {
                        hue: 0.0,
                        saturation: 0.0,
                        value: 1.0,
                        alpha: 1.0,
                    })
                } else {
                    Color::Hsva(Hsva {
                        hue: 0.0,
                        saturation: 0.0,
                        value: 0.7,
                        alpha: 1.0,
                    })
                }
            }),
        ));

        if let Some(blocker) = blockers_for_action_execution {
            commands.spawn((
                Text::new(blocker.0),
                TextFont {
                    font_size: FontSize::Vh(1.5),
                    ..default()
                },
                ChildOf(button),
            ));
        }

        if let Some(cache) = action_manager.process_cache() {
            match &cache {
                core_game_logic::requests::ActionProcessCache::TileAction(cache) => {
                    let selection_progress_bar = commands
                        .spawn((
                            Node {
                                width: Val::Percent(100.0),
                                min_height: Val::Percent(20.0),
                                justify_content: JustifyContent::SpaceEvenly,
                                padding: UiRect::all(Val::Px(2.0)),
                                ..default()
                            },
                            BackgroundColor(ZINC_900.into()),
                            ChildOf(panel.entity()),
                        ))
                        .id();

                    for box_number in 0..cache.selection_bounds().end {
                        commands.spawn((
                            Node {
                                height: Val::Percent(90.0),
                                width: Val::Percent(100.0 / (cache.selection_bounds().end as f32)),
                                border: UiRect::all(Val::Px(3.0)),
                                ..default()
                            },
                            {
                                if box_number >= cache.amount_currently_selected() {
                                    (
                                        BackgroundColor(STONE_900.into()),
                                        BorderColor::all(STONE_950),
                                    )
                                } else if box_number >= cache.selection_bounds().start {
                                    (
                                        BackgroundColor(EMERALD_500.into()),
                                        BorderColor::all(EMERALD_700),
                                    )
                                } else {
                                    (BackgroundColor(SKY_500.into()), BorderColor::all(SKY_700))
                                }
                            },
                            ChildOf(selection_progress_bar),
                        ));
                    }
                }

                core_game_logic::requests::ActionProcessCache::Ex1 => (),
            }
        }
        Ok(())
    }

    struct Blocker(String);

    fn action_blockers(
        action_manager: &ActionInputManager,
        logical_world: &LogicalWorld,
        operating_player: &OperatingPlayer,
    ) -> Result<Option<Blocker>, BevyError> {
        let action = action_manager.loaded_action().ok_or(
            "Tried to calculate action execution blockers, but there was no loaded action",
        )?;

        if let Some(cache) = {
            match action {
                FrontendAction::UseCard { cache, .. } => Some(cache),
                FrontendAction::UseOrder { cache, .. } => Some(cache),
                FrontendAction::PurchaseCard { .. } => None,
            }
        } {
            match cache {
                core_game_logic::requests::ActionProcessCache::TileAction(cache) => {
                    if cache.selection_bounds().start > cache.amount_currently_selected() {
                        return Ok(Some(Blocker(format!(
                            "select at least {} more tiles",
                            cache.selection_bounds().start - cache.amount_currently_selected() // note this will not result in a negative number, because the cache will not allow for selections beyond the maximum allowed number of selections.
                        ))));
                    }
                }
                core_game_logic::requests::ActionProcessCache::Ex1 => (),
            }
        }

        match action {
            FrontendAction::UseCard { .. } => (),
            FrontendAction::UseOrder { .. } => {
                let logical_piece = logical_world
                    .0
                    .get::<OccupiedByPiece>(
                        logical_world
                            .0
                            .resource::<TileDirectory>()
                            .get_entity(action_manager.active_tile().ok_or("Tried to generate execution blockers for an order, but there was no active tile.")?)?,
                    )
                    .ok_or("An order was loaded but the active tile was vacant")?
                    .piece();

                if logical_world.0.get::<OrdersReceivable>(logical_piece).expect("all pieces should have a component detailing how many orders they have and should have each round.").currently == 0 {
                                    return Ok(Some(Blocker("Piece is out of orders this round".into())))
                                }

                let owner = logical_world.0.get::<PieceOwnedByPlayer>(logical_piece);

                if owner.is_none()
                    || owner.unwrap().0
                        != *logical_world
                            .0
                            .resource::<PlayerDirectory>()
                            .get(operating_player.0)
                {
                    return Ok(Some(Blocker("You do not own this piece.".into())));
                }
                if logical_world
                    .0
                    .get::<PlayerOrdersRemaining>(
                        *logical_world
                            .0
                            .resource::<PlayerDirectory>()
                            .get(operating_player.0),
                    )
                    .ok_or(
                        "A player lacked information about how many remaining orders they have.",
                    )?
                    .remaining
                    == 0
                {
                    return Ok(Some(Blocker("You are out of orders this round".into())));
                }
            }
            FrontendAction::PurchaseCard { slot } => {
                let active_tile = action_manager.active_tile().ok_or("Tried to calculate execution blockers for a purchase action, but there was no active tile.")?;

                let logical_tile = logical_world
                    .0
                    .resource::<TileDirectory>()
                    .get_entity(active_tile)?;

                let MarketTile(browsed_market) = logical_world
                    .0
                    .get::<MarketTile>(logical_tile)
                    .ok_or("This tile is not a market tile")?;

                let a_piece_the_player_owns_occupies_this_tile: bool = if let Some(piece) =
                    logical_world.0.get::<OccupiedByPiece>(logical_tile)
                    && let Some(owner) = logical_world.0.get::<PieceOwnedByPlayer>(piece.piece())
                    && let Some(owner_id) = logical_world.0.get::<PlayerId>(owner.0)
                    && *owner_id == operating_player.0
                {
                    let inventory = logical_world
                        .0
                        .get::<PlayerCardInventory>(owner.0)
                        .expect("all players should have a card inventory");

                    if inventory.all_cards().len() >= inventory.max_card_capacity() as usize {
                        return Ok(Some(Blocker("Inventory is full".into())));
                    }

                    true
                } else {
                    false
                };

                if !a_piece_the_player_owns_occupies_this_tile {
                    return Ok(Some(Blocker("You do not occupy this market.".into())));
                }

                let coins_of_operating_player = logical_world.0.get::<Coins>(*logical_world
                    .0
                    .resource::<PlayerDirectory>()
                    .get(operating_player.0)).ok_or("A player lacked a component detailing the amount of currency they possesed.")?.0;

                let price_of_card = logical_world
                    .0
                    .resource::<MarketDirectory>()
                    .get_market(*browsed_market)?
                    .get_card_and_price(*slot)
                    .1
                    .0;

                if coins_of_operating_player < price_of_card {
                    return Ok(Some(Blocker("Insufficient funds".into())));
                }
            }
        }

        if operating_player.0 != logical_world.0.resource::<ActivePlayer>().0 {
            return Ok(Some(Blocker("It is not your turn".into())));
        }

        Ok(None)
    }

    fn try_start_execution_request(
        mut click: On<Pointer<Click>>,
        operating_player: Res<OperatingPlayer>,
        logical_world: Res<LogicalWorld>,
        action_manager: Res<ActionInputManager>,
        mut commands: Commands,
    ) -> Result<(), BevyError> {
        click.propagate(false);

        if let Some(Blocker(message)) =
            action_blockers(&action_manager, &logical_world, &operating_player)?
        {
            warn!(message);
        } else {
            commands.trigger(TryExecuteLoadedAction);
        }

        Ok(())
    }
}
