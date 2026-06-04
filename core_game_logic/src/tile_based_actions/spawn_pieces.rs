use crate::{
    pieces::{
        self, FacingHexDirection, GetsFreeRotation, GivesExtraPlayerOrder, Health, IsWinCondition,
        OccupiedByPiece, OccupiesTile, OrdersReceivable, PieceOwnedByPlayer,
    },
    players::{PlayerDirectory, PlayerId, PlayerState},
    requests::{ActionEffect, ChangeLog},
    tile_based_actions::{TileActionFunctionality, TileActionFunctionalityCapabilityConstants},
    tile_mapping::TileId,
    tiles::TileDirectory,
};
#[derive(Debug)]
pub struct SpawnPieces {
    pub archetype: crate::pieces::ArchetypeId,
    pub owner: PlayerId,
}

impl TileActionFunctionalityCapabilityConstants for SpawnPieces {
    const ACCEPTABLE_SELECTION_COUNTS: std::ops::Range<usize> = 0..usize::MAX;
}
impl TileActionFunctionality for SpawnPieces {
    fn execute(
        &self,
        validated_selections: &[crate::tile_mapping::TileId],
        world: &mut bevy::ecs::world::World,
    ) -> crate::requests::ChangeLog {
        let blueprint = world
            .resource::<pieces::ArchetypeDirectory>()
            .get_archetype(self.archetype)
            .unwrap()
            .clone();

        let player_entity = world
            .resource::<PlayerDirectory>()
            .get_player(self.owner)
            .unwrap();

        let tile_entities = world.resource::<TileDirectory>();

        let mut log = ChangeLog::default();

        for (id, tile) in validated_selections
            .iter()
            .map(|id| (*id, tile_entities.get_entity(*id).unwrap()))
            .collect::<Box<[(TileId, bevy::ecs::entity::Entity)]>>()
        {
            let mut piece = world.spawn((
                Health {
                    max: blueprint.max_health,
                    current: blueprint.max_health,
                },
                PieceOwnedByPlayer(player_entity),
                OccupiesTile(tile),
                OrdersReceivable {
                    per_round: blueprint.starting_orders_per_round,
                    currently: 0,
                },
                blueprint.orders.clone(),
                FacingHexDirection::default(),
                GetsFreeRotation,
            ));

            if blueprint.gives_extra_player_order {
                piece.insert(GivesExtraPlayerOrder);
            }

            if blueprint.is_win_condition {
                piece.insert(IsWinCondition);
                *world.get_mut::<PlayerState>(player_entity).unwrap() = PlayerState::Alive;
            }

            log.write(ActionEffect::SpawnedPiece {
                tile: id,
                player: self.owner,
                archetype: self.archetype,
                facing_direction: FacingHexDirection::default(),
            });

            log.write(ActionEffect::GaveFreeRotationComponent {
                to_piece_on_tile: id,
            });
        }

        log
    }

    fn update_eligibility(
        &self,
        selection_status: &mut super::selection_mechanics::SelectionData,
        world: &bevy::ecs::world::World,
    ) {
        for (id, ent) in world.resource::<TileDirectory>().id_entity_pairs() {
            if world.get::<OccupiedByPiece>(ent).is_some() {
                selection_status
                    .try_set_state(id, super::selection_mechanics::State::Neither)
                    .unwrap();
            } else {
                selection_status
                    .try_set_state(id, super::selection_mechanics::State::Elligible)
                    .unwrap();
            }

            // unwrap is fine, since we're not trying to select anything and we already know the tile ids are within the game's bounds.
        }
    }
}
