use bevy::{color::palettes::tailwind::*, prelude::*};

use crate::{
    functional_assets::SetUpBoard, inputs_interface::LoadedAction,
    ui_panels::upper_panel::VisualInventoryPlugin,
};

mod upper_panel;

pub struct UiPanelsPlugin;

impl Plugin for UiPanelsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(SetUpBoard, spawn_basic_ui_layout);
        app.add_observer(hoverable_elements::hover_colors);
        app.add_observer(hoverable_elements::un_hover_colors);

        app.add_systems(
            Update,
            execution_button::update_panel.run_if(resource_exists_and_changed::<LoadedAction>),
        );

        app.add_plugins(VisualInventoryPlugin);
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
                        justify_content: JustifyContent::SpaceAround,
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
                        align_self: AlignSelf::Center,
                        border: UiRect::all(UNIVERSAL_BORDER_WIDTH),
                        ..default()
                    },
                    BorderColor::all(PANEL_BORDERS),
                    BackgroundColor(PANEL_BACKGROUNDS)
                ),
                (
                    MarketPanel,
                    Node {
                        width: OVERALL_PANEL_WIDTHS,
                        align_self: AlignSelf::Center,
                        height: OVERALL_PANEL_HEIGHTS,
                        border: UiRect::all(UNIVERSAL_BORDER_WIDTH),
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

    use crate::inputs_interface::LoadedAction;

    #[derive(Debug, Component)]
    pub struct ExecutionButtonPanel;

    pub fn update_panel(
        mut commands: Commands,
        loaded_action: Res<LoadedAction>,
        panel: Single<Entity, With<ExecutionButtonPanel>>,
    ) {
        match &loaded_action.cache {
            core_game_logic::requests::ActionProcessCache::TileAction(cache) => {
                commands.entity(panel.entity()).despawn_children();

                let ready_for_execution =
                    cache.selection_bounds().start <= cache.currently_selected();

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
                            if ready_for_execution {
                                (
                                    BackgroundColor(VIOLET_600.into()),
                                    BorderColor::all(VIOLET_800),
                                )
                            } else {
                                (
                                    BackgroundColor(VIOLET_900.into()),
                                    BorderColor::all(VIOLET_950),
                                )
                            }
                        },
                    ))
                    .id();

                commands.spawn((
                    Text::new("Use Item"),
                    TextFont {
                        font_size: 24.0,
                        ..default()
                    },
                    ChildOf(button),
                    TextColor({
                        if ready_for_execution {
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

                commands.spawn((
                    Text::new(if !ready_for_execution {
                        format!(
                            "select at least {} more tiles",
                            cache.selection_bounds().start - cache.currently_selected() // note this will not result in a negative number, because the cache will not allow for selections beyond the maximum allowed number of selections.
                        )
                    } else {
                        format!(
                            "select up to {} more tiles",
                            cache.selection_bounds().end - cache.currently_selected() // note this will not result in a negative number, because the cache will not allow for selections beyond the maximum allowed number of selections.
                        )
                    }),
                    TextFont {
                        font_size: 12.0,
                        ..default()
                    },
                    ChildOf(button),
                ));

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
                            if box_number >= cache.currently_selected() {
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

            core_game_logic::requests::ActionProcessCache::Ex1 => todo!(),
        }
    }
}
