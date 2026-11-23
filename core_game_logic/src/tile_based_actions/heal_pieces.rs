use crate::{
    forensic_action_descriptions::{ForensicDescribe, TextSnippet},
    pieces::{self, OccupiedByPiece},
    requests::ChangeLog,
    tile_based_actions::{
        SelectedTile, TileActionFunctionality, TileActionFunctionalityCapabilityConstants,
    },
    tile_mapping::{HexVector2d, TileId, TileIdServer},
    tiles::TileDirectory,
};
#[derive(Debug)]
pub struct PieceHealsAdjacent {
    pub healer_occupies_tile: TileId,
    pub heal: u32,
    pub adjacency_depth: u8,
    pub allow_self: bool,
}

impl TileActionFunctionalityCapabilityConstants for PieceHealsAdjacent {
    const ACCEPTABLE_SELECTION_COUNTS: std::ops::Range<usize> = 1..usize::MAX;
}

impl TileActionFunctionality for PieceHealsAdjacent {
    fn execute(
        &self,
        validated_selections: &[super::SelectedTile],
        world: &mut bevy::ecs::world::World,
    ) -> crate::requests::ChangeLog {
        let mut log = ChangeLog::default();

        for SelectedTile { id: tile, .. } in validated_selections.iter() {
            log.append(&mut pieces::heal(
                world
                    .get::<OccupiedByPiece>(world.resource::<TileDirectory>().get_entity(*tile))
                    .unwrap()
                    .piece(),
                self.heal,
                world,
                *tile,
            ));
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

        if self.allow_self {
            selection_status.maybe_set_elligible(self.healer_occupies_tile);
        }

        for tile in HexVector2d::from(self.healer_occupies_tile)
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

impl ForensicDescribe for PieceHealsAdjacent {
    fn forensic_description(&self) -> Box<[crate::forensic_action_descriptions::TextSnippet]> {
        Box::new([
            TextSnippet::new_basic_text("Restore "),
            TextSnippet::PlainText {
                text: format!("{} health", self.heal),
                color: Some(crate::forensic_action_descriptions::ColorIndicators::Health),
            },
            TextSnippet::new_basic_text(" to pieces on selected tiles within a "),
            TextSnippet::PlainText {
                text: format!("{} ring", self.adjacency_depth),
                color: Some(crate::forensic_action_descriptions::ColorIndicators::GeneralHighlight),
            },
            TextSnippet::new_basic_text(" radius."),
        ])
    }
}
