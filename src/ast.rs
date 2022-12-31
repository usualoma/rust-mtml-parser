extern crate serde;
use serde::{Serialize};

#[derive(Debug,PartialEq,Eq,Serialize)]
pub struct AttributeValue<'a> {
    pub value: &'a str,
    pub line: u32,
    pub offset: usize,
}

#[derive(Debug,PartialEq,Eq,Serialize)]
pub struct Attribute<'a> {
    pub name: &'a str,
    pub values: Vec<AttributeValue<'a>>,
    pub line: u32,
    pub offset: usize,
}

#[derive(Debug,PartialEq,Eq,Serialize)]
pub struct RootNode<'a> {
    pub children: Vec<Node<'a>>,
}

#[derive(Debug,PartialEq,Eq,Serialize)]
pub struct TextNode<'a> {
    pub value: &'a str,
    pub line: u32,
    pub offset: usize,
}

#[derive(Debug,PartialEq,Eq,Serialize)]
pub struct FunctionTagNode<'a> {
    pub name: &'a str,
    pub attributes: Vec<Attribute<'a>>,
    pub line: u32,
    pub offset: usize,
}

#[derive(Debug,PartialEq,Eq,Serialize)]
pub struct BlockTagNode<'a> {
    pub name: &'a str,
    pub attributes: Vec<Attribute<'a>>,
    pub children: Vec<Node<'a>>,
    pub line: u32,
    pub offset: usize,
}

#[derive(Debug,PartialEq,Eq,Serialize)]
#[serde(tag = "type")]
pub enum Node<'a> {
    Root(RootNode<'a>),
    Text(TextNode<'a>),
    FunctionTag(FunctionTagNode<'a>),
    BlockTag(BlockTagNode<'a>),
}
