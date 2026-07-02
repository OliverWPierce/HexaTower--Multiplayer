use bevy::ecs::resource::Resource;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::cards::CardId;
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Deserialize, Serialize)]
pub struct MarketId(pub u32);

#[derive(Debug, Resource)]
pub struct MarketDirectory(Box<[LogicalMarket]>);

#[derive(Debug, Clone, Copy)]
pub struct CardPrice(pub u32);

#[derive(Debug, Copy, Clone)]
pub struct LogicalMarket(pub [(CardId, CardPrice); 3]);

#[derive(Debug, Copy, Clone, Deserialize, Serialize)]
pub enum SlotInMarket {
    First,
    Second,
    Third,
}

#[derive(Debug, Error)]
#[error{"Tried to retrieve market data using an invalid MarketId {0:?}."}]
pub struct InvaildIDErr(pub MarketId);

impl MarketDirectory {
    pub fn get_market(&self, id: MarketId) -> Result<&LogicalMarket, InvaildIDErr> {
        self.0.get(id.0 as usize).ok_or(InvaildIDErr(id))
    }

    pub(crate) fn new(markets: Box<[LogicalMarket]>) -> Self {
        Self(markets)
    }
}

impl LogicalMarket {
    pub fn get_card_and_price(&self, slot: SlotInMarket) -> &(CardId, CardPrice) {
        match slot {
            SlotInMarket::First => self.0.first().unwrap(),
            SlotInMarket::Second => self.0.get(1).unwrap(),
            SlotInMarket::Third => self.0.get(2).unwrap(),
        }
    }
}
