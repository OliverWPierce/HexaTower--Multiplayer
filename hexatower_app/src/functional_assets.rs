use bevy::{ecs::schedule::ScheduleLabel, prelude::*};

struct StartupPlugin;

impl Plugin for StartupPlugin {
    fn build(&self, app: &mut App) {
        todo!()
    }
}
#[derive(Debug, ScheduleLabel, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct SetUpBoard;
