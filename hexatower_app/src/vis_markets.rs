use bevy::prelude::*;
use core_game_logic::{requests::ActionEffect, tile_mapping::HexVector2d};

use crate::{AppState, functional_assets::VisMarketDirectory, inputs_interface::EffectToDisplay};

pub struct VisMarketsPlugin;

impl Plugin for VisMarketsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, spin_markets.run_if(in_state(AppState::InGame)));

        app.add_systems(
            Update,
            add_markets.run_if(resource_exists_and_changed::<EffectToDisplay>),
        );
    }
}

#[derive(Debug, Component)]
struct MarketModel;

fn spin_markets(
    mut models_to_spin: Query<&mut Transform, With<MarketModel>>,
    delta_time: Res<Time>,
) {
    const SPEED: f32 = 0.17;

    for mut transform in models_to_spin.iter_mut() {
        transform.rotate_y(SPEED * delta_time.delta_secs());
    }
}

fn add_markets(
    effect: Res<EffectToDisplay>,
    mut commands: Commands,
    visual_details: Res<VisMarketDirectory>,
) -> Result<(), BevyError> {
    let ActionEffect::SpawnedNewMarket { tile, market } = effect.0 else {
        return Ok(());
    };

    commands.spawn((
        Transform::from_translation(Vec3::from(HexVector2d::from(tile))),
        MarketModel,
        WorldAssetRoot(visual_details.get_market(market)?.model.clone()),
    ));

    Ok(())
}
