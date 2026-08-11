use bevy::{color::palettes::tailwind, prelude::*};

use crate::{
    functional_assets::SetUpBoard,
    main_menu::ClickThroughSelector,
    ui_panels::{MidPanelUpper, UNIVERSAL_BACKGROUND, spawn_basic_ui_layout},
};

pub struct UpperBarPlugin;

impl Plugin for UpperBarPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(SetUpBoard, effects.after(spawn_basic_ui_layout));

        // app.add_systems(
        //     Update,
        //     manage_orders_panel
        //         .run_if(in_state(AppState::InGame))
        //         .run_if(resource_exists_and_changed::<ActionInputManager>),
        // );

        // app.add_observer(load_order.run_if(in_state(AppState::InGame)));
    }
}

fn effects(mut commands: Commands, parent: Single<Entity, With<MidPanelUpper>>) {
    commands.spawn((
        Node {
            width: Val::Percent(30.0),
            height: Val::Vh(11.0),
            flex_direction: FlexDirection::Column,
            ..default()
        },
        BackgroundColor(Color::BLACK),
        ChildOf(parent.entity()),
        children![
            (
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Vh(8.0),
                    ..default()
                },
                BackgroundColor(Color::WHITE),
                children![
                    (
                        // the effect name and image
                        Node {
                            width: Val::Percent(85.0),
                            height: Val::Percent(100.0),
                            ..default()
                        },
                        BackgroundColor(Color::from(Srgba::RED)),
                    ),
                    (
                        // the number of remaining effects to display
                        Node {
                            width: Val::Percent(15.0),
                            height: Val::Percent(50.0),
                            ..default()
                        },
                        BackgroundColor(Color::from(Srgba::GREEN)),
                    )
                ]
            ),
            (
                // display speed.
                Node {
                    width: Val::Percent(60.0),
                    height: Val::Vh(3.0),
                    ..default()
                },
                BackgroundColor(Color::from(Srgba::BLUE)),
            )
        ],
    ));

    let overall_panel = commands
        .spawn((
            Node {
                width: Val::Percent(30.0),
                height: Val::Vh(11.0),
                flex_direction: FlexDirection::Column,
                ..default()
            },
            ChildOf(parent.entity()),
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
        ))
        .id();

    let parent_for_description_image_and_progress_bar = commands
        .spawn((
            Node {
                width: Val::Percent(85.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::ColumnReverse,
                ..default()
            },
            ChildOf(container_for_description_block_and_count),
            BackgroundColor(UNIVERSAL_BACKGROUND),
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
                    ..default()
                },
                Text::default(),
                EffectDescription,
            ),
            (
                Node {
                    height: Val::Percent(100.0),
                    aspect_ratio: Some(1.0),
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
            ..default()
        },
        BackgroundColor(UNIVERSAL_BACKGROUND),
        ChildOf(container_for_description_block_and_count),
        Text::new("0"),
        RemainingEffects,
    ));

    // speed clickthrough
    commands.spawn((
        Node {
            width: Val::Percent(60.0),
            height: Val::Vh(3.0),
            ..default()
        },
        BackgroundColor(Color::from(Srgba::BLUE)),
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
struct RemainingEffects;

#[derive(Debug, Resource)]
struct EffectPlaythroughSpeedMultiplyer(f32);

impl Default for EffectPlaythroughSpeedMultiplyer {
    fn default() -> Self {
        Self(1.0)
    }
}

impl ClickThroughSelector for EffectPlaythroughSpeedMultiplyer {
    fn next_option(&mut self) {
        match self.0 {
            0.1 => self.0 = 0.25,
            0.25 => self.0 = 0.5,
            0.5 => self.0 = 0.75,
            0.75 => self.0 = 1.0,
            1.0 => self.0 = 1.5,
            1.5 => self.0 = 2.0,
            2.0 => self.0 = 3.0,
            3.0 => self.0 = 5.0,
            5.0 => self.0 = 7.5,
            7.5 => self.0 = 10.0,
            10.0 => self.0 = 0.1,
            _ => self.0 = 1.0,
        }
    }

    fn previous_option(&mut self) {
        match self.0 {
            0.1 => self.0 = 10.0,
            0.25 => self.0 = 0.1,
            0.5 => self.0 = 0.25,
            0.75 => self.0 = 0.5,
            1.0 => self.0 = 0.75,
            1.5 => self.0 = 1.0,
            2.0 => self.0 = 1.5,
            3.0 => self.0 = 2.0,
            5.0 => self.0 = 3.0,
            7.5 => self.0 = 5.0,
            10.0 => self.0 = 7.5,
            _ => self.0 = 1.0,
        }
    }

    fn display_text(&self) -> String {
        format!("{:?}x", self.0)
    }
}
