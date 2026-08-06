/// Notes on this module.
/// 1. Systems should only reference the "effect to display" resource once: when starting the sequence. Other systems (ie. ones that move particles) should not reference that resource because it is liable to change frequently.
/// 2. Sequences should not reference visual world data that they do not create, because those things may be destroyed by other reactions. (ie. A particle should fly to a logical tile location, not to the location of a visual piece model, because a concurrent reaction could despawn the piece model.)
use bevy::prelude::*;

mod coin_flying;
mod item_purchased;
mod player_orders;

use crate::{AppState, inputs_interface::EffectToDisplay};

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
