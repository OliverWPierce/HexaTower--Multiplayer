use bevy::ecs::query::With;

use crate::{
    forensic_action_descriptions::{ForensicDescribe, LinkedGamplayElement, TextSnippet},
    pieces::{
        self, GivesExtraPlayerOrder, Health, IsSpawnPoint, IsWinCondition, OccupiedByPiece,
        OccupiesTile, OrdersReceivable, PieceOwnedByPlayer,
    },
    players::{PlayerDirectory, PlayerId, PlayerState},
    requests::{ActionEffect, ChangeLog},
    tile_based_actions::{
        TileActionFunctionality, TileActionFunctionalityCapabilityConstants,
        selection_mechanics::SelectedTile,
    },
    tile_mapping::{
        HexVector2d, NORTH, NORTH_EAST, NORTH_WEST, SOUTH, SOUTH_EAST, SOUTH_WEST, TileId,
    },
    tiles::TileDirectory,
};
#[derive(Debug)]
pub struct SpawnPieces {
    pub archetype: crate::pieces::ArchetypeId,
    pub owner: PlayerId,
    pub restrictions: SpawningRestrictions,
}
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum SpawningRestrictions {
    Anywhere,
    StandardRestrictions,
}

impl TileActionFunctionalityCapabilityConstants for SpawnPieces {
    const ACCEPTABLE_SELECTION_COUNTS: std::ops::Range<usize> = 0..usize::MAX;
}
impl TileActionFunctionality for SpawnPieces {
    fn execute(
        &self,
        validated_selections: &[SelectedTile],
        world: &mut bevy::ecs::world::World,
    ) -> crate::requests::ChangeLog {
        let blueprint = world
            .resource::<pieces::ArchetypeDirectory>()
            .get_archetype(self.archetype)
            .unwrap()
            .clone();

        let player_entity = *world.resource::<PlayerDirectory>().get(self.owner);

        let tile_entities = world.resource::<TileDirectory>();

        let mut log = ChangeLog::default();

        for (selection, tile_entity) in validated_selections
            .iter()
            .map(|tile| (*tile, tile_entities.get_entity(tile.id)))
            .collect::<Box<[(SelectedTile, bevy::ecs::entity::Entity)]>>()
        {
            let mut piece = world.spawn((
                Health {
                    max: blueprint.max_health,
                    current: blueprint.max_health,
                },
                PieceOwnedByPlayer(player_entity),
                OccupiesTile(tile_entity),
                OrdersReceivable {
                    per_round: blueprint.starting_orders_per_round,
                    currently: 0,
                },
                blueprint.orders.clone(),
                selection.direction,
            ));

            if blueprint.gives_extra_player_order {
                piece.insert(GivesExtraPlayerOrder);
            }

            if blueprint.is_spawnpoint {
                piece.insert(IsSpawnPoint);
            }

            if blueprint.is_win_condition {
                piece.insert(IsWinCondition);
                *world.get_mut::<PlayerState>(player_entity).unwrap() = PlayerState::Alive;
            }

            log.write(ActionEffect::SpawnedPiece {
                tile: selection.id,
                player: self.owner,
                archetype: self.archetype,
                facing_direction: selection.direction,
            });
        }

        log
    }

    fn update_eligibility(
        &self,
        selection_status: &mut super::selection_mechanics::SelectionData,
        world: &bevy::ecs::world::World,
    ) {
        match self.restrictions {
            SpawningRestrictions::Anywhere => {
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
            SpawningRestrictions::StandardRestrictions => {
                // selection_status.set_all_possible_inelligible();

                // let mut spawnpoints = world
                //     .try_query_filtered::<(&OccupiesTile, &PieceOwnedByPlayer), With<IsSpawnPoint>>(
                //     )
                //     .unwrap();

                // let owner_of_peiece_being_spawned =
                //     world.resource::<PlayerDirectory>().get(self.owner);

                // let maximum_tile_id = world.resource::<TileDirectory>().tile_count() - 1;

                // let valid_tiles_by_proximity_to_friendly_spawn = spawnpoints
                //     .iter(world)
                //     .filter_map(|(tile, owner)| {
                //         if owner.0 == *owner_of_peiece_being_spawned {
                //             let tile_location = HexVector2d::from(
                //                 *world
                //                     .get::<TileId>(tile.0)
                //                     .expect("A piece must occupy a tile."),
                //             );

                //             Some(
                //                 [
                //                     tile_location + NORTH,
                //                     tile_location + NORTH_EAST,
                //                     tile_location + NORTH_WEST,
                //                     tile_location + SOUTH_EAST,
                //                     tile_location + SOUTH,
                //                     tile_location + SOUTH_WEST,
                //                 ]
                //                 .iter()
                //                 .filter_map(|vector| {
                //                     let id = TileId::from(*vector);
                //                     if id.id() as usize > maximum_tile_id {
                //                         None
                //                     } else {
                //                         Some(id)
                //                     }
                //                 })
                //                 .collect::<Vec<_>>(),
                //             )
                //         } else {
                //             None
                //         }
                //     })
                //     .flatten()
                //     .collect::<Vec<_>>();

                // let mut tiles_adjacent_to_enemy_spawn_points = spawnpoints
                //     .iter(world)
                //     .filter_map(|(tile, owner)| {
                //         if owner.0 != *owner_of_peiece_being_spawned {
                //             let tile_location = HexVector2d::from(
                //                 *world
                //                     .get::<TileId>(tile.0)
                //                     .expect("A piece must occupy a tile."),
                //             );

                //             Some(
                //                 [
                //                     tile_location + NORTH,
                //                     tile_location + NORTH_EAST,
                //                     tile_location + NORTH_WEST,
                //                     tile_location + SOUTH_EAST,
                //                     tile_location + SOUTH,
                //                     tile_location + SOUTH_WEST,
                //                 ]
                //                 .iter()
                //                 .filter_map(|vector| {
                //                     let id = TileId::from(*vector);
                //                     if id.id() as usize > maximum_tile_id {
                //                         None
                //                     } else {
                //                         Some(id)
                //                     }
                //                 })
                //                 .collect::<Vec<_>>(),
                //             )
                //         } else {
                //             None
                //         }
                //     })
                //     .flatten();

                // for tile in valid_tiles_by_proximity_to_friendly_spawn {
                //     if world
                //         .get::<OccupiedByPiece>(
                //             world.resource::<TileDirectory>().get_entity(tile).unwrap(),
                //         )
                //         .is_none()
                //         && tiles_adjacent_to_enemy_spawn_points
                //             .find(|inelligible_tile| *inelligible_tile == tile)
                //             .is_none()
                //     {
                //         _ = selection_status.try_set_state(tile, super::State::Elligible);
                //     }
                // }
                //
                todo!()
            }
        }
    }
}

impl ForensicDescribe for SpawnPieces {
    fn forensic_description(&self) -> Box<[crate::forensic_action_descriptions::TextSnippet]> {
        Box::new([
            TextSnippet::new_basic_text("Spawn "),
            TextSnippet::Link(LinkedGamplayElement::Piece(self.archetype)),
            TextSnippet::new_basic_text(" on selected tiles."),
            TextSnippet::Link(LinkedGamplayElement::Player(self.owner)),
            TextSnippet::new_basic_text(" will own and command this piece."),
        ])
    }
}
