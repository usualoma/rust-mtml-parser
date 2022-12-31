//! # mtml-parser

mod ast;
mod parser;
pub mod serializer;
mod json;
mod tag;

pub use parser::parse;
pub use serializer::serialize;
pub use json::to_json;
