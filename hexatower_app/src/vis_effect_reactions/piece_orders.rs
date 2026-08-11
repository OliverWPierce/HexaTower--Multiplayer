use bevy::prelude::*;
use core_game_logic::{requests::ActionEffect, tile_mapping::HexVector2d};

use crate::{
    inputs_interface::EffectToDisplay,
    vis_effect_reactions::{EFFECT_HOVER_HEIGHT, anim_bundle_spawn_await_disappear},
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

    let basis_loc = Vec3::from(HexVector2d::from(tile)).with_y(EFFECT_HOVER_HEIGHT);

    let mut rng = rand::rng();

    // central bit
    commands.spawn((
        WorldAssetRoot(asset_server.load::<WorldAsset>(
            GltfAssetLabel::Scene(0).from_asset("particles_and_effects/order_envelope.glb"),
        )),
        anim_bundle_spawn_await_disappear(basis_loc, &mut rng, 0.0, 0.0..=0.0),
    ));

    const ACCENT_PARTICLE_COUNT: u8 = 30;

    commands.spawn_batch(
        (0..ACCENT_PARTICLE_COUNT)
            .map(|_| {
                (
                    anim_bundle_spawn_await_disappear(basis_loc, &mut rng, 0.2, 0.8..=1.2),
                    WorldAssetRoot(accent_asset.clone()),
                )
            })
            .collect::<Box<[_]>>(),
    );
}
