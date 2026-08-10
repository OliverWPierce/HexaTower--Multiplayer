use bevy::prelude::*;
use core_game_logic::{requests::ActionEffect, tile_mapping::HexVector2d};

use crate::{
    functional_assets::LogicalWorld,
    inputs_interface::EffectToDisplay,
    vis_effect_reactions::{
        EFFECT_HOVER_HEIGHT, FlyToAnim, flight_animation_presets::SpawnInPlaceThenFly,
        tower_of_player,
    },
};

pub fn purchase_item(
    action_effect: Res<EffectToDisplay>,
    logical_world: Res<LogicalWorld>,
    mut commands: Commands,
    asset_server: ResMut<AssetServer>,
) {
    let ActionEffect::AddedCardToInventory {
        player,
        source: Some(tile),
        ..
    } = action_effect.0
    else {
        return;
    };

    let start_pos = Vec3::from(HexVector2d::from(tile)).with_y(EFFECT_HOVER_HEIGHT);
    let end_pos = {
        if let Some(tower) = tower_of_player(&logical_world, player) {
            Vec3::from(HexVector2d::from(tower)).with_y(EFFECT_HOVER_HEIGHT)
        } else {
            Vec3::from(HexVector2d::from(tile)).with_y(EFFECT_HOVER_HEIGHT + 100.0)
        }
    };
    let mut rng = rand::rng();

    commands.spawn((
        SpawnInPlaceThenFly::anim_bundle(start_pos, end_pos, -0.1..0.1, 1.0..1.0, 0.1, &mut rng),
        WorldAssetRoot(asset_server.load::<WorldAsset>(
            GltfAssetLabel::Scene(0).from_asset("particles_and_effects/marbled_box.glb"),
        )),
    ));

    const ACCENT_PARTICLE_COUNT: u8 = 30;

    commands.spawn_batch(
        (0..ACCENT_PARTICLE_COUNT)
            .map(|_| {
                (
                    SpawnInPlaceThenFly::anim_bundle(
                        start_pos,
                        end_pos,
                        0.5..0.8,
                        0.8..1.2,
                        0.1,
                        &mut rng,
                    ),
                    WorldAssetRoot(asset_server.load::<WorldAsset>(
                        GltfAssetLabel::Scene(0).from_asset("particles_and_effects/green_plu.glb"),
                    )),
                )
            })
            .collect::<Box<[_]>>(),
    );
}
