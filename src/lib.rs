//! # mtml-parser

pub mod ast;
pub mod parser;
pub mod serializer;
mod json;
pub mod tag;

pub use parser::parse;
pub use serializer::serialize;
pub use json::to_json;
