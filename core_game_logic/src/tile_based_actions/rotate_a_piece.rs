use crate::{
    forensic_action_descriptions::ForensicDescribe,
    pieces::{FacingHexDirection, OccupiedByPiece},
    requests::{ActionEffect, ChangeLog},
    tile_based_actions::{TileActionFunctionality, TileActionFunctionalityCapabilityConstants},
    tile_mapping::TileId,
    tiles::TileDirectory,
};

#[derive(Debug, Clone, Copy)]
pub struct RotatePiece {
    pub piece_on_tile: TileId,
}

impl TileActionFunctionalityCapabilityConstants for RotatePiece {
    const ACCEPTABLE_SELECTION_COUNTS: std::ops::Range<usize> = 1..1;
}

impl TileActionFunctionality for RotatePiece {
    fn execute(
        &self,
        validated_selections: &[super::SelectedTile],
        world: &mut bevy::ecs::world::World,
    ) -> crate::requests::ChangeLog {
        let piece = world
            .get::<OccupiedByPiece>(
                world
                    .resource::<TileDirectory>()
                    .get_entity(validated_selections.first().unwrap().id),
            )
            .unwrap()
            .piece();

        *world.get_mut::<FacingHexDirection>(piece).unwrap() =
            validated_selections.first().unwrap().direction;

        let mut log = ChangeLog::default();
        log.write(ActionEffect::PieceRotated {
            on_tile: validated_selections.first().unwrap().id,
            new_rotation: validated_selections.first().unwrap().direction,
        });

        log
    }

    fn update_eligibility(
        &self,
        selection_status: &mut super::selection_mechanics::SelectionData,
        world: &bevy::ecs::world::World,
    ) {
        if world
            .get::<OccupiedByPiece>(
                world
                    .resource::<TileDirectory>()
                    .get_entity(self.piece_on_tile),
            )
            .is_some()
        {
            selection_status.maybe_set_elligible(self.piece_on_tile);
        }
    }
}

impl ForensicDescribe for RotatePiece {
    fn forensic_description(&self) -> Box<[crate::forensic_action_descriptions::TextSnippet]> {
        Box::new([
            crate::forensic_action_descriptions::TextSnippet::new_basic_text(
                "Rotate selected pieces.",
            ),
        ])
    }
}
