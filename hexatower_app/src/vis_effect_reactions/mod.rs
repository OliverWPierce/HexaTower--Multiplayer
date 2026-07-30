use std::f32::consts::TAU;

/// Notes on this module.
/// 1. Systems should only reference the "effect to display" resource once: when starting the sequence. Other systems (ie. ones that move particles) should not reference that resource because it is liable to change frequently.
/// 2. Sequences should not reference visual world data that they do not create, because those things may be destroyed by other reactions. (ie. A particle should fly to a logical tile location, not to the location of a visual piece model, because a concurrent reaction could despawn the piece model.)
use bevy::prelude::*;
use core_game_logic::{
    pieces::{IsWinCondition, OccupiesTile, OwnsPieces},
    players::PlayerDirectory,
    requests::ActionEffect,
    tile_mapping::{HexVector2d, TileId},
};
use rand::RngExt;

use crate::{AppState, functional_assets::LogicalWorld, inputs_interface::EffectToDisplay};

pub struct VisEffectReactions;

impl Plugin for VisEffectReactions {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            update_anim_progress.run_if(in_state(AppState::InGame)),
        );

        app.add_systems(
            Update,
            spawn_coins.run_if(resource_exists_and_changed::<EffectToDisplay>),
        );

        app.add_systems(
            Update,
            (animate_translation)
                .after(update_anim_progress)
                .run_if(in_state(AppState::InGame)),
        );
    }
}

#[derive(Debug, Component)]
pub struct AnimateTranslation(pub EasingCurve<Vec3>);
#[derive(Debug, Component)]
pub struct AnimProgress(f32);

fn animate_translation(
    mut translations: Query<(&AnimateTranslation, &mut Transform, &AnimProgress)>,
) {
    translations
        .iter_mut()
        .for_each(|(new_translation, mut current_transform, anim_progress)| {
            if let Some(new_translation) = new_translation.0.sample(anim_progress.0) {
                current_transform.translation = new_translation;
            }
        })
}

fn update_anim_progress(mut prog: Query<&mut AnimProgress>, time: Res<Time>) {
    const ANIM_SPEED: f32 = 0.8;

    prog.iter_mut().for_each(|mut anim_prog| {
        anim_prog.0 = (anim_prog.0 + time.delta_secs() * ANIM_SPEED).clamp(0.0, 1.0);
    });
}

fn spawn_coins(
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

    let target_info = if let Some(owned_pieces) = logical_world
        .0
        .get::<OwnsPieces>(*logical_world.0.resource::<PlayerDirectory>().get(player))
        && let Some(&tile_of_player_tower) = owned_pieces.list().iter().find_map(|&piece| {
            if logical_world.0.get::<IsWinCondition>(piece).is_some()
                && let Some(OccupiesTile(tile)) = logical_world.0.get::<OccupiesTile>(piece)
            {
                logical_world.0.get::<TileId>(*tile)
            } else {
                None
            }
        }) {
        if delta_coins.is_negative() {
            TargetingInfo {
                from_loc: HexVector2d::from(tile_of_player_tower).into(),
                to_loc: match from_tile {
                    Some(recipient) => HexVector2d::from(recipient).into(),
                    None => Vec3::from(HexVector2d::from(tile_of_player_tower)).with_y(30.0),
                },
                accent_particle: asset_server.load::<WorldAsset>(
                    GltfAssetLabel::Scene(0).from_asset("particles_and_effects/red_minus.glb"),
                ),
            }
        } else {
            TargetingInfo {
                from_loc: match from_tile {
                    Some(source) => HexVector2d::from(source).into(),
                    None => Vec3::from(HexVector2d::from(tile_of_player_tower)).with_y(30.0),
                },
                to_loc: HexVector2d::from(tile_of_player_tower).into(),
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
                    to_loc: HexVector2d::from(recipient).into(),
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
                    from_loc: HexVector2d::from(source).into(),
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

    for _ in 0..delta_coins.abs() {
        let start_pos =
            target_info.from_loc + rng.random::<Vec3>().normalize() * rng.random_range(0.1..1.2);
        let end_pos =
            target_info.to_loc + rng.random::<Vec3>().normalize() * rng.random_range(0.1..1.2);

        commands.spawn((
            Transform::from_translation(start_pos).rotate_local_z(rng.random_range(-TAU..TAU)),
            WorldAssetRoot(asset_server.load::<WorldAsset>(
                GltfAssetLabel::Scene(0).from_asset("particles_and_effects/Coin.glb"),
            )),
            AnimateTranslation(EasingCurve::new(
                start_pos,
                end_pos,
                EaseFunction::SmoothStep,
            )),
            AnimProgress(0.0),
        ));
    }

    for _ in 0..delta_coins.abs() {
        let start_pos =
            target_info.from_loc + rng.random::<Vec3>().normalize() * rng.random_range(0.1..1.2);
        let end_pos =
            target_info.to_loc + rng.random::<Vec3>().normalize() * rng.random_range(0.1..1.2);

        commands.spawn((
            Transform::from_translation(
                target_info.from_loc
                    + rng.random::<Vec3>().normalize() * rng.random_range(0.1..1.2),
            )
            .rotate_local_z(rng.random_range(-TAU..TAU)),
            WorldAssetRoot(target_info.accent_particle.clone()),
            AnimateTranslation(EasingCurve::new(
                start_pos,
                end_pos,
                EaseFunction::SmoothStep,
            )),
            AnimProgress(0.0),
        ));
    }
}
