use crate::{
    markets::MarketId,
    requests::ActionEffect,
    tile_based_actions::{
        TileActionFunctionality, TileActionFunctionalityCapabilityConstants,
        selection_mechanics::SelectedTile,
    },
    tiles::{MarketTile, TileDirectory},
};
#[derive(Debug)]
pub struct MakeMarketTile {
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
                .map(|tile| {
                    (
                        tile_entities.get_entity(tile.id).unwrap(),
                        MarketTile(self.market),
                    )
                })
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
        _world: &bevy::ecs::world::World,
    ) {
        selection_status.set_all_possible_elligible();
    }
}

impl TileActionFunctionalityCapabilityConstants for MakeMarketTile {
    const ACCEPTABLE_SELECTION_COUNTS: std::ops::Range<usize> = 0..usize::MAX;
}
