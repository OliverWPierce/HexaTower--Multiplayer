use std::f32::consts::TAU;

use bevy::prelude::*;
use core_game_logic::{requests::ActionEffect, tile_mapping::HexVector2d};
use rand::RngExt;

use crate::{
    functional_assets::LogicalWorld,
    inputs_interface::EffectToDisplay,
    vis_effect_reactions::{
        AnimatedProperty, AnimatedPropertyInterval, AnimatedScale, AnimatedTranslation,
        EFFECT_HOVER_HEIGHT, FlyToAnim, flight_animation_presets::SpawnInPlaceThenFly,
        tower_of_player,
    },
};

pub fn player_order_change(
    effect: Res<EffectToDisplay>,
    mut commands: Commands,
    asset_server: ResMut<AssetServer>,
    logical_world: Res<LogicalWorld>,
) {
    const ACCENT_PARTICLE_COUNT: u8 = 30;

    match effect.0 {
        ActionEffect::IncreasedRemainingOrdersOfPlayer {
            receipient,
            source: Some(tile),
        } if let Some(tower_tile) = tower_of_player(&logical_world, receipient) => {
            let mut rng = rand::rng();

            commands.spawn_batch(
                (0..ACCENT_PARTICLE_COUNT)
                    .map(|_| {
                        (
                            SpawnInPlaceThenFly::anim_bundle(
                                Vec3::from(HexVector2d::from(tile)).with_y(EFFECT_HOVER_HEIGHT),
                                Vec3::from(HexVector2d::from(tower_tile))
                                    .with_y(EFFECT_HOVER_HEIGHT),
                                0.5..0.8,
                                0.8..1.2,
                                0.1,
                                &mut rng,
                            ),
                            WorldAssetRoot(
                                asset_server.load::<WorldAsset>(
                                    GltfAssetLabel::Scene(0)
                                        .from_asset("particles_and_effects/green_plus.glb"),
                                ),
                            ),
                        )
                    })
                    .collect::<Box<[_]>>(),
            );

            commands.spawn((
                SpawnInPlaceThenFly::anim_bundle(
                    Vec3::from(HexVector2d::from(tile)).with_y(EFFECT_HOVER_HEIGHT),
                    Vec3::from(HexVector2d::from(tower_tile)).with_y(EFFECT_HOVER_HEIGHT),
                    0.0..0.0,
                    1.0..1.0,
                    0.1,
                    &mut rng,
                ),
                WorldAssetRoot(asset_server.load::<WorldAsset>(
                    GltfAssetLabel::Scene(0).from_asset("particles_and_effects/red_button.glb"),
                )),
            ));
        }
        ActionEffect::IncreasedRemainingOrdersOfPlayer {
            receipient,
            source: None,
        } if let Some(tower_tile) = tower_of_player(&logical_world, receipient) => {
            const APPEAR_DURATION: f32 = 1.0;
            const IN_OUT_SPEED: f32 = 0.5;

            let basis_loc = Vec3::from(HexVector2d::from(tower_tile)).with_y(EFFECT_HOVER_HEIGHT);

            // central bit
            commands.spawn((
                WorldAssetRoot(asset_server.load::<WorldAsset>(
                    GltfAssetLabel::Scene(0).from_asset("particles_and_effects/red_button.glb"),
                )),
                Transform::from_translation(basis_loc).with_scale(Vec3::ZERO),
                AnimatedScale(
                    AnimatedProperty::new_seamless(
                        Vec3::ZERO,
                        [
                            AnimatedPropertyInterval {
                                next_value: Vec3::ONE,
                                duration: IN_OUT_SPEED,
                                mode: EaseFunction::BackOut,
                            },
                            AnimatedPropertyInterval {
                                next_value: Vec3::ONE,
                                duration: APPEAR_DURATION,
                                mode: EaseFunction::Linear,
                            },
                            AnimatedPropertyInterval {
                                next_value: Vec3::ZERO,
                                duration: IN_OUT_SPEED,
                                mode: EaseFunction::BackIn,
                            },
                        ]
                        .into(),
                        0.0,
                    ),
                    true,
                ),
            ));

            let mut rng = rand::rng();

            commands.spawn_batch(
                (0..ACCENT_PARTICLE_COUNT)
                    .map(|_| {
                        (
                            Transform::from_translation(
                                basis_loc
                                    + Vec3::X.rotate_y(rng.random_range(0.0..TAU))
                                        * rng.random_range(0.8..1.2),
                            )
                            .with_scale(Vec3::ZERO),
                            AnimatedScale(
                                AnimatedProperty::new_seamless(
                                    Vec3::ZERO,
                                    [
                                        AnimatedPropertyInterval {
                                            next_value: Vec3::ONE,
                                            duration: IN_OUT_SPEED + rng.random_range(0.0..0.2),
                                            mode: EaseFunction::BackOut,
                                        },
                                        AnimatedPropertyInterval {
                                            next_value: Vec3::ONE,
                                            duration: APPEAR_DURATION + rng.random_range(0.0..0.2),
                                            mode: EaseFunction::Linear,
                                        },
                                        AnimatedPropertyInterval {
                                            next_value: Vec3::ZERO,
                                            duration: IN_OUT_SPEED + rng.random_range(0.0..0.2),
                                            mode: EaseFunction::BackIn,
                                        },
                                    ]
                                    .into(),
                                    0.0,
                                ),
                                true,
                            ),
                            WorldAssetRoot(
                                asset_server
                                    .load::<WorldAsset>(
                                        GltfAssetLabel::Scene(0)
                                            .from_asset("particles_and_effects/green_plus.glb"),
                                    )
                                    .clone(),
                            ),
                        )
                    })
                    .collect::<Box<[_]>>(),
            );
        }
        ActionEffect::ReducedRemaingOrdersOfPlayer(player)
            if let Some(tower_tile) = super::tower_of_player(&logical_world, player) =>
        {
            const APPEAR_DURATION: f32 = 1.0;
            const IN_OUT_SPEED: f32 = 0.5;

            let basis_loc = Vec3::from(HexVector2d::from(tower_tile)).with_y(EFFECT_HOVER_HEIGHT);

            // central bit
            commands.spawn((
                WorldAssetRoot(asset_server.load::<WorldAsset>(
                    GltfAssetLabel::Scene(0).from_asset("particles_and_effects/red_button.glb"),
                )),
                Transform::from_translation(basis_loc).with_scale(Vec3::ZERO),
                AnimatedScale(
                    AnimatedProperty::new_seamless(
                        Vec3::ZERO,
                        [
                            AnimatedPropertyInterval {
                                next_value: Vec3::ONE,
                                duration: IN_OUT_SPEED,
                                mode: EaseFunction::BackOut,
                            },
                            AnimatedPropertyInterval {
                                next_value: Vec3::ONE,
                                duration: APPEAR_DURATION,
                                mode: EaseFunction::Linear,
                            },
                            AnimatedPropertyInterval {
                                next_value: Vec3::ZERO,
                                duration: IN_OUT_SPEED,
                                mode: EaseFunction::BackIn,
                            },
                        ]
                        .into(),
                        0.0,
                    ),
                    true,
                ),
            ));

            let mut rng = rand::rng();

            commands.spawn_batch(
                (0..ACCENT_PARTICLE_COUNT)
                    .map(|_| {
                        (
                            Transform::from_translation(
                                basis_loc
                                    + Vec3::X.rotate_y(rng.random_range(0.0..TAU))
                                        * rng.random_range(0.8..1.2),
                            )
                            .with_scale(Vec3::ZERO),
                            AnimatedScale(
                                AnimatedProperty::new_seamless(
                                    Vec3::ZERO,
                                    [
                                        AnimatedPropertyInterval {
                                            next_value: Vec3::ONE,
                                            duration: IN_OUT_SPEED + rng.random_range(0.0..0.2),
                                            mode: EaseFunction::BackOut,
                                        },
                                        AnimatedPropertyInterval {
                                            next_value: Vec3::ONE,
                                            duration: APPEAR_DURATION + rng.random_range(0.0..0.2),
                                            mode: EaseFunction::Linear,
                                        },
                                        AnimatedPropertyInterval {
                                            next_value: Vec3::ZERO,
                                            duration: IN_OUT_SPEED + rng.random_range(0.0..0.2),
                                            mode: EaseFunction::BackIn,
                                        },
                                    ]
                                    .into(),
                                    0.0,
                                ),
                                true,
                            ),
                            WorldAssetRoot(
                                asset_server
                                    .load::<WorldAsset>(
                                        GltfAssetLabel::Scene(0)
                                            .from_asset("particles_and_effects/red_minus.glb"),
                                    )
                                    .clone(),
                            ),
                        )
                    })
                    .collect::<Box<[_]>>(),
            );
        }
        _ => (),
    }
}
