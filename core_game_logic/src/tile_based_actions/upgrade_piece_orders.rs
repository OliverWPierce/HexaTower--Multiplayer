use crate::{
    forensic_action_descriptions::{ForensicDescribe, TextSnippet},
    pieces::{OccupiedByPiece, OrdersReceivable},
    requests::{ActionEffect, ChangeLog},
    tile_based_actions::{
        TileActionFunctionality, TileActionFunctionalityCapabilityConstants,
        selection_mechanics::SelectedTile,
    },
    tiles::TileDirectory,
};

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct UpgradePieceOrders;

impl TileActionFunctionalityCapabilityConstants for UpgradePieceOrders {
    const ACCEPTABLE_SELECTION_COUNTS: std::ops::Range<usize> = 0..usize::MAX;
}
impl TileActionFunctionality for UpgradePieceOrders {
    fn execute(
        &self,
        validated_selections: &[SelectedTile],
        world: &mut bevy::ecs::world::World,
    ) -> crate::requests::ChangeLog {
        let mut log = ChangeLog::default();

        validated_selections
            .iter()
            .for_each(|SelectedTile { id, .. }| {
                let mut per_round = world
                    .get_mut::<OrdersReceivable>(
                        world
                            .get::<OccupiedByPiece>(
                                world.resource::<TileDirectory>().get_entity(*id),
                            )
                            .unwrap()
                            .piece(),
                    )
                    .unwrap();

                per_round.tier += 1;

                per_round.per_round += 1;

                log.write(ActionEffect::UpgradeOrdersPerRound(*id));
            });

        log
    }

    fn update_eligibility(
        &self,
        selection_status: &mut super::selection_mechanics::SelectionData,
        world: &bevy::ecs::world::World,
    ) {
        for (id, entity) in world.resource::<TileDirectory>().id_entity_pairs() {
            match world.get::<OccupiedByPiece>(entity) {
                Some(_) => selection_status.maybe_set_elligible(id),
                None => selection_status.maybe_set_inelligible(id),
            }
        }
    }
}

impl ForensicDescribe for UpgradePieceOrders {
    fn forensic_description(&self) -> Box<[crate::forensic_action_descriptions::TextSnippet]> {
        ([
            TextSnippet::new_basic_text("Permanently increase selected pieces' order capacity by"),
            TextSnippet::PlainText {
                text: format!(" {}.", 1),
                color: Some(crate::forensic_action_descriptions::ColorIndicators::Health),
            },
        ])
        .into()
    }
}
