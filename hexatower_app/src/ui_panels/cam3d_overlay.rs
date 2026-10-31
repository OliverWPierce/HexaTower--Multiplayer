use bevy::{color::palettes::tailwind, prelude::*};
use core_game_logic::{
    forensic_action_descriptions::ForensicDescribe, players::ActivePlayer, requests::ActionEffect,
};

use crate::{
    AppState, OperatingPlayer,
    functional_assets::{LogicalWorld, PlayerNames, VisCardDirectory, VisMarketDirectory},
    inputs_interface::{EffectToDisplay, EffectsQueue, NextEffectStartsIn, TryEndTurn},
    ui_panels::{
        MidPanelLower, MidPanelUpper, UNIVERSAL_BACKGROUND, UNIVERSAL_BORDER, add_description,
        spawn_basic_ui_layout,
    },
    vis_pieces::visual_piece_archetypes_storage::VisualPieceArchetypeDirectory,
};

pub struct Cam3dOverlayPlugin;

impl Plugin for Cam3dOverlayPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(AppState::InGame),
            (effects, spawn_end_turn_button).after(spawn_basic_ui_layout),
        );

        app.add_systems(
            Update,
            animate_progress_bar.run_if(resource_exists_and_changed::<NextEffectStartsIn>),
        );

        app.add_systems(
            Update,
            animate_remaining_effects_counter.run_if(resource_exists_and_changed::<EffectsQueue>),
        );

        app.add_systems(
            Update,
            (describe_effects, edit_began_turn_text)
                .distributive_run_if(resource_changed_or_removed::<EffectToDisplay>),
        );
    }
}

const BORDER_WITDH: Val = Val::Px(3.0);

fn effects(mut commands: Commands, parent: Single<Entity, With<MidPanelUpper>>) {
    info!("Spawned the UI which appears over the board!");

    let overall_panel = commands
        .spawn((
            Node {
                width: Val::Percent(30.0),
                height: Val::Vh(11.0),
                flex_direction: FlexDirection::Column,
                ..default()
            },
            ChildOf(parent.entity()),
            Pickable {
                should_block_lower: false,
                is_hoverable: false,
            },
        ))
        .id();

    let container_for_description_block_and_count = commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Vh(8.0),
                ..default()
            },
            ChildOf(overall_panel),
            Pickable {
                should_block_lower: false,
                is_hoverable: false,
            },
        ))
        .id();

    let parent_for_description_image_and_progress_bar = commands
        .spawn((
            Node {
                width: Val::Percent(85.0),
                height: Val::Percent(100.0),
                border: UiRect::all(BORDER_WITDH),
                flex_direction: FlexDirection::ColumnReverse,
                ..default()
            },
            ChildOf(container_for_description_block_and_count),
            BackgroundColor(UNIVERSAL_BACKGROUND),
            BorderColor::all(UNIVERSAL_BORDER),
        ))
        .id();

    // progress bar
    commands.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(7.0),
            ..default()
        },
        ChildOf(parent_for_description_image_and_progress_bar),
        BackgroundColor(tailwind::TEAL_700.into()),
        ProgressBar,
    ));

    // description and image
    commands.spawn((
        Node {
            width: Val::Percent(100.0),
            flex_grow: 2.0,

            ..default()
        },
        ChildOf(parent_for_description_image_and_progress_bar),
        BackgroundColor(UNIVERSAL_BACKGROUND),
        children![
            (
                Node {
                    height: Val::Percent(100.0),
                    justify_content: JustifyContent::Center,
                    align_content: AlignContent::Center,
                    ..default()
                },
                children![(Text::default(), EffectDescription)]
            ),
            (
                Node {
                    height: Val::Percent(100.0),
                    aspect_ratio: Some(1.0),
                    justify_content: JustifyContent::Center,
                    align_content: AlignContent::Center,
                    ..default()
                },
                ImageNode::default(),
                EffectImage,
            )
        ],
    ));

    //counter
    commands.spawn((
        Node {
            width: Val::Percent(15.0),
            height: Val::Percent(50.0),
            justify_content: JustifyContent::Center,
            align_content: AlignContent::Center,
            border: UiRect::all(BORDER_WITDH),
            border_radius: BorderRadius::bottom_right(Val::Percent(100.0)),
            ..default()
        },
        BackgroundColor(UNIVERSAL_BACKGROUND),
        BorderColor::all(UNIVERSAL_BORDER),
        ChildOf(container_for_description_block_and_count),
        children![(Text::new("0"), RemainingEffectsCounter)],
    ));

    // speed clickthrough
    commands.spawn((
        Node {
            width: Val::Percent(60.0),
            height: Val::Vh(3.0),
            border: UiRect::all(BORDER_WITDH),
            ..default()
        },
        BackgroundColor(UNIVERSAL_BACKGROUND),
        BorderColor::all(UNIVERSAL_BORDER),
        ChildOf(overall_panel),
    ));
}

