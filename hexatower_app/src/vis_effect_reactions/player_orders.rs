use std::f32::consts::TAU;

use bevy::prelude::*;
use core_game_logic::{
    pieces::{IsWinCondition, OccupiesTile, OwnsPieces},
    players::PlayerDirectory,
    requests::ActionEffect,
    tile_mapping::{HexVector2d, TileId},
};
use rand::RngExt;

use crate::{
    functional_assets::LogicalWorld,
    inputs_interface::EffectToDisplay,
    vis_effect_reactions::{
        AnimatedProperty, AnimatedPropertyInterval, AnimatedScale, AnimatedTranslation,
        EFFECT_HOVER_HEIGHT,
    },
};

pub fn player_order_change(
    effect: Res<EffectToDisplay>,
    mut commands: Commands,
    asset_server: ResMut<AssetServer>,
    logical_world: Res<LogicalWorld>,
) {
    const ACCENT_PARTICLE_NUMBER: u8 = 20;

    match effect.0 {
        ActionEffect::IncreasedRemainingOrdersOfPlayer { receipient, source } => {
            let Some(owned_pieces) = logical_world.0.get::<OwnsPieces>(
                *logical_world
                    .0
                    .resource::<PlayerDirectory>()
                    .get(receipient),
            ) else {
                return;
            };
            let Some(&tile_of_player_tower) = owned_pieces.list().iter().find_map(|&piece| {
                if logical_world.0.get::<IsWinCondition>(piece).is_some()
                    && let Some(OccupiesTile(tile)) = logical_world.0.get::<OccupiesTile>(piece)
                {
                    logical_world.0.get::<TileId>(*tile)
                } else {
                    None
                }
            }) else {
                return;
            };

            const PHASE_1_DUR: f32 = 0.05;
            const PHASE_2_DUR: f32 = 0.5;
            const PHASE_3_DUR: f32 = 1.4;
            const PHASE_4_DUR: f32 = 0.5;
            const PHASE_5_DUR: f32 = 0.05;

            const DUR_VARIANCE: f32 = 0.05;

            if let Some(source) = source {
                let source_position = HexVector2d::from(source).into();
                let tower_pos: Vec3 = HexVector2d::from(tile_of_player_tower).into();

                // first we spawn the button.

                // then we spawn helper particles:

                let mut rng = rand::rng();

                commands.spawn((
                    (
                        Transform::from_translation(source_position).with_scale(Vec3::ZERO),
                        AnimatedTranslation(AnimatedProperty::new_seamless(
                            source_position,
                            [
                                AnimatedPropertyInterval {
                                    next_value: source_position,
                                    duration: PHASE_1_DUR,
                                    mode: EaseFunction::SmoothStep,
                                },
                                AnimatedPropertyInterval {
                                    next_value: source_position,
                                    duration: PHASE_2_DUR,
                                    mode: EaseFunction::SmoothStep,
                                },
                                AnimatedPropertyInterval {
                                    next_value: tower_pos,
                                    duration: PHASE_3_DUR,
                                    mode: EaseFunction::SmoothStep,
                                },
                                AnimatedPropertyInterval {
                                    next_value: tower_pos,
                                    duration: PHASE_4_DUR,
                                    mode: EaseFunction::SmoothStep,
                                },
                                AnimatedPropertyInterval {
                                    next_value: tower_pos.with_y(0.0),
                                    duration: PHASE_5_DUR,
                                    mode: EaseFunction::QuadraticIn,
                                },
                            ]
                            .into(),
                            0.0,
                        )),
                        AnimatedScale(
                            AnimatedProperty::new_seamless(
                                Vec3::ZERO,
                                [
                                    AnimatedPropertyInterval {
                                        next_value: Vec3::ONE * 5.0,
                                        duration: PHASE_1_DUR + rng.random_range(0.0..DUR_VARIANCE),
                                        mode: EaseFunction::SmoothStep,
                                    },
                                    AnimatedPropertyInterval {
                                        next_value: Vec3::ONE * 5.0,
                                        duration: PHASE_2_DUR
                                            + PHASE_3_DUR
                                            + PHASE_5_DUR
                                            + rng.random_range(0.0..DUR_VARIANCE) * 3.0,
                                        mode: EaseFunction::SmoothStep,
                                    },
                                    AnimatedPropertyInterval {
                                        next_value: Vec3::ZERO,
                                        duration: PHASE_5_DUR + rng.random_range(0.0..DUR_VARIANCE),
                                        mode: EaseFunction::QuadraticIn,
                                    },
                                ]
                                .into(),
                                0.0,
                            ),
                            true,
                        ),
                    ),
                    WorldAssetRoot(asset_server.load::<WorldAsset>(
                        GltfAssetLabel::Scene(0).from_asset("particles_and_effects/green_plus.glb"),
                    )),
                ));

                for _ in 0..ACCENT_PARTICLE_NUMBER {
                    let offset =
                        Vec3::X.rotate_y(rng.random_range(0.0..TAU)) * rng.random_range(0.8..1.2);
                    let scale = Vec3::splat(rng.random_range(0.8..1.2));

                    commands.spawn((
                        (
                            Transform::from_translation(source_position).with_scale(Vec3::ZERO),
                            AnimatedTranslation(AnimatedProperty::new_seamless(
                                source_position,
                                [
                                    AnimatedPropertyInterval {
                                        next_value: source_position + offset,
                                        duration: PHASE_1_DUR + rng.random_range(0.0..DUR_VARIANCE),
                                        mode: EaseFunction::SmoothStep,
                                    },
                                    AnimatedPropertyInterval {
                                        next_value: source_position + offset,
                                        duration: PHASE_2_DUR + rng.random_range(0.0..DUR_VARIANCE),
                                        mode: EaseFunction::SmoothStep,
                                    },
                                    AnimatedPropertyInterval {
                                        next_value: tower_pos + offset * 0.5,
                                        duration: PHASE_3_DUR + rng.random_range(0.0..DUR_VARIANCE),
                                        mode: EaseFunction::SmoothStep,
                                    },
                                    AnimatedPropertyInterval {
                                        next_value: tower_pos + offset * 0.5,
                                        duration: PHASE_4_DUR + rng.random_range(0.0..DUR_VARIANCE),
                                        mode: EaseFunction::SmoothStep,
                                    },
                                    AnimatedPropertyInterval {
                                        next_value: tower_pos + offset,
                                        duration: PHASE_5_DUR + rng.random_range(0.0..DUR_VARIANCE),
                                        mode: EaseFunction::QuadraticIn,
                                    },
                                ]
                                .into(),
                                0.0,
                            )),
                            AnimatedScale(
                                AnimatedProperty::new_seamless(
                                    Vec3::ZERO,
                                    [
                                        AnimatedPropertyInterval {
                                            next_value: scale,
                                            duration: PHASE_1_DUR
                                                + rng.random_range(0.0..DUR_VARIANCE),
                                            mode: EaseFunction::SmoothStep,
                                        },
                                        AnimatedPropertyInterval {
                                            next_value: scale,
                                            duration: PHASE_2_DUR
                                                + PHASE_3_DUR
                                                + PHASE_5_DUR
                                                + rng.random_range(0.0..DUR_VARIANCE) * 3.0,
                                            mode: EaseFunction::SmoothStep,
                                        },
                                        AnimatedPropertyInterval {
                                            next_value: Vec3::ZERO,
                                            duration: PHASE_5_DUR
                                                + rng.random_range(0.0..DUR_VARIANCE),
                                            mode: EaseFunction::QuadraticIn,
                                        },
                                    ]
                                    .into(),
                                    0.0,
                                ),
                                true,
                            ),
                        ),
                        WorldAssetRoot(
                            asset_server.load::<WorldAsset>(
                                GltfAssetLabel::Scene(0)
                                    .from_asset("particles_and_effects/green_plus.glb"),
                            ),
                        ),
                    ));
                }
            } else {
                const TIME_TO_APPEAR: f32 = 0.3;
                const BUTTON_HOVER_TIME: f32 = 1.0;
                const TIME_TO_DISAPPEAR: f32 = 0.3;
                // spawn the central 3d icon.
                commands.spawn((
                    WorldAssetRoot(asset_server.load(
                        GltfAssetLabel::Scene(0).from_asset("particles_and_effects/red_button.glb"),
                    )),
                    Transform::from_translation(
                        Vec3::from(HexVector2d::from(tile_of_player_tower))
                            .with_y(EFFECT_HOVER_HEIGHT + 0.1),
                    ),
                    AnimatedScale(
                        AnimatedProperty::new_seamless(
                            Vec3::ZERO,
                            [
                                AnimatedPropertyInterval {
                                    next_value: Vec3::ONE * 5.0,
                                    duration: TIME_TO_APPEAR,
                                    mode: EaseFunction::QuinticOut,
                                },
                                AnimatedPropertyInterval {
                                    next_value: Vec3::ONE * 5.0,
                                    duration: BUTTON_HOVER_TIME,
                                    mode: EaseFunction::Linear,
                                },
                                AnimatedPropertyInterval {
                                    next_value: Vec3::ZERO,
                                    duration: TIME_TO_DISAPPEAR,
                                    mode: EaseFunction::QuinticIn,
                                },
                            ]
                            .into(),
                            0.0,
                        ),
                        true,
                    ),
                ));

                let mut rng = rand::rng();

                // Spawn the auxilliary plusses.
                for _ in 0..ACCENT_PARTICLE_NUMBER {
                    let basis_pos = Vec3::from(HexVector2d::from(tile_of_player_tower))
                        .with_y(EFFECT_HOVER_HEIGHT);

                    let offset =
                        Vec3::X.rotate_y(rng.random_range(0.0..TAU)) * rng.random_range(0.8..1.2);
                    let scale = Vec3::splat(rng.random_range(0.8..1.2));

                    commands.spawn((
                        WorldAssetRoot(
                            asset_server.load(
                                GltfAssetLabel::Scene(0)
                                    .from_asset("particles_and_effects/green_plus.glb"),
                            ),
                        ),
                        Transform::from_translation(basis_pos),
                        AnimatedScale(
                            AnimatedProperty::new_seamless(
                                Vec3::ZERO,
                                [
                                    AnimatedPropertyInterval {
                                        next_value: scale,
                                        duration: TIME_TO_APPEAR + rng.random_range(0.0..0.1),
                                        mode: EaseFunction::QuinticOut,
                                    },
                                    AnimatedPropertyInterval {
                                        next_value: scale,
                                        duration: BUTTON_HOVER_TIME + rng.random_range(0.0..0.1),
                                        mode: EaseFunction::Linear,
                                    },
                                    AnimatedPropertyInterval {
                                        next_value: Vec3::ZERO,
                                        duration: TIME_TO_DISAPPEAR + rng.random_range(0.0..0.1),
                                        mode: EaseFunction::QuinticIn,
                                    },
                                ]
                                .into(),
                                0.0,
                            ),
                            true,
                        ),
                        AnimatedTranslation(AnimatedProperty::new_seamless(
                            Vec3::ZERO,
                            [
                                AnimatedPropertyInterval {
                                    next_value: basis_pos + offset,
                                    duration: TIME_TO_APPEAR + rng.random_range(0.0..0.1),
                                    mode: EaseFunction::QuinticOut,
                                },
                                AnimatedPropertyInterval {
                                    next_value: basis_pos + offset,
                                    duration: BUTTON_HOVER_TIME + rng.random_range(0.0..0.1),
                                    mode: EaseFunction::Linear,
                                },
                                AnimatedPropertyInterval {
                                    next_value: basis_pos + offset * 2.0,
                                    duration: TIME_TO_DISAPPEAR + rng.random_range(0.0..0.1),
                                    mode: EaseFunction::QuinticIn,
                                },
                            ]
                            .into(),
                            0.0,
                        )),
                    ));
                }
            }
        }
        ActionEffect::ReducedRemaingOrdersOfPlayer(player) => {
            let Some(owned_pieces) = logical_world
                .0
                .get::<OwnsPieces>(*logical_world.0.resource::<PlayerDirectory>().get(player))
            else {
                return;
            };
            let Some(&tile_of_player_tower) = owned_pieces.list().iter().find_map(|&piece| {
                if logical_world.0.get::<IsWinCondition>(piece).is_some()
                    && let Some(OccupiesTile(tile)) = logical_world.0.get::<OccupiesTile>(piece)
                {
                    logical_world.0.get::<TileId>(*tile)
                } else {
                    None
                }
            }) else {
                return;
            };

            const TIME_TO_APPEAR: f32 = 0.3;
            const BUTTON_HOVER_TIME: f32 = 1.0;
            const TIME_TO_DISAPPEAR: f32 = 0.3;

            // spawn the central 3d icon.
            commands.spawn((
                WorldAssetRoot(asset_server.load(
                    GltfAssetLabel::Scene(0).from_asset("particles_and_effects/red_button.glb"),
                )),
                Transform::from_translation(
                    Vec3::from(HexVector2d::from(tile_of_player_tower))
                        .with_y(EFFECT_HOVER_HEIGHT + 0.1),
                ),
                AnimatedScale(
                    AnimatedProperty::new_seamless(
                        Vec3::ZERO,
                        [
                            AnimatedPropertyInterval {
                                next_value: Vec3::ONE * 5.0,
                                duration: TIME_TO_APPEAR,
                                mode: EaseFunction::QuinticOut,
                            },
                            AnimatedPropertyInterval {
                                next_value: Vec3::ONE * 5.0,
                                duration: BUTTON_HOVER_TIME,
                                mode: EaseFunction::Linear,
                            },
                            AnimatedPropertyInterval {
                                next_value: Vec3::ZERO,
                                duration: TIME_TO_DISAPPEAR,
                                mode: EaseFunction::QuinticIn,
                            },
                        ]
                        .into(),
                        0.0,
                    ),
                    true,
                ),
            ));

            let mut rng = rand::rng();

            // Spawn the auxilliary red.
            for _ in 0..ACCENT_PARTICLE_NUMBER {
                let basis_pos =
                    Vec3::from(HexVector2d::from(tile_of_player_tower)).with_y(EFFECT_HOVER_HEIGHT);

                let offset =
                    Vec3::X.rotate_y(rng.random_range(0.0..TAU)) * rng.random_range(0.8..1.2);
                let scale = Vec3::splat(rng.random_range(0.8..1.2));

                commands.spawn((
                    WorldAssetRoot(asset_server.load(
                        GltfAssetLabel::Scene(0).from_asset("particles_and_effects/red_minus.glb"),
                    )),
                    Transform::from_translation(basis_pos),
                    AnimatedScale(
                        AnimatedProperty::new_seamless(
                            Vec3::ZERO,
                            [
                                AnimatedPropertyInterval {
                                    next_value: scale,
                                    duration: TIME_TO_APPEAR + rng.random_range(0.0..0.1),
                                    mode: EaseFunction::QuinticOut,
                                },
                                AnimatedPropertyInterval {
                                    next_value: scale,
                                    duration: BUTTON_HOVER_TIME + rng.random_range(0.0..0.1),
                                    mode: EaseFunction::Linear,
                                },
                                AnimatedPropertyInterval {
                                    next_value: Vec3::ZERO,
                                    duration: TIME_TO_DISAPPEAR + rng.random_range(0.0..0.1),
                                    mode: EaseFunction::QuinticIn,
                                },
                            ]
                            .into(),
                            0.0,
                        ),
                        true,
                    ),
                    AnimatedTranslation(AnimatedProperty::new_seamless(
                        Vec3::ZERO,
                        [
                            AnimatedPropertyInterval {
                                next_value: basis_pos + offset,
                                duration: TIME_TO_APPEAR + rng.random_range(0.0..0.1),
                                mode: EaseFunction::QuinticOut,
                            },
                            AnimatedPropertyInterval {
                                next_value: basis_pos + offset,
                                duration: BUTTON_HOVER_TIME + rng.random_range(0.0..0.1),
                                mode: EaseFunction::Linear,
                            },
                            AnimatedPropertyInterval {
                                next_value: basis_pos + offset * 2.0,
                                duration: TIME_TO_DISAPPEAR + rng.random_range(0.0..0.1),
                                mode: EaseFunction::QuinticIn,
                            },
                        ]
                        .into(),
                        0.0,
                    )),
                ));
            }
        }
        _ => (),
    }
}
