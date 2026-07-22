use std::{
    cmp::max,
    f32,
    ops::{Add, Mul, Sub},
};

use bevy::{
    ecs::{component::Component, resource::Resource},
    math::{Vec2, Vec3},
};
use serde::{Deserialize, Serialize};

use crate::InvalidIdErr;

/// Tile ids start at zero and work counter clockwise from the origin, starting at the tile directly beneath the origin.
#[derive(Debug, Component, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Deserialize, Serialize)]
#[component(immutable)]
pub struct TileId(u32);

#[derive(Debug, Resource, Clone, Copy)]
pub struct TileIdServer {
    /// the number of tiles on the board, as one would count them. (so, if the maximum id is 59, this number is 60.)
    pub total_tiles_on_board: u32,
}

impl TileId {
    pub fn id(&self) -> u32 {
        self.0
    }
}

impl TileIdServer {
    pub fn construct_tile_id(&self, id: u32) -> Result<TileId, InvalidIdErr<TileId>> {
        if (0..self.total_tiles_on_board).contains(&id) {
            Ok(TileId(id))
        } else {
            Err(InvalidIdErr::new(TileId(id)))
        }
    }
}

pub const SQRT_3: f32 = 1.7320508;

/// The total number of tiles on the board, including the tile with id zero. This is based on the number of rings the board was created with, with tile zero counted as ring zero. (ie. the first ring to actually look like a ring is ring 1.)
pub fn tiles_on_board(rings_on_board: u32) -> u32 {
    (3 * (rings_on_board + 1) * rings_on_board) + 1
}

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

impl From<u32> for HexVector2d {
    fn from(id: u32) -> Self {
        if id == 0 {
            return HexVector2d { a: 0, b: 0 };
        }

        let id = id as i32;

        let ring = {
            let mut examined_ring = 1;
            // this loop can be avoided using floating points and square roots, but then percision suffers and becomes per-platform.
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
            _ => unreachable!(),
        }
    }
}

impl From<HexVector2d> for u32 {
    fn from(vec: HexVector2d) -> Self {
        if vec.a.signum() != vec.b.signum() {
            let ring = max(vec.a.abs(), vec.b.abs());

            if vec.a.abs() >= vec.b.abs() {
                if vec.a > 0 {
                    let tile_at_ring_top = (3 * ring * (ring - 1) + 1) + 3 * ring;
                    (tile_at_ring_top + vec.b.abs()) as u32
                } else {
                    let tile_at_bottom = 3 * ring * (ring - 1) + 1;
                    (tile_at_bottom + vec.b) as u32
                }
            } else if vec.b > 0 {
                let tile_at_two_sixths = (3 * ring * (ring - 1) + 1) + 2 * ring;
                (tile_at_two_sixths + vec.a) as u32
            } else {
                let tile_at_five_sixths = (3 * ring * (ring - 1) + 1) + 5 * ring;
                (tile_at_five_sixths - vec.a.abs()) as u32
            }
        } else {
            // The normal hex_vec has a blindspot, so if we're in the blindspot, we'll just pick a new coordinate system.
            // The converted vec uses North/South and Southeast/Northwest.
            let converted_vec = (vec.a + vec.b, vec.b);
            let ring = converted_vec.0.abs();

            if ring == 0 {
                0
            } else if converted_vec.0 > 0 {
                let tile_at_ring_top = (3 * ring * (ring - 1) + 1) + 3 * ring;
                (tile_at_ring_top - converted_vec.1) as u32
            } else {
                let tile_at_bottom_with_offset = 3 * ring * (ring + 1) + 1;
                (tile_at_bottom_with_offset + converted_vec.1) as u32
            }
        }
    }
}

impl From<HexVector2d> for Vec2 {
    fn from(hex_vec: HexVector2d) -> Self {
        let vec_from_a = Vec2 {
            x: 0.0,
            y: hex_vec.a as f32 * SQRT_3,
        };
        let vec_from_b = Vec2 {
            x: 1.5 * hex_vec.b as f32,
            y: SQRT_3 / 2.0 * hex_vec.b as f32, // FIX ME
        };

        vec_from_a + vec_from_b
    }
}

impl From<HexVector2d> for Vec3 {
    fn from(value: HexVector2d) -> Self {
        let v2 = Vec2::from(value);

        Self {
            x: v2.x,
            y: 0.0,
            z: v2.y,
        }
    }
}

impl HexVector2d {
    pub fn to_valid_tile_id(self, server: &TileIdServer) -> Option<TileId> {
        server.construct_tile_id(self.into()).ok()
    }

    pub fn adjacencies(self) -> [Self; 6] {
        [
            self + NORTH,
            self + SOUTH,
            self + NORTH_EAST,
            self + SOUTH_EAST,
            self + NORTH_WEST,
            self + SOUTH_WEST,
        ]
    }
}

impl From<TileId> for HexVector2d {
    fn from(value: TileId) -> Self {
        value.0.into()
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tile_conversions() {
        for id in 0..10000 {
            let start_id = id;

            let end_id: u32 = HexVector2d::from(start_id).into();

            assert_eq!(
                start_id, end_id,
                "Fail. Started with id {start_id:?}, ended with {end_id:?}"
            );
        }
    }
    #[test]
    fn test_adjacencies() {
        for id in 0..61 {
            let northern_adjaceny: u32 = (HexVector2d::from(id) + NORTH).into();
            println!("The tile north of {id} is {}", northern_adjaceny);
        }
    }
}
