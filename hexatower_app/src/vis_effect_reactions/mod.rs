/// Notes on this module.
/// 1. Systems should only reference the "effect to display" resource once: when starting the sequence. Other systems (ie. ones that move particles) should not reference that resource because it is liable to change frequently.
/// 2. Sequences should not reference visual world data that they do not create, because those things may be destroyed by other reactions. (ie. A particle should fly to a logical tile location, not to the location of a visual piece model, because a concurrent reaction could despawn the piece model.)
use bevy::{
    ecs::component::{ComponentMutability, Mutable},
    prelude::*,
};
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
            (spawn_coins, player_order_change)
                .run_if(resource_exists_and_changed::<EffectToDisplay>),
        );

        app.add_systems(
            Update,
            (animate_translation, animate_scale)
                .after(update_anim_progress)
                .run_if(in_state(AppState::InGame)),
        );
    }
}

const EFFECT_HOVER_HEIGHT: f32 = 1.3;
const GENERAL_SPPED_MULTIPLYER: f32 = 1.0;

#[derive(Debug, Component)]
pub struct AnimateTranslation(pub EasingCurve<Vec3>);
#[derive(Debug, Component)]
pub struct AnimateScale(pub EasingCurve<Vec3>);
#[derive(Debug, Component)]
pub struct AnimProgress {
    progress: f32,
    speed_multiplyer: f32,
}

fn animate_translation(
    mut translations: Query<(&AnimateTranslation, &mut Transform, &AnimProgress)>,
) {
    translations
        .iter_mut()
        .for_each(|(new_translation, mut current_transform, anim_progress)| {
            if let Some(new_translation) = new_translation.0.sample(anim_progress.progress) {
                current_transform.translation = new_translation;
            }
        })
}

fn animate_scale(mut translations: Query<(&AnimateScale, &mut Transform, &AnimProgress)>) {
    translations
        .iter_mut()
        .for_each(|(new_scale, mut current_transform, anim_progress)| {
            if let Some(new_scale) = new_scale.0.sample(anim_progress.progress) {
                current_transform.scale = new_scale;
            }
        })
}

