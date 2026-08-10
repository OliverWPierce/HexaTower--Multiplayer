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
pub fn spawn_coins(
    effect: Res<EffectToDisplay>,
    mut commands: Commands,
    asset_server: ResMut<AssetServer>,
    logical_world: Res<LogicalWorld>,
) {
    let ActionEffect::AlteredCoins {
        player,
        delta_coins,
        from_tile,
    } = effect.0
    else {
        return;
    };

    struct TargetingInfo {
        from_loc: Vec3,
        to_loc: Vec3,
        accent_particle: Handle<WorldAsset>,
    }

    let target_info = if let Some(tile_of_player_tower) =
        super::tower_of_player(&logical_world, player)
    {
        if delta_coins.is_negative() {
            TargetingInfo {
                from_loc: Vec3::from(HexVector2d::from(tile_of_player_tower))
                    .with_y(EFFECT_HOVER_HEIGHT),
                to_loc: match from_tile {
                    Some(recipient) => {
                        Vec3::from(HexVector2d::from(recipient)).with_y(EFFECT_HOVER_HEIGHT)
                    }
                    None => Vec3::from(HexVector2d::from(tile_of_player_tower)).with_y(30.0),
                },
                accent_particle: asset_server.load::<WorldAsset>(
                    GltfAssetLabel::Scene(0).from_asset("particles_and_effects/red_minus.glb"),
                ),
            }
        } else {
            TargetingInfo {
                from_loc: match from_tile {
                    Some(source) => {
                        Vec3::from(HexVector2d::from(source)).with_y(EFFECT_HOVER_HEIGHT)
                    }
                    None => Vec3::from(HexVector2d::from(tile_of_player_tower)).with_y(30.0),
                },
                to_loc: Vec3::from(HexVector2d::from(tile_of_player_tower))
                    .with_y(EFFECT_HOVER_HEIGHT),
                accent_particle: asset_server.load::<WorldAsset>(
                    GltfAssetLabel::Scene(0).from_asset("particles_and_effects/green_plus.glb"),
                ),
            }
        }
    } else {
        // the player has no tile to represent them.
        if delta_coins.is_negative() {
            match from_tile {
                Some(recipient) => TargetingInfo {
                    from_loc: Vec3::from(HexVector2d::from(recipient)).with_y(30.0),
                    to_loc: Vec3::from(HexVector2d::from(recipient)).with_y(EFFECT_HOVER_HEIGHT),
                    accent_particle: asset_server.load::<WorldAsset>(
                        GltfAssetLabel::Scene(0)
                            .from_asset("particles_and_effects/question_mark.glb"),
                    ),
                },
                None => return,
            }
        } else {
            match from_tile {
                Some(source) => TargetingInfo {
                    from_loc: Vec3::from(HexVector2d::from(source)).with_y(EFFECT_HOVER_HEIGHT),
                    to_loc: Vec3::from(HexVector2d::from(source)).with_y(30.0),
                    accent_particle: asset_server.load::<WorldAsset>(
                        GltfAssetLabel::Scene(0)
                            .from_asset("particles_and_effects/question_mark.glb"),
                    ),
                },
                None => return,
            }
        }
    };

    let mut rng = rand::rng();

    const SCALE_IN_DUR: f32 = 0.05;
    const HOLD_ABOVE_SOURCE_DUR: f32 = 0.5;
    const FLIGHT_TIME_SCALE: f32 = 0.3;
    const HOLD_ABOVE_TARGET_DUR: f32 = 0.5;
    const ENTER_TARGET_DUR: f32 = 0.05;

    const DUR_VARIANCE: f32 = 0.05;

    let flight_time = FLIGHT_TIME_SCALE * target_info.from_loc.distance(target_info.to_loc);

    for index in 0..delta_coins.abs() * 2 {
        let offset = Vec3::X.rotate_y(rng.random_range(0.0..TAU)) * rng.random_range(0.1..1.2);
        let scale = Vec3::splat(rng.random_range(0.8..1.2));

        commands.spawn((
            (
                Transform::from_translation(target_info.from_loc).with_scale(Vec3::ZERO),
                AnimatedTranslation(AnimatedProperty::new_seamless(
                    target_info.from_loc,
                    [
                        AnimatedPropertyInterval {
                            next_value: target_info.from_loc + offset,
                            duration: SCALE_IN_DUR + rng.random_range(0.0..DUR_VARIANCE),
                            mode: EaseFunction::SmoothStep,
                        },
                        AnimatedPropertyInterval {
                            next_value: target_info.from_loc + offset,
                            duration: HOLD_ABOVE_SOURCE_DUR + rng.random_range(0.0..DUR_VARIANCE),
                            mode: EaseFunction::SmoothStep,
                        },
                        AnimatedPropertyInterval {
                            next_value: target_info.to_loc + offset * 0.5,
                            duration: flight_time + rng.random_range(0.0..DUR_VARIANCE),
                            mode: EaseFunction::SmoothStep,
                        },
                        AnimatedPropertyInterval {
                            next_value: target_info.to_loc + offset * 0.5,
                            duration: HOLD_ABOVE_TARGET_DUR + rng.random_range(0.0..DUR_VARIANCE),
                            mode: EaseFunction::SmoothStep,
                        },
                        AnimatedPropertyInterval {
                            next_value: target_info.to_loc.with_y(0.0),
                            duration: ENTER_TARGET_DUR + rng.random_range(0.0..DUR_VARIANCE),
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
                                duration: SCALE_IN_DUR + rng.random_range(0.0..DUR_VARIANCE),
                                mode: EaseFunction::SmoothStep,
                            },
                            AnimatedPropertyInterval {
                                next_value: scale,
                                duration: HOLD_ABOVE_SOURCE_DUR
                                    + flight_time
                                    + ENTER_TARGET_DUR
                                    + rng.random_range(0.0..DUR_VARIANCE) * 3.0,
                                mode: EaseFunction::SmoothStep,
                            },
                            AnimatedPropertyInterval {
                                next_value: Vec3::ZERO,
                                duration: ENTER_TARGET_DUR + rng.random_range(0.0..DUR_VARIANCE),
                                mode: EaseFunction::QuadraticIn,
                            },
                        ]
                        .into(),
                        0.0,
                    ),
                    true,
                ),
            ),
            WorldAssetRoot(if index < delta_coins.abs() {
                asset_server.load::<WorldAsset>(
                    GltfAssetLabel::Scene(0).from_asset("particles_and_effects/Coin.glb"),
                )
            } else {
                target_info.accent_particle.clone()
            }),
        ));
    }
}
