use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

use tocken::tokenizer::Tokenizer;

fn main() {
    let mut tokenizer = Tokenizer::default();
    let file = File::open("wiki.txt").expect("file not found");
    let reader = BufReader::new(file);
    let mut contents = Vec::new();
    for line in reader.lines() {
        contents.push(line.unwrap());
    }
    println!("fit");
    tokenizer.fit(&contents);
    println!("dump");
    tokenizer.dump(&Path::new("wiki_tocken.json"));
}
