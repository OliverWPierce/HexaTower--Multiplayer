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

#[derive(Debug, Error)]
#[error{"Tried to retrieve market data using an invalid MarketId {0:?}."}]
pub struct InvaildIDErr(pub MarketId);

impl MarketDirectory {
    pub fn get_market(&self, id: MarketId) -> Result<&LogicalMarket, InvaildIDErr> {
        self.0.get(id.0 as usize).ok_or(InvaildIDErr(id))
    }
}
