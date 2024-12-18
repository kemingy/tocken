//! Naive tokenizer for Unicode.

use std::{
    borrow::Cow,
    collections::{HashMap, HashSet},
};

use serde::{Deserialize, Serialize};
use tantivy_stemmers::algorithms::english_porter_2;
use unicode_normalization::UnicodeNormalization;
use unicode_segmentation::UnicodeSegmentation;

use crate::stopwords::ENGLISH;

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
    Porter2,
    None,
}

impl Stemmer {
    pub fn stem<'a>(&self, text: &'a str) -> Cow<'a, str> {
        match self {
            Stemmer::Porter2 => english_porter_2(text),
            Stemmer::None => Cow::Borrowed(&text),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tokenizer {
    stopwords: HashSet<String>,
    norm: Normalization,
    lowercase: bool,
    stemmer: Stemmer,
    table: HashMap<String, u32>,
}

impl Default for Tokenizer {
    fn default() -> Self {
        Self {
            stopwords: HashSet::from_iter(ENGLISH.iter().map(|&s| s.to_string())),
            norm: Normalization::None,
            lowercase: true,
            stemmer: Stemmer::Porter2,
            table: HashMap::new(),
        }
    }
}

impl Tokenizer {
    pub fn fit(&mut self, contents: &[String]) {
        for content in contents {
            let expect_case = match self.lowercase {
                true => content.to_lowercase(),
                false => content.to_owned(),
            };
            for word in expect_case.unicode_words() {
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
        let expect_case = match self.lowercase {
            true => content.to_lowercase(),
            false => content.to_string(),
        };
        let mut tokens = Vec::new();
        for word in expect_case.unicode_words() {
            if self.stopwords.contains(word) {
                continue;
            }
            let token = self.norm.normalize(self.stemmer.stem(word).as_ref());
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
