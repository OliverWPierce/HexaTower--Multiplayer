use bevy::ecs::{component::Component, entity::Entity, resource::Resource, world::World};
use thiserror::Error;

#[derive(Debug, Component)]
pub struct PlayerId(pub u8);

#[derive(Debug, Resource)]
pub struct PlayerDirectory(Box<[Entity]>);

#[derive(Debug, Error)]
#[error("Tried to get a player with an invalid id: {:?}", self.0.0)]
pub struct InvalidIdErr(PlayerId);

impl PlayerDirectory {
    fn get_player(&self, id: PlayerId) -> Result<Entity, InvalidIdErr> {
        self.0.get(id.0 as usize).copied().ok_or(InvalidIdErr(id))
    }
}
#[derive(Debug, Component)]
struct Inventory(i8);

fn initialize_players(world: &mut World, player_count: u8) {
    let players = world
        .spawn_batch((0..player_count).map(|id| (PlayerId(id), Inventory(2))))
        .collect::<Vec<Entity>>()
        .into_boxed_slice();
    world.insert_resource(PlayerDirectory(players));
}
