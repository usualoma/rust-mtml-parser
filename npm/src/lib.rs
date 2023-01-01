use wasm_bindgen::prelude::*;
use mtml_parser::{ast::{RootNode, Node, Node::*}, parse as parse_mtml, serialize as serialize_mtml};

#[wasm_bindgen]
pub fn parse(input: &str) -> Result<JsValue, JsValue> {
    return match parse_mtml(input)? {
        Root(node) => Ok(serde_wasm_bindgen::to_value(&node)?),
        node => Ok(serde_wasm_bindgen::to_value(&node)?),
    };
}

#[wasm_bindgen]
pub fn serialize(node: JsValue) -> String {
    match serde_wasm_bindgen::from_value::<RootNode>(node.clone()) {
        Ok(node) => return serialize_mtml(Root(node), None),
        Err(_) => return serialize_mtml(serde_wasm_bindgen::from_value::<Node>(node).unwrap(), None)
    }
}
