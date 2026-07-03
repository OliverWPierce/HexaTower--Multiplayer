use crate::{cards::CardId, markets::MarketId, pieces::ArchetypeId};

pub enum LinkedGamplayElement {
    Card(CardId),
    Piece(ArchetypeId),
    Market(MarketId),
}

pub enum ColorIndicators {
    Money,
    Damage,
    Health,
    GeneralHighlight,
    Unimportant,
}

pub enum SnippetContent {
    PlainText(String),
    Link(LinkedGamplayElement),
}

pub struct TextSnippet {
    pub special_color: Option<ColorIndicators>,
    pub content: SnippetContent,
}

impl TextSnippet {
    fn new(text: &str) -> Self {
        Self {
            special_color: None,
            content: SnippetContent::PlainText(text.into()),
        }
    }
}

pub trait ForensicDescribe {
    fn forensic_description(&self) -> Box<[TextSnippet]> {
        Box::new([TextSnippet {
            special_color: None,
            content: SnippetContent::PlainText("No description implemented yet.".into()),
        }])
    }
}
