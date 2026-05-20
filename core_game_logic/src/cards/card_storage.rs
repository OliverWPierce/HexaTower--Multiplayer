use bevy::ecs::resource::Resource;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::cards::CardFunction;

#[derive(Debug, PartialEq)]
pub struct LogicalCard {
    pub functionality: CardFunction,
}
/// The index of a card in the card assets.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Deserialize, Serialize)]
pub struct CardId(pub u32);

#[derive(Debug, Resource)]
pub struct CardDirectory(Box<[LogicalCard]>);

#[derive(Debug, Error)]
#[error{"Tried to retrieve card data using an invalid CardId {0:?}."}]
pub struct InvaildIDErr(pub CardId);

impl CardDirectory {
    pub fn get_card(&self, id: CardId) -> Result<&LogicalCard, InvaildIDErr> {
        self.0.get(id.0 as usize).ok_or(InvaildIDErr(id))
    }
    pub fn new(cards: Box<[LogicalCard]>) -> Self {
        CardDirectory(cards)
    }
}
