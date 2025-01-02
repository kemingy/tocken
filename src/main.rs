use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

use argh::FromArgs;
use env_logger::Env;
use log::debug;
use tocken::tokenizer::Tokenizer;

#[derive(FromArgs, Debug)]
/// Tokenize text file
struct Args {
    /// input file
    #[argh(option, short = 'i')]
    input: String,
    /// output file
    #[argh(option, short = 'o')]
    output: String,
    /// min frequency
    #[argh(option, short = 'f', default = "1")]
    min_freq: u32,
}

fn main() {
    let args: Args = argh::from_env();

    let env = Env::default().filter_or("TOCKEN_LOG", "debug");
    env_logger::init_from_env(env);
    debug!("{:?}", args);

    let mut tokenizer = Tokenizer::default();
    tokenizer.min_freq = args.min_freq;
    let file = File::open(args.input).expect("file not found");
    let reader = BufReader::new(file);
    let mut contents = Vec::new();
    for line in reader.lines() {
        contents.push(line.unwrap());
    }
    tokenizer.fit(&contents);
    tokenizer.trim();
    tokenizer.dump(&Path::new(&args.output));
}
