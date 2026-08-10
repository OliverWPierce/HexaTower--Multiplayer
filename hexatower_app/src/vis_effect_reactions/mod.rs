use std::ops::Range;

/// Notes on this module.
/// 1. Systems should only reference the "effect to display" resource once: when starting the sequence. Other systems (ie. ones that move particles) should not reference that resource because it is liable to change frequently.
/// 2. Sequences should not reference visual world data that they do not create, because those things may be destroyed by other reactions. (ie. A particle should fly to a logical tile location, not to the location of a visual piece model, because a concurrent reaction could despawn the piece model.)
use bevy::{ecs::bundle::NoBundleEffect, prelude::*};
use core_game_logic::{
    pieces::{IsWinCondition, OccupiesTile, OwnsPieces},
    players::{PlayerDirectory, PlayerId},
    tile_mapping::TileId,
};
use rand::rngs::ThreadRng;

mod coin_flying;
mod item_purchased;
mod piece_orders;
mod player_orders;

use crate::{AppState, functional_assets::LogicalWorld, inputs_interface::EffectToDisplay};

pub struct VisEffectReactions;

impl Plugin for VisEffectReactions {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (animate_translation, animate_scale).run_if(in_state(AppState::InGame)),
        );
        app.add_systems(
            Update,
            (
                coin_flying::spawn_coins,
                player_orders::player_order_change,
                item_purchased::purchase_item,
                piece_orders::piece_orders,
            )
                .run_if(resource_exists_and_changed::<EffectToDisplay>),
        );
    }
}

pub const EFFECT_HOVER_HEIGHT: f32 = 1.3;
const GENERAL_SPPED_MULTIPLYER: f32 = 1.0;

#[derive(Debug, Component)]
pub struct AnimatedProperty<A> {
    pub fxn: Vec<(EasingCurve<A>, f32)>,
    pub elapsed_in_segment: f32,
}

pub struct AnimatedPropertyInterval<A> {
    /// The next value the animation will reach.
    pub next_value: A,
    /// How fast it will approach this value after the previous value.
    pub duration: f32,
    /// How it will arrive at that value.
    pub mode: EaseFunction,
}

impl<A: Clone> AnimatedProperty<A> {
    pub fn new_seamless(
        start_value: A,
        animations: Box<[AnimatedPropertyInterval<A>]>,
        with_elapsed: f32,
    ) -> Self {
        let mut previous_value = start_value;

        let mut curves = Vec::new();

        for interval in animations {
            curves.push((
                EasingCurve::new(previous_value, interval.next_value.clone(), interval.mode),
                interval.duration,
            ));

            previous_value = interval.next_value;
        }

        curves.reverse();

        Self {
            fxn: curves,
            elapsed_in_segment: with_elapsed,
        }
    }
}

impl<A> AnimatedProperty<A>
where
    EasingCurve<A>: Curve<A>,
{
    fn current_val(&mut self) -> Option<A> {
        while let Some((curve, duration)) = self.fxn.last() {
            if let Some(val) = curve.sample(self.elapsed_in_segment / *duration) {
                return Some(val);
            }

            self.elapsed_in_segment = (self.elapsed_in_segment - duration).max(0.0);
            self.fxn.pop();
        }

        None
    }
}

#[derive(Debug, Component)]
pub struct AnimatedTranslation(pub AnimatedProperty<Vec3>);

fn animate_translation(
    mut transforms: Query<(Entity, &mut Transform, &mut AnimatedTranslation)>,
    mut commands: Commands,
    time: Res<Time>,
) {
    transforms
        .iter_mut()
        .for_each(|(entity, mut transform, mut animation)| {
            if let Some(new_translation) = animation.0.current_val() {
                transform.translation = new_translation;
                animation.0.elapsed_in_segment += GENERAL_SPPED_MULTIPLYER * time.delta_secs();
            } else {
                commands.entity(entity).try_remove::<AnimatedTranslation>();
            }
        });
}

/// Note: if the bool is true, when this animation is finished, the entity will despawn.
#[derive(Debug, Component)]
pub struct AnimatedScale(pub AnimatedProperty<Vec3>, pub bool);

fn animate_scale(
    mut transforms: Query<(Entity, &mut Transform, &mut AnimatedScale)>,
    mut commands: Commands,
    time: Res<Time>,
) {
    transforms
        .iter_mut()
        .for_each(|(entity, mut transform, mut animation)| {
            if let Some(new_scale) = animation.0.current_val() {
                transform.scale = new_scale;
                animation.0.elapsed_in_segment += GENERAL_SPPED_MULTIPLYER * time.delta_secs();
            } else if animation.1 {
                commands.entity(entity).try_despawn();
            } else {
                commands.entity(entity).try_remove::<AnimatedScale>();
            }
        });
}

