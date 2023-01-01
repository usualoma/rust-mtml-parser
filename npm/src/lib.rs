use mtml_parser::{
    ast::{Node, Node::*, RootNode},
    parse as parse_mtml, serialize as serialize_mtml,
};
use serde::Deserialize;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn parse(input: &str) -> Result<JsValue, JsValue> {
    return match parse_mtml(input)? {
        Root(node) => Ok(serde_wasm_bindgen::to_value(&node)?),
        node => Ok(serde_wasm_bindgen::to_value(&node)?),
    };
}

#[wasm_bindgen]
#[derive(Debug, Deserialize)]
pub enum FunctionTagStyle {
    Dollar = 1,
    SelfClosing = 2,
}

#[derive(Debug, Deserialize)]
#[allow(non_snake_case)]
struct SerializeOptions {
    prefix: String,
    functionTagStyle: u8,
}

impl SerializeOptions {
    fn to_options(&self) -> mtml_parser::serializer::Options {
        return mtml_parser::serializer::Options {
            prefix: self.prefix.clone(),
            function_tag_style: match self.functionTagStyle {
                2 => {
                    mtml_parser::serializer::FunctionTagStyle::SelfClosing
                },
                1 | _ => {
                    mtml_parser::serializer::FunctionTagStyle::Dollar
                },
            },
        };
    }
}

#[wasm_bindgen]
pub fn serialize(node: JsValue, opts: JsValue) -> String {
    let opts = match serde_wasm_bindgen::from_value::<SerializeOptions>(opts) {
        Ok(opts) => Some(opts.to_options()),
        Err(_) => None,
    };
    match serde_wasm_bindgen::from_value::<RootNode>(node.clone()) {
        Ok(node) => return serialize_mtml(Root(node), opts),
        Err(_) => {
            return serialize_mtml(serde_wasm_bindgen::from_value::<Node>(node).unwrap(), opts)
        }
    };
}
