use bevy::ecs::{component::Component, entity::Entity, resource::Resource};
use thiserror::Error;

#[derive(Debug, Resource)]
pub struct ArchetypeDirectory(Vec<LogicalPieceArchetype>);

#[derive(Debug, Clone)]
pub struct LogicalPieceArchetype {
    pub max_health: u32,
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
    pub fn log_piece(&self) -> Entity {
        self.0
    }
}

#[derive(Component)]
#[relationship_target(relationship = LogPieceOwnedByPlayer, linked_spawn)]
pub struct OwnsLogPieces(Vec<Entity>);

impl OwnsLogPieces {
    pub fn list(&self) -> &Vec<Entity> {
        &self.0
    }
}

#[derive(Component)]
#[relationship(relationship_target = OwnsLogPieces)]
pub struct LogPieceOwnedByPlayer(pub Entity);

#[derive(Component)]
pub struct Health {
    pub max: u32,
    pub current: u32,
}
