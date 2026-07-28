use crate::{
    forensic_action_descriptions::ForensicDescribe,
    pieces::{FacingHexDirection, OccupiedByPiece, OccupiesTile},
    requests::{ActionEffect, ChangeLog},
    tile_based_actions::{TileActionFunctionality, TileActionFunctionalityCapabilityConstants},
    tile_mapping::{HexVector2d, TileId, TileIdServer},
    tiles::TileDirectory,
};

#[derive(Debug)]
pub struct MovePiece {
    pub piece_on_tile: TileId,
    pub method: MovementMethod,
}

#[derive(Debug, Clone, Copy)]
pub enum MovementMethod {
    Adjacent { depth: u8 },
    Forward { depth: u8 },
}

impl TileActionFunctionalityCapabilityConstants for MovePiece {
    const ACCEPTABLE_SELECTION_COUNTS: std::ops::Range<usize> = 1..1;
}

impl TileActionFunctionality for MovePiece {
    fn execute(
        &self,
        validated_selections: &[super::SelectedTile],
        world: &mut bevy::ecs::world::World,
    ) -> crate::requests::ChangeLog {
        let tile_directory = world.resource::<TileDirectory>();
        let current_tile = tile_directory.get_entity(self.piece_on_tile);
        let new_tile = tile_directory.get_entity(validated_selections.first().unwrap().id);

        let piece = world.get::<OccupiedByPiece>(current_tile).unwrap().piece();

        world.entity_mut(piece).insert(OccupiesTile(new_tile));

        let mut log = ChangeLog::default();

        log.write(ActionEffect::PieceMoved {
            from_tile: self.piece_on_tile,
            to_tile: validated_selections.first().unwrap().id,
        });

        log
    }

    fn update_eligibility(
        &self,
        selection_status: &mut super::selection_mechanics::SelectionData,
        world: &bevy::ecs::world::World,
    ) {
        let tile_directory = world.resource::<TileDirectory>();
        let candidate_tiles = match self.method {
            MovementMethod::Adjacent { depth } => HexVector2d::from(self.piece_on_tile)
                .adjacent_ids_with_depth(depth, world.resource::<TileIdServer>()),
            MovementMethod::Forward { depth } => {
                let forward_direction = (*world
                    .get::<FacingHexDirection>(
                        world
                            .get::<OccupiedByPiece>(tile_directory.get_entity(self.piece_on_tile))
                            .unwrap()
                            .piece(),
                    )
                    .expect("All pieces should face a direction."))
                .into();
                let mut vectors = vec![HexVector2d::from(self.piece_on_tile) + forward_direction];

                for _ in 1..depth {
                    vectors.push(*vectors.last().unwrap() + forward_direction)
                }

                vectors
                    .iter()
                    .filter_map(|vector| vector.to_valid_tile_id(world.resource::<TileIdServer>()))
                    .collect::<_>()
            }
        };

        selection_status.set_all_possible_inelligible();
        for tile in candidate_tiles {
            if world
                .get::<OccupiedByPiece>(tile_directory.get_entity(tile))
                .is_none()
            {
                selection_status.maybe_set_elligible(tile);
            }
        }
    }
}

impl ForensicDescribe for MovePiece {
    fn forensic_description(&self) -> Box<[crate::forensic_action_descriptions::TextSnippet]> {
        Box::new([
            crate::forensic_action_descriptions::TextSnippet::new_basic_text(
                "No description implemented yet.",
            ),
        ])
    }
}
