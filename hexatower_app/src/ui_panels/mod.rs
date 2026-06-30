use bevy::{color::palettes::tailwind::*, prelude::*};

use crate::{
    functional_assets::SetUpBoard,
    inputs_interface::LoadedAction,
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
        app.add_observer(unload_action_button);

        app.add_systems(
            Update,
            execution_button::update_panel.run_if(resource_exists_and_changed::<LoadedAction>),
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
        const OVERALL_PANEL_WIDTHS: Val = Val::Percent(96.0);
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
                        width: OVERALL_PANEL_WIDTHS,
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
                        width: OVERALL_PANEL_WIDTHS,
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
                        width: OVERALL_PANEL_WIDTHS,
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

    commands.spawn((
        Node {
            width: Val::Percent(SIDE_PANELS_WIDTH_AS_A_PERCENT),
            height: Val::Percent(100.0),
            border: UiRect::all(UNIVERSAL_BORDER_WIDTH),
            ..default()
        },
        BorderColor::all(UNIVERSAL_BORDER),
        BackgroundColor(UNIVERSAL_BACKGROUND),
        ChildOf(overall_parent),
    ));
}

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
    pub text_size_px: f32,
}

pub const LEFT_SIDE_HEADER_PARAMS: HeaderParameters = HeaderParameters {
    border_color: Color::Srgba(ZINC_900),
    background_color: Color::Srgba(ZINC_600),
    border_thickness: Val::Px(3.0),
    height: Val::Px(30.0),
    width: Val::Percent(96.0),
    text_size_px: 24.0,
};

#[derive(Debug, Component)]
pub struct UnloadActionButton;

fn unload_action_button(
    trigger: On<Pointer<Click>>,
    buttons: Query<(), With<UnloadActionButton>>,
    mut commands: Commands,
) {
    if buttons.get(trigger.entity).is_ok() {
        commands.remove_resource::<LoadedAction>();
    }
}

mod hoverable_elements {
    use bevy::prelude::*;

    /// This module provides an easy way to make UI elements hoverable. Simply use the "create_hoverable_ui_bundle" function when spawning an entity, and the rest is handled.

    #[derive(Debug, Component, Default)]
    #[require(BackgroundColor, BorderColor, ColorsForDormantUI)]
    pub struct ColorsForHoveredUI {
        border: BorderColor,
        background: BackgroundColor,
    }

    #[derive(Debug, Component, Default)]
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
        players::{ActivePlayer, Coins, PlayerDirectory, PlayerOrdersRemaining},
        tiles::{MarketTile, TileDirectory},
    };

    use crate::{
        OperatingPlayer,
        functional_assets::LogicalWorld,
        inputs_interface::{LoadedAction, Source, TryExecuteLoadedAction},
        vis_tiles::ActiveTile,
    };

    #[derive(Debug, Component)]
    pub struct ExecutionButtonPanel;

    pub fn update_panel(
        mut commands: Commands,
        loaded_action: Res<LoadedAction>,
        panel: Single<Entity, With<ExecutionButtonPanel>>,
        operating_player: Res<OperatingPlayer>,
        active_tile: Option<Res<ActiveTile>>,
        logical_world: Res<LogicalWorld>,
    ) -> Result<(), BevyError> {
        let blockers_for_action_execution = action_blockers(
            &loaded_action,
            &logical_world,
            &operating_player,
            active_tile.as_ref(),
        )?;

        struct ColorScheme {
            unready_background: BackgroundColor,
            unready_border: BorderColor,
            ready_background: BackgroundColor,
            ready_border: BorderColor,
        }

        let colors = match loaded_action.source {
            Source::Card(..) => ColorScheme {
                unready_background: BackgroundColor(BLUE_900.into()),
                unready_border: BorderColor::all(BLUE_950),
                ready_background: BackgroundColor(BLUE_500.into()),
                ready_border: BorderColor::all(BLUE_600),
            },
            Source::Order(..) => ColorScheme {
                unready_background: BackgroundColor(RED_900.into()),
                unready_border: BorderColor::all(RED_950),
                ready_background: BackgroundColor(RED_500.into()),
                ready_border: BorderColor::all(RED_600),
            },
            Source::Market(slot_in_market) => ColorScheme {
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
            Text::new(match loaded_action.source {
                Source::Card(..) => String::from("USE ITEM!"),
                Source::Order(..) => String::from("ORDER!"),
                Source::Market(..) => String::from("Purchase!"),
            }),
            TextFont {
                font_size: 24.0,
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
                    font_size: 12.0,
                    ..default()
                },
                ChildOf(button),
            ));
        }

        match &loaded_action.cache {
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
        Ok(())
    }

    struct Blocker(String);

    fn action_blockers(
        loaded_action: &LoadedAction,
        logical_world: &LogicalWorld,
        operating_player: &OperatingPlayer,
        active_tile: Option<&Res<ActiveTile>>,
    ) -> Result<Option<Blocker>, BevyError> {
        match &loaded_action.cache {
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

        match loaded_action.source {
            Source::Card(..) => (),
            Source::Order(..) => {
                let Some(tile) = active_tile else {
                    return Err(BevyError::from(
                        "An order was loaded with no accompanying active tile.",
                    ));
                };

                let logical_piece = logical_world
                    .0
                    .get::<OccupiedByPiece>(
                        logical_world
                            .0
                            .resource::<TileDirectory>()
                            .get_entity(tile.0)?,
                    )
                    .ok_or("An order was loaded but the active tile was vacant")?
                    .piece();

                if logical_world.0.get::<OrdersReceivable>(logical_piece).expect("all pieces should have a component detailing how many orders they have and should have each round.").currently == 0 {
                                    return Ok(Some(Blocker("Piece is out of orders this round".into())))
                                }

                let owner = logical_world.0.get::<PieceOwnedByPlayer>(logical_piece);

                if owner.is_none()
                    || owner.unwrap().0
                        != logical_world
                            .0
                            .resource::<PlayerDirectory>()
                            .get_player(operating_player.0)?
                {
                    return Ok(Some(Blocker("You do not own this piece.".into())));
                }
                if logical_world
                    .0
                    .get::<PlayerOrdersRemaining>(
                        logical_world
                            .0
                            .resource::<PlayerDirectory>()
                            .get_player(operating_player.0)?,
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
            Source::Market(slot_in_market) => {
                let coins_of_operating_player = logical_world.0.get::<Coins>(logical_world.0.resource::<PlayerDirectory>().get_player(operating_player.0)?).ok_or("A player lacked a component detailing the amount of currency they possesed.")?.0;
                let browsed_market = logical_world.0.get::<MarketTile>(logical_world.0.resource::<TileDirectory>().get_entity(active_tile.ok_or("There was a loaded action with a market source, but no active tile.")?.0)?).ok_or("Loaded action had a market as its source, but the active tile was not a market tile")?.0;

                if coins_of_operating_player
                    < logical_world
                        .0
                        .resource::<MarketDirectory>()
                        .get_market(browsed_market)?
                        .get_card_and_price(slot_in_market)
                        .1
                        .0
                {
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
        active_tile: Option<Res<ActiveTile>>,
        action: Res<LoadedAction>,
        mut commands: Commands,
    ) -> Result<(), BevyError> {
        click.propagate(false);

        if let Some(Blocker(message)) = action_blockers(
            &action,
            &logical_world,
            &operating_player,
            active_tile.as_ref(),
        )? {
            warn!(message);
        } else {
            commands.trigger(TryExecuteLoadedAction);
        }

        Ok(())
    }
}
