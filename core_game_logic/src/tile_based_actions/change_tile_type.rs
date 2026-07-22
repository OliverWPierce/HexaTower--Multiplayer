use std::ops::Range;

use crate::{
    forensic_action_descriptions::{ForensicDescribe, LinkedGamplayElement, TextSnippet},
    requests::{ActionEffect, ChangeLog},
    tile_based_actions::{
        TileActionFunctionality, TileActionFunctionalityCapabilityConstants,
        selection_mechanics::SelectedTile,
    },
    tile_mapping::TileId,
    tiles::{TileDirectory, TileType},
};
#[derive(Debug, Clone)]
pub struct ConvertTileTo {
    pub target_type: TileType,
    pub restrictions: Option<AdjecentRestriction>,
}
#[derive(Debug, Clone)]
pub struct AdjecentRestriction {
    pub adjacent_to: TileId,
}

impl TileActionFunctionalityCapabilityConstants for ConvertTileTo {
    const ACCEPTABLE_SELECTION_COUNTS: Range<usize> = 0..usize::MAX;
}

impl TileActionFunctionality for ConvertTileTo {
    fn execute(
        &self,
        validated_selections: &[SelectedTile],
        world: &mut bevy::ecs::world::World,
    ) -> crate::requests::ChangeLog {
        let mut log = ChangeLog::default();

        for tile in validated_selections {
            let directory = world.resource::<TileDirectory>();

            let Some(mut tile_type) = world
                .entity_mut(directory.get_entity(tile.id))
                .into_mut::<TileType>()
            else {
                panic!("A tile entity had no component indicating the type of tile it was.")
            };

            *tile_type = self.target_type.clone();
            log.write(ActionEffect::ConvertedTileType {
                tile: tile.id,
                new_type: self.target_type.clone(),
            });
        }

        log
    }

    fn update_eligibility(
        &self,
        selection_status: &mut super::selection_mechanics::SelectionData,
        world: &bevy::ecs::world::World,
    ) {
        if self.restrictions.is_none() {
            selection_status.set_all_possible_elligible();
            return;
        }

        selection_status.set_all_possible_inelligible();

        use crate::tile_mapping::*;
        let basis_vector: HexVector2d = self.restrictions.as_ref().unwrap().adjacent_to.into();

        for tile in basis_vector.adjacencies() {
            let Some(id) = tile.to_valid_tile_id(world.resource::<TileIdServer>()) else {
                continue;
            };
            selection_status.maybe_set_elligible(id);
        }
    }
}

impl ForensicDescribe for ConvertTileTo {
    fn forensic_description(&self) -> Box<[crate::forensic_action_descriptions::TextSnippet]> {
        Box::new([
            TextSnippet::new_basic_text("Convert selected tiles into "),
            TextSnippet::Link(LinkedGamplayElement::Tile(self.target_type.clone())),
            TextSnippet::new_basic_text(" tiles."),
        ])
    }
}
