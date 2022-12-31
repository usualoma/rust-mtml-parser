//! # mtml-parser

mod ast;
mod parser;
mod serializer;
mod json;

pub use parser::parse;
pub use serializer::serialize;
pub use json::to_json;
