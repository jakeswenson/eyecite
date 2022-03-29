/*!
# *** Tokenizer regexes: ***
Regexes used from tokenizers.py
 */

use const_format::formatcp;
use reporters_db::regexes::ResolvedRegex;

/**!
We need a regex that matches roman numerals but not the empty string,
without using lookahead assertions that aren't supported by hyperscan.
We *don't* want to match roman numerals 'v', 'l', or 'c', or numerals over
200, or uppercase, as these are usually false positives
(see https://github.com/freelawproject/eyecite/issues/56 ).
Match roman numerals 1 to 199 except for 5, 50, 100:
 */
pub const ROMAN_NUMERAL_REGEX: &str = formatcp!(
    "{}|{}|{}",
    // 10-199, but not 50-59 or 100-109 or 150-159:
    r"c?(?:xc|xl|l?x{1,3})(?:ix|iv|v?i{0,3})",
    // 1-9, 51-59, 101-109, 151-159, but not 5, 55, 105, 155:
    r"(?:c?l?)(?:ix|iv|v?i{1,3})",
    // 55, 105, 150, 155:
    r"(?:lv|cv|cl|clv)",
);

/**!
Page number regex to match one of the following:
(ordered in descending order of likelihood)
 1) A plain digit. E.g. "123"
 2) A roman numeral.
 */
pub const PAGE_NUMBER_REGEX: &str = formatcp!(r"(?:\d+|{})", ROMAN_NUMERAL_REGEX);

pub const PAGE_REGEX: &str = formatcp!("(?P<page>{})", PAGE_NUMBER_REGEX);

/// Wrap regex with space or end of string.
macro_rules! space_boundaries_re {
    ($regex:expr) => {
        formatcp!(r"(?:^|\s)({})(?:\s|$)", $regex)
    };
}

/// Wrap regex with punctuation pattern.
macro_rules! strip_punctuation_re {
    ($regex:expr) => {
        formatcp!(
            r"{PUNCTUATION_REGEX}{}{PUNCTUATION_REGEX}",
            $regex,
            PUNCTUATION_REGEX = PUNCTUATION_REGEX
        )
    };
}

/// Regex for IdToken
pub const ID_REGEX: &str = space_boundaries_re!(r"id\.,?|ibid\.");

/// Regex for SupraToken
pub const SUPRA_REGEX: &str = space_boundaries_re!(strip_punctuation_re!("supra"));

/// Regex for ParagraphToken
pub const PARAGRAPH_REGEX: &str = r"(\n)";

/// Wrap regex with punctuation pattern.
macro_rules! join_with {
    ($sep:literal, [ $s:literal ] ) => { $s };
    ($sep:literal, [ $s:literal, $t:literal ] ) => { formatcp!("{}{}{}", $s, $sep, $t) };
    ($sep:literal, [ $s:literal, $($rest:literal),+ ] ) => {
        formatcp!("{}{}{}", $s, $sep, join_with!($sep, [ $( $rest ),* ]))
    };
    ($sep:literal, [ $s:literal, $($rest:literal),+ , ] ) => {
        formatcp!("{}{}{}", $s, $sep, join_with!($sep, [ $( $rest ),* ]))
    };
}

/// Regex for StopWordToken
pub const STOP_WORDS_JOINED: &str = join_with!(
    "|",
    [
        "v",
        "re",
        "parte",
        "denied",
        "citing",
        "aff'd",
        "affirmed",
        "remanded",
        "see",
        "granted",
        "dismissed",
    ]
);

pub const STOP_WORDS: [&str; 11] = [
    "v",
    "re",
    "parte",
    "denied",
    "citing",
    "aff'd",
    "affirmed",
    "remanded",
    "see",
    "granted",
    "dismissed",
];

/// Regex for StopWordToken
pub const STOP_WORD_REGEX: &str = space_boundaries_re!(strip_punctuation_re!(formatcp!(
    r"(?P<stop_word>{})",
    STOP_WORDS_JOINED
)));

/// Regex for SectionToken
pub const SECTION_REGEX: &str = r"(\S*§\S*)";

/// Regex to match punctuation around volume numbers and stopwords.
/// This could potentially be more precise.
pub const PUNCTUATION_REGEX: &str = r"[^\sa-zA-Z0-9]*";

/// Wrap regex to require non-alphanumeric characters on left and right.
pub fn nonalphanum_boundaries_re(regex: &ResolvedRegex) -> ResolvedRegex {
    ResolvedRegex::of(format!(
        r"(?:^|[^a-zA-Z0-9])({})(?:[^a-zA-Z0-9]|$)",
        regex.value()
    ))
}

/// Convert a full citation regex into a short citation regex.
///
/// Currently this just means we turn
/// > '(?P<reporter>...),? (?P<page>...'
/// to
/// > '(?P<reporter>...),? at (?P<page>...'

// clippy doesn't like '\ ' but i think its ok since it is set to ignore whitespace
#[allow(clippy::invalid_regex)]
pub fn short_cite_re(regex: &str) -> ResolvedRegex {
    let replaced = regex::RegexBuilder::new(
        r#"# reporter group:
            (
                \(\?P<reporter>[^)]+\)
            )
            (?:,\?)?\  # comma and space
            # page group:
            (
                \(\?P<page>
            )"#,
    )
    .ignore_whitespace(true)
    .build()
    .unwrap()
    .replace_all(regex, r"$1,? at $2");

    ResolvedRegex::of(replaced.to_string())
}
