/// Tile ids start at zero and work counter clockwise from the origin, starting at the tile directly beneath the origin.
pub struct TileId(usize);

/// "i" is north+/south-. "j" is northeast+/southwest-. "k" is northwest+/southeast-.
#[derive(Debug)]
pub struct HexVector {
    i: i32,
    j: i32,
    k: i32,
}

impl TileId {
    pub fn new(id: usize) -> Self {
        TileId(id)
    }
    pub fn id(&self) -> usize {
        self.0
    }
}

impl From<TileId> for HexVector {
    fn from(id: TileId) -> Self {
        if id.0 == 0 {
            return HexVector { i: 0, j: 0, k: 0 };
        }

        let id = id.0 as i32;

        let ring = {
            let mut examined_ring = 1;
            loop {
                if id <= 3 * (examined_ring + 1) * examined_ring {
                    break examined_ring;
                }
                examined_ring += 1;
            }
        };

        let steps_on_ring = id - (3 * ring * (ring - 1) + 1);

        let edges_traversed = steps_on_ring / ring; // because these are i32 divisions, the answer is rounded down.

        let remaining_steps = steps_on_ring - edges_traversed * ring; // this math cannot be collapsed with the previous line, because we're dealing with integers.I know it looks weird.

        match edges_traversed {
            0 => HexVector {
                i: -ring,
                j: remaining_steps,
                k: 0,
            },
            1 => HexVector {
                i: -ring + remaining_steps,
                j: ring,
                k: 0,
            },
            2 => HexVector {
                i: 0,
                j: ring,
                k: -remaining_steps,
            },
            3 => HexVector {
                i: ring,
                j: -remaining_steps,
                k: 0,
            },
            4 => HexVector {
                i: -remaining_steps,
                j: 0,
                k: -ring,
            },
            5 => HexVector {
                i: 0,
                j: -ring,
                k: remaining_steps,
            },
            _ => panic!(),
        }
    }
}
