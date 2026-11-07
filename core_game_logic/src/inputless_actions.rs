use std::fmt::Debug;

use bevy::ecs::world::World;

use crate::{forensic_action_descriptions::ForensicDescribe, requests::ChangeLog};

pub trait InputlessAction: ForensicDescribe + Debug + Send + Sync {
    fn execute(&self, world: &mut World) -> ChangeLog;

    ///Keep the output string short. It must display on the button.
    fn execution_blocker(&self, world: &World) -> Option<String>;
}
