use crate::tokenizers::extractors::TokenExtractor;
use crate::tokenizers::models::{Token, Tokens};
use crate::EyeciteError;
use std::collections::HashMap;

pub mod extractors;
pub mod models;

pub trait Tokenizer<'a> {
    fn get_extractors(&'a self, text: &'a str)
        -> Box<dyn Iterator<Item = &'a TokenExtractor> + 'a>;

    fn extract_tokens(&'a self, text: &'a str) -> Vec<Token<'a>> {
        self.get_extractors(text)
            .flat_map(|e| e.get_matches(text).into_iter().map(move |m| (e, m)))
            .map(|(e, m)| e.get_token(m))
            .collect()
    }

    fn tokenize(&'a self, text: &'a str) -> (Tokens<'a>, Vec<(usize, Token<'a>)>) {
        let mut citation_tokens: Vec<(usize, Token)> = Vec::new();
        let mut all_tokens: Vec<Token> = Vec::new();

        let tokens = self.extract_tokens(text);
        let mut last_token: Option<Token> = None;
        let mut offset: usize = 0;

        /// Split text into words, treating whitespace as a word, and append
        /// to tokens. NOTE this is a significant portion of total runtime of
        /// get_citations(), so benchmark if changing
        fn append_text<'a>(tokens: &mut Vec<Token<'a>>, text: &'a str) {
            for part in text.split(' ') {
                // TODO: maybe filter repeated strings which will be empty
                if !part.is_empty() {
                    tokens.push(Token::Word(part));
                    tokens.push(Token::Space);
                } else {
                    tokens.push(Token::Space);
                }
            }

            tokens.pop(); // remove final extra space
        }

        for token in tokens {
            if let Some(last) = last_token.as_mut() {
                // Sometimes the exact same cite is matched by two different
                // regexes. Attempt to merge rather than discarding one or the
                // other:
                let merged = last.merge(&token);
                if let Some(merged) = merged {
                    citation_tokens.pop();
                    all_tokens.pop();

                    citation_tokens.push((all_tokens.len(), merged.clone()));
                    all_tokens.push(merged);

                    continue;
                }
            }

            if offset > token.start() {
                continue;
            }

            if offset < token.start() {
                // capture plain text before each match
                append_text(&mut all_tokens, &text[offset..token.start()]);
            }

            // capture match
            citation_tokens.push((all_tokens.len(), token.clone()));
            all_tokens.push(token.clone());
            offset = token.end();
            last_token = Some(token)
        }

        // capture plain text after final match
        if offset < text.len() {
            append_text(&mut all_tokens, &text[offset..]);
        }

        (all_tokens, citation_tokens)
    }
}

pub struct Ahocorasick<'a> {
    extractors: HashMap<String, Vec<&'a TokenExtractor>>,
    strings: Vec<String>,
    corasick: daachorse::DoubleArrayAhoCorasick,
}

impl<'a> Ahocorasick<'a> {
    pub fn new(items: &'a [TokenExtractor]) -> Result<Self, EyeciteError> {
        let mut extractors: HashMap<String, Vec<_>> = HashMap::new();

        for e in items {
            for s in e.strings.iter().cloned() {
                let _v = extractors.entry(s).or_default().push(e);
            }
        }

        let strings: Vec<_> = extractors.keys().cloned().collect();

        let corasick = daachorse::DoubleArrayAhoCorasickBuilder::new().build(strings.as_slice())?;

        Ok(Self {
            extractors,
            strings,
            corasick,
        })
    }
}

impl<'a> Tokenizer<'a> for Ahocorasick<'a> {
    fn get_extractors(
        &'a self,
        text: &'a str,
    ) -> Box<dyn Iterator<Item = &'a TokenExtractor> + 'a> {
        Box::new(self.corasick.find_iter(text).flat_map(|m| {
            self.extractors[self.strings[m.value()].as_str()]
                .as_slice()
                .iter()
                .copied()
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::extractors::EXTRACTORS;
    use crate::tokenizers::extractors::TokenExtractorExtra;
    use crate::tokenizers::models::{Token, TokenData};
    use crate::tokenizers::{Ahocorasick, Tokenizer};
    use reporters_db::laws::NaiveDateTime;
    use reporters_db::reporters::Edition;
    use std::str::FromStr;

    #[test]
    fn tokenize() {
        let tokenizer = Ahocorasick::new(EXTRACTORS.as_slice()).unwrap();

        let (all_tokens, tokens) = tokenizer.tokenize("See Roe v. Wade, 410 U. S. 113 (1973)");

        let stop_word_extra = TokenExtractorExtra {
            exact_editions: vec![],
            variation_editions: vec![],
            short: false,
        };

        let edition_extra = TokenExtractorExtra {
            exact_editions: vec![],
            variation_editions: vec![Edition {
                end: None,
                start: Some(NaiveDateTime::from_str("1875-01-01T00:00:00").unwrap()),
                regexes: None,
            }],
            short: false,
        };

        let see_token = Token::StopWord(TokenData {
            data: "See",
            start: 0,
            end: 3,
            extra: &stop_word_extra,
            groups: vec![("stop_word".into(), "See")].into_iter().collect(),
        });

        let v_token = Token::StopWord(TokenData {
            data: "v.",
            start: 8,
            end: 10,
            extra: &stop_word_extra,
            groups: vec![("stop_word".into(), "v")].into_iter().collect(),
        });

        let us_citation = Token::Citation(TokenData {
            data: "410 U. S. 113",
            start: 17,
            end: 30,
            extra: &edition_extra,
            groups: vec![
                ("reporter".into(), "U. S."),
                ("volume".into(), "410"),
                ("page".into(), "113"),
            ]
            .into_iter()
            .collect(),
        });

        let expected_tokens = vec![
            see_token.clone(),
            Token::Space,
            Token::Word("Roe"),
            Token::Space,
            v_token.clone(),
            Token::Space,
            Token::Word("Wade,"),
            Token::Space,
            us_citation.clone(),
            Token::Space,
            Token::Word("(1973)"),
        ];

        assert_eq!(all_tokens, expected_tokens);
        assert_eq!(tokens, vec![(0, see_token), (4, v_token), (8, us_citation)]);
    }
}
