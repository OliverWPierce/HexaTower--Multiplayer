use std::f32::consts::TAU;

use bevy::prelude::*;
use core_game_logic::{
    players::PlayerId,
    tile_mapping::{HexVector2d, TileId},
};
use rand::RngExt;

use crate::AppState;

pub struct VisEffects3DPlugin;

impl Plugin for VisEffects3DPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (update_gravity_particles, spawn_particles).run_if(in_state(AppState::InGame)),
        );

        app.add_message::<AlteredCoins>();
    }
}

#[derive(Debug, Component)]
struct GravityParticle {
    present_velocity: Vec3,
}

fn update_gravity_particles(
    mut particles: Query<(Entity, &mut GravityParticle, &mut Transform)>,
    time: Res<Time>,
    mut commands: Commands,
) {
    const ACC_GRAVITY: f32 = 9.8;

    for (entity, mut particle, mut transform) in particles.iter_mut() {
        particle.present_velocity.y -= ACC_GRAVITY * time.delta_secs();
        transform.translation += particle.present_velocity * time.delta_secs();

        if transform.translation.y < -0.1 {
            commands.entity(entity).despawn();
        }
    }
}

#[derive(Debug, Message)]
pub struct AlteredCoins {
    pub player: PlayerId,
    pub delta_coins: i32,
    pub from_tile: Option<TileId>,
}

fn spawn_particles(
    mut asset_server: ResMut<AssetServer>,
    mut coins: MessageReader<AlteredCoins>,
    mut commands: Commands,
) {
    let mut rng = rand::rng();

    for &AlteredCoins {
        player,
        delta_coins,
        from_tile,
    } in coins.read()
    {
        if from_tile.is_none() {
            continue;
        }

        let spawn_location = Vec3::from(HexVector2d::from(from_tile.unwrap()));

        for _ in 0..delta_coins {
            commands.spawn((
                Transform::from_translation(spawn_location)
                    .with_scale(Vec3::splat(0.3))
                    .rotate_local_z(rng.random_range(-TAU..TAU)),
                WorldAssetRoot(asset_server.load::<WorldAsset>(
                    GltfAssetLabel::Scene(0).from_asset("base_plates/yellow_baseplate.glb"),
                )),
                GravityParticle {
                    present_velocity: Vec3 {
                        x: rng.random_range(-1.0..1.0),
                        y: rng.random_range(1.0..2.0),
                        z: rng.random_range(-1.0..1.0),
                    },
                },
            ));
        }

        let accent = if delta_coins.signum().is_negative() {
            asset_server.load::<WorldAsset>(
                GltfAssetLabel::Scene(0).from_asset("base_plates/red_baseplate.glb"),
            )
        } else {
            asset_server.load::<WorldAsset>(
                GltfAssetLabel::Scene(0).from_asset("base_plates/green_baseplate.glb"),
            )
        };

        for _ in 0..delta_coins {
            commands.spawn((
                Transform::from_translation(spawn_location)
                    .with_scale(Vec3::splat(0.05))
                    .rotate_local_z(rng.random_range(-TAU..TAU)),
                WorldAssetRoot(accent.clone()),
                GravityParticle {
                    present_velocity: Vec3 {
                        x: rng.random_range(-1.0..1.0),
                        y: rng.random_range(1.0..2.0),
                        z: rng.random_range(-1.0..1.0),
                    },
                },
            ));
        }
    }
}
