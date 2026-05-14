use thiserror::Error;

use crate::{directories::InvalidIdErr, tile_mapping::TileId};

#[derive(Debug, Default, PartialEq, Clone)]
pub enum State {
    Elligible,
    Selected,
    #[default]
    Neither,
}

pub struct SelectionData {
    all_tile_states: Box<[State]>,
    ordered_selections: Vec<TileId>,
}
#[derive(Debug, Error)]
pub enum SelectionError {
    #[error("Tried to select a tile which was not elligible for selection.")]
    AttemptedToSelectInelligibleTile,
    #[error("Attempted to select a tile of invalid id. Id number {0}")]
    InvalidIdError(#[from] InvalidIdErr<TileId>),
}

impl SelectionData {
    pub fn new(tiles_on_board: usize) -> Self {
        SelectionData {
            all_tile_states: vec![State::Neither; tiles_on_board].into_boxed_slice(),
            ordered_selections: Vec::new(),
        }
    }

    /// If you try to select a tile that is either already selected or innelligible, it will error. If you try to change a tile which is already selected, nothing will happen. This function protects invalid data from being created. (ie: selecting an inelligible tile, or selecting a tile twice.)
    pub fn try_set_state(&mut self, tile: TileId, target: State) -> Result<(), SelectionError> {
        let state = self
            .all_tile_states
            .get_mut(tile.id() as usize)
            .ok_or(SelectionError::InvalidIdError(InvalidIdErr(tile)))?;

        match target {
            State::Elligible => {
                if *state == State::Neither {
                    *state = State::Elligible;
                };
                Ok(())
            }

            State::Selected => {
                if *state != State::Elligible {
                    Err(SelectionError::AttemptedToSelectInelligibleTile)
                } else {
                    *state = State::Selected;
                    self.ordered_selections.push(tile);
                    Ok(())
                }
            }

            State::Neither => {
                if *state == State::Elligible {
                    *state = State::Neither;
                };
                Ok(())
            }
        }
    }

    pub fn get_validated_ordered_selections(&self) -> &[TileId] {
        self.ordered_selections.as_slice()
    }

    pub fn get_states(&self) -> &[State] {
        &self.all_tile_states
    }

    pub fn selection_count(&self) -> usize {
        self.ordered_selections.len()
    }

    pub fn set_all_possible_elligible(&mut self) {
        for tile in self.all_tile_states.iter_mut() {
            if *tile != State::Selected {
                *tile = State::Elligible
            }
        }
    }

    pub fn set_all_possible_inelligible(&mut self) {
        for tile in self.all_tile_states.iter_mut() {
            if *tile != State::Selected {
                *tile = State::Neither
            }
        }
    }

    pub fn clear_elligibles(&mut self) {
        self.all_tile_states.iter_mut().for_each(|state| {
            if *state == State::Elligible {
                *state = State::Neither
            }
        });
    }
}
