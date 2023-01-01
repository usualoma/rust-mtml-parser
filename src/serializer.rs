extern crate clap;
use clap::{Parser, ValueEnum};

use super::ast::{Node::*, *};

#[derive(Debug, Clone, PartialEq, Eq, ValueEnum)]
pub enum FunctionTagStyle {
    Dollar = 1,
    SelfClosing,
}

#[derive(Debug, Clone, Parser)]
#[command(author, version, about, long_about = None)]
pub struct Options {
    #[arg(short, long, default_value = "mt:")]
    pub prefix: String,
    #[arg(short, long, value_enum, default_value = "dollar")]
    pub function_tag_style: FunctionTagStyle,
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
        prefix: "mt:".to_string(),
        function_tag_style: FunctionTagStyle::Dollar,
    });

    match node {
        Root(RootNode { children }) => {
            for child in children {
                s.push_str(&serialize(child, Some(options.clone())));
            }
        }
        Text(TextNode { value, .. }) => {
            s.push_str(value.as_str());
        }
        FunctionTag(FunctionTagNode {
            name, attributes, ..
        }) => {
            let pre_sign = if options.function_tag_style == FunctionTagStyle::Dollar {
                "$"
            } else {
                ""
            };
            let post_sign = if options.function_tag_style == FunctionTagStyle::Dollar {
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
                s.push_str(&serialize(child, Some(options.clone())));
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
        let serialized = serialize(
            root,
            Some(Options {
                prefix: "mt:".to_string(),
                function_tag_style: FunctionTagStyle::SelfClosing,
            }),
        );
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
        let serialized = serialize(
            root,
            Some(Options {
                prefix: "MT".to_string(),
                function_tag_style: FunctionTagStyle::Dollar,
            }),
        );
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
