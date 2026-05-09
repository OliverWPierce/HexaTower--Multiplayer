use std::{fmt::Debug, ops::Range};

use bevy::ecs::world::World;
use thiserror::Error;

use crate::{
    player_actions::ChangeLog,
    tile_based_actions::selection_mechanics::{SelectionData, SelectionError},
    tile_mapping::TileId,
};

pub mod change_tile_type;
mod selection_mechanics;

pub trait TileActionFunctionality: Debug {
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

pub struct LoadedTileAction {
    action: TileAction,
    selections: SelectionData,
}

impl LoadedTileAction {
    pub fn initialize(action: TileAction, tiles_on_board: usize, world: &World) -> Self {
        let mut initial_selection_data = SelectionData::new(tiles_on_board);

        action
            .action_functionality
            .update_eligibility(&mut initial_selection_data, world);

        LoadedTileAction {
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

    pub(crate) fn execute(self, world: &mut World) -> ChangeLog {
        self.action
            .action_functionality
            .execute(self.selections.get_validated_ordered_selections(), world)
    }
}
