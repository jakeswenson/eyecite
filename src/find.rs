use crate::find::models::Citation;
use crate::tokenizers::Tokenizer;

pub mod models;

/**!
This is eyecite's main workhorse function. Given a string of text
(e.g., a judicial opinion or other legal document), return a list of
[`eyecite.models.Citation`](models::Citation) objects representing the citations found
in the document.

Args:
    plain_text:
        The text to parse. You may wish to use the 'eyecite.clean.clean_text'
        function to pre-process your text
        before passing it here.
    remove_ambiguous:
        Whether to remove citations that might refer to more
        than one reporter and can't be narrowed down by date.
    tokenizer:
        An instance of a Tokenizer object. See 'eyecite.tokenizers'
        for information about available tokenizers. Uses the
        'eyecite.tokenizers.AhocorasickTokenizer' by default.

Returns:
    A list of 'eyecite.models.CitationBase' objects
 */
pub fn get_citations<'a>(
    plain_text: &'a str,
    _remove_ambiguous: bool,
    tokenizer: &'a (dyn Tokenizer<'a>),
) -> Vec<Citation<'a>> {
    let (_words, citation_tokens) = tokenizer.tokenize(plain_text);
    let citations = Vec::new();

    for (_i, _token) in citation_tokens {}

    citations
}
