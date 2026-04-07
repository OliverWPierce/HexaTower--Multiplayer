use std::ops::{Add, Mul, Sub};

/// Tile ids start at zero and work counter clockwise from the origin, starting at the tile directly beneath the origin.
pub struct TileId(u32);

impl TileId {
    pub fn new(id: u32) -> Self {
        TileId(id)
    }
    pub fn id(&self) -> u32 {
        self.0
    }
}

/// "i" is north+/south-. "j" is northeast+/southwest-. "k" is northwest+/southeast-.
#[derive(Debug, Clone, Copy)]
pub struct HexVector2d {
    pub a: i32,
    pub b: i32,
}

pub const NORTH: HexVector2d = HexVector2d { a: 1, b: 0 };
pub const NORTH_EAST: HexVector2d = HexVector2d { a: 0, b: 1 };
pub const NORTH_WEST: HexVector2d = HexVector2d { a: 1, b: -1 };
pub const SOUTH: HexVector2d = HexVector2d { a: -1, b: 0 };
pub const SOUTH_EAST: HexVector2d = HexVector2d { a: -1, b: 1 };
pub const SOUTH_WEST: HexVector2d = HexVector2d { a: 0, b: -1 };

impl HexVector2d {
    pub fn scalar_mult(mut self, scalar: i32) {
        self.a *= scalar;
        self.b *= scalar;
    }
}

impl Mul<i32> for HexVector2d {
    type Output = Self;

    fn mul(self, rhs: i32) -> Self::Output {
        HexVector2d {
            a: self.a * rhs,
            b: self.b * rhs,
        }
    }
}

impl Mul<HexVector2d> for i32 {
    type Output = HexVector2d;

    fn mul(self, rhs: HexVector2d) -> Self::Output {
        HexVector2d {
            a: rhs.a * self,
            b: rhs.b * self,
        }
    }
}

impl Add for HexVector2d {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        HexVector2d {
            a: self.a + rhs.a,
            b: self.b + rhs.b,
        }
    }
}

impl Sub for HexVector2d {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        HexVector2d {
            a: self.a - rhs.a,
            b: self.b - rhs.b,
        }
    }
}

impl From<TileId> for HexVector2d {
    fn from(id: TileId) -> Self {
        if id.0 == 0 {
            return HexVector2d { a: 0, b: 0 };
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

        let edges_traversed = steps_on_ring / ring;
        let remaining_steps = steps_on_ring % ring;

        match edges_traversed {
            0 => SOUTH * ring + NORTH_EAST * remaining_steps,
            1 => SOUTH_EAST * ring + NORTH * remaining_steps,
            2 => NORTH_EAST * ring + NORTH_WEST * remaining_steps,
            3 => NORTH * ring + SOUTH_WEST * remaining_steps,
            4 => NORTH_WEST * ring + SOUTH * remaining_steps,
            5 => SOUTH_WEST * ring + SOUTH_EAST * remaining_steps,
            _ => panic!(),
        }
    }
}
