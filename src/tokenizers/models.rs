use crate::tokenizers::extractors::TokenExtractorExtra;
use std::collections::HashMap;
use std::fmt::Debug;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct TokenData<'a> {
    pub data: &'a str,
    pub start: usize,
    pub end: usize,
    pub extra: &'a TokenExtractorExtra,
    pub groups: HashMap<&'a str, &'a str>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Token<'a> {
    /// A word
    Word(&'a str),
    /// A Space
    Space,
    /// String matching a citation regex from `reporters_db/reporters.json`.
    Citation(TokenData<'a>),
    /// Word containing a section symbol.
    Section(TokenData<'a>),
    /// Word matching "supra" with or without punctuation.
    Supra(TokenData<'a>),
    /// Word matching "id" or "ibid".
    Id(TokenData<'a>),
    /// Word matching a break between paragraphs.
    Paragraph(TokenData<'a>),
    /// Word matching one of the STOP_TOKENS.
    StopWord(TokenData<'a>),
}

impl Token<'_> {
    fn data(&self) -> &TokenData {
        match self {
            Token::Citation(data)
            | Token::StopWord(data)
            | Token::Supra(data)
            | Token::Id(data)
            | Token::Paragraph(data)
            | Token::Section(data) => data,
            Token::Word(_) | Token::Space => todo!("Words don't have data"),
        }
    }

    pub fn start(&self) -> usize {
        self.data().start
    }

    pub fn end(&self) -> usize {
        self.data().end
    }

    pub(crate) fn merge(&self, _other: &Self) -> Option<Self> {
        None
    }
}

pub trait TokenFactory: Clone + Debug {
    fn create<'a, 'b>(&'a self, data: TokenData<'b>) -> Token<'b>
    where
        'b: 'a;
}

#[derive(Debug, Clone)]
pub enum TokenFactories {
    Paragraph,
    Id,
    Supra,
    Citation,
    StopWord,
    Section,
}

impl TokenFactory for TokenFactories {
    fn create<'a, 'b>(&'a self, data: TokenData<'b>) -> Token<'b>
    where
        'b: 'a,
    {
        match self {
            TokenFactories::Paragraph => Token::Paragraph(data),
            TokenFactories::Id => Token::Id(data),
            TokenFactories::Supra => Token::Supra(data),
            TokenFactories::Citation => Token::Citation(data),
            TokenFactories::Section => Token::Section(data),
            TokenFactories::StopWord => Token::StopWord(data),
        }
    }
}

pub type Tokens<'a> = Vec<Token<'a>>;
