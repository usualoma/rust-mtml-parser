use super::ast::{Node::*, *};

pub const FUNCTION_TAG_STYLE_DOLLAR: u8 = 0;
pub const FUNCTION_TAG_STYLE_SELF_CLOSING: u8 = 1;

#[derive(Debug, Clone, Copy)]
pub struct Options<'a> {
    prefix: &'a str,
    function_tag_style: u8,
}

/// Serialize AST to MTML document.
///
/// # Examples
///
/// ```
/// use mtml_parser::{parse, serialize};
///
/// let node = match parse("<body><mt:Entries><mt:EntryTitle /></mt:Entries></body>") {
///   Ok(node) => node,
///   Err(err) => panic!("{}", err),
/// };
/// serialize(node, None);
/// ```
pub fn serialize(node: Node, options: Option<Options>) -> String {
    let mut s = String::new();
    let options = options.unwrap_or(Options {
        prefix: "mt:",
        function_tag_style: FUNCTION_TAG_STYLE_DOLLAR,
    });

    match node {
        Root(RootNode { children }) => {
            for child in children {
                s.push_str(&serialize(child, Some(options)));
            }
        }
        Text(TextNode { value, .. }) => {
            s.push_str(value);
        }
        FunctionTag(FunctionTagNode {
            name, attributes, ..
        }) => {
            let pre_sign = if options.function_tag_style == FUNCTION_TAG_STYLE_DOLLAR {
                "$"
            } else {
                ""
            };
            let post_sign = if options.function_tag_style == FUNCTION_TAG_STYLE_DOLLAR {
                "$"
            } else {
                "/"
            };
            s.push_str(&format!("<{}{}{}", pre_sign, options.prefix, name));
            for attr in attributes {
                s.push_str(&format!(r#" {}="{}""#, attr.name, attr.values[0].value));
            }
            s.push_str(&format!("{}>", post_sign));
        }
        BlockTag(BlockTagNode {
            name,
            children,
            attributes,
            ..
        }) => {
            s.push_str(&format!("<{}{}", options.prefix, name));
            for attr in attributes {
                s.push_str(&format!(r#" {}="{}""#, attr.name, attr.values[0].value));
            }
            s.push_str(">");
            for child in children {
                s.push_str(&serialize(child, Some(options)));
            }
            s.push_str(&format!("</{}{}>", options.prefix, name));
        }
    }

    return s;
}

#[cfg(test)]
mod tests {
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
        let serialized = serialize(root, None);
        assert_eq!(
            serialized,
            r#"
<html>
  <body>
    <mt:Entries limit="10\"20">
      <$mt:EntryTitle encode_html="1"$>
    </mt:Entries>
  </body>
</html>"#
        )
    }
    
    #[test]
    fn test_serialize_self_closing() {
        let root = parse(INPUT).unwrap();
        let serialized = serialize(root, Some(Options {
            prefix: "mt:",
            function_tag_style: FUNCTION_TAG_STYLE_SELF_CLOSING,
        }));
        assert_eq!(
            serialized,
            r#"
<html>
  <body>
    <mt:Entries limit="10\"20">
      <mt:EntryTitle encode_html="1"/>
    </mt:Entries>
  </body>
</html>"#
        )
    }
    
    #[test]
    fn test_serialize_prefix() {
        let root = parse(INPUT).unwrap();
        let serialized = serialize(root, Some(Options {
            prefix: "MT",
            function_tag_style: FUNCTION_TAG_STYLE_DOLLAR,
        }));
        assert_eq!(
            serialized,
            r#"
<html>
  <body>
    <MTEntries limit="10\"20">
      <$MTEntryTitle encode_html="1"$>
    </MTEntries>
  </body>
</html>"#
        )
    }
}
