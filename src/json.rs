extern crate serde_json;
use super::ast::{Node::*, *};

/// Serialize AST to JSON.
///
/// # Examples
///
/// ```
/// use mtml_parser::{parse, to_json};
///
/// let node = match parse("<body><mt:Entries><mt:EntryTitle /></mt:Entries></body>") {
///   Ok(node) => node,
///   Err(err) => panic!("{}", err),
/// };
/// to_json(node);
/// ```
pub fn to_json(node: Node) -> String {
    return match node {
        Root(node) => serde_json::to_string(&node),
        _ => serde_json::to_string(&node),
    }
    .unwrap();
}

#[cfg(test)]
mod tests {
    extern crate serde_json;
    use super::super::parser::*;
    use super::*;

    const INPUT: &str = r#"
<html>
  <body>
    <mt:Entries    limit="10\"20">
      <mtEntryTitle encode_html="1"/>
    </mt:Entries>
  </body>
</html>"#;

    #[test]
    fn test_serialize() {
        let root = parse(INPUT).unwrap();
        let json = to_json(root);
        println!("{}", json);
        assert_eq!(
            json,
            r#"{"children":[{"type":"Text","value":"\n<html>\n  <body>\n    ","line":1,"column":1,"offset":0},{"type":"BlockTag","name":"Entries","attributes":[{"name":"limit","values":[{"value":"10\\\"20","line":4,"column":26,"offset":42}],"line":4,"column":20,"offset":36}],"children":[{"type":"Text","value":"\n      ","line":4,"column":35,"offset":51},{"type":"FunctionTag","name":"EntryTitle","attributes":[{"name":"encode_html","values":[{"value":"1","line":5,"column":33,"offset":84}],"line":5,"column":21,"offset":72}],"line":5,"column":7,"offset":58},{"type":"Text","value":"\n    ","line":5,"column":38,"offset":89}],"line":4,"column":5,"offset":21},{"type":"Text","value":"\n  </body>\n</html>","line":6,"column":18,"offset":107}]}"#
        )
    }
}
