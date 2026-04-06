mod tile_mapping;

pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

#[cfg(test)]
mod tests {
    use crate::tile_mapping::{HexVector, TileId};

    #[test]
    fn it_works() {
        let tiles = [
            TileId::new(0),
            TileId::new(6),
            TileId::new(29),
            TileId::new(18),
            TileId::new(36),
            TileId::new(37),
            TileId::new(38),
        ];

        for tile in tiles {
            println!();

            println!("Tile {} coordinates are:", tile.id());
            println!("{:?}", HexVector::from(tile));
        }
        panic!()
    }
}
