//! # mtml-parser

extern crate nom;
mod ast;

use ast::{Node::*, *};
use nom::{
    branch::alt,
    bytes::complete::{escaped, is_not, tag, tag_no_case, take_until},
    character::complete::{alpha1, alphanumeric1, anychar, char, multispace0, one_of},
    combinator::{opt, recognize},
    multi::many0_count,
    sequence::pair,
    IResult, InputTake,
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
pub fn parse(input: &str) -> Result<Node, String> {
    match parse_internal(Span::new(input), None) {
        Ok((_, children)) => {
            return Ok(Root(RootNode { children }));
        }
        Err(e) => {
            return Err(format!("Parse error: {}", e));
        }
    };
}

fn take_until_tag(input: Span) -> IResult<Span, Span> {
    let str = input.to_string();
    let mut pos = 0usize;
    loop {
        match str[pos..].find('<') {
            Some(index) => {
                pos += index;
                let offset = match str.chars().nth(pos + 1) {
                    Some('$') | Some('/') => 1,
                    _ => 0,
                };
                let next = &str[pos + offset + 1..pos + offset + 3];
                if next.eq_ignore_ascii_case("mt") {
                    break;
                }
                pos += 1;
            }
            None => {
                pos = str.len();
                break;
            }
        }
    }

    return Ok(input.take_split(pos));
}

fn parse_internal<'a>(
    mut input: Span<'a>,
    current_tag: Option<&'a str>,
) -> IResult<Span<'a>, Vec<Node<'a>>> {
    let mut children = vec![];

    while input.len() > 0 {
        let (_, pos) = position(input)?;
        let (rest, text) = match opt(take_until_tag)(input)? {
            (rest, Some(text)) => (rest, text),
            _ => (Span::new(""), input),
        };

        if text.len() > 0 {
            children.push(Text(TextNode {
                value: text.fragment(),
                line: pos.location_line(),
                offset: pos.location_offset(),
            }))
        }

        if rest.len() == 0 {
            break;
        }

        let (_, end_tag) = opt(tag_no_case("</"))(rest)?;
        if end_tag.is_some() && current_tag.is_some() {
            let current_tag_str = current_tag.unwrap();
            let (rest, _) = alt((
                tag_no_case(format!("</mt:{}>", current_tag_str).as_str()),
                tag_no_case(format!("</mt{}>", current_tag_str).as_str()),
            ))(rest)?;
            input = rest;
            break;
        } else {
            let (rest, node) = parse_tag(rest)?;
            children.push(node);
            input = rest;
        };
    }

    return Ok((input, children));
}

fn parse_attribute_values(mut input: Span) -> IResult<Span, Vec<ast::AttributeValue>> {
    let mut values: Vec<ast::AttributeValue> = vec![];

    while input.len() > 0 {
        let (_, pos) = position(input)?;
        let (rest, ch) = opt(alt((char('"'), char('\''))))(input)?;
        let ch = match ch {
            Some(ch) => ch,
            None => break,
        };
        let (rest, value) = escaped(
            is_not(format!("{}\\", ch).as_str()),
            '\\',
            one_of(format!("{}", ch).as_str()),
        )(rest)?;
        let (rest, _) = char(ch)(rest)?;
        values.push(ast::AttributeValue {
            value: value.fragment(),
            line: pos.location_line(),
            offset: pos.location_offset(),
        });

        input = rest;

        let (rest, separator) = opt(char(','))(rest)?;
        if separator.is_none() {
            break;
        }

        input = rest;
    }

    Ok((input, values))
}

fn name_parser(input: Span) -> IResult<Span, Span> {
    recognize(pair(
        alt((alpha1, tag("_"))),
        many0_count(alt((alphanumeric1, tag("_"), tag(":")))),
    ))(input)
}

fn parse_attribute(input: Span) -> IResult<Span, Option<ast::Attribute>> {
    let (rest, _) = multispace0(input)?;
    let (_, pos) = position(rest)?;

    let (rest, name) = opt(name_parser)(rest)?;
    let name = match name {
        Some(name) => name,
        None => return Ok((input, None)),
    };

    let (rest, _) = char('=')(rest)?;
    let (rest, values) = parse_attribute_values(rest)?;

    return Ok((
        rest,
        Some(ast::Attribute {
            name: name.fragment(),
            values,
            line: pos.location_line(),
            offset: pos.location_offset(),
        }),
    ));
}

