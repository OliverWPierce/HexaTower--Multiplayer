use crate::{
    forensic_action_descriptions::{ForensicDescribe, LinkedGamplayElement, TextSnippet},
    markets::MarketId,
    requests::ActionEffect,
    tile_based_actions::{
        TileActionFunctionality, TileActionFunctionalityCapabilityConstants,
        selection_mechanics::SelectedTile,
    },
    tiles::{MarketTile, TileDirectory},
};
#[derive(Debug)]
pub(crate) struct MakeMarketTile {
    pub market: MarketId,
}

impl TileActionFunctionality for MakeMarketTile {
    fn execute(
        &self,
        validated_selections: &[SelectedTile],
        world: &mut bevy::ecs::world::World,
    ) -> crate::requests::ChangeLog {
        let tile_entities = world.resource::<TileDirectory>();

        world.insert_batch(
            validated_selections
                .iter()
                .map(|tile| (tile_entities.get_entity(tile.id), MarketTile(self.market)))
                .collect::<Box<[_]>>(),
        );

        validated_selections
            .iter()
            .map(|tile| ActionEffect::SpawnedNewMarket {
                tile: tile.id,
                market: self.market,
            })
            .collect::<Vec<_>>()
            .into()
    }

    fn update_eligibility(
        &self,
        selection_status: &mut super::selection_mechanics::SelectionData,
        world: &bevy::ecs::world::World,
    ) {
        selection_status.set_all_possible_elligible();

        for (tile_id, entity) in world.resource::<TileDirectory>().id_entity_pairs() {
            if world.get::<MarketTile>(entity).is_none() {
                continue;
            }

            selection_status.maybe_set_inelligible(tile_id);
        }
    }
}

impl TileActionFunctionalityCapabilityConstants for MakeMarketTile {
    const ACCEPTABLE_SELECTION_COUNTS: std::ops::Range<usize> = 0..usize::MAX;
}

impl ForensicDescribe for MakeMarketTile {
    fn forensic_description(&self) -> Box<[crate::forensic_action_descriptions::TextSnippet]> {
        Box::new([
            TextSnippet::new_basic_text("Spawns "),
            TextSnippet::Link(LinkedGamplayElement::Market(self.market)),
            TextSnippet::new_basic_text(
                " on selected tiles. You must occupy a market in order to purchase its wares.",
            ),
        ])
    }
}
