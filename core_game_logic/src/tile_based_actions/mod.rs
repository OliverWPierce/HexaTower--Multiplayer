use std::ops::Range;

use bevy::ecs::world::World;
use thiserror::Error;

use crate::{player_actions::ChangeLog, tile_mapping::TileId};

#[derive(Debug, Clone)]
struct ValidTileBasedAction<A: TileBasedAction> {
    game_design_bounds: Range<usize>,
    tile_action: A,
}

trait TileBasedAction {
    const EXECUTABLE_SELECTION_BOUNDS: Range<usize>;
    /// If the selections are dependent, it means that the eligible tiles evolve with each tile that is selected. For example, chain lightning is dependent,
    /// because selecting a tile determines influences the next set of eligible tiles. An "all pieces" critera is independent, because selecting a piece doesn't change
    /// what other pieces can be selected afterwards.
    const DEPENDENT_SELECTIONS: bool;

    /// This function has permission to actually modify the board. It should only run once all checks have happened. If an error occurs, the program will panic.
    fn execute(&self, tiles: &[TileId], world: &mut World) -> ChangeLog;

    /// When implementing this function, do not worry about handling when the player has selected the maximum amount of tiles. That is handled elsewhere.
    fn calculate_eligible_tiles_using_method(
        &self,
        selected_tiles: &[TileId],
        world: &World,
    ) -> &[TileId];
}

#[derive(Debug, Error)]
#[error(
    "Attempted to create a game_action, but the game design bounds were incompatible with the code's functionality."
)]
pub struct ContradictorySelectionBoundsError;

impl<A: TileBasedAction> ValidTileBasedAction<A> {
    fn new(
        action: A,
        selection_bounds: Range<usize>,
    ) -> Result<Self, ContradictorySelectionBoundsError> {
        if selection_bounds.start >= A::EXECUTABLE_SELECTION_BOUNDS.start
            && selection_bounds.end <= A::EXECUTABLE_SELECTION_BOUNDS.end
        {
            Ok(Self {
                game_design_bounds: selection_bounds,
                tile_action: action,
            })
        } else {
            Err(ContradictorySelectionBoundsError)
        }
    }
}

struct LoadedTileBasedAction<A: TileBasedAction> {
    action: ValidTileBasedAction<A>,
    validated_selections: Vec<TileId>,
}

impl<A: TileBasedAction + Clone> LoadedTileBasedAction<A> {
    fn load(action: &ValidTileBasedAction<A>) -> Self {
        Self {
            action: action.clone(),
            validated_selections: Vec::new(),
        }
    }

    fn eligible_tiles(&self, world: &World) -> &[TileId] {
        if self.validated_selections.len() >= self.action.game_design_bounds.end {
            &[]
        } else {
            self.action
                .tile_action
                .calculate_eligible_tiles_using_method(&self.validated_selections, world)
            // still need to handle when the player has selected the maximum number of tiles. Hold up, if its already a resource, just use the ECS like in the last version.
        }
    }
}

#[cfg(test)]
mod tests {}
