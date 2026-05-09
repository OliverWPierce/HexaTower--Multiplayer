use crate::{tile_based_actions::TileActionProcessCache, tile_mapping::TileId, tiles::TileType};
#[derive(Debug, PartialEq)]
pub enum ActionEffect {
    DamagedPieceOnTile(TileId),
    ConvertedTileType { tile: TileId, new_type: TileType },
}
#[derive(Default)]
pub struct ChangeLog(Vec<ActionEffect>);

impl ChangeLog {
    pub fn read(&self) -> &Vec<ActionEffect> {
        &self.0
    }

    pub fn write(&mut self, effect: ActionEffect) {
        self.0.push(effect);
    }
}

pub enum ActionProcessCache {
    TileAction(TileActionProcessCache),
}
