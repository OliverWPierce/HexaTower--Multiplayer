use bevy::ecs::resource::Resource;
use serde::{Deserialize, Serialize};

use crate::{cards::CardFunction, directories::InvalidIdErr};

/// This struct contains all the data that makes one card fundamentally different from another card.
#[derive(Debug, PartialEq)]
pub struct LogicalCard {
    pub card_fxn: CardFunction,
}
/// The index of a card in the card assets.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Deserialize, Serialize)]
pub struct CardId(pub u32);

impl crate::directories::DirectoryId for CardId {}

#[derive(Debug, Resource)]
pub struct CardDirectory(Box<[LogicalCard]>);

impl crate::directories::Directory for CardDirectory {
    type Id = CardId;

    type Contains = LogicalCard;

    fn get(
        &self,
        id: Self::Id,
    ) -> Result<&Self::Contains, crate::directories::InvalidIdErr<Self::Id>> {
        self.0.get(id.0 as usize).ok_or(InvalidIdErr(id))
    }
}

#[derive(Debug, Default)]
pub struct CardAssetsConstructor(Vec<LogicalCard>);

impl From<CardAssetsConstructor> for CardDirectory {
    fn from(value: CardAssetsConstructor) -> Self {
        CardDirectory(value.0.into_boxed_slice())
    }
}

impl CardAssetsConstructor {
    /// tries to add a card if it isn't in the vector yet. Returns the index of the card.
    pub fn add_card(&mut self, card: LogicalCard) -> CardId {
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

    use crate::cards::card_storage::{CardAssetsConstructor, CardDirectory, CardId, LogicalCard};

    #[test]
    fn add_cards_to_world() {
        let mut constructor = CardAssetsConstructor(Vec::new());

        let index1 = constructor.add_card(LogicalCard {
            card_fxn: crate::cards::CardFunction::Ex2,
        });
        let index2 = constructor.add_card(LogicalCard {
            card_fxn: crate::cards::CardFunction::TileConversionToSingleType {
                selection_bounds: 0..4,
                target_type: crate::tiles::TileType::Ex1,
            },
        });
        let index3 = constructor.add_card(LogicalCard {
            card_fxn: crate::cards::CardFunction::TileConversionToSingleType {
                selection_bounds: 0..2,
                target_type: crate::tiles::TileType::Ex1,
            },
        });
        let index4 = constructor.add_card(LogicalCard {
            card_fxn: crate::cards::CardFunction::Ex2,
        });
        let index5 = constructor.add_card(LogicalCard {
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

        world.insert_resource::<CardDirectory>(constructor.into());
    }
}
