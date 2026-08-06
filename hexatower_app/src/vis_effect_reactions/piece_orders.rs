use std::f32::consts::TAU;

use bevy::prelude::*;
use core_game_logic::{requests::ActionEffect, tile_mapping::HexVector2d};
use rand::RngExt;

use crate::{
    inputs_interface::EffectToDisplay,
    vis_effect_reactions::{
        AnimatedProperty, AnimatedPropertyInterval, AnimatedScale, EFFECT_HOVER_HEIGHT,
    },
};

pub fn piece_orders(
    effect: Res<EffectToDisplay>,
    mut commands: Commands,
    asset_server: ResMut<AssetServer>,
) {
    let (tile, accent_asset) = match effect.0 {
        ActionEffect::IncreasedRemainingOrdersOfPiece { tile_of_piece } => (
            tile_of_piece,
            asset_server.load::<WorldAsset>(
                GltfAssetLabel::Scene(0).from_asset("particles_and_effects/green_plus.glb"),
            ),
        ),
        ActionEffect::ReducedRemainingOrdersOfPiece { tile_of_piece } => (
            tile_of_piece,
            asset_server.load::<WorldAsset>(
                GltfAssetLabel::Scene(0).from_asset("particles_and_effects/red_minus.glb"),
            ),
        ),
        _ => return,
    };

    const APPEAR_DURATION: f32 = 1.0;
    const IN_OUT_SPEED: f32 = 0.5;

    let basis_loc = Vec3::from(HexVector2d::from(tile)).with_y(EFFECT_HOVER_HEIGHT);

    // central bit
    commands.spawn((
        WorldAssetRoot(asset_server.load::<WorldAsset>(
            GltfAssetLabel::Scene(0).from_asset("particles_and_effects/order_envelope.glb"),
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

    const ACCENT_PARTICLE_COUNT: u8 = 30;

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
                    WorldAssetRoot(accent_asset.clone()),
                )
            })
            .collect::<Box<[_]>>(),
    );
}
