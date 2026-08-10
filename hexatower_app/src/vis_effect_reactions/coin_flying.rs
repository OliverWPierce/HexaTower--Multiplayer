use bevy::prelude::*;
use core_game_logic::{requests::ActionEffect, tile_mapping::HexVector2d};

use crate::{
    functional_assets::LogicalWorld,
    inputs_interface::EffectToDisplay,
    vis_effect_reactions::{
        EFFECT_HOVER_HEIGHT, FlyToAnim, flight_animation_presets::SpawnInPlaceThenFly,
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

    commands.spawn_batch(
        (0..delta_coins.abs())
            .map(|_| {
                (
                    SpawnInPlaceThenFly::anim_bundle(
                        target_info.from_loc,
                        target_info.to_loc,
                        0.5..0.8,
                        0.8..1.2,
                        0.1,
                        &mut rng,
                    ),
                    WorldAssetRoot(asset_server.load::<WorldAsset>(
                        GltfAssetLabel::Scene(0).from_asset("particles_and_effects/Coin.glb"),
                    )),
                )
            })
            .collect::<Box<[_]>>(),
    );

    commands.spawn_batch(
        (0..delta_coins.abs())
            .map(|_| {
                (
                    SpawnInPlaceThenFly::anim_bundle(
                        target_info.from_loc,
                        target_info.to_loc,
                        0.5..0.8,
                        0.8..1.2,
                        0.1,
                        &mut rng,
                    ),
                    WorldAssetRoot(target_info.accent_particle.clone()),
                )
            })
            .collect::<Box<[_]>>(),
    );
}