pub fn spawn_pluse(
    commands: &mut Commands,
    location: Vec3,
    quantity: u8,
    delay_between: f32,
    mesh: Handle<WorldAsset>,
    duration_of_individual_pulse: f32,
) {
    for delay_mult in 0..quantity {
        commands.spawn((
            Transform::from_translation(location),
            AnimatedScale(
                AnimatedProperty::new_seamless(
                    Vec3::ZERO,
                    [
                        AnimatedPropertyInterval {
                            next_value: Vec3::ZERO,
                            duration: delay_between * delay_mult as f32,
                            mode: EaseFunction::Linear,
                        },
                        AnimatedPropertyInterval {
                            next_value: Vec3::ONE,
                            duration: duration_of_individual_pulse,
                            mode: EaseFunction::QuinticIn,
                        },
                    ]
                    .into(),
                    0.0,
                ),
                true,
            ),
            WorldAssetRoot(mesh.clone()),
        ));
    }
}
pub fn tower_of_player(logical_world: &LogicalWorld, player: PlayerId) -> Option<TileId> {
    let owned_pieces = logical_world
        .0
        .get::<OwnsPieces>(*logical_world.0.resource::<PlayerDirectory>().get(player))?;
    owned_pieces
        .list()
        .iter()
        .find_map(|&piece| {
            if logical_world.0.get::<IsWinCondition>(piece).is_some()
                && let Some(OccupiesTile(tile)) = logical_world.0.get::<OccupiesTile>(piece)
            {
                logical_world.0.get::<TileId>(*tile)
            } else {
                None
            }
        })
        .copied()
}

use std::f32::consts::TAU;

use bevy::{
    ecs::bundle::Bundle,
    math::{Vec3, curve::EaseFunction},
    transform::components::Transform,
};
use rand::RngExt;

pub struct SpawnInPlaceThenFly;

impl SpawnInPlaceThenFly {
    pub fn anim_bundle(
        from: bevy::math::Vec3,
        to: bevy::math::Vec3,
        pos_offset: std::ops::Range<f32>,
        scale_range: std::ops::Range<f32>,
        max_time_offset: f32,
        rng: &mut rand::prelude::ThreadRng,
    ) -> impl Bundle<Effect: bevy::ecs::bundle::NoBundleEffect> + use<> {
        const OFFSET_SHRINK: f32 = 0.5;
        const SCALE_IN_DUR: f32 = 0.3;
        const AWAIT_DUR: f32 = 0.3;
        const FLIGHT_TIME_MULTIPLYER: f32 = 0.2;

        let pos_offset = if !pos_offset.is_empty() {
            Vec3::X.rotate_y(rng.random_range(0.0..TAU)) * rng.random_range(pos_offset)
        } else {
            Vec3::ZERO
        };
        let flight_time = FLIGHT_TIME_MULTIPLYER * from.distance(to);
        let biggest_scale = if !scale_range.is_empty() {
            Vec3::splat(rng.random_range(scale_range))
        } else {
            Vec3::ONE
        };

        (
            Transform::from_translation(from + pos_offset).with_scale(Vec3::ZERO),
            AnimatedTranslation(AnimatedProperty::new_seamless(
                from + pos_offset,
                [
                    // hold the current positon.
                    AnimatedPropertyInterval {
                        next_value: from + pos_offset,
                        duration: SCALE_IN_DUR + AWAIT_DUR + rng.random_range(0.0..max_time_offset),
                        mode: EaseFunction::Linear,
                    },
                    // fly to the target
                    AnimatedPropertyInterval {
                        next_value: to + pos_offset * OFFSET_SHRINK,
                        duration: flight_time,
                        mode: EaseFunction::SmoothStep,
                    },
                ]
                .into(),
                0.0,
            )),
            AnimatedScale(
                AnimatedProperty::new_seamless(
                    Vec3::ZERO,
                    [
                        // scale in
                        AnimatedPropertyInterval {
                            next_value: biggest_scale,
                            duration: SCALE_IN_DUR + rng.random_range(0.0..max_time_offset),
                            mode: EaseFunction::BackOut,
                        },
                        // wait until after it's arrived at target and waited.
                        AnimatedPropertyInterval {
                            next_value: biggest_scale,
                            duration: flight_time
                                + AWAIT_DUR
                                + rng.random_range(0.0..max_time_offset),
                            mode: EaseFunction::Linear,
                        },
                        // scale out
                        AnimatedPropertyInterval {
                            next_value: Vec3::ZERO,
                            duration: SCALE_IN_DUR + rng.random_range(0.0..max_time_offset),
                            mode: EaseFunction::BackIn,
                        },
                    ]
                    .into(),
                    0.0,
                ),
                true,
            ),
        )
    }
}
