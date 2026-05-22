use bevy::ecs::{component::Component, entity::Entity, resource::Resource, world::World};
use thiserror::Error;

use crate::orders::OrderId;

#[derive(Debug, Resource)]
pub struct ArchetypeDirectory(Box<[LogicalPieceArchetype]>);

pub fn initialize_pieces(world: &mut World, piece_archetypes: Box<[LogicalPieceArchetype]>) {
    world.insert_resource(ArchetypeDirectory(piece_archetypes));
}

#[derive(Debug, Clone)]
pub struct LogicalPieceArchetype {
    pub max_health: u32,
    pub starting_orders_per_round: u8,
    pub orders: Orders,
}

#[derive(Debug, Component, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct ArchetypeId(pub u32);

#[derive(Debug, Error)]
#[error{"Tried to get a piece archetype which did not exist for this game {0:?}."}]
pub struct InvaildIDErr(pub ArchetypeId);

impl ArchetypeDirectory {
    pub fn get_archetype(
        &self,
        archetype: ArchetypeId,
    ) -> Result<&LogicalPieceArchetype, InvaildIDErr> {
        self.0
            .get(archetype.0 as usize)
            .ok_or(InvaildIDErr(archetype))
    }
}

#[derive(Clone, Component, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[relationship(relationship_target = OccupiedByPiece)]
pub struct OccupiesTile(pub Entity);
#[derive(Clone, Component, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[relationship_target(relationship = OccupiesTile)]
pub struct OccupiedByPiece(Entity);

impl OccupiedByPiece {
    pub fn piece(&self) -> Entity {
        self.0
    }
}

#[derive(Component)]
#[relationship_target(relationship = PieceOwnedByPlayer, linked_spawn)]
pub struct OwnsLogPieces(Vec<Entity>);

impl OwnsLogPieces {
    pub fn list(&self) -> &Vec<Entity> {
        &self.0
    }
}

#[derive(Component)]
#[relationship(relationship_target = OwnsLogPieces)]
pub struct PieceOwnedByPlayer(pub Entity);

#[derive(Component)]
pub struct Health {
    pub max: u32,
    pub current: u32,
}

#[derive(Debug, Component, Clone)]
pub struct Orders(pub [Option<OrderId>; 5]);

#[derive(Debug, Component)]
pub struct OrdersReceivable {
    pub per_round: u8,
    pub currently: u8,
}
