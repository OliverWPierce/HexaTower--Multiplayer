/// Rules of this module:
///
///  1. action executions are free to panic when they feel like it. The elligibility step should prevent problems from ever reaching the execution step.
///  2. Never assume that all pieces have owners.
///  3. Never use the "Active Player" resource... this creates problems for players who are experimenting with the board while it is not their turn. If you need such information,
///     get it from the fields of structs implementing TileActionFunctionality.
use std::{
    fmt::Debug,
    ops::{Index, Range},
};

use bevy::ecs::world::World;
use thiserror::Error;

use crate::{
    forensic_action_descriptions::{ForensicDescribe, TextSnippet},
    requests::ChangeLog,
    tile_based_actions::selection_mechanics::{InvalidSelection, SelectionData},
    tile_mapping::{TileId, TileIdServer},
};

pub mod change_tile_type;
pub mod damage_piece;
pub mod make_market_tile;
pub mod move_piece;
mod selection_mechanics;
pub mod spawn_pieces;

pub use selection_mechanics::SelectedTile;
pub use selection_mechanics::State;

pub trait TileActionFunctionality: Debug + Send + Sync + ForensicDescribe {
    fn execute(&self, validated_selections: &[SelectedTile], world: &mut World) -> ChangeLog;

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
        let mut initial_selection_data = SelectionData::new(*world.resource::<TileIdServer>());

        action
            .action_functionality
            .update_eligibility(&mut initial_selection_data, world);

        TileActionProcessCache {
            action,
            selections: initial_selection_data,
        }
    }

    pub fn forensic_describe(&self) -> Box<[TextSnippet]> {
        self.action.action_functionality.forensic_description()
    }

    pub fn try_select_tile_and_update_elligibility(
        &mut self,
        hopeful_tile: SelectedTile,
        world: &World,
    ) -> Result<(), InvalidSelection> {
        self.selections
            .try_select(hopeful_tile.id, hopeful_tile.direction)?;

        if self.selections.selection_count() >= self.action.tile_range_for_execution.end {
            self.selections.set_all_possible_inelligible();
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

    pub fn amount_currently_selected(&self) -> usize {
        self.selections.selection_count()
    }

    pub fn selection_bounds(&self) -> &Range<usize> {
        &self.action.tile_range_for_execution
    }

    pub fn selected_tiles(&self) -> &[SelectedTile] {
        self.selections.get_validated_ordered_selections()
    }

    pub fn get_tile_state(&self, tile: TileId) -> &State {
        self.selections.get_states().index(tile.id() as usize)
    }
}

#[derive(Debug, Error)]
#[error("Tried to execute an action, but too few tiles were selected.")]
pub struct SelectedTooFewTiles;

#[cfg(test)]
mod tests {
    use crate::{
        forensic_action_descriptions::ForensicDescribe,
        requests::ChangeLog,
        tile_based_actions::{
            TileAction, TileActionFunctionality, TileActionFunctionalityCapabilityConstants,
            selection_mechanics::SelectedTile,
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
                _validated_selections: &[SelectedTile],
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

        impl ForensicDescribe for FailingAction {
            fn forensic_description(
                &self,
            ) -> Box<[crate::forensic_action_descriptions::TextSnippet]> {
                Box::new([
                    crate::forensic_action_descriptions::TextSnippet::new_basic_text(
                        "No description implemented yet.",
                    ),
                ])
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
