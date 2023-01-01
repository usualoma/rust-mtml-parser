use std::io::Read;
use clap::Parser;

use mtml_parser::parse;
use mtml_parser::serializer::{serialize, Options};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let opts = Options::parse();

    let mut input = String::new();
    std::io::stdin().read_to_string(&mut input)?;
    let root = parse(input.as_str())?;
    print!("{}", serialize(root, Some(opts)));

    return Ok(());
}
