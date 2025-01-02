//! Naive tokenizer for Unicode.

use std::{
    borrow::Cow,
    collections::{HashMap, HashSet},
    time::Instant,
};

use log::debug;
use serde::{Deserialize, Serialize};
use tantivy_stemmers::algorithms::english_porter as stemmer;
use unicode_normalization::UnicodeNormalization;
use unicode_segmentation::UnicodeSegmentation;

use crate::stopwords::{CHINESE_NLTK_SINGLE, CJK_LUCENE, ENGLISH_LUCENE, ENGLISH_NLTK};

#[derive(Debug, Clone, Serialize, Deserialize)]
/// Unicode normalization.
/// Ref: https://unicode.org/reports/tr15/
pub enum Normalization {
    /// Canonical Decomposition
    NFD,
    /// Canonical Decomposition, followed by Canonical Composition
    NFC,
    /// Compatibility Decomposition
    NFKD,
    /// Compatibility Decomposition, followed by Canonical Composition
    NFKC,
    /// No normalization
    None,
}

impl Normalization {
    /// Normalize the text.
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
/// Stemmer method.
pub enum Stemmer {
    /// https://snowballstem.org/algorithms/
    Snowball,
    /// No stemmer
    None,
}

impl Stemmer {
    /// Stem the text.
    pub fn stem<'a>(&self, text: &'a str) -> Cow<'a, str> {
        match self {
            Stemmer::Snowball => stemmer(text),
            Stemmer::None => Cow::Borrowed(text),
        }
    }
}

/// English possessive filter.
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
/// Tokenizer for text keyword match.
pub struct Tokenizer {
    /// The minimum frequency of the token to be kept when calling `trim`.
    pub min_freq: u32,
    /// The stopwords set.
    pub stopwords: HashSet<String>,
    /// The normalization method.
    pub norm: Normalization,
    /// The stemmer method.
    pub stemmer: Stemmer,
    table: HashMap<String, u32>,
    counter: Vec<u32>,
}

impl Default for Tokenizer {
    fn default() -> Self {
        Self {
            stopwords: [
                ENGLISH_LUCENE,
                CJK_LUCENE,
                ENGLISH_NLTK,
                CHINESE_NLTK_SINGLE,
            ]
            .into_iter()
            .flat_map(|slice| slice.iter().map(|x| x.to_string()))
            .collect(),
            norm: Normalization::None,
            stemmer: Stemmer::Snowball,
            table: HashMap::new(),
            counter: Vec::new(),
            min_freq: 5,
        }
    }
}

impl Tokenizer {
    fn get_token(&self, content: &str) -> Vec<String> {
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
            if token.is_empty() {
                continue;
            }
            tokens.push(token);
        }

        tokens
    }

    /// Fit the tokenizer with the contents. Re-call this function will update the tokenizer.
    pub fn fit(&mut self, contents: &[String]) {
        let instant = Instant::now();
        let exist_token = self.table.len();
        for content in contents {
            let tokens = self.get_token(content);
            for token in tokens {
                let length = self.table.len();
                let entry = self.table.entry(token).or_insert(length as u32);
                if *entry == self.counter.len() as u32 {
                    self.counter.push(0);
                }
                self.counter[*entry as usize] += 1;
            }
        }
        debug!(
            "fitting took {:?}, parsed {:?} lines of text, found {:?} tokens",
            instant.elapsed().as_secs_f32(),
            contents.len(),
            self.table.len() - exist_token
        );
    }

    /// Tokenize the content and return the token ids.
    pub fn tokenize(&self, content: &str) -> Vec<u32> {
        let tokens = self.get_token(content);
        let mut ids = Vec::with_capacity(tokens.len());
        for token in tokens {
            if let Some(&id) = self.table.get(&token) {
                ids.push(id);
            }
        }
        ids
    }

    /// This will trim the `table` according to the `min_freq` and clean the `counter`.
    pub fn trim(&mut self) {
        let mut selected = HashMap::new();
        for (token, &id) in self.table.iter() {
            if self.counter[id as usize] >= self.min_freq {
                selected.insert(token.clone(), selected.len() as u32);
            }
        }
        debug!(
            "trim {:?} tokens into {:?} tokens",
            self.table.len(),
            selected.len()
        );
        self.table = selected;
        self.counter.clear();
    }

    /// Serialize the tokenizer into a JSON string.
    pub fn dumps(&self) -> String {
        serde_json::to_string(self).expect("failed to serialize")
    }

    /// Serialize the tokenizer into a JSON file.
    pub fn dump(&self, path: &impl AsRef<std::path::Path>) {
        std::fs::write(path, self.dumps()).expect("failed to write");
    }

    /// Deserialize the tokenizer from a JSON string.
    pub fn loads(data: &str) -> Self {
        serde_json::from_str(data).unwrap()
    }

    /// Deserialize the tokenizer from a JSON file.
    pub fn load(path: &impl AsRef<std::path::Path>) -> Self {
        serde_json::from_slice(&std::fs::read(path).expect("failed to read"))
            .expect("failed to deserialize")
    }

    /// Get the total token number.
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
