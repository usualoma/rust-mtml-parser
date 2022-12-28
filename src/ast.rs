#[derive(Debug)]
pub enum NodeKind {
    Root,
    Text,
    BlockTag,
    FunctionTag,
}

#[derive(Debug)]
pub struct AttributeValue<'a> {
    pub value: &'a str,
    pub line: u32,
    pub offset: usize,
}

#[derive(Debug)]
pub struct Attribute<'a> {
    pub name: &'a str,
    pub values: Vec<AttributeValue<'a>>,
    pub line: u32,
    pub offset: usize,
}

#[derive(Debug)]
pub struct Node<'a> {
    pub kind: NodeKind,
    pub name: &'a str,
    pub value: Option<&'a str>,
    pub children: Vec<Node<'a>>,
    pub attributes: Vec<Attribute<'a>>,
    pub line: u32,
    pub offset: usize,
}
