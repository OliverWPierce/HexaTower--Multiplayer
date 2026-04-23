use std::ops::{Range, RangeBounds};

use bevy::ecs::world::World;

use crate::tile_mapping::TileId;

struct TileBasedGameAction<F: TileBasedActionFunctionality, M: TileBasedEligibilityMethod> {
    functionality: F,
    eligibility_critera: M,
}

trait TileBasedActionFunctionality {
    const EXPECTED_TILE_COUNT: Range<u16>;

    fn execute(&self, affected_tiles: &[TileId], world: &mut World);

    fn expected_input_range(&self) -> Range<u16> {
        Self::EXPECTED_TILE_COUNT
    }
}

trait TileBasedEligibilityMethod {
    // the inputs for this are a work in progress.
    fn eligible_tiles(&self, selected_tiles: &[TileId], world: &World);
}

impl<F: TileBasedActionFunctionality, M: TileBasedEligibilityMethod> TileBasedGameAction<F, M> {
    pub fn try_execute(&self, on_tiles: &[TileId], world: &mut World) {
        if !F::EXPECTED_TILE_COUNT.contains(&(on_tiles.len() as u16)) {
            todo!()
            //return an error.
        }
        todo!()

        //we replay the selections. If the selection returns an error, we know the action was bad. This is the last "checks" phase.

        // Last, we execute the action and return the ChangeLog.
    }
}
