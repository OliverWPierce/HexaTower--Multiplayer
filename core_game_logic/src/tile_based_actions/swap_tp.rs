use crate::{
    forensic_action_descriptions::{ForensicDescribe, TextSnippet},
    pieces::{OccupiedByPiece, OccupiesTile, PieceOwnedByPlayer},
    requests::{ActionEffect, ChangeLog},
    tile_based_actions::{TileActionFunctionality, TileActionFunctionalityCapabilityConstants},
    tile_mapping::TileId,
    tiles::TileDirectory,
};
#[derive(Debug)]
pub struct SwapPiecesWithFriendly {
    pub acting_piece_on_tile: TileId,
}

impl TileActionFunctionalityCapabilityConstants for SwapPiecesWithFriendly {
    const ACCEPTABLE_SELECTION_COUNTS: std::ops::Range<usize> = 1..1;
}

impl TileActionFunctionality for SwapPiecesWithFriendly {
    fn execute(
        &self,
        validated_selections: &[super::SelectedTile],
        world: &mut bevy::ecs::world::World,
    ) -> crate::requests::ChangeLog {
        let mut log = ChangeLog::default();

        let tiles = world.resource::<TileDirectory>();
        let tile1 = tiles.get_entity(self.acting_piece_on_tile);
        let tile2 = tiles.get_entity(validated_selections.first().unwrap().id);

        let p1 = world.get::<OccupiedByPiece>(tile1).unwrap().piece();
        let p2 = world.get::<OccupiedByPiece>(tile2).unwrap().piece();

        world.entity_mut(p1).remove::<OccupiesTile>();
        world.entity_mut(p2).insert(OccupiesTile(tile1));
        world.entity_mut(p1).insert(OccupiesTile(tile2));

        log.write(ActionEffect::PiecesSwapped(
            self.acting_piece_on_tile,
            validated_selections.first().unwrap().id,
        ));

        log
    }

    fn update_eligibility(
        &self,
        selection_status: &mut super::selection_mechanics::SelectionData,
        world: &bevy::ecs::world::World,
    ) {
        if let Some(piece) = world.get::<OccupiedByPiece>(
            world
                .resource::<TileDirectory>()
                .get_entity(self.acting_piece_on_tile),
        ) && let Some(PieceOwnedByPlayer(target_owner)) =
            world.get::<PieceOwnedByPlayer>(piece.piece())
        {
            for (tile, piece) in world
                .try_query::<(&TileId, &OccupiedByPiece)>()
                .unwrap()
                .iter(world)
            {
                if let Some(PieceOwnedByPlayer(owner)) =
                    world.get::<PieceOwnedByPlayer>(piece.piece())
                    && target_owner == owner
                {
                    selection_status.maybe_set_elligible(*tile);
                } else {
                    selection_status.maybe_set_inelligible(*tile);
                }
            }
            return;
        }
        selection_status.set_all_possible_inelligible();
    }
}

impl ForensicDescribe for SwapPiecesWithFriendly {
    fn forensic_description(&self) -> Box<[TextSnippet]> {
        Box::new([TextSnippet::new_basic_text(
            "Swap this piece's location with one of it's friends.",
        )])
    }
}
