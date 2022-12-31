//! # mtml-parser

mod ast;
mod parser;
mod serializer;

pub use parser::parse;
pub use serializer::serialize;
