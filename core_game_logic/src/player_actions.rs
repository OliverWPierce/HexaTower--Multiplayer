use crate::{tile_mapping::TileId, tiles::TileType};

pub enum ActionEffect {
    DamagedPieceOnTile(TileId),
    ConvertedTileType { tile: TileId, new_tile: TileType },
}

pub struct ChangeLog(Vec<ActionEffect>);

impl ChangeLog {
    pub fn read(&self) -> &Vec<ActionEffect> {
        &self.0
    }

    pub fn write(&mut self, effect: ActionEffect) {
        self.0.push(effect);
    }
}
