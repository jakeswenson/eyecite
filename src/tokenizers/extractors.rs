use crate::regexes;
use crate::tokenizers::models::{Token, TokenData, TokenFactories, TokenFactory};
use lazy_static::lazy_static;
use reporters_db::regexes::{RegexTemplate, ResolvedRegex};
use reporters_db::reporters::{reporters, Edition, EditionName};
use reporters_db::utils::process_variables;
use std::collections::{HashMap, HashSet};

#[derive(Default, Debug, Clone, Eq, PartialEq)]
pub struct TokenExtractorExtra {
    pub exact_editions: Vec<Edition>,
    pub variation_editions: Vec<Edition>,
    pub short: bool,
}

pub struct TokenMatch<'a> {
    pub(crate) regex_match: regex::Captures<'a>,
    pub(crate) names: Vec<&'a str>,
}

#[derive(Debug)]
pub struct TokenExtractor {
    pub regex: ResolvedRegex,
    pub token_factory: TokenFactories,
    pub extra: TokenExtractorExtra,
    pub strings: HashSet<String>,
    pub ignore_case: bool,
    built_regex: regex::Regex,
}

impl TokenExtractor {
    pub fn new(
        regex: ResolvedRegex,
        token_factory: TokenFactories,
        ignore_case: bool,
        strings: HashSet<String>,
        extra: TokenExtractorExtra,
    ) -> Self {
        let built_regex = regex::RegexBuilder::new(regex.value())
            .case_insensitive(ignore_case)
            .build()
            .expect("unable to build regex");

        Self {
            regex,
            token_factory,
            built_regex,
            ignore_case,
            strings,
            extra,
        }
    }

    /// Return match objects for all matches in text.
    pub fn get_matches<'a>(&'a self, text: &'a str) -> Vec<TokenMatch<'a>> {
        let matches = self.built_regex.captures_iter(text);
        let names: Vec<_> = self.built_regex.capture_names().flatten().collect();

        matches
            .into_iter()
            .map(|regex_match| TokenMatch {
                regex_match,
                names: names.clone(),
            })
            .collect()
    }

    /// For a given match object, return a Token.
    pub fn get_token<'a>(&'a self, token_match: TokenMatch<'a>) -> Token<'a> {
        let m = token_match.regex_match.get(1).unwrap();
        let start = m.start();
        let end = m.end();
        let data: &'a str = m.as_str();

        let extra: &'a TokenExtractorExtra = &self.extra;

        self.token_factory.create(TokenData {
            start,
            end,
            data,
            extra,
            groups: token_match
                .names
                .into_iter()
                .flat_map(|name| {
                    token_match
                        .regex_match
                        .name(name)
                        .map(move |m| (name, m.as_str()))
                })
                .collect(),
        })
    }
}