#[derive(Debug, Component)]
struct ProgressBar;

#[derive(Debug, Component)]
struct EffectDescription;

#[derive(Debug, Component)]
struct EffectImage;

#[derive(Debug, Component)]
struct RemainingEffectsCounter;

fn animate_progress_bar(
    mut width: Single<&mut Node, With<ProgressBar>>,
    remaining_time: Res<NextEffectStartsIn>,
) {
    width.width = Val::Percent(
        100.0
            - 100.0 * (remaining_time.0.elapsed_secs() / remaining_time.0.duration().as_secs_f32()),
    );
}

fn animate_remaining_effects_counter(
    mut text: Single<&mut Text, With<RemainingEffectsCounter>>,
    queue: Res<EffectsQueue>,
) {
    text.0 = format!("{}", { queue.len() });
}

fn describe_effects(
    parent: Single<Entity, With<EffectDescription>>,
    effect: Option<Res<EffectToDisplay>>,
    mut commands: Commands,
    visual_cards: Res<VisCardDirectory>,
    vis_markets: Res<VisMarketDirectory>,
    vis_pieces: Res<VisualPieceArchetypeDirectory>,
    operating_player: Res<OperatingPlayer>,
    player_names: Res<PlayerNames>,
) -> Result<(), BevyError> {
    commands.entity(parent.entity()).despawn_children();

    if let Some(effect) = effect {
        add_description(
            parent.entity(),
            &effect.0.forensic_description(),
            &mut commands,
            FontSize::Vh(1.2),
            &visual_cards,
            &vis_markets,
            &vis_pieces,
            &operating_player,
            &player_names,
        )?;
    }

    Ok(())
}

#[derive(Debug, Component)]
struct EndTurnDisplay;

fn spawn_end_turn_button(mut commands: Commands, mid_panel: Single<Entity, With<MidPanelLower>>) {
    commands
        .spawn((
            Node {
                width: Val::Percent(30.0),
                height: Val::Vh(11.0),
                flex_direction: FlexDirection::Column,
                border: UiRect::all(BORDER_WITDH),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                border_radius: BorderRadius::top_right(Val::Percent(100.0)),
                ..default()
            },
            ChildOf(mid_panel.entity()),
            BackgroundColor(UNIVERSAL_BACKGROUND),
            BorderColor::all(UNIVERSAL_BORDER),
            EndTurnDisplay,
            children![(Text::from("..."), EndTurnDisplay)],
        ))
        .observe(|_: On<Pointer<Click>>, mut commands: Commands| -> () {
            commands.trigger(TryEndTurn);
        });
}

fn edit_began_turn_text(
    mut text: Single<&mut Text, With<EndTurnDisplay>>,
    player_names: Res<PlayerNames>,
    operating_player: Res<OperatingPlayer>,
    effect: Option<Res<EffectToDisplay>>,
    logical_world: Res<LogicalWorld>,
) {
    if let Some(effect) = effect {
        if let ActionEffect::EndedTurn(_) = effect.0 {
            let active_player = logical_world.0.resource::<ActivePlayer>().0;

            if active_player == operating_player.0 {
                text.0 = String::from("End Turn!");
            } else {
                text.0 = format!("Siwitching to {}'s turn.", player_names.get(active_player));
            }
        } else if let ActionEffect::BeganTurn(_) = effect.0 {
            let active_player = logical_world.0.resource::<ActivePlayer>().0;

            if active_player == operating_player.0 {
                text.0 = String::from("End Turn!");
            } else {
                text.0 = format!("It is {}'s turn.", player_names.get(active_player));
            }
        }
    }
}
