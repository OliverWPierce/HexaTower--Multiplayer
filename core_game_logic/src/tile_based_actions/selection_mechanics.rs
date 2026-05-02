use thiserror::Error;

use crate::{tile_mapping::TileId, tiles};

#[derive(Debug, Default, PartialEq, Clone)]
pub enum State {
    Elligible,
    Selected,
    #[default]
    Neither,
}
#[derive(Debug, Clone)]
pub struct SelectionState(State);

impl SelectionState {
    pub fn state(&self) -> &State {
        &self.0
    }

    pub fn try_set_elligble(&mut self) {
        if self.0 != State::Selected {
            self.0 = State::Elligible
        }
    }

    pub fn try_set_inelligible(&mut self) {
        if self.0 != State::Selected {
            self.0 = State::Neither
        }
    }
}

pub struct SelectionData {
    all_tiles: Box<[SelectionState]>,
    ordered_selections: Vec<TileId>,
}
#[derive(Debug, Error)]
pub enum SelectionError {
    #[error("Tried to select a tile which was not elligible for selection.")]
    AttemptedToSelectInelligibleTile,
    #[error("Attempted to select a tile of invalid id. Id number {0}")]
    InvalidIdError(#[from] tiles::InvaildIDErr),
}

impl SelectionData {
    pub fn new(tiles_on_board: usize) -> Self {
        SelectionData {
            all_tiles: vec![SelectionState(State::Neither); tiles_on_board].into_boxed_slice(),
            ordered_selections: Vec::new(),
        }
    }

    pub fn try_select(&mut self, tile: TileId) -> Result<(), SelectionError> {
        let state = &mut self
            .all_tiles
            .get_mut(tile.id() as usize)
            .ok_or(SelectionError::InvalidIdError(tiles::InvaildIDErr(tile)))?
            .0;

        if *state != State::Selected {
            Err(SelectionError::AttemptedToSelectInelligibleTile)
        } else {
            *state = State::Selected;
            self.ordered_selections.push(tile);
            Ok(())
        }
    }

    pub fn get_validated_ordered_selections(&self) -> &[TileId] {
        self.ordered_selections.as_slice()
    }

    fn get_states(&self) -> &[State] {
        self.all_tiles
            .iter()
            .map(|s| s.0)
            .collect::<Vec<State>>()
            .as_slice();
    }

    pub fn selection_count(&self) -> usize {
        self.ordered_selections.len()
    }

    pub fn clear_elligibles(&mut self) {
        self.all_tiles
            .iter_mut()
            .for_each(|tile_status| tile_status.try_set_inelligible());
    }
}