pub fn _populate_reporter_extractors() -> Vec<TokenExtractor> {
    let mut raw_regex_variables = reporters_db::regexes::raw_regexes();

    raw_regex_variables
        .get_mut("full_cite")
        .expect("full_cite should already exist")
        .add("", RegexTemplate::of("$volume $reporter,? $page"));

    raw_regex_variables
        .get_mut("page")
        .expect("page should already exist")
        .add("", RegexTemplate::of(regexes::PAGE_REGEX));

    let regex_vars = process_variables(raw_regex_variables);

    fn _substitute_edition(template: RegexTemplate, edition_name: &[EditionName]) -> RegexTemplate {
        let mut map: HashMap<String, RegexTemplate> = HashMap::new();
        let editions: Vec<String> = edition_name
            .iter()
            .map(|e| e.value())
            .map(regex::escape)
            .collect();
        map.insert("edition".into(), RegexTemplate::of(editions.join("|")));
        template.resolve(&map)
    }

    // # Extractors step one: add an extractor for each reporter string
    //
    //     # Build a lookup of regex -> edition.
    //     # Keys in this dict will be regular expressions to handle a
    //     # particular reporter string, like (simplified)
    //     # r"(?P<volume>\d+) (?P<reporter>U\.S\.) (?P<page>\d+)"
    #[derive(Default, Debug)]
    struct Lookup {
        editions: Vec<Edition>,
        variations: Vec<Edition>,
        strings: HashSet<String>,
        short: bool,
    }

    fn _add_regex(
        reporters: &[EditionName],
        edition: &Edition,
        regex: ResolvedRegex,
        is_short: bool,
        result: &mut HashMap<ResolvedRegex, Lookup>,
        func: fn(&mut Lookup) -> &mut Vec<Edition>,
    ) {
        let entry = result.entry(regex.clone()).or_default();

        entry.short = is_short;

        let result = func(entry);
        result.push(edition.clone());

        let has_strings = regex.value().contains(&regex::escape(reporters[0].value()));

        if has_strings {
            let cloned = reporters.iter().map(|r| r.value().into());

            for s in cloned {
                entry.strings.insert(s);
            }
        }
    }

    fn _add_regexes(
        regex_templates: &[RegexTemplate],
        edition_name: EditionName,
        edition: Edition,
        variations: Vec<EditionName>,
        variables: &HashMap<String, RegexTemplate>,
        result: &mut HashMap<ResolvedRegex, Lookup>,
    ) {
        for template in regex_templates {
            let template = reporters_db::utils::recursive_substitute(template.clone(), variables);
            let arg = vec![edition_name.clone()];
            let regex = _substitute_edition(template.clone(), arg.as_slice())
                .resolved()
                .expect("edition should have been the last thing to resolve");

            let short_regex = regexes::short_cite_re(regex.value());
            _add_regex(arg.as_slice(), &edition, regex, false, result, |l| {
                &mut l.editions
            });
            _add_regex(arg.as_slice(), &edition, short_regex, true, result, |l| {
                &mut l.editions
            });

            if !variations.is_empty() {
                let variation_regex = _substitute_edition(template, variations.as_slice())
                    .resolved()
                    .expect("edition should have been the last thing to resolve");

                let short_variation_regex = regexes::short_cite_re(variation_regex.value());

                _add_regex(
                    variations.as_slice(),
                    &edition,
                    variation_regex,
                    false,
                    result,
                    |l| &mut l.variations,
                );
                _add_regex(
                    variations.as_slice(),
                    &edition,
                    short_variation_regex,
                    false,
                    result,
                    |l| &mut l.variations,
                );
            }
        }
    }

    let mut editions_by_regex: HashMap<ResolvedRegex, Lookup> = HashMap::new();

    // # add reporters.json:
    let reporters = reporters();
    for (_key, cluster) in reporters {
        for source in cluster {
            let variations = source.variations;

            for (edition_name, edition_data) in source.editions {
                let regexes = edition_data
                    .regexes
                    .clone()
                    .unwrap_or_else(|| vec![RegexTemplate::of("$full_cite")]);

                let edition_variations: Vec<_> = variations
                    .iter()
                    .filter(|(_, v)| edition_name == (*v).clone())
                    .map(|(k, _)| k.clone())
                    .collect();

                _add_regexes(
                    &regexes,
                    edition_name,
                    edition_data,
                    edition_variations,
                    &regex_vars,
                    &mut editions_by_regex,
                )
            }
        }
    }

    // # add laws.json

    // # add journals.json

    let mut extractors = Vec::new();

    // # Add each regex to EXTRACTORS
    for (regex, lookup) in editions_by_regex {
        extractors.push(TokenExtractor::new(
            regexes::nonalphanum_boundaries_re(&regex),
            TokenFactories::Citation,
            false,
            lookup.strings,
            TokenExtractorExtra {
                exact_editions: lookup.editions,
                variation_editions: lookup.variations,
                short: lookup.short,
            },
        ));
    }

    extractors.push(TokenExtractor::new(
        ResolvedRegex::of(regexes::ID_REGEX.into()),
        TokenFactories::Id,
        true,
        vec!["id.".into(), "ibid.".into()].into_iter().collect(),
        Default::default(),
    ));

    extractors.push(TokenExtractor::new(
        ResolvedRegex::of(regexes::SUPRA_REGEX.into()),
        TokenFactories::Supra,
        true,
        vec!["supra".into()].into_iter().collect(),
        Default::default(),
    ));

    extractors.push(TokenExtractor::new(
        ResolvedRegex::of(regexes::PARAGRAPH_REGEX.into()),
        TokenFactories::Paragraph,
        false,
        Default::default(),
        Default::default(),
    ));

    extractors.push(TokenExtractor::new(
        ResolvedRegex::of(regexes::STOP_WORD_REGEX.into()),
        TokenFactories::StopWord,
        true,
        regexes::STOP_WORDS.into_iter().map(|s| s.into()).collect(),
        Default::default(),
    ));

    extractors.push(TokenExtractor::new(
        ResolvedRegex::of(regexes::SECTION_REGEX.into()),
        TokenFactories::Section,
        false,
        vec!["§"].into_iter().map(|s| s.into()).collect(),
        Default::default(),
    ));

    extractors
}

lazy_static! {
    pub static ref EXTRACTORS: Vec<TokenExtractor> = _populate_reporter_extractors();
}

#[cfg(test)]
mod tests {
    use super::EXTRACTORS;

    #[test]
    fn build_extractors() {
        assert_eq!(EXTRACTORS.is_empty(), false);
    }
}
