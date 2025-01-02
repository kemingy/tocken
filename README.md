# Tocken

[![CI](https://github.com/kemingy/tocken/actions/workflows/check.yml/badge.svg)](https://github.com/kemingy/tocken/actions/workflows/check.yml)
[![crates.io](https://img.shields.io/crates/v/tocken.svg)](https://crates.io/crates/tocken)
[![docs.rs](https://docs.rs/tocken/badge.svg)](https://docs.rs/tocken)

Tokenizer implemented in Rust.

This tokenizer is based on [Lucene's EnglishAnalyzer](https://github.com/apache/lucene/blob/525b963be076fe8c58dd1162f083b6a9911e4efd/lucene/analysis/common/src/java/org/apache/lucene/analysis/en/EnglishAnalyzer.java#L37).

## Usage

- as a library: check the [main.rs](./src/main.rs) file and [docs](https://docs.rs/tocken).
- as a CLI:
  - `cargo r -r --help`
  - `cargo r -r -- -i wiki.txt -o wiki_tocken_f10.json -f 10`
