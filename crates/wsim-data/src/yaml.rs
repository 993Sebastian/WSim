//! Minimal YAML tree with source positions.
//!
//! Built from the event stream of `saphyr-parser` so that every value keeps its line
//! and column. Plain scalars are resolved here (numbers may contain `_` separators);
//! anchors, aliases and multiple documents are rejected to keep data files simple.

use std::collections::BTreeSet;

use saphyr_parser::{Event, Parser, ScalarStyle, Span};

use crate::messages;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct Position {
    pub line: u32,
    pub column: u32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Node {
    pub kind: Kind,
    pub position: Position,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Kind {
    Null,
    Bool(bool),
    /// Number with its source text (for messages and map keys).
    Number(f64, String),
    Str(String),
    Seq(Vec<Node>),
    Map(Vec<(Node, Node)>),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SyntaxError {
    pub position: Position,
    pub message: String,
}

impl Node {
    /// Text of a scalar as written in the file; used for map keys and messages.
    pub fn scalar_text(&self) -> Option<&str> {
        match &self.kind {
            Kind::Str(s) | Kind::Number(_, s) => Some(s),
            Kind::Bool(true) => Some("true"),
            Kind::Bool(false) => Some("false"),
            _ => None,
        }
    }

    /// Looks up a map value by key.
    pub fn get(&self, key: &str) -> Option<&Node> {
        self.entry(key).map(|(_, v)| v)
    }

    /// Looks up a map entry (key node and value node) by key.
    pub fn entry(&self, key: &str) -> Option<(&Node, &Node)> {
        match &self.kind {
            Kind::Map(entries) => entries
                .iter()
                .find(|(k, _)| k.scalar_text() == Some(key))
                .map(|(k, v)| (k, v)),
            _ => None,
        }
    }
}

fn position(span: &Span) -> Position {
    let to_u32 = |v: usize| u32::try_from(v).unwrap_or(u32::MAX);
    Position {
        line: to_u32(span.start.line()),
        column: to_u32(span.start.col() + 1),
    }
}

enum Frame {
    Seq {
        items: Vec<Node>,
        position: Position,
    },
    Map {
        entries: Vec<(Node, Node)>,
        pending_key: Option<Node>,
        keys: BTreeSet<String>,
        position: Position,
    },
}

/// Parses a YAML document. An empty document yields a `Null` node.
pub fn parse(text: &str) -> Result<Node, SyntaxError> {
    let mut stack: Vec<Frame> = Vec::new();
    let mut root: Option<Node> = None;
    let mut documents = 0;

    for event in Parser::new_from_str(text) {
        let (event, span) = event.map_err(|e| SyntaxError {
            position: Position {
                line: u32::try_from(e.marker().line()).unwrap_or(u32::MAX),
                column: u32::try_from(e.marker().col() + 1).unwrap_or(u32::MAX),
            },
            message: messages::yaml_syntax(e.info()),
        })?;
        let pos = position(&span);
        let fail = |message: String| {
            Err(SyntaxError {
                position: pos,
                message,
            })
        };
        match event {
            Event::Nothing | Event::StreamStart | Event::StreamEnd | Event::DocumentEnd => {}
            Event::DocumentStart(_) => {
                documents += 1;
                if documents > 1 {
                    return fail(messages::yaml_multiple_documents());
                }
            }
            Event::Alias(_) => return fail(messages::yaml_alias()),
            Event::Scalar(value, style, anchor, _) => {
                if anchor != 0 {
                    return fail(messages::yaml_alias());
                }
                let node = Node {
                    kind: resolve_scalar(&value, style),
                    position: pos,
                };
                push(&mut stack, &mut root, node)?;
            }
            Event::SequenceStart(anchor, _) | Event::MappingStart(anchor, _) if anchor != 0 => {
                return fail(messages::yaml_alias());
            }
            Event::SequenceStart(..) => stack.push(Frame::Seq {
                items: Vec::new(),
                position: pos,
            }),
            Event::MappingStart(..) => stack.push(Frame::Map {
                entries: Vec::new(),
                pending_key: None,
                keys: BTreeSet::new(),
                position: pos,
            }),
            Event::SequenceEnd | Event::MappingEnd => {
                let node = match stack.pop() {
                    Some(Frame::Seq { items, position }) => Node {
                        kind: Kind::Seq(items),
                        position,
                    },
                    Some(Frame::Map {
                        entries, position, ..
                    }) => Node {
                        kind: Kind::Map(entries),
                        position,
                    },
                    None => unreachable!("parser emitted an unbalanced end event"),
                };
                push(&mut stack, &mut root, node)?;
            }
        }
    }
    Ok(root.unwrap_or(Node {
        kind: Kind::Null,
        position: Position { line: 1, column: 1 },
    }))
}

fn push(stack: &mut [Frame], root: &mut Option<Node>, node: Node) -> Result<(), SyntaxError> {
    match stack.last_mut() {
        None => *root = Some(node),
        Some(Frame::Seq { items, .. }) => items.push(node),
        Some(Frame::Map {
            entries,
            pending_key,
            keys,
            ..
        }) => match pending_key.take() {
            Some(key) => entries.push((key, node)),
            None => {
                let Some(text) = node.scalar_text() else {
                    return Err(SyntaxError {
                        position: node.position,
                        message: messages::yaml_complex_key(),
                    });
                };
                if !keys.insert(text.to_owned()) {
                    return Err(SyntaxError {
                        position: node.position,
                        message: messages::yaml_duplicate_key(text),
                    });
                }
                *pending_key = Some(node);
            }
        },
    }
    Ok(())
}

fn resolve_scalar(value: &str, style: ScalarStyle) -> Kind {
    if style != ScalarStyle::Plain {
        return Kind::Str(value.to_owned());
    }
    match value {
        "" | "~" | "null" => Kind::Null,
        "true" => Kind::Bool(true),
        "false" => Kind::Bool(false),
        _ => match parse_number(value) {
            Some(n) => Kind::Number(n, value.to_owned()),
            None => Kind::Str(value.to_owned()),
        },
    }
}

/// Decimal numbers with optional sign, fraction, exponent and `_` digit separators.
fn parse_number(text: &str) -> Option<f64> {
    let bytes = text.as_bytes();
    let mut i = 0;
    if matches!(bytes.first(), Some(b'+' | b'-')) {
        i += 1;
    }
    let digits = |i: &mut usize| {
        let start = *i;
        if !bytes.get(*i).is_some_and(u8::is_ascii_digit) {
            return false;
        }
        while bytes
            .get(*i)
            .is_some_and(|b| b.is_ascii_digit() || *b == b'_')
        {
            *i += 1;
        }
        bytes[*i - 1] != b'_' && *i > start
    };
    if !digits(&mut i) {
        return None;
    }
    if bytes.get(i) == Some(&b'.') {
        i += 1;
        if !digits(&mut i) {
            return None;
        }
    }
    if matches!(bytes.get(i), Some(b'e' | b'E')) {
        i += 1;
        if matches!(bytes.get(i), Some(b'+' | b'-')) {
            i += 1;
        }
        if !digits(&mut i) {
            return None;
        }
    }
    if i != bytes.len() {
        return None;
    }
    text.replace('_', "").parse().ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn number(text: &str) -> Option<f64> {
        parse_number(text)
    }

    #[test]
    fn parses_numbers_with_separators() {
        assert_eq!(number("60_000_000"), Some(60_000_000.0));
        assert_eq!(number("-1.5"), Some(-1.5));
        assert_eq!(number("2.5e3"), Some(2500.0));
        assert_eq!(number("1_000.25"), Some(1000.25));
    }

    #[test]
    fn rejects_non_numbers() {
        for text in ["1,5", "abc", "1_", "_1", "1.", ".5", "1e", "DEU", "1.2.3"] {
            assert_eq!(number(text), None, "{text}");
        }
    }

    #[test]
    fn keeps_positions() {
        let doc = parse("a: 1\nb:\n  - x\n  - 'y'\n").unwrap();
        let b = doc.get("b").unwrap();
        assert_eq!(b.position.line, 3);
        let Kind::Seq(items) = &b.kind else { panic!() };
        assert_eq!(items[1].position, Position { line: 4, column: 5 });
        assert_eq!(items[1].kind, Kind::Str("y".into()));
        assert_eq!(doc.get("a").unwrap().kind, Kind::Number(1.0, "1".into()));
    }

    #[test]
    fn quoted_numbers_stay_text() {
        let doc = parse("a: '12'").unwrap();
        assert_eq!(doc.get("a").unwrap().kind, Kind::Str("12".into()));
    }

    #[test]
    fn empty_document_is_null() {
        assert_eq!(parse("# nur ein Kommentar\n").unwrap().kind, Kind::Null);
    }

    #[test]
    fn rejects_duplicate_keys() {
        let err = parse("a: 1\nb: 2\na: 3\n").unwrap_err();
        assert_eq!(err.position.line, 3);
        assert!(err.message.contains("„a“"), "{}", err.message);
    }

    #[test]
    fn rejects_aliases_and_multiple_documents() {
        assert!(parse("a: &x 1\nb: *x\n").is_err());
        assert!(parse("a: 1\n---\nb: 2\n").is_err());
    }

    #[test]
    fn reports_syntax_errors_with_line() {
        let err = parse("a: [1, 2\nb: 3\n").unwrap_err();
        assert!(err.position.line >= 1);
        assert!(err.message.starts_with("Die Datei ist kein gültiges YAML"));
    }
}
