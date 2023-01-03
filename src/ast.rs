extern crate serde;
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AttributeValue {
    pub value: String,
    pub line: u32,
    pub column: usize,
    pub offset: usize,
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Attribute {
    pub name: String,
    pub values: Vec<AttributeValue>,
    pub line: u32,
    pub column: usize,
    pub offset: usize,
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RootNode {
    pub children: Vec<Node>,
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TextNode {
    pub value: String,
    pub line: u32,
    pub column: usize,
    pub offset: usize,
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct FunctionTagNode {
    pub name: String,
    pub attributes: Vec<Attribute>,
    pub line: u32,
    pub column: usize,
    pub offset: usize,
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct BlockTagNode {
    pub name: String,
    pub attributes: Vec<Attribute>,
    pub children: Vec<Node>,
    pub line: u32,
    pub column: usize,
    pub offset: usize,
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Node {
    Root(RootNode),
    Text(TextNode),
    FunctionTag(FunctionTagNode),
    BlockTag(BlockTagNode),
}
