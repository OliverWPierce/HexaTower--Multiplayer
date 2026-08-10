use bevy::prelude::*;
use core_game_logic::{requests::ActionEffect, tile_mapping::HexVector2d};

use crate::{
    inputs_interface::EffectToDisplay,
    vis_effect_reactions::{EFFECT_HOVER_HEIGHT, anim_bundle_spawn_await_disappear},
};

pub fn piece_damage_or_heal(
    effect: Res<EffectToDisplay>,
    mut commands: Commands,
    asset_server: ResMut<AssetServer>,
) {
    let (tile, accent_asset, hp_change) = match effect.0 {
        ActionEffect::DamagedPiece {
            on_tile,
            hp_removed,
        } => (
            on_tile,
            asset_server.load::<WorldAsset>(
                GltfAssetLabel::Scene(0).from_asset("particles_and_effects/red_minus.glb"),
            ),
            hp_removed,
        ),
        ActionEffect::HealedPiece { on_tile, hp_added } => (
            on_tile,
            asset_server.load::<WorldAsset>(
                GltfAssetLabel::Scene(0).from_asset("particles_and_effects/green_plus.glb"),
            ),
            hp_added,
        ),
        _ => return,
    };

    let basis_loc = Vec3::from(HexVector2d::from(tile)).with_y(EFFECT_HOVER_HEIGHT);

    let mut rng = rand::rng();

    // central bit
    commands.spawn((
        WorldAssetRoot(asset_server.load::<WorldAsset>(
            GltfAssetLabel::Scene(0).from_asset("particles_and_effects/heart_gem.glb"),
        )),
        anim_bundle_spawn_await_disappear(basis_loc, &mut rng, 0.0, 0.0..=0.0),
    ));

    commands.spawn_batch(
        (0..hp_change)
            .map(|_| {
                (
                    anim_bundle_spawn_await_disappear(basis_loc, &mut rng, 0.2, 0.8..=1.2),
                    WorldAssetRoot(accent_asset.clone()),
                )
            })
            .collect::<Box<[_]>>(),
    );
}
