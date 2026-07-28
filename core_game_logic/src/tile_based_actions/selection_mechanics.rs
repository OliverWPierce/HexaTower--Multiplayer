use std::ops::IndexMut;

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{
    pieces::FacingHexDirection,
    tile_mapping::{TileId, TileIdServer},
};

#[derive(Debug, Default, PartialEq, Clone)]
pub enum State {
    Elligible,
    Selected(FacingHexDirection),
    #[default]
    Neither,
}
#[derive(Debug)]
pub struct SelectionData {
    all_tile_states: Box<[State]>,
    ordered_selections: Vec<SelectedTile>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct SelectedTile {
    pub id: TileId,
    pub direction: FacingHexDirection,
}

#[derive(Debug, Error)]
#[error("Tried to select {0:?} which was not elligible by the current cache state.")]
pub struct InvalidSelection(TileId);

impl SelectionData {
    pub fn new(total_tile_count: TileIdServer) -> Self {
        SelectionData {
            all_tile_states: vec![State::Neither; total_tile_count.total_tiles_on_board as usize]
                .into_boxed_slice(),
            ordered_selections: Vec::new(),
        }
    }

    pub fn get_validated_ordered_selections(&self) -> &[SelectedTile] {
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
            if *tile == State::Neither {
                *tile = State::Elligible
            }
        }
    }

    pub fn set_all_possible_inelligible(&mut self) {
        for tile in self.all_tile_states.iter_mut() {
            if *tile == State::Elligible {
                *tile = State::Neither
            }
        }
    }

    pub fn maybe_set_inelligible(&mut self, tile: TileId) {
        let s = self.all_tile_states.index_mut(tile.id() as usize);
        if *s == State::Elligible {
            *s = State::Neither;
        }
    }

    pub fn maybe_set_elligible(&mut self, tile: TileId) {
        let s = self.all_tile_states.index_mut(tile.id() as usize);
        if *s == State::Neither {
            *s = State::Elligible;
        }
    }

    pub fn try_select(
        &mut self,
        tile: TileId,
        direction: FacingHexDirection,
    ) -> Result<(), InvalidSelection> {
        let s = self.all_tile_states.index_mut(tile.id() as usize);
        if *s == State::Elligible {
            *s = State::Selected(direction);
            self.ordered_selections.push(SelectedTile {
                id: tile,
                direction,
            });

            Ok(())
        } else {
            Err(InvalidSelection(tile))
        }
    }
}
