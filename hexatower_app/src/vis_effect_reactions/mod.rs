/// Notes on this module.
/// 1. Systems should only reference the "effect to display" resource once: when starting the sequence. Other systems (ie. ones that move particles) should not reference that resource because it is liable to change frequently.
/// 2. Sequences should not reference visual world data that they do not create, because those things may be destroyed by other reactions. (ie. A particle should fly to a logical tile location, not to the location of a visual piece model, because a concurrent reaction could despawn the piece model.)
use bevy::prelude::*;

mod coin_flying;

use crate::{AppState, vis_effect_reactions::coin_flying::CoinFlying};

pub struct VisEffectReactions;

impl Plugin for VisEffectReactions {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (animate_translation, animate_scale).run_if(in_state(AppState::InGame)),
        );

        app.add_plugins(CoinFlying);
    }
}

pub const EFFECT_HOVER_HEIGHT: f32 = 1.3;
const GENERAL_SPPED_MULTIPLYER: f32 = 1.0;

#[derive(Debug, Component)]
pub struct AnimatedProperty<A> {
    pub fxn: Box<[(EasingCurve<A>, f32)]>,
    pub prog: f32,
}

pub struct AnimatedPropertyInterval<A> {
    /// The next value the animation will reach.
    pub next_value: A,
    /// How fast it will approach this value after the previous value.
    pub speed_multiplyer: f32,
    /// How it will arrive at that value.
    pub mode: EaseFunction,
}

impl<A: Clone> AnimatedProperty<A> {
    pub fn new_seamless(
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
            if let Some((new_translation, anim_speed_multiplyer)) =
                animation.0.get_current_value_and_speed()
            {
                transform.translation = new_translation;
                animation.0.prog +=
                    anim_speed_multiplyer * GENERAL_SPPED_MULTIPLYER * time.delta_secs();
            }
            commands.entity(entity).remove::<AnimatedTranslation>();
        });
}

/// Note: when this animation is finished, the entity will despawn.
#[derive(Debug, Component)]
pub struct AnimatedScale(pub AnimatedProperty<Vec3>);

fn animate_scale(
    mut transforms: Query<(Entity, &mut Transform, &mut AnimatedScale)>,
    mut commands: Commands,
    time: Res<Time>,
) {
    transforms
        .iter_mut()
        .for_each(|(entity, mut transform, mut animation)| {
            if let Some((new_scale, anim_speed_multiplyer)) =
                animation.0.get_current_value_and_speed()
            {
                transform.scale = new_scale;
                animation.0.prog +=
                    anim_speed_multiplyer * GENERAL_SPPED_MULTIPLYER * time.delta_secs();
            }
            commands.entity(entity).despawn();
        });
}
