mod tile_mapping;

pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

#[cfg(test)]
mod tests {
    use std::cmp::max;

    use crate::tile_mapping::{HexVector2d, TileId};

    #[test]
    fn it_works() {
        // for id in 0..1000 {
        //     let looping_ring = {
        //         let mut examined_ring = 0;
        //         loop {
        //             if id <= 3 * (examined_ring + 1) * examined_ring {
        //                 break examined_ring;
        //             }
        //             examined_ring += 1;
        //         }
        //     };

        //     let id = id as f32;

        //     let math_ring = (0.5886 * id.powf(0.4968)).round() as i32;

        //     assert_eq!(looping_ring, math_ring)
        // }

        for id in 0..1000 {
            let predicted_ring = {
                let mut examined_ring = 0;
                loop {
                    if id <= 3 * (examined_ring + 1) * examined_ring {
                        break examined_ring;
                    }
                    examined_ring += 1;
                }
            };

            let hex_vec: HexVector2d = TileId::new(id).into();

            let maybe_ring = {
                if hex_vec.a.signum() != hex_vec.b.signum() {
                    max(hex_vec.a.abs(), hex_vec.b.abs())
                } else {
                    let new_vec = (hex_vec.a + hex_vec.b, hex_vec.b);
                    max(new_vec.0.abs(), new_vec.1.abs())
                }
            };

            if maybe_ring - (predicted_ring as i32) != 0 {
                println!(
                    "id: {}, ring is off by: {}",
                    id,
                    (maybe_ring - predicted_ring as i32)
                );

                if hex_vec.a.signum() == hex_vec.b.signum() {
                    println!("signs are equal: YES");
                } else {
                    println!("signs are equal: NO");
                }
            }
        }

        panic!()
    }
}
