use bevy::prelude::*;
use core_game_logic::{
    markets::MarketId,
    tile_mapping::{HexVector2d, TileId},
};

use crate::{AppState, functional_assets::VisMarketDirectory};

pub struct VisMarketsPlugin;

impl Plugin for VisMarketsPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<MarketSpawned>();

        app.add_systems(
            Update,
            (spin_markets, add_markets).run_if(in_state(AppState::InGame)),
        );
    }
}

// TODO: Make this a relationship component with the visual tile.
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

#[derive(Debug, Message)]
pub struct MarketSpawned {
    pub tile: TileId,
    pub market: MarketId,
}

fn add_markets(
    mut markets_to_visualize: MessageReader<MarketSpawned>,
    mut commands: Commands,
    visual_details: Res<VisMarketDirectory>,
) -> Result<(), BevyError> {
    for MarketSpawned { tile, market } in markets_to_visualize.read() {
        commands.spawn((
            Transform::from_translation(Vec3::from(HexVector2d::from(*tile))),
            MarketModel,
            WorldAssetRoot(visual_details.get_market(*market)?.model.clone()),
        ));
    }

    Ok(())
}
