use crate::{
    cards::CardId, markets::MarketId, pieces::ArchetypeId, players::PlayerId, tiles::TileType,
};

pub enum LinkedGamplayElement {
    Card(CardId),
    Piece(ArchetypeId),
    Market(MarketId),
    Tile(TileType),
    Player(PlayerId),
}

pub enum ColorIndicators {
    Money,
    Damage,
    Health,
    GeneralHighlight,
    Unimportant,
}

pub enum TextSnippet {
    PlainText {
        text: String,
        color: Option<ColorIndicators>,
    },
    Link(LinkedGamplayElement),
}

impl TextSnippet {
    pub fn new_basic_text(text: &str) -> Self {
        Self::PlainText {
            text: text.into(),
            color: None,
        }
    }
}

pub trait ForensicDescribe {
    fn forensic_description(&self) -> Box<[TextSnippet]> {
        Box::new([TextSnippet::new_basic_text(
            "No description implemented yet.",
        )])
    }
}
