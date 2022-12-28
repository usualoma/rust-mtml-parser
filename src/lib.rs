//! # mtml-parser

extern crate nom;
mod ast;

use ast::{Node, NodeKind};
use nom::{
    branch::alt,
    bytes::complete::{escaped, is_not, tag_no_case, take_until},
    character::complete::{alphanumeric0, anychar, char, multispace0, one_of},
    combinator::opt,
    IResult,
};
use nom_locate::{position, LocatedSpan};

type Span<'a> = LocatedSpan<&'a str>;

/// Parse MTML document and return AST.
///
/// # Examples
///
/// ```
/// use mtml_parser::parse;
/// 
/// parse("<body><mt:Entries><mt:EntryTitle /></mt:Entries></body>");
/// ```
pub fn parse(s: &str) -> Node {
    let (_, children) = parse_internal(Span::new(s), None).unwrap();
    return Node {
        kind: NodeKind::Root,
        name: "",
        value: None,
        children: children,
        attributes: vec![],
        line: 0,
        offset: 0,
    };
}

fn parse_internal<'a>(
    s: Span<'a>,
    current_tag: Option<&'a str>,
) -> IResult<Span<'a>, Vec<Node<'a>>> {
    let mut _s = s;
    let mut children = vec![];

    while _s.len() > 0 {
        let (_, pos) = position(_s)?;
        let (s, text) = match opt(alt((
            take_until("<mt"),
            take_until("</mt"),
            take_until("<$mt"),
        )))(_s)?
        {
            (s, Some(text)) => (s, text),
            _ => (Span::new(""), _s),
        };

        if text.len() > 0 {
            children.push(Node {
                kind: NodeKind::Text,
                name: "",
                value: Some(text.fragment()),
                children: vec![],
                attributes: vec![],
                line: pos.location_line(),
                offset: pos.location_offset(),
            });
        }

        if s.len() == 0 {
            break;
        }

        let (_, end_tag) = opt(tag_no_case("</"))(s)?;
        if end_tag.is_some() && current_tag.is_some() {
            let (s, _) = tag_no_case(format!("</mt:{}>", current_tag.unwrap()).as_str())(s)?;
            _s = s;
            break;
        } else {
            let (s, node) = parse_tag(s)?;
            children.push(node);
            _s = s;
        };
    }

    return Ok((_s, children));
}

fn parse_attribute_values(s: Span) -> IResult<Span, Vec<ast::AttributeValue>> {
    let (mut _s, _) = opt(char(','))(s)?;
    let mut values: Vec<ast::AttributeValue> = vec![];

    while _s.len() > 0 {
        match _s.chars().nth(0) {
            Some('"') | Some('\'') => {}
            _ => break,
        }
        let (s, ch) = alt((char('"'), char('\'')))(_s)?;
        let (s, value) = escaped(
            is_not(format!("{}\\", ch).as_str()),
            '\\',
            one_of(format!("{}", ch).as_str()),
        )(s)?;
        let (s, _) = char(ch)(s)?;
        let (_, pos) = position(s)?;
        values.push(ast::AttributeValue {
            value: value.fragment(),
            line: pos.location_line(),
            offset: pos.location_offset(),
        });
        _s = s;
    }

    return if values.len() > 0 {
        Ok((_s, values))
    } else {
        Ok((s, values))
    };
}

fn parse_attribute(s0: Span) -> IResult<Span, Option<ast::Attribute>> {
    let (s, _) = multispace0(s0)?;
    let (_, name) = alphanumeric0(s)?;
    if name.len() == 0 {
        return Ok((s, None));
    }
    let (_, pos) = position(s)?;
    let (s, name) = take_until("=")(s)?;
    let (s, _) = char('=')(s)?;
    let (s, values) = parse_attribute_values(s)?;

    return Ok((
        s,
        Some(ast::Attribute {
            name: name.fragment(),
            values,
            line: pos.location_line(),
            offset: pos.location_offset(),
        }),
    ));
}

fn parse_attributes(s: Span) -> IResult<Span, Vec<ast::Attribute>> {
    let mut s0 = s;
    let mut attributes = vec![];

    loop {
        let (s, attribute) = parse_attribute(s0)?;
        match attribute {
            Some(attribute) => {
                s0 = s;
                attributes.push(attribute)
            }
            None => break,
        }
    }

    return Ok((s0, attributes));
}

fn parse_tag(s: Span) -> IResult<Span, Node> {
    let (s, head) = alt((tag_no_case("<mt"), tag_no_case("<$mt")))(s)?;
    let (s, _) = opt(char(':'))(s)?;
    let (s, name) = alphanumeric0(s)?;
    let (s, attributes) = parse_attributes(s)?;
    let (s, pos) = position(s)?;
    let (s, tail) = take_until(">")(s)?;
    let (s, _) = anychar(s)?;

    let kind = if tail.len() >= 1
        && (head.chars().nth(1).unwrap() == '$' || tail.chars().rev().nth(0).unwrap() == '/')
    {
        NodeKind::FunctionTag
    } else {
        NodeKind::BlockTag
    };

    if matches!(kind, NodeKind::BlockTag) {
        let (s, children) = parse_internal(s, Some(name.fragment()))?;
        return Ok((
            s,
            Node {
                kind,
                name: name.fragment(),
                value: None,
                children,
                attributes,
                line: pos.location_line(),
                offset: pos.location_offset(),
            },
        ));
    }

    return Ok((
        s,
        Node {
            kind,
            name: name.fragment(),
            value: None,
            children: vec![],
            attributes,
            line: pos.location_line(),
            offset: pos.location_offset(),
        },
    ));
}

/// Serialize AST to MTML document.
///
/// # Examples
///
/// ```
/// use mtml_parser::{parse, serialize};
/// 
/// serialize(parse("<body><mt:Entries><mt:EntryTitle /></mt:Entries></body>"));
/// ```
pub fn serialize(node: Node) -> String {
    let mut s = String::new();

    match node.kind {
        NodeKind::Text => {
            s.push_str(node.value.unwrap());
        }
        NodeKind::FunctionTag => {
            s.push_str(&format!("<$mt:{}", node.name));
            for attr in node.attributes {
                s.push_str(&format!(r#" {}="{}""#, attr.name, attr.values[0].value));
            }
            s.push_str("$>");
        }
        NodeKind::BlockTag => {
            s.push_str(&format!("<mt:{}", node.name));
            for attr in node.attributes {
                s.push_str(&format!(r#" {}="{}""#, attr.name, attr.values[0].value));
            }
            s.push_str(">");
            for child in node.children {
                s.push_str(&serialize(child));
            }
            s.push_str(&format!("</mt:{}>", node.name));
        }
        NodeKind::Root => {
            for child in node.children {
                s.push_str(&serialize(child));
            }
        }
    }

    return s;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_serialize() {
        let input = r#"
<html>
  <body>
    <mt:Entries    limit="10\"20">
      <mtEntryTitle encode_html="1"/>
    </mt:Entries>
  </body>
</html>"#;
        let root = parse(input);

        let serialized = serialize(root);

        assert_eq!(serialized, r#"
<html>
  <body>
    <mt:Entries limit="10\"20">
      <$mt:EntryTitle encode_html="1"$>
    </mt:Entries>
  </body>
</html>"#)
    }
}
