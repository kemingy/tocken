//! Naive tokenizer for Unicode.

use std::{
    borrow::Cow,
    collections::{HashMap, HashSet},
};

use serde::{Deserialize, Serialize};
use tantivy_stemmers::algorithms::english_porter as stemmer;
use unicode_normalization::UnicodeNormalization;
use unicode_segmentation::UnicodeSegmentation;

use crate::stopwords::ENGLISH_LUCENE;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Normalization {
    NFD,
    NFC,
    NFKD,
    NFKC,
    None,
}

impl Normalization {
    pub fn normalize(&self, text: &str) -> String {
        match self {
            Normalization::NFD => text.nfd().collect(),
            Normalization::NFC => text.nfc().collect(),
            Normalization::NFKD => text.nfkd().collect(),
            Normalization::NFKC => text.nfkc().collect(),
            Normalization::None => text.to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Stemmer {
    Snowball,
    None,
}

impl Stemmer {
    pub fn stem<'a>(&self, text: &'a str) -> Cow<'a, str> {
        match self {
            Stemmer::Snowball => stemmer(text),
            Stemmer::None => Cow::Borrowed(text),
        }
    }
}

pub fn english_possessive_filter(text: &str) -> Option<String> {
    match text.len() > 2 && text.ends_with("s") {
        true => {
            let chars = text.chars().collect::<Vec<_>>();
            let c = chars[chars.len() - 2];
            match c {
                '\'' | '\u{2019}' | '\u{FF07}' => Some(chars[..chars.len() - 2].iter().collect()),
                _ => None,
            }
        }
        false => None,
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tokenizer {
    stopwords: HashSet<String>,
    norm: Normalization,
    stemmer: Stemmer,
    table: HashMap<String, u32>,
}

impl Default for Tokenizer {
    fn default() -> Self {
        Self {
            stopwords: HashSet::from_iter(ENGLISH_LUCENE.iter().map(|&s| s.to_string())),
            norm: Normalization::None,
            stemmer: Stemmer::Snowball,
            table: HashMap::new(),
        }
    }
}

impl Tokenizer {
    pub fn fit(&mut self, contents: &[String]) {
        for content in contents {
            let lowercase = content.to_lowercase();
            for word in lowercase.unicode_words() {
                if self.stopwords.contains(word) {
                    continue;
                }
                let token = self.norm.normalize(self.stemmer.stem(word).as_ref());
                let length = self.table.len();
                self.table.entry(token).or_insert(length as u32);
            }
        }
    }

    pub fn tokenize(&self, content: &str) -> Vec<u32> {
        let lowercase = content.to_lowercase();
        let mut tokens = Vec::new();
        for word in lowercase.unicode_words() {
            let word = match english_possessive_filter(word) {
                Some(w) => w,
                None => word.to_string(),
            };
            if self.stopwords.contains(&word) {
                continue;
            }
            let token = self.norm.normalize(self.stemmer.stem(&word).as_ref());
            if let Some(&id) = self.table.get(&token) {
                tokens.push(id);
            }
        }

        tokens
    }

    pub fn dumps(&self) -> String {
        serde_json::to_string(self).unwrap()
    }

    pub fn dump(&self, path: &impl AsRef<std::path::Path>) {
        std::fs::write(
            path,
            serde_json::to_string(&self).expect("failed to serialize"),
        )
        .expect("failed to write");
    }

    pub fn loads(data: &str) -> Self {
        serde_json::from_str(data).unwrap()
    }

    pub fn load(path: &impl AsRef<std::path::Path>) -> Self {
        serde_json::from_slice(&std::fs::read(path).expect("failed to read"))
            .expect("failed to deserialize")
    }

    pub fn vocab_len(&self) -> usize {
        self.table.len()
    }
}

#[cfg(test)]
mod tests {
    use crate::tokenizer::english_possessive_filter;

    #[test]
    fn test_english_possessive_filter() {
        let cases = [
            ("John's", "John"),
            ("John’s", "John"),
            ("John＇s", "John"),
            ("Johns", "Johns"),
            ("John", "John"),
            ("Johns'", "Johns'"),
            ("John'ss", "John'ss"),
            ("'s", "'s"),
        ];

        for (text, expected) in cases.iter() {
            if let Some(res) = english_possessive_filter(text) {
                assert_eq!(res, *expected);
            }
        }
    }
}