fn update_anim_progress(
    mut prog: Query<(Entity, &mut AnimProgress)>,
    time: Res<Time>,
    mut commands: Commands,
) {
    const GENERAL_SPEED_MULTIPLYER: f32 = 1.0;

    prog.iter_mut().for_each(|(ent, mut anim_prog)| {
        anim_prog.progress =
            (anim_prog.progress + time.delta_secs() * GENERAL_SPEED_MULTIPLYER).clamp(0.0, 1.0);

        if anim_prog.progress == 1.0 {
            commands.entity(ent).despawn();
        }
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

    for _ in 0..delta_coins.abs() {
        let start_pos =
            target_info.from_loc + rng.random::<Vec3>().normalize() * rng.random_range(0.1..1.2);
        let end_pos = target_info.to_loc;

        commands.spawn((
            Transform::from_translation(start_pos).rotate_local_z(rng.random_range(-0.5..0.5)),
            WorldAssetRoot(asset_server.load::<WorldAsset>(
                GltfAssetLabel::Scene(0).from_asset("particles_and_effects/Coin.glb"),
            )),
            AnimateTranslation(EasingCurve::new(
                start_pos,
                end_pos,
                EaseFunction::SmoothStep,
            )),
            AnimProgress {
                progress: 0.0,
                speed_multiplyer: 0.8,
            },
        ));
    }

    for _ in 0..delta_coins.abs() {
        let start_pos =
            target_info.from_loc + rng.random::<Vec3>().normalize() * rng.random_range(0.1..1.2);
        let end_pos = target_info.to_loc;

        commands.spawn((
            Transform::from_translation(start_pos).rotate_local_z(rng.random_range(-0.5..0.5)),
            WorldAssetRoot(target_info.accent_particle.clone()),
            AnimateTranslation(EasingCurve::new(
                start_pos,
                end_pos,
                EaseFunction::SmoothStep,
            )),
            AnimProgress {
                progress: 0.0,
                speed_multiplyer: 0.8,
            },
        ));
    }
}

fn player_order_change(
    effect: Res<EffectToDisplay>,
    mut commands: Commands,
    asset_server: ResMut<AssetServer>,
    logical_world: Res<LogicalWorld>,
) {
    const ACCENT_PARTICLE_NUMBER: u8 = 7;
    const SPEED: f32 = 0.4;

    match effect.0 {
        ActionEffect::IncreasedRemainingOrdersOfPlayer { receipient, source } => {
            let Some(owned_pieces) = logical_world.0.get::<OwnsPieces>(
                *logical_world
                    .0
                    .resource::<PlayerDirectory>()
                    .get(receipient),
            ) else {
                return;
            };
            let Some(&tile_of_player_tower) = owned_pieces.list().iter().find_map(|&piece| {
                if logical_world.0.get::<IsWinCondition>(piece).is_some()
                    && let Some(OccupiesTile(tile)) = logical_world.0.get::<OccupiesTile>(piece)
                {
                    logical_world.0.get::<TileId>(*tile)
                } else {
                    None
                }
            }) else {
                return;
            };

            match source {
                Some(source_tile) => {
                    commands.spawn((
                        Transform::from_translation(
                            Vec3::from(HexVector2d::from(source_tile)).with_y(EFFECT_HOVER_HEIGHT),
                        ),
                        AnimProgress {
                            progress: 0.0,
                            speed_multiplyer: SPEED,
                        },
                        WorldAssetRoot(
                            asset_server.load(
                                GltfAssetLabel::Scene(0)
                                    .from_asset("particles_and_effects/red_button.glb"),
                            ),
                        ),
                        AnimateTranslation(EasingCurve::new(
                            Vec3::from(HexVector2d::from(source_tile)).with_y(EFFECT_HOVER_HEIGHT),
                            Vec3::from(HexVector2d::from(tile_of_player_tower))
                                .with_y(EFFECT_HOVER_HEIGHT),
                            EaseFunction::SmoothStep,
                        )),
                    ));

                    let mut rng = rand::rng();

                    for _ in 0..ACCENT_PARTICLE_NUMBER {
                        let start_pos = Vec3::from(HexVector2d::from(source_tile))
                            .with_y(EFFECT_HOVER_HEIGHT)
                            + rng.random::<Vec3>().normalize() * rng.random_range(0.1..1.2);
                        let end_pos = Vec3::from(HexVector2d::from(tile_of_player_tower))
                            .with_y(EFFECT_HOVER_HEIGHT);

                        commands.spawn((
                            Transform::from_translation(start_pos)
                                .rotate_local_z(rng.random_range(-0.5..0.5)),
                            WorldAssetRoot(
                                asset_server.load(
                                    GltfAssetLabel::Scene(0)
                                        .from_asset("particles_and_effects/green_plus.glb"),
                                ),
                            ),
                            AnimateTranslation(EasingCurve::new(
                                start_pos,
                                end_pos,
                                EaseFunction::SmoothStep,
                            )),
                            AnimProgress {
                                progress: 0.0,
                                speed_multiplyer: SPEED,
                            },
                        ));
                    }
                }
                None => {
                    commands.spawn((
                        Transform::from_translation(
                            Vec3::from(HexVector2d::from(tile_of_player_tower))
                                .with_y(EFFECT_HOVER_HEIGHT),
                        ),
                        AnimProgress {
                            progress: 0.0,
                            speed_multiplyer: SPEED,
                        },
                        WorldAssetRoot(
                            asset_server.load(
                                GltfAssetLabel::Scene(0)
                                    .from_asset("particles_and_effects/red_button.glb"),
                            ),
                        ),
                        AnimateScale(EasingCurve::new(
                            Vec3::ZERO,
                            Vec3::ONE,
                            EaseFunction::ExponentialIn,
                        )),
                    ));

                    let mut rng = rand::rng();

                    for _ in 0..ACCENT_PARTICLE_NUMBER {
                        commands.spawn((
                            Transform::from_translation(
                                Vec3::from(HexVector2d::from(tile_of_player_tower))
                                    .with_y(EFFECT_HOVER_HEIGHT)
                                    + rng.random::<Vec3>().normalize() * rng.random_range(0.1..1.2),
                            ),
                            AnimProgress {
                                progress: rng.random_range(0.0..0.1),
                                speed_multiplyer: SPEED,
                            },
                            WorldAssetRoot(
                                asset_server.load(
                                    GltfAssetLabel::Scene(0)
                                        .from_asset("particles_and_effects/green_plus.glb"),
                                ),
                            ),
                            AnimateScale(EasingCurve::new(
                                Vec3::ZERO,
                                Vec3::ONE,
                                EaseFunction::ExponentialIn,
                            )),
                        ));
                    }
                }
            }
        }
        ActionEffect::ReducedRemaingOrdersOfPlayer(player) => {
            let Some(owned_pieces) = logical_world
                .0
                .get::<OwnsPieces>(*logical_world.0.resource::<PlayerDirectory>().get(player))
            else {
                return;
            };
            let Some(&tile_of_player_tower) = owned_pieces.list().iter().find_map(|&piece| {
                if logical_world.0.get::<IsWinCondition>(piece).is_some()
                    && let Some(OccupiesTile(tile)) = logical_world.0.get::<OccupiesTile>(piece)
                {
                    logical_world.0.get::<TileId>(*tile)
                } else {
                    None
                }
            }) else {
                return;
            };

            commands.spawn((
                Transform::from_translation(
                    Vec3::from(HexVector2d::from(tile_of_player_tower)).with_y(EFFECT_HOVER_HEIGHT),
                ),
                AnimProgress {
                    progress: 0.0,
                    speed_multiplyer: SPEED,
                },
                WorldAssetRoot(asset_server.load(
                    GltfAssetLabel::Scene(0).from_asset("particles_and_effects/red_button.glb"),
                )),
                AnimateScale(EasingCurve::new(
                    Vec3::ZERO,
                    Vec3::ONE,
                    EaseFunction::ExponentialIn,
                )),
            ));

            let mut rng = rand::rng();

            for _ in 0..ACCENT_PARTICLE_NUMBER {
                commands.spawn((
                    Transform::from_translation(
                        Vec3::from(HexVector2d::from(tile_of_player_tower))
                            .with_y(EFFECT_HOVER_HEIGHT)
                            + rng.random::<Vec3>().normalize() * rng.random_range(0.1..1.2),
                    ),
                    AnimProgress {
                        progress: rng.random_range(0.0..0.1),
                        speed_multiplyer: SPEED,
                    },
                    WorldAssetRoot(asset_server.load(
                        GltfAssetLabel::Scene(0).from_asset("particles_and_effects/red_minus.glb"),
                    )),
                    AnimateScale(EasingCurve::new(
                        Vec3::ZERO,
                        Vec3::ONE,
                        EaseFunction::ExponentialIn,
                    )),
                ));
            }
        }
        _ => (),
    }
}
#[derive(Debug, Component)]
struct AnimatedProperty<A> {
    fxn: Box<[(EasingCurve<A>, f32)]>,
    prog: f32,
}

struct AnimatedPropertyInterval<A> {
    /// The next value the animation will reach.
    next_value: A,
    /// How fast it will approach this value after the previous value.
    speed_multiplyer: f32,
    /// How it will arrive at that value.
    mode: EaseFunction,
}

impl<A: Clone> AnimatedProperty<A> {
    fn new_seamless(
        start_value: A,
        animations: Box<[AnimatedPropertyInterval<A>]>,
        with_progress: f32,
    ) -> Self {
        let mut previous_value = start_value;

        let mut curves = Vec::new();

        for interval in animations {
            curves.push((
                EasingCurve::new(previous_value, interval.next_value.clone(), interval.mode),
                interval.speed_multiplyer,
            ));

            previous_value = interval.next_value;
        }

        Self {
            fxn: curves.into_boxed_slice(),
            prog: with_progress,
        }
    }
}

impl<A> AnimatedProperty<A>
where
    EasingCurve<A>: Curve<A>,
{
    fn get_current_value_and_speed(&self) -> Option<(A, f32)> {
        self.fxn
            .iter()
            .enumerate()
            .find_map(|(index, (curve, speed))| {
                curve
                    .sample(self.prog - index as f32 * 1.0)
                    .map(|new_value| (new_value, *speed))
            })
    }
}

impl<A: Component<Mutability = Mutable>> AnimatedProperty<A>
where
    EasingCurve<A>: Curve<A>,
{
    fn update(
        mut query: Query<(Entity, &mut AnimatedProperty<A>, &mut A)>,
        mut commands: Commands,
        time: Res<Time>,
    ) {
        query
            .iter_mut()
            .for_each(|(ent, mut animation, mut value)| {
                if let Some((new_value, speed_multiplyer)) = animation.get_current_value_and_speed()
                {
                    *value = new_value;
                    animation.prog +=
                        time.delta_secs() * GENERAL_SPPED_MULTIPLYER * speed_multiplyer;
                } else {
                    commands.entity(ent).despawn();
                }
            });
    }
}
