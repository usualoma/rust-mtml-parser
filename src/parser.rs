extern crate nom;

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

use super::ast::{Node::*, *};
use super::tag::FUNCTION_TAGS;

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
    current_tag: Option<String>,
) -> IResult<Span<'a>, Vec<Node>> {
    let mut children = vec![];

    while input.len() > 0 {
        let (_, pos) = position(input)?;
        let (rest, text) = match opt(take_until_tag)(input)? {
            (rest, Some(text)) => (rest, text),
            _ => (Span::new(""), input),
        };

        if text.len() > 0 {
            children.push(Text(TextNode {
                value: text.to_string(),
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

fn parse_attribute_values(mut input: Span) -> IResult<Span, Vec<AttributeValue>> {
    let mut values: Vec<AttributeValue> = vec![];

    while input.len() > 0 {
        let (_, pos) = position(input)?;
        let (rest, ch) = opt(alt((char('"'), char('\''))))(input)?;
        let ch = match ch {
            Some(ch) => ch,
            None => break,
        };
        let (rest, value) = opt(escaped(
            is_not(format!("{}\\", ch).as_str()),
            '\\',
            one_of(format!("{}", ch).as_str()),
        ))(rest)?;
        let (rest, _) = char(ch)(rest)?;
        values.push(AttributeValue {
            value: match value {
                Some(value) => value.to_string(),
                None => "".to_string(),
            },
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

fn parse_attribute(input: Span) -> IResult<Span, Option<Attribute>> {
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
        Some(Attribute {
            name: name.to_string(),
            values,
            line: pos.location_line(),
            offset: pos.location_offset(),
        }),
    ));
}

fn parse_attributes(mut input: Span) -> IResult<Span, Vec<Attribute>> {
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
    let (_, pos) = position(input)?;
    let (rest, head) = alt((tag_no_case("<mt"), tag_no_case("<$mt")))(input)?;
    let (rest, _) = opt(char(':'))(rest)?;
    let (rest, name) = name_parser(rest)?;
    let (rest, attributes) = parse_attributes(rest)?;
    let (rest, tail) = take_until(">")(rest)?;
    let (rest, _) = anychar(rest)?;

    if FUNCTION_TAGS.lock().unwrap().contains(&name.to_lowercase())
        || &name.to_lowercase() == "else"
        || (tail.len() >= 1
            && (head.chars().nth(1).unwrap() == '$' || tail.chars().rev().nth(0).unwrap() == '/'))
    {
        return Ok((
            rest,
            FunctionTag(FunctionTagNode {
                name: name.to_string(),
                attributes,
                line: pos.location_line(),
                offset: pos.location_offset(),
            }),
        ));
    } else {
        let (rest, children) = parse_internal(rest, Some(name.to_string()))?;
        return Ok((
            rest,
            BlockTag(BlockTagNode {
                name: name.to_string(),
                children,
                attributes,
                line: pos.location_line(),
                offset: pos.location_offset(),
            }),
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_blank_attribute() {
        let (rest, tag) = parse_tag(Span::new(r#"<$mt:Var name="search_link" strip="" trim="1" encode_html="1" setvar="search_link"$>"#)).unwrap();
        assert_eq!(*rest.fragment(), "");
        assert_eq!(
            tag,
            FunctionTag(FunctionTagNode {
                name: "Var".to_string(),
                attributes: vec![
                    Attribute {
                        name: "name".to_string(),
                        values: vec![AttributeValue {
                            value: "search_link".to_string(),
                            line: 1,
                            offset: 14,
                        }],
                        line: 1,
                        offset: 9,
                    },
                    Attribute {
                        name: "strip".to_string(),
                        values: vec![AttributeValue {
                            value: "".to_string(),
                            line: 1,
                            offset: 34,
                        }],
                        line: 1,
                        offset: 28,
                    },
                    Attribute {
                        name: "trim".to_string(),
                        values: vec![AttributeValue {
                            value: "1".to_string(),
                            line: 1,
                            offset: 42,
                        }],
                        line: 1,
                        offset: 37,
                    },
                    Attribute {
                        name: "encode_html".to_string(),
                        values: vec![AttributeValue {
                            value: "1".to_string(),
                            line: 1,
                            offset: 58,
                        }],
                        line: 1,
                        offset: 46,
                    },
                    Attribute {
                        name: "setvar".to_string(),
                        values: vec![AttributeValue {
                            value: "search_link".to_string(),
                            line: 1,
                            offset: 69,
                        }],
                        line: 1,
                        offset: 62,
                    },
                ],
                line: 1,
                offset: 0
            })
        );
    }

    #[test]
    fn test_parse_if_else() {
        let (rest, tag) = parse_tag(Span::new(r#"<mt:If name="blog_lang" eq="ja">ja_JP<mt:else><$mt:Var name="blog_lang"$></mt:If>"#)).unwrap();
        assert_eq!(*rest.fragment(), "");
        assert_eq!(
            tag,
            BlockTag(BlockTagNode {
                name: "If".to_string(),
                attributes: vec![
                    Attribute {
                        name: "name".to_string(),
                        values: vec![AttributeValue {
                            value: "blog_lang".to_string(),
                            line: 1,
                            offset: 12,
                        }],
                        line: 1,
                        offset: 7,
                    },
                    Attribute {
                        name: "eq".to_string(),
                        values: vec![AttributeValue {
                            value: "ja".to_string(),
                            line: 1,
                            offset: 27,
                        }],
                        line: 1,
                        offset: 24,
                    },
                ],
                line: 1,
                offset: 0,
                children: vec![
                    Text(TextNode {
                        value: "ja_JP".to_string(),
                        line: 1,
                        offset: 32,
                    }),
                    FunctionTag(FunctionTagNode {
                        name: "else".to_string(),
                        attributes: vec![],
                        line: 1,
                        offset: 37,
                    }),
                    FunctionTag(FunctionTagNode {
                        name: "Var".to_string(),
                        attributes: vec![Attribute {
                            name: "name".to_string(),
                            values: vec![AttributeValue {
                                value: "blog_lang".to_string(),
                                line: 1,
                                offset: 60,
                            }],
                            line: 1,
                            offset: 55,
                        }],
                        line: 1,
                        offset: 46,
                    }),
                ],
            })
        );
    }

    #[test]
    fn test_parse_tag_function_tag() {
        let (rest, tag) = parse_tag(Span::new(r#"<mt:EntryTitle>"#)).unwrap();
        assert_eq!(*rest.fragment(), "");
        assert_eq!(
            tag,
            FunctionTag(FunctionTagNode {
                name: "EntryTitle".to_string(),
                attributes: vec![],
                line: 1,
                offset: 0
            })
        );
    }

    #[test]
    fn test_parse_attribute() {
        let (rest, attribute) = parse_attribute(Span::new(r#"limit="10""#)).unwrap();
        assert_eq!(*rest.fragment(), "");
        let attribute = attribute.unwrap();
        assert_eq!(attribute.name, "limit");
        assert_eq!(
            attribute.values,
            vec![AttributeValue {
                value: "10".to_string(),
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
            vec![AttributeValue {
                value: "10".to_string(),
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
                AttributeValue {
                    value: "a".to_string(),
                    line: 1,
                    offset: 8
                },
                AttributeValue {
                    value: "b".to_string(),
                    line: 1,
                    offset: 12
                }
            ]
        );
    }
}
