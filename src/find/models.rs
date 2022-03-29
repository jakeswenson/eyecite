use crate::tokenizers::models::Token;
use std::collections::{HashMap, HashSet};

#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, Hash, Clone)]
pub enum CitationMetadata<'a> {
    Parenthetical(&'a str),
    PinCite(&'a str),
    Year(&'a str),
    Court(&'a str),
    Plaintiff(&'a str),
    Defendant(&'a str),
    Extra(&'a str),
    AntecedentGuess(&'a str),
    Volume(&'a str),
}

pub struct CitationSource<'a> {
    pub token: Token<'a>,
    pub index: usize,
    pub span_start: Option<usize>,
    pub span_end: Option<usize>,
    pub groups: HashMap<String, String>,
    pub metadata: HashSet<CitationMetadata<'a>>,
}

pub enum Citation<'a> {
    Resource {
        source: CitationSource<'a>,
        pin_cite: Option<&'a str>,
        year: Option<&'a str>,
    },
    Law {
        source: CitationSource<'a>,
        publisher: Option<&'a str>,
        day: Option<&'a str>,
        month: Option<&'a str>,
    },
    Journal {
        source: CitationSource<'a>,
    },
    Case {
        source: CitationSource<'a>,
        pin_cite: Option<&'a str>,
        year: Option<&'a str>,
        court: Option<&'a str>,
    },
    /**
    Convenience class which represents a standard, fully named citation,
    i.e., the kind of citation that marks the first time a document is cited.

    Example:
    ```text
    Adarand Constructors, Inc. v. Peña, 515 U.S. 200, 240
    ```
    **/
    FullCase {
        source: CitationSource<'a>,
        pin_cite: Option<&'a str>,
        year: Option<&'a str>,
        court: Option<&'a str>,
        plaintiff: Option<&'a str>,
        defendant: Option<&'a str>,
        extra: Option<&'a str>,
    },
    /**
    Convenience class which represents a short form citation, i.e., the kind
    of citation made after a full citation has already appeared. This kind of
    citation lacks a full case name and usually has a different page number
    than the canonical citation.

    Examples:
    ```text
    Adarand, 515 U.S., at 241
    Adarand, 515 U.S. at 241
    515 U.S., at 241
    ```
    **/
    ShortCase {
        source: CitationSource<'a>,
        pin_cite: Option<&'a str>,
        year: Option<&'a str>,
        court: Option<&'a str>,
        antecedent_guess: Option<&'a str>,
    },
    /**
    Convenience class which represents a 'supra' citation, i.e., a citation
    to something that is above in the document. Like a short form citation,
    this kind of citation lacks a full case name and usually has a different
    page number than the canonical citation.


    Examples:
    ```text
    Adarand, supra, at 240
    Adarand, 515 supra, at 240
    Adarand, supra, somethingelse
    Adarand, supra. somethingelse
    ```
     **/
    Supra {
        source: CitationSource<'a>,
        pin_cite: Option<&'a str>,
        year: Option<&'a str>,
        court: Option<&'a str>,
        antecedent_guess: Option<&'a str>,
        volume: Option<&'a str>,
    },
    /**
    Convenience class which represents an 'id' or 'ibid' citation, i.e., a
    citation to the document referenced immediately prior. An 'id' citation is
    unlike a regular citation object since it has no knowledge of its reporter,
    volume, or page. Instead, the only helpful information that this reference
    possesses is a record of the pin cite after the 'id' token.

    Example: `"... foo bar," id., at 240`
    **/
    Id {
        source: CitationSource<'a>,
        pin_cite: Option<&'a str>,
        year: Option<&'a str>,
        court: Option<&'a str>,
        antecedent_guess: Option<&'a str>,
        volume: Option<&'a str>,
    },
    /**
    Convenience class which represents an unknown citation. A recognized
    citation should theoretically be parsed as a CaseCitation, FullLawCitation,
    or a FullJournalCitation. If it's something else, this class serves as
    a naive catch-all.
    **/
    Unknown {
        source: CitationSource<'a>,
    },
}
