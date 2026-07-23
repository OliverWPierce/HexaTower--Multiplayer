use crate::{
    forensic_action_descriptions::{ForensicDescribe, TextSnippet},
    pieces::{OccupiedByPiece, damage_and_maybe_kill_piece},
    players::ActivePlayer,
    requests::ChangeLog,
    tile_based_actions::{
        SelectedTile, TileActionFunctionality, TileActionFunctionalityCapabilityConstants,
    },
    tile_mapping::{HexVector2d, TileId, TileIdServer},
    tiles::TileDirectory,
};
#[derive(Debug)]
pub struct PieceAttacksAdjacent {
    pub attacker_occupies_tile: TileId,
    pub damage: u32,
    pub adjacency_depth: u8,
}

impl TileActionFunctionalityCapabilityConstants for PieceAttacksAdjacent {
    const ACCEPTABLE_SELECTION_COUNTS: std::ops::Range<usize> = 1..usize::MAX;
}

impl TileActionFunctionality for PieceAttacksAdjacent {
    fn execute(
        &self,
        validated_selections: &[super::SelectedTile],
        world: &mut bevy::ecs::world::World,
    ) -> crate::requests::ChangeLog {
        let mut log = ChangeLog::default();
        let attacking_player = world.resource::<ActivePlayer>().0;

        for SelectedTile { id: tile, .. } in validated_selections.iter() {
            damage_and_maybe_kill_piece(
                self.damage,
                *tile,
                Some(attacking_player),
                &mut log,
                world,
            );
        }

        log
    }

    fn update_eligibility(
        &self,
        selection_status: &mut super::selection_mechanics::SelectionData,
        world: &bevy::ecs::world::World,
    ) {
        selection_status.set_all_possible_inelligible();
        let tile_directory = world.resource::<TileDirectory>();

        for tile in HexVector2d::from(self.attacker_occupies_tile)
            .adjacent_ids_with_depth(self.adjacency_depth, world.resource::<TileIdServer>())
        {
            if world
                .get::<OccupiedByPiece>(tile_directory.get_entity(tile))
                .is_some()
            {
                selection_status.maybe_set_elligible(tile);
            }
        }
    }
}

impl ForensicDescribe for PieceAttacksAdjacent {
    fn forensic_description(&self) -> Box<[crate::forensic_action_descriptions::TextSnippet]> {
        Box::new([
            TextSnippet::new_basic_text("Deal "),
            TextSnippet::PlainText {
                text: format!("{} damage", self.damage),
                color: Some(crate::forensic_action_descriptions::ColorIndicators::Damage),
            },
            TextSnippet::new_basic_text(" to pieces on selected tiles within a "),
            TextSnippet::PlainText {
                text: format!("{} ring", self.damage),
                color: Some(crate::forensic_action_descriptions::ColorIndicators::GeneralHighlight),
            },
            TextSnippet::new_basic_text(" radius"),
        ])
    }
}
