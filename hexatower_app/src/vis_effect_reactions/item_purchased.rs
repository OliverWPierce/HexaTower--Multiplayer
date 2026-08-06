use bevy::prelude::*;
use core_game_logic::{requests::ActionEffect, tile_mapping::HexVector2d};

use crate::{
    inputs_interface::EffectToDisplay,
    vis_effect_reactions::{
        AnimatedProperty, AnimatedPropertyInterval, AnimatedScale, EFFECT_HOVER_HEIGHT,
    },
};

pub fn purchase_item(
    action_effect: Res<EffectToDisplay>,
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

    const APPEAR_DURATION: f32 = 1.0;
    const IN_OUT_SPEED: f32 = 0.5;

    let basis_loc = Vec3::from(HexVector2d::from(tile)).with_y(EFFECT_HOVER_HEIGHT);

    commands.spawn((
        WorldAssetRoot(asset_server.load::<WorldAsset>(
            GltfAssetLabel::Scene(0).from_asset("particles_and_effects/marbled_box.glb"),
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
}
