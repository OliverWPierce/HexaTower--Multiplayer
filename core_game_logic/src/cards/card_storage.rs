use bevy::ecs::resource::Resource;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::cards::CardFunction;

/// This struct contains all the data that makes one card fundamentally different from another card.
#[derive(Debug, PartialEq)]
pub struct CardAsset {
    pub(crate) card_fxn: CardFunction,
}
/// The index of a card in the card assets.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Deserialize, Serialize)]
pub struct CardId(pub u32);

#[derive(Debug, Resource)]
pub struct CardAssets(Box<[CardAsset]>);

#[derive(Debug, Error)]
#[error{"Tried to retrieve card data using an invalid CardId."}]
pub struct InvaildIDErr(pub CardId);

impl CardAssets {
    pub fn get_card(&self, id: CardId) -> Result<&CardAsset, InvaildIDErr> {
        self.0.get(id.0 as usize).ok_or(InvaildIDErr(id))
    }
}
#[derive(Debug, Default)]
pub struct CardAssetsConstructor(Vec<CardAsset>);

impl From<CardAssetsConstructor> for CardAssets {
    fn from(value: CardAssetsConstructor) -> Self {
        CardAssets(value.0.into_boxed_slice())
    }
}

impl CardAssetsConstructor {
    /// tries to add a card if it isn't in the vector yet. Returns the index of the card.
    pub fn add_card(&mut self, card: CardAsset) -> CardId {
        let identical_card = self
            .0
            .iter()
            .enumerate()
            .find(|(_index, stored_card)| **stored_card == card);

        if let Some((index, _)) = identical_card {
            CardId(index as u32)
        } else {
            self.0.push(card);

            CardId(self.0.len() as u32 - 1)
        }
    }
}
#[cfg(test)]
mod tests {
    use bevy::ecs::world::World;

    use crate::cards::card_storage::{CardAsset, CardAssets, CardAssetsConstructor, CardId};

    #[test]
    fn add_cards_to_world() {
        let mut constructor = CardAssetsConstructor(Vec::new());

        let index1 = constructor.add_card(CardAsset {
            card_fxn: crate::cards::CardFunction::Ex2,
        });
        let index2 = constructor.add_card(CardAsset {
            card_fxn: crate::cards::CardFunction::TileConversionToSingleType {
                selection_bounds: 0..4,
                target_type: crate::tiles::TileType::Ex1,
            },
        });
        let index3 = constructor.add_card(CardAsset {
            card_fxn: crate::cards::CardFunction::TileConversionToSingleType {
                selection_bounds: 0..2,
                target_type: crate::tiles::TileType::Ex1,
            },
        });
        let index4 = constructor.add_card(CardAsset {
            card_fxn: crate::cards::CardFunction::Ex2,
        });
        let index5 = constructor.add_card(CardAsset {
            card_fxn: crate::cards::CardFunction::TileConversionToSingleType {
                selection_bounds: 0..4,
                target_type: crate::tiles::TileType::Basic,
            },
        });

        assert_eq!(index1, CardId(0));
        assert_eq!(index2, CardId(1));
        assert_eq!(index3, CardId(2));
        assert_eq!(index4, CardId(0));
        assert_eq!(index5, CardId(3));

        let mut world = World::new();

        world.insert_resource::<CardAssets>(constructor.into());
    }
}
