use crate::{
    forensic_action_descriptions::{ForensicDescribe, TextSnippet},
    pieces::{
        FacingHexDirection, OccupiedByPiece, PieceOwnedByPlayer, damage_and_maybe_kill_piece,
    },
    players::PlayerId,
    requests::ChangeLog,
    tile_based_actions::{
        SelectedTile, TileActionFunctionality, TileActionFunctionalityCapabilityConstants,
    },
    tile_mapping::{HexVector2d, TileId, TileIdServer},
    tiles::TileDirectory,
};
#[derive(Debug)]
pub struct PieceAttacksForward {
    pub attacker_occupies_tile: TileId,
    pub damage: u32,
    pub depth: u8,
}

impl TileActionFunctionalityCapabilityConstants for PieceAttacksForward {
    const ACCEPTABLE_SELECTION_COUNTS: std::ops::Range<usize> = 1..usize::MAX;
}

impl TileActionFunctionality for PieceAttacksForward {
    fn execute(
        &self,
        validated_selections: &[super::SelectedTile],
        world: &mut bevy::ecs::world::World,
    ) -> crate::requests::ChangeLog {
        let mut log = ChangeLog::default();
        let attacking_player = {
            let attacking_piece = world
                .get::<OccupiedByPiece>(
                    world
                        .resource::<TileDirectory>()
                        .get_entity(self.attacker_occupies_tile),
                )
                .unwrap()
                .piece();
            world
                .get::<PieceOwnedByPlayer>(attacking_piece)
                .map(|owner| {
                    *world
                        .get::<PlayerId>(owner.0)
                        .expect("All players should have an ID")
                })
        };

        for SelectedTile { id: tile, .. } in validated_selections.iter() {
            damage_and_maybe_kill_piece(self.damage, *tile, attacking_player, &mut log, world);
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

        let Some(piece) =
            world.get::<OccupiedByPiece>(tile_directory.get_entity(self.attacker_occupies_tile))
        else {
            return;
        };
        let Some(&direction) = world.get::<FacingHexDirection>(piece.piece()) else {
            return;
        };

        for tile in HexVector2d::from(self.attacker_occupies_tile).ids_in_direction(
            direction.into(),
            self.depth,
            world.resource::<TileIdServer>(),
        ) {
            if world
                .get::<OccupiedByPiece>(tile_directory.get_entity(tile))
                .is_some()
            {
                selection_status.maybe_set_elligible(tile);
            }
        }
    }
}

impl ForensicDescribe for PieceAttacksForward {
    fn forensic_description(&self) -> Box<[crate::forensic_action_descriptions::TextSnippet]> {
        Box::new([
            TextSnippet::new_basic_text("Deal "),
            TextSnippet::PlainText {
                text: format!("{} damage", self.damage),
                color: Some(crate::forensic_action_descriptions::ColorIndicators::Damage),
            },
            TextSnippet::new_basic_text(" to selected pieces. Pieces can be up to "),
            TextSnippet::PlainText {
                text: format!("{} tiles", self.depth),
                color: Some(crate::forensic_action_descriptions::ColorIndicators::GeneralHighlight),
            },
            TextSnippet::new_basic_text(" in front of this piece"),
        ])
    }
}
