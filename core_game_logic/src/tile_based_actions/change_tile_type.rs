use std::ops::Range;

use crate::{
    requests::{ActionEffect, ChangeLog},
    tile_based_actions::{TileActionFunctionality, TileActionFunctionalityCapabilityConstants},
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
        validated_selections: &[crate::tile_mapping::TileId],
        world: &mut bevy::ecs::world::World,
    ) -> crate::requests::ChangeLog {
        let mut log = ChangeLog::default();

        for tile in validated_selections {
            let directory = world.resource::<TileDirectory>();

            let Some(mut tile_type) = world
                .entity_mut(directory.get_entity(*tile).unwrap())
                .into_mut::<TileType>()
            else {
                panic!("A tile entity had no component indicating the type of tile it was.")
            };

            *tile_type = self.target_type.clone();
            log.write(ActionEffect::ConvertedTileType {
                tile: *tile,
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

        selection_status.clear_elligibles();

        use crate::tile_mapping::*;
        let basis_vector: HexVector2d = self.restrictions.as_ref().unwrap().adjacent_to.into();

        let maximum_id_on_board =
            TileId::new(world.resource::<TileDirectory>().tile_count() as u32 - 1);

        let directions = [NORTH, NORTH_EAST, NORTH_WEST, SOUTH, SOUTH_EAST, SOUTH_WEST];

        for id in directions.iter().filter_map(|direction| {
            let id: TileId = (basis_vector + *direction).into();
            if id > maximum_id_on_board {
                None
            } else {
                Some(id)
            }
        }) {
            selection_status
                .try_set_state(
                    id,
                    crate::tile_based_actions::selection_mechanics::State::Elligible,
                )
                .unwrap(); // Since we already made sure the TileId is valid for this board size, and we're not trying to select a tile, this is fine.
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        CreationParameters,
        requests::ActionEffect,
        tile_based_actions::{TileAction, TileActionProcessCache, change_tile_type::ConvertTileTo},
        tile_mapping::TileId,
    };

    #[test]
    fn test_tile_conversion() {
        let mut world = CreationParameters::testing_default();

        let mut loaded_action = TileActionProcessCache::initialize(
            TileAction::new(
                ConvertTileTo {
                    target_type: crate::tiles::TileType::Ex1,
                    restrictions: None,
                },
                2..4,
            )
            .unwrap(),
            &world,
        );

        assert!(loaded_action.try_execute(&mut world).is_err());

        loaded_action
            .try_select_tile_and_update_elligibility(TileId::new(0), &world)
            .unwrap();

        assert!(loaded_action.try_execute(&mut world).is_err());

        loaded_action
            .try_select_tile_and_update_elligibility(TileId::new(2), &world)
            .unwrap();

        loaded_action
            .try_select_tile_and_update_elligibility(TileId::new(23), &world)
            .unwrap();

        assert!(
            loaded_action
                .try_select_tile_and_update_elligibility(TileId::new(61), &world)
                .is_err()
        );
        assert!(
            loaded_action
                .try_select_tile_and_update_elligibility(TileId::new(2), &world)
                .is_err()
        );

        loaded_action
            .try_select_tile_and_update_elligibility(TileId::new(60), &world)
            .unwrap();

        assert!(
            loaded_action
                .try_select_tile_and_update_elligibility(TileId::new(27), &world)
                .is_err()
        );

        let exprected_change_log = [
            ActionEffect::ConvertedTileType {
                tile: TileId::new(0),
                new_type: crate::tiles::TileType::Ex1,
            },
            ActionEffect::ConvertedTileType {
                tile: TileId::new(2),
                new_type: crate::tiles::TileType::Ex1,
            },
            ActionEffect::ConvertedTileType {
                tile: TileId::new(23),
                new_type: crate::tiles::TileType::Ex1,
            },
            ActionEffect::ConvertedTileType {
                tile: TileId::new(60),
                new_type: crate::tiles::TileType::Ex1,
            },
        ];

        let created_change_log = loaded_action.try_execute(&mut world).unwrap();

        assert_eq!(created_change_log.read()[0], exprected_change_log[0]);

        assert_eq!(created_change_log.read()[1], exprected_change_log[1]);

        assert_eq!(created_change_log.read()[2], exprected_change_log[2]);

        assert_eq!(created_change_log.read()[3], exprected_change_log[3]);
    }
}
