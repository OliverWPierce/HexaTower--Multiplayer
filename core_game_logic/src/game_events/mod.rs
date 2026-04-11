use bevy::ecs::world::World;

pub trait GameEvent {
    fn validate(&self, world: &World) -> Result<(), ()>;

    fn consume(self, world: &mut World) -> Result<(), ()>;
}
