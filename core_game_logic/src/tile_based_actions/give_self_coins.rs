use std::ops::Range;

use crate::{
    forensic_action_descriptions::{ForensicDescribe, TextSnippet},
    pieces::{OccupiedByPiece, PieceOwnedByPlayer},
    players::{Coins, PlayerId},
    requests::{ActionEffect, ChangeLog},
    tile_based_actions::{
        TileActionFunctionality, TileActionFunctionalityCapabilityConstants,
        selection_mechanics::{self, SelectedTile},
    },
    tile_mapping::TileId,
    tiles::TileDirectory,
};

#[derive(Debug)]
pub struct GiveOwnerCoins {
    pub tile_of_piece: TileId,
    pub ammount: u32,
}

impl TileActionFunctionalityCapabilityConstants for GiveOwnerCoins {
    const ACCEPTABLE_SELECTION_COUNTS: Range<usize> = 1..1;
}

impl TileActionFunctionality for GiveOwnerCoins {
    fn execute(
        &self,
        validated_selections: &[SelectedTile],
        world: &mut bevy::ecs::world::World,
    ) -> crate::requests::ChangeLog {
        let mut log = ChangeLog::default();

        let piece = world
            .get::<OccupiedByPiece>(
                world
                    .resource::<TileDirectory>()
                    .get_entity(validated_selections.first().unwrap().id),
            )
            .unwrap();

        let player = world.get::<PieceOwnedByPlayer>(piece.piece()).unwrap().0;

        world
            .get_mut::<Coins>(player)
            .expect("All players have info about their finances.")
            .0 += self.ammount;

        log.write(ActionEffect::AlteredCoins {
            player: *world.get::<PlayerId>(player).unwrap(),
            delta_coins: self.ammount as i32,
            from_tile: Some(validated_selections.first().unwrap().id),
        });

        log
    }

    fn update_eligibility(
        &self,
        selection_status: &mut super::selection_mechanics::SelectionData,
        world: &bevy::ecs::world::World,
    ) {
        let Some(piece) = world.get::<OccupiedByPiece>(
            world
                .resource::<TileDirectory>()
                .get_entity(self.tile_of_piece),
        ) else {
            return;
        };

        let Some(..) = world.get::<PieceOwnedByPlayer>(piece.piece()) else {
            return;
        };

        selection_status.maybe_set_elligible(self.tile_of_piece);
        if selection_status.state_of_tile(self.tile_of_piece)
            == &selection_mechanics::State::Elligible
        {
            selection_status
                .try_select(self.tile_of_piece, crate::pieces::FacingHexDirection::North)
                .unwrap()
        }
    }
}

impl ForensicDescribe for GiveOwnerCoins {
    fn forensic_description(&self) -> Box<[crate::forensic_action_descriptions::TextSnippet]> {
        Box::new([
            TextSnippet::new_basic_text("Give the owner +"),
            TextSnippet::PlainText {
                text: format!("+{}", self.ammount),
                color: Some(crate::forensic_action_descriptions::ColorIndicators::Money),
            },
            TextSnippet::new_basic_text(" coins."),
        ])
    }
}
