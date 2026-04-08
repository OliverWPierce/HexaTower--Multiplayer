use std::{
    cmp::max,
    ops::{Add, Mul, Sub},
};

/// Tile ids start at zero and work counter clockwise from the origin, starting at the tile directly beneath the origin.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
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
    a: i32,
    b: i32,
}

pub const NORTH: HexVector2d = HexVector2d { a: 1, b: 0 };
pub const NORTH_EAST: HexVector2d = HexVector2d { a: 0, b: 1 };
pub const NORTH_WEST: HexVector2d = HexVector2d { a: 1, b: -1 };
pub const SOUTH: HexVector2d = HexVector2d { a: -1, b: 0 };
pub const SOUTH_EAST: HexVector2d = HexVector2d { a: -1, b: 1 };
pub const SOUTH_WEST: HexVector2d = HexVector2d { a: 0, b: -1 };

impl Mul<i32> for HexVector2d {
    type Output = Self;

    fn mul(self, rhs: i32) -> Self::Output {
        HexVector2d {
            a: self.a * rhs,
            b: self.b * rhs,
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

        let ring = (0.5 + ((4 * id - 1) as f32 / 12.0).powf(0.5)).floor() as i32;

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
            _ => unreachable!(),
        }
    }
}

impl From<HexVector2d> for TileId {
    fn from(vec: HexVector2d) -> Self {
        if vec.a.signum() != vec.b.signum() {
            let ring = max(vec.a.abs(), vec.b.abs());

            if vec.a.abs() >= vec.b.abs() {
                if vec.a > 0 {
                    let tile_at_ring_top = (3 * ring * (ring - 1) + 1) + 3 * ring;
                    TileId((tile_at_ring_top + vec.b.abs()) as u32)
                } else {
                    let tile_at_bottom = 3 * ring * (ring - 1) + 1;
                    TileId((tile_at_bottom + vec.b) as u32)
                }
            } else if vec.b > 0 {
                let tile_at_two_sixths = (3 * ring * (ring - 1) + 1) + 2 * ring;
                TileId((tile_at_two_sixths + vec.a) as u32)
            } else {
                let tile_at_five_sixths = (3 * ring * (ring - 1) + 1) + 5 * ring;
                TileId((tile_at_five_sixths - vec.a.abs()) as u32)
            }
        } else {
            // The normal hex_vec has a blindspot, so if we're in the blindspot, we'll just pick a new coordinate system.
            // The converted vec uses North/South and Southeast/Northwest.
            let converted_vec = (vec.a + vec.b, vec.b);
            let ring = converted_vec.0.abs();

            if ring == 0 {
                TileId(0)
            } else if converted_vec.0 > 0 {
                let tile_at_ring_top = (3 * ring * (ring - 1) + 1) + 3 * ring;
                TileId((tile_at_ring_top - converted_vec.1) as u32)
            } else {
                let tile_at_bottom_with_offset = 3 * ring * (ring + 1) + 1;
                TileId((tile_at_bottom_with_offset + converted_vec.1) as u32)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tile_conversions() {
        for id in 0..10000000 {
            let start_id = TileId::new(id);

            let end_id: TileId = HexVector2d::from(start_id).into();

            assert_eq!(
                start_id, end_id,
                "Fail. Started with id {start_id:?}, ended with {end_id:?}"
            );
        }
    }
    #[test]
    fn test_adjacencies() {
        for id in 0..61 {
            let northern_adjaceny: TileId = (HexVector2d::from(TileId::new(id)) + NORTH).into();
            println!("The tile north of {id} is {}", northern_adjaceny.id());
        }
    }
}
