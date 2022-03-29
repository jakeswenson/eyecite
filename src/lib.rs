extern crate core;

use thiserror::Error;

pub mod find;
pub mod regexes;
pub mod tokenizers;

#[derive(Error, Debug)]
pub enum EyeciteError {
    #[error("Error building tokenizer: {source}")]
    AhocorasickError {
        #[from]
        source: daachorse::errors::DaachorseError,
    },
    #[error("Error building regex: {source}")]
    RegexError {
        #[from]
        source: regex::Error,
    },
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
