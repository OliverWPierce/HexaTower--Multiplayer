use bevy::ecs::{component::Component, entity::Entity, resource::Resource, world::World};
use thiserror::Error;

use crate::{
    orders::OrderId,
    players::{Coins, PlayerId},
    requests::{ActionEffect, ChangeLog},
    tile_mapping::TileId,
};

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
    pub gives_extra_player_order: bool,
    pub default_monetary_value: u32,
    pub is_win_condition: bool,
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
pub struct OwnsPieces(Vec<Entity>);

impl OwnsPieces {
    pub fn list(&self) -> &Vec<Entity> {
        &self.0
    }
}

#[derive(Component)]
#[relationship(relationship_target = OwnsPieces)]
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
#[derive(Debug, Component)]
#[component(storage = "SparseSet")]
pub struct GivesExtraPlayerOrder;

#[derive(Debug, Component)]
pub struct MonetaryValue(u32);

pub fn kill_piece(
    piece: Entity,
    killing_player: Option<Entity>,
    changelog: &mut ChangeLog,
    world: &mut World,
) {
    let tile_piece_was_on = *world
        .get::<TileId>(world.get::<OccupiesTile>(piece).unwrap().0)
        .unwrap();

    changelog.write(ActionEffect::PieceKilled {
        on_tile: tile_piece_was_on,
    });

    if let Some(player) = killing_player {
        let coins_to_give_killer = world.get::<MonetaryValue>(piece).unwrap().0;
        world.get_mut::<Coins>(player).unwrap().0 += coins_to_give_killer;
        changelog.write(ActionEffect::AlteredCoins {
            player: *world.get::<PlayerId>(player).unwrap(),
            delta_coins: coins_to_give_killer as i32,
            from_tile: Some(tile_piece_was_on),
        });
    }
}

pub enum AlterHealthMethod {
    Constant(i32),
    FractionOfMissing(f32),
    FractionOfMax(f32),
}

pub fn get_delta_health(health: Health, method: AlterHealthMethod) -> i32 {
    let base_damage = match method {
        AlterHealthMethod::Constant(damage) => damage as f32,
        AlterHealthMethod::FractionOfMissing(fraction) => {
            (health.max.saturating_sub(health.current)) as f32 * fraction
        }
        AlterHealthMethod::FractionOfMax(fraction) => health.max as f32 * fraction,
    };

    base_damage as i32
}
#[derive(Debug, Component)]
pub struct IsWinCondition;
