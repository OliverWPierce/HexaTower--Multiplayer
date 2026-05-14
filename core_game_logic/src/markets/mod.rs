use thiserror::Error;

use crate::cards::CardId;
#[derive(Debug)]
pub struct MarketId(pub u32);

pub struct MarketDirectory(Box<[Market]>);

#[derive(Debug, Copy, Clone)]
pub struct Market(pub [(CardId, u32); 3]);

#[derive(Debug, Error)]
#[error{"Tried to retrieve market data using an invalid MarketId {0:?}."}]
pub struct InvaildIDErr(pub MarketId);

impl MarketDirectory {
    fn get_market(&self, id: MarketId) -> Result<&Market, InvaildIDErr> {
        self.0.get(id.0 as usize).ok_or(InvaildIDErr(id))
    }
}