fn parse_attributes(mut input: Span) -> IResult<Span, Vec<ast::Attribute>> {
    let mut attributes = vec![];

    loop {
        let (rest, attribute) = parse_attribute(input)?;
        match attribute {
            Some(attribute) => {
                input = rest;
                attributes.push(attribute)
            }
            None => break,
        }
    }

    return Ok((input, attributes));
}

fn parse_tag(input: Span) -> IResult<Span, Node> {
    let (rest, head) = alt((tag_no_case("<mt"), tag_no_case("<$mt")))(input)?;
    let (rest, _) = opt(char(':'))(rest)?;
    let (rest, name) = name_parser(rest)?;
    let (rest, attributes) = parse_attributes(rest)?;
    let (rest, pos) = position(rest)?;
    let (rest, tail) = take_until(">")(rest)?;
    let (rest, _) = anychar(rest)?;

    if tail.len() >= 1
        && (head.chars().nth(1).unwrap() == '$' || tail.chars().rev().nth(0).unwrap() == '/')
    {
        return Ok((
            rest,
            FunctionTag(FunctionTagNode {
                name: name.fragment(),
                attributes,
                line: pos.location_line(),
                offset: pos.location_offset(),
            }),
        ));
    } else {
        let (rest, children) = parse_internal(rest, Some(name.fragment()))?;
        return Ok((
            rest,
            BlockTag(BlockTagNode {
                name: name.fragment(),
                children,
                attributes,
                line: pos.location_line(),
                offset: pos.location_offset(),
            }),
        ));
    }
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
/// serialize(node);
/// ```
pub fn serialize(node: Node) -> String {
    let mut s = String::new();

    match node {
        Root(RootNode { children }) => {
            for child in children {
                s.push_str(&serialize(child));
            }
        }
        Text(TextNode { value, .. }) => {
            s.push_str(value);
        }
        FunctionTag(FunctionTagNode {
            name, attributes, ..
        }) => {
            s.push_str(&format!("<$mt:{}", name));
            for attr in attributes {
                s.push_str(&format!(r#" {}="{}""#, attr.name, attr.values[0].value));
            }
            s.push_str("$>");
        }
        BlockTag(BlockTagNode {
            name,
            children,
            attributes,
            ..
        }) => {
            s.push_str(&format!("<mt:{}", name));
            for attr in attributes {
                s.push_str(&format!(r#" {}="{}""#, attr.name, attr.values[0].value));
            }
            s.push_str(">");
            for child in children {
                s.push_str(&serialize(child));
            }
            s.push_str(&format!("</mt:{}>", name));
        }
    }

    return s;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_attribute() {
        let (rest, attribute) = parse_attribute(Span::new(r#"limit="10""#)).unwrap();
        assert_eq!(*rest.fragment(), "");
        let attribute = attribute.unwrap();
        assert_eq!(attribute.name, "limit");
        assert_eq!(
            attribute.values,
            vec![ast::AttributeValue {
                value: "10",
                line: 1,
                offset: 6
            }]
        );
    }

    #[test]
    fn test_parse_attribute_single_quote() {
        let (rest, attribute) = parse_attribute(Span::new(r#"limit='10'"#)).unwrap();
        assert_eq!(*rest.fragment(), "");
        let attribute = attribute.unwrap();
        assert_eq!(attribute.name, "limit");
        assert_eq!(
            attribute.values,
            vec![ast::AttributeValue {
                value: "10",
                line: 1,
                offset: 6
            }]
        );
    }

    #[test]
    fn test_parse_attribute_replace() {
        let (rest, attribute) = parse_attribute(Span::new(r#"replace="a","b""#)).unwrap();
        assert_eq!(*rest.fragment(), "");
        let attribute = attribute.unwrap();
        assert_eq!(attribute.name, "replace");
        assert_eq!(
            attribute.values,
            vec![
                ast::AttributeValue {
                    value: "a",
                    line: 1,
                    offset: 8
                },
                ast::AttributeValue {
                    value: "b",
                    line: 1,
                    offset: 12
                }
            ]
        );
    }

    #[test]
    fn test_parse_then_serialize() {
        let input = r#"
<html>
  <body>
    <mt:Entries    limit="10\"20">
      <mtEntryTitle encode_html="1"/>
    </mt:Entries>
  </body>
</html>"#;
        let root = parse(input).unwrap();

        let serialized = serialize(root);

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
}
