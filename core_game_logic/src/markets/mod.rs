use crate::{cards::CardId, directories::InvalidIdErr};

pub struct Market([(CardId, u32); 3]);

#[derive(Debug)]
pub struct MarketId(pub u32);

struct MarketDirectory(Box<[Market]>);

impl crate::directories::DirectoryId for MarketId {}

impl crate::directories::Directory for MarketDirectory {
    type Id = MarketId;

    type Contains = Market;

    fn get(
        &self,
        id: Self::Id,
    ) -> Result<&Self::Contains, crate::directories::InvalidIdErr<Self::Id>> {
        self.0.get(id.0 as usize).ok_or(InvalidIdErr(id))
    }
}
