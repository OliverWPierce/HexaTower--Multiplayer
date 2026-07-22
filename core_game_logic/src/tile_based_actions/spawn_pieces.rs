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
    tile_mapping::{HexVector2d, TileId, TileIdServer},
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
    TowerSpawns,
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
                        selection_status.maybe_set_inelligible(id)
                    } else {
                        selection_status.maybe_set_elligible(id)
                    }

                    // unwrap is fine, since we're not trying to select anything and we already know the tile ids are within the game's bounds.
                }
            }
            SpawningRestrictions::StandardRestrictions => {
                selection_status.set_all_possible_inelligible();

                let friendly_player = *world.resource::<PlayerDirectory>().get(self.owner);
                let tile_server = world.resource::<TileIdServer>();

                let mut tiles_adjacent_to_friendly_spawnpoints = Vec::new();
                let mut tiles_adjacent_to_enemy_spawnpoints = Vec::new();

                for (spawnpoint, owner) in world
                    .try_query_filtered::<(&OccupiesTile, &PieceOwnedByPlayer), With<IsSpawnPoint>>(
                    )
                    .unwrap()
                    .iter(world)
                {
                    if owner.0 == friendly_player {
                        for tile in HexVector2d::from(*world.get::<TileId>(spawnpoint.0).unwrap())
                            .adjacencies()
                            .iter()
                            .filter_map(|vector| vector.to_valid_tile_id(tile_server))
                        {
                            tiles_adjacent_to_friendly_spawnpoints.push(tile);
                        }
                    } else {
                        for tile in HexVector2d::from(*world.get::<TileId>(spawnpoint.0).unwrap())
                            .adjacencies()
                            .iter()
                            .filter_map(|vector| vector.to_valid_tile_id(tile_server))
                        {
                            tiles_adjacent_to_enemy_spawnpoints.push(tile);
                        }
                    }
                }

                for tile in tiles_adjacent_to_friendly_spawnpoints {
                    if world
                        .get::<OccupiedByPiece>(world.resource::<TileDirectory>().get_entity(tile))
                        .is_some()
                    {
                        continue;
                    }

                    selection_status.maybe_set_elligible(tile);
                }

                for tile in tiles_adjacent_to_enemy_spawnpoints {
                    selection_status.maybe_set_inelligible(tile);
                }
            }
            SpawningRestrictions::TowerSpawns => {
                selection_status.set_all_possible_elligible();

                let friendly_player = *world.resource::<PlayerDirectory>().get(self.owner);
                let tile_server = world.resource::<TileIdServer>();

                for (tile, piece) in world
                    .try_query::<(&TileId, &OccupiedByPiece)>()
                    .unwrap()
                    .iter(world)
                {
                    selection_status.maybe_set_inelligible(*tile);
                    if world.get::<IsSpawnPoint>(piece.piece()).is_some()
                        && let Some(PieceOwnedByPlayer(owner)) =
                            world.get::<PieceOwnedByPlayer>(piece.piece())
                        && *owner != friendly_player
                    {
                        let vectors_adjacent_to_enemy_spawnpoints =
                            HexVector2d::from(*tile).adjacencies();

                        for vector in vectors_adjacent_to_enemy_spawnpoints {
                            if let Some(tile) = vector.to_valid_tile_id(tile_server) {
                                selection_status.maybe_set_inelligible(tile);
                            }
                        }
                    }
                }
            }
        }
    }
}

impl ForensicDescribe for SpawnPieces {
    fn forensic_description(&self) -> Box<[crate::forensic_action_descriptions::TextSnippet]> {
        let mut text_fragments = vec![
            TextSnippet::new_basic_text("Spawn "),
            TextSnippet::Link(LinkedGamplayElement::Piece(self.archetype)),
            TextSnippet::new_basic_text(" on selected tiles."),
            TextSnippet::Link(LinkedGamplayElement::Player(self.owner)),
            TextSnippet::new_basic_text(" will own and command this piece."),
        ];

        match self.restrictions {
            SpawningRestrictions::Anywhere => (),
            SpawningRestrictions::StandardRestrictions => text_fragments.push(TextSnippet::new_basic_text(
                "Piece must be spawned next to a friendly spawnpoint and away from an enemy spawnpoint.",
            )),
            SpawningRestrictions::TowerSpawns => text_fragments.push(TextSnippet::new_basic_text(
                "Piece can be spawned anywhere, except next to enemy spawnpoints.",
            )),
        }

        text_fragments.into_boxed_slice()
    }
}
