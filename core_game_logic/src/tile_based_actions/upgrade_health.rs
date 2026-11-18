use crate::{
    forensic_action_descriptions::{ForensicDescribe, TextSnippet},
    pieces::{Health, OccupiedByPiece},
    requests::{ActionEffect, ChangeLog},
    tile_based_actions::{
        TileActionFunctionality, TileActionFunctionalityCapabilityConstants,
        selection_mechanics::SelectedTile,
    },
    tiles::TileDirectory,
};

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct UpgradeHealth;

const UPGRADE_AMOUNT: u32 = 5;

impl TileActionFunctionalityCapabilityConstants for UpgradeHealth {
    const ACCEPTABLE_SELECTION_COUNTS: std::ops::Range<usize> = 0..usize::MAX;
}
impl TileActionFunctionality for UpgradeHealth {
    fn execute(
        &self,
        validated_selections: &[SelectedTile],
        world: &mut bevy::ecs::world::World,
    ) -> crate::requests::ChangeLog {
        let mut log = ChangeLog::default();

        validated_selections
            .iter()
            .for_each(|SelectedTile { id, .. }| {
                let mut health = world
                    .get_mut::<Health>(
                        world
                            .get::<OccupiedByPiece>(
                                world.resource::<TileDirectory>().get_entity(*id),
                            )
                            .unwrap()
                            .piece(),
                    )
                    .unwrap();

                health.tier += 1;

                health.max += UPGRADE_AMOUNT;

                log.write(ActionEffect::UpgradeMaxHealth(*id, UPGRADE_AMOUNT));
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

impl ForensicDescribe for UpgradeHealth {
    fn forensic_description(&self) -> Box<[crate::forensic_action_descriptions::TextSnippet]> {
        ([
            TextSnippet::new_basic_text("Increase max health of selected pieces by"),
            TextSnippet::PlainText {
                text: format!(" {}.", UPGRADE_AMOUNT),
                color: Some(crate::forensic_action_descriptions::ColorIndicators::Health),
            },
        ])
        .into()
    }
}
