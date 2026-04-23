use bevy::ecs::world::World;

pub trait BoardEvent {
    fn validate(&self, world: &World) -> anyhow::Result<()>;

    fn consume(self, world: &mut World) -> anyhow::Result<()>;
}

pub mod modify_tile_type;
