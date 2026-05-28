use std::{fmt::Debug, ops::Range};

use bevy::ecs::world::World;
use thiserror::Error;

use crate::{
    requests::ChangeLog,
    tile_based_actions::selection_mechanics::{SelectionData, SelectionError},
    tile_mapping::TileId,
    tiles::TileDirectory,
};

pub mod change_tile_type;
pub mod make_market_tile;
mod selection_mechanics;
pub mod spawn_pieces;

pub use selection_mechanics::State;

pub trait TileActionFunctionality: Debug + Send + Sync {
    fn execute(&self, validated_selections: &[TileId], world: &mut World) -> ChangeLog;

    fn update_eligibility(&self, selection_status: &mut SelectionData, world: &World);
}

pub trait TileActionFunctionalityCapabilityConstants: TileActionFunctionality {
    const ACCEPTABLE_SELECTION_COUNTS: Range<usize>;
}

#[derive(Debug)]
pub struct TileAction {
    action_functionality: Box<dyn TileActionFunctionality>,
    tile_range_for_execution: Range<usize>,
}

#[derive(Debug, Error)]
#[error(
    "Tried to create a valid tile action, but its game design selection bounds did not match the functionality."
)]
pub struct InvalidSelectionBounds;

impl TileAction {
    pub fn new<A: TileActionFunctionalityCapabilityConstants + 'static>(
        action: A,
        game_design_selection_bounds: Range<usize>,
    ) -> Result<Self, InvalidSelectionBounds> {
        if A::ACCEPTABLE_SELECTION_COUNTS.start <= game_design_selection_bounds.start
            && A::ACCEPTABLE_SELECTION_COUNTS.end >= game_design_selection_bounds.end
        {
            Ok(Self {
                action_functionality: Box::new(action),
                tile_range_for_execution: game_design_selection_bounds,
            })
        } else {
            Err(InvalidSelectionBounds)
        }
    }
}

#[derive(Debug)]
pub struct TileActionProcessCache {
    action: TileAction,
    selections: SelectionData,
}

impl TileActionProcessCache {
    pub fn initialize(action: TileAction, world: &World) -> Self {
        let tiles_on_board = world.resource::<TileDirectory>().tile_count();
        let mut initial_selection_data = SelectionData::new(tiles_on_board);

        action
            .action_functionality
            .update_eligibility(&mut initial_selection_data, world);

        TileActionProcessCache {
            action,
            selections: initial_selection_data,
        }
    }

    pub fn try_select_tile_and_update_elligibility(
        &mut self,
        tile: TileId,
        world: &World,
    ) -> Result<(), SelectionError> {
        self.selections
            .try_set_state(tile, selection_mechanics::State::Selected)?;

        if self.selections.selection_count() >= self.action.tile_range_for_execution.end {
            self.selections.clear_elligibles();
        } else {
            self.action
                .action_functionality
                .update_eligibility(&mut self.selections, world);
        }

        Ok(())
    }

    pub fn view_selection_states(&self) -> &[selection_mechanics::State] {
        self.selections.get_states()
    }

    pub(crate) fn try_execute(&self, world: &mut World) -> Result<ChangeLog, SelectedTooFewTiles> {
        if self.selections.selection_count() >= self.action.tile_range_for_execution.start {
            Ok(self
                .action
                .action_functionality
                .execute(self.selections.get_validated_ordered_selections(), world))
        } else {
            Err(SelectedTooFewTiles)
        }
    }
}

#[derive(Debug, Error)]
#[error("Tried to execute an action, but too few tiles were selected.")]
pub struct SelectedTooFewTiles;

#[cfg(test)]
mod tests {
    use crate::{
        requests::ChangeLog,
        tile_based_actions::{
            TileAction, TileActionFunctionality, TileActionFunctionalityCapabilityConstants,
        },
    };

    #[test]
    fn test_improper_tile_action_creation() {
        #[derive(Debug, Clone, Copy)]
        struct FailingAction;

        impl TileActionFunctionalityCapabilityConstants for FailingAction {
            const ACCEPTABLE_SELECTION_COUNTS: std::ops::Range<usize> = 2..3;
        }

        impl TileActionFunctionality for FailingAction {
            fn execute(
                &self,
                _validated_selections: &[crate::tile_mapping::TileId],
                _world: &mut bevy::ecs::world::World,
            ) -> crate::requests::ChangeLog {
                ChangeLog::default()
            }

            fn update_eligibility(
                &self,
                _selection_status: &mut super::selection_mechanics::SelectionData,
                _world: &bevy::ecs::world::World,
            ) {
            }
        }

        let failing_action_functionality = FailingAction;

        assert!(TileAction::new(failing_action_functionality, 1..2).is_err());
        assert!(TileAction::new(failing_action_functionality, 2..2).is_ok());
        assert!(TileAction::new(failing_action_functionality, 2..3).is_ok());
        assert!(TileAction::new(failing_action_functionality, 3..4).is_err());
        assert!(TileAction::new(failing_action_functionality, 0..usize::MAX).is_err());
    }
}
