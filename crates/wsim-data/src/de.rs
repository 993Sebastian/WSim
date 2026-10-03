//! Serde deserializer over the YAML tree.
//!
//! Own implementation so that every error carries line, path and a German message.

use std::fmt;

use serde::de::{
    self, DeserializeSeed, Deserializer, Expected, IntoDeserializer, MapAccess, SeqAccess,
    Unexpected, Visitor,
};

use crate::messages;
use crate::report::Path;
use crate::suggest;
use crate::yaml::{Kind, Node, Position};

#[derive(Debug, Clone, PartialEq)]
pub struct DeError {
    pub message: String,
    pub position: Option<Position>,
    pub path: Option<Path>,
}

impl fmt::Display for DeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for DeError {}

fn plain(message: String) -> DeError {
    DeError {
        message,
        position: None,
        path: None,
    }
}

impl de::Error for DeError {
    fn custom<T: fmt::Display>(msg: T) -> Self {
        plain(msg.to_string())
    }

    fn invalid_type(unexpected: Unexpected<'_>, expected: &dyn Expected) -> Self {
        let mut message = messages::wrong_type(
            &describe_unexpected(&unexpected),
            &describe_expected(expected),
        );
        if let Unexpected::Str(s) = unexpected
            && s.contains(',')
            && s.chars()
                .all(|c| c.is_ascii_digit() || matches!(c, ',' | '.' | '-' | '_'))
        {
            message.push_str(messages::decimal_comma_hint());
        }
        plain(message)
    }

    fn invalid_value(unexpected: Unexpected<'_>, expected: &dyn Expected) -> Self {
        plain(messages::invalid_value(
            &describe_unexpected(&unexpected),
            &describe_expected(expected),
        ))
    }

    fn invalid_length(len: usize, expected: &dyn Expected) -> Self {
        plain(messages::wrong_length(len, &describe_expected(expected)))
    }

    fn unknown_variant(variant: &str, expected: &'static [&'static str]) -> Self {
        plain(messages::unknown_value(
            variant,
            expected,
            suggest::closest(variant, expected.iter().copied()),
        ))
    }

    fn unknown_field(field: &str, expected: &'static [&'static str]) -> Self {
        plain(messages::unknown_field(
            field,
            expected,
            suggest::closest(field, expected.iter().copied()),
        ))
    }

    fn missing_field(field: &'static str) -> Self {
        plain(messages::missing_field(field))
    }

    fn duplicate_field(field: &'static str) -> Self {
        plain(messages::duplicate_field(field))
    }
}

fn describe_unexpected(unexpected: &Unexpected<'_>) -> String {
    match unexpected {
        Unexpected::Bool(b) => format!("„{b}“"),
        Unexpected::Unsigned(n) => format!("die Zahl {n}"),
        Unexpected::Signed(n) => format!("die Zahl {n}"),
        Unexpected::Float(n) => format!("die Zahl {n}"),
        Unexpected::Str(s) => format!("der Text „{s}“"),
        Unexpected::Unit => "ein leerer Wert".into(),
        Unexpected::Seq => "eine Liste".into(),
        Unexpected::Map => "eine Zuordnung".into(),
        other => other.to_string(),
    }
}

/// Translates serde's English descriptions of expected types.
fn describe_expected(expected: &dyn Expected) -> String {
    let text = expected.to_string();
    let lower = text.to_lowercase();
    if lower.contains("string") || lower.contains("identifier") {
        "ein Text".into()
    } else if lower == "f64" || lower == "f32" {
        "eine Zahl".into()
    } else if is_integer_type(&lower, 'u') {
        "eine ganze Zahl ab 0".into()
    } else if is_integer_type(&lower, 'i') {
        "eine ganze Zahl".into()
    } else if lower.contains("bool") {
        "true oder false".into()
    } else if lower.contains("sequence") || lower.contains("tuple") || lower.contains("array") {
        "eine Liste".into()
    } else if lower.contains("map") || lower.starts_with("struct ") {
        "eine Zuordnung (Feld: Wert)".into()
    } else if lower.starts_with("variant") || lower.starts_with("enum ") {
        "einer der erlaubten Werte".into()
    } else {
        text
    }
}

fn is_integer_type(name: &str, prefix: char) -> bool {
    name.strip_prefix(prefix)
        .is_some_and(|bits| bits.parse::<u8>().is_ok())
}

/// Deserializes `T` from a node. `path` is the node's location for error messages.
pub fn from_node<'de, T: de::Deserialize<'de>>(node: &'de Node, path: &Path) -> Result<T, DeError> {
    T::deserialize(NodeDe {
        node,
        path: path.clone(),
    })
}

struct NodeDe<'a> {
    node: &'a Node,
    path: Path,
}

impl NodeDe<'_> {
    /// Attaches this node's location to errors raised without one.
    fn locate<T>(&self, result: Result<T, DeError>) -> Result<T, DeError> {
        result.map_err(|mut e| {
            if e.position.is_none() {
                e.position = Some(self.node.position);
                e.path = Some(self.path.clone());
            }
            e
        })
    }

    fn unexpected(&self) -> Unexpected<'_> {
        match &self.node.kind {
            Kind::Null => Unexpected::Unit,
            Kind::Bool(b) => Unexpected::Bool(*b),
            Kind::Number(n, _) => Unexpected::Float(*n),
            Kind::Str(s) => Unexpected::Str(s),
            Kind::Seq(_) => Unexpected::Seq,
            Kind::Map(_) => Unexpected::Map,
        }
    }

    fn type_error<T>(&self, visitor: &dyn Expected) -> Result<T, DeError> {
        Err(de::Error::invalid_type(self.unexpected(), visitor))
    }

    fn integer(&self, visitor: &dyn Expected) -> Result<i64, DeError> {
        match &self.node.kind {
            #[allow(clippy::cast_possible_truncation)]
            Kind::Number(n, _) if n.fract() == 0.0 && n.abs() < 9.0e15 => Ok(*n as i64),
            _ => self.type_error(visitor),
        }
    }
}

macro_rules! deserialize_integer {
    ($($method:ident),*) => {$(
        fn $method<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, DeError> {
            let result = self.integer(&visitor).and_then(|n| visitor.visit_i64(n));
            self.locate(result)
        }
    )*};
}

impl<'de> Deserializer<'de> for NodeDe<'de> {
    type Error = DeError;

    fn deserialize_any<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, DeError> {
        let result = match &self.node.kind {
            Kind::Null => visitor.visit_unit(),
            Kind::Bool(b) => visitor.visit_bool(*b),
            #[allow(clippy::cast_possible_truncation)]
            Kind::Number(n, _) if n.fract() == 0.0 && n.abs() < 9.0e15 => {
                visitor.visit_i64(*n as i64)
            }
            Kind::Number(n, _) => visitor.visit_f64(*n),
            Kind::Str(s) => visitor.visit_str(s),
            Kind::Seq(items) => visitor.visit_seq(SeqDe {
                items: items.iter().enumerate(),
                path: &self.path,
            }),
            Kind::Map(entries) => visitor.visit_map(MapDe::new(entries, &self.path)),
        };
        self.locate(result)
    }

    fn deserialize_bool<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, DeError> {
        let result = match &self.node.kind {
            Kind::Bool(b) => visitor.visit_bool(*b),
            _ => self.type_error(&visitor),
        };
        self.locate(result)
    }

    deserialize_integer!(
        deserialize_i8,
        deserialize_i16,
        deserialize_i32,
        deserialize_i64,
        deserialize_u8,
        deserialize_u16,
        deserialize_u32,
        deserialize_u64
    );

    fn deserialize_f32<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, DeError> {
        self.deserialize_f64(visitor)
    }

    fn deserialize_f64<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, DeError> {
        let result = match &self.node.kind {
            Kind::Number(n, _) => visitor.visit_f64(*n),
            _ => self.type_error(&visitor),
        };
        self.locate(result)
    }

    fn deserialize_str<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, DeError> {
        let result = match &self.node.kind {
            Kind::Str(s) => visitor.visit_str(s),
            _ => self.type_error(&visitor),
        };
        self.locate(result)
    }

    fn deserialize_string<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, DeError> {
        self.deserialize_str(visitor)
    }

    fn deserialize_identifier<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, DeError> {
        self.deserialize_str(visitor)
    }

    fn deserialize_char<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, DeError> {
        self.deserialize_str(visitor)
    }

    fn deserialize_bytes<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, DeError> {
        self.deserialize_any(visitor)
    }

    fn deserialize_byte_buf<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, DeError> {
        self.deserialize_any(visitor)
    }

    fn deserialize_option<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, DeError> {
        match &self.node.kind {
            Kind::Null => self.locate(visitor.visit_none()),
            _ => visitor.visit_some(self),
        }
    }

    fn deserialize_unit<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, DeError> {
        let result = match &self.node.kind {
            Kind::Null => visitor.visit_unit(),
            _ => self.type_error(&visitor),
        };
        self.locate(result)
    }

    fn deserialize_unit_struct<V: Visitor<'de>>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value, DeError> {
        self.deserialize_unit(visitor)
    }

    fn deserialize_newtype_struct<V: Visitor<'de>>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value, DeError> {
        visitor.visit_newtype_struct(self)
    }

    fn deserialize_seq<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, DeError> {
        let result = match &self.node.kind {
            Kind::Seq(items) => visitor.visit_seq(SeqDe {
                items: items.iter().enumerate(),
                path: &self.path,
            }),
            _ => self.type_error(&visitor),
        };
        self.locate(result)
    }

    fn deserialize_tuple<V: Visitor<'de>>(
        self,
        _len: usize,
        visitor: V,
    ) -> Result<V::Value, DeError> {
        self.deserialize_seq(visitor)
    }

    fn deserialize_tuple_struct<V: Visitor<'de>>(
        self,
        _name: &'static str,
        _len: usize,
        visitor: V,
    ) -> Result<V::Value, DeError> {
        self.deserialize_seq(visitor)
    }

    fn deserialize_map<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, DeError> {
        let result = match &self.node.kind {
            Kind::Map(entries) => visitor.visit_map(MapDe::new(entries, &self.path)),
            _ => self.type_error(&visitor),
        };
        self.locate(result)
    }

    fn deserialize_struct<V: Visitor<'de>>(
        self,
        _name: &'static str,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, DeError> {
        self.deserialize_map(visitor)
    }

    fn deserialize_enum<V: Visitor<'de>>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, DeError> {
        // Data files only use unit variants written as plain text, e.g. `art: rohstoff`.
        let result = match &self.node.kind {
            Kind::Str(s) => {
                visitor.visit_enum(IntoDeserializer::<DeError>::into_deserializer(s.as_str()))
            }
            _ => self.type_error(&visitor),
        };
        self.locate(result)
    }

    fn deserialize_ignored_any<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, DeError> {
        visitor.visit_unit()
    }
}

struct SeqDe<'a, 'p> {
    items: std::iter::Enumerate<std::slice::Iter<'a, Node>>,
    path: &'p Path,
}

impl<'de> SeqAccess<'de> for SeqDe<'de, '_> {
    type Error = DeError;

    fn next_element_seed<S: DeserializeSeed<'de>>(
        &mut self,
        seed: S,
    ) -> Result<Option<S::Value>, DeError> {
        match self.items.next() {
            Some((index, node)) => seed
                .deserialize(NodeDe {
                    node,
                    path: self.path.index(index),
                })
                .map(Some),
            None => Ok(None),
        }
    }

    fn size_hint(&self) -> Option<usize> {
        Some(self.items.len())
    }
}

struct MapDe<'a, 'p> {
    entries: std::slice::Iter<'a, (Node, Node)>,
    path: &'p Path,
    value: Option<(&'a Node, Path)>,
}

impl<'a, 'p> MapDe<'a, 'p> {
    fn new(entries: &'a [(Node, Node)], path: &'p Path) -> Self {
        Self {
            entries: entries.iter(),
            path,
            value: None,
        }
    }
}

impl<'de> MapAccess<'de> for MapDe<'de, '_> {
    type Error = DeError;

    fn next_key_seed<S: DeserializeSeed<'de>>(
        &mut self,
        seed: S,
    ) -> Result<Option<S::Value>, DeError> {
        let Some((key, value)) = self.entries.next() else {
            return Ok(None);
        };
        let name = key.scalar_text().unwrap_or_default();
        self.value = Some((value, self.path.field(name)));
        seed.deserialize(NodeDe {
            node: key,
            path: self.path.key(name),
        })
        .map(Some)
    }

    fn next_value_seed<S: DeserializeSeed<'de>>(&mut self, seed: S) -> Result<S::Value, DeError> {
        let (node, path) = self
            .value
            .take()
            .expect("next_value_seed called before next_key_seed");
        seed.deserialize(NodeDe { node, path })
    }

    fn size_hint(&self) -> Option<usize> {
        Some(self.entries.len())
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use serde::Deserialize;

    use super::*;
    use crate::yaml::parse;

    #[derive(Debug, Deserialize, PartialEq)]
    #[serde(deny_unknown_fields)]
    struct Example {
        id: String,
        menge: f64,
        tage: u32,
        #[serde(default)]
        eingang: BTreeMap<String, f64>,
        art: Art,
        werte: Option<BTreeMap<i32, f64>>,
    }

    #[derive(Debug, Deserialize, PartialEq)]
    #[serde(rename_all = "snake_case")]
    enum Art {
        Rohstoff,
        Halbzeug,
    }

    fn read(text: &str) -> Result<Example, DeError> {
        let node = parse(text).unwrap();
        from_node(&node, &Path::default().field("produkte").index(0))
    }

    #[test]
    fn reads_valid_entry() {
        let e = read("id: stahl\nmenge: 1_000\ntage: 2\neingang: {roheisen: 1.1}\nart: halbzeug\nwerte: {1900: 5}\n")
            .unwrap();
        assert_eq!(e.menge, 1000.0);
        assert_eq!(e.eingang["roheisen"], 1.1);
        assert_eq!(e.art, Art::Halbzeug);
        assert_eq!(e.werte.unwrap()[&1900], 5.0);
    }

    #[test]
    fn unknown_field_with_suggestion() {
        let e = read("id: a\nmenge: 1\ntage: 1\nart: rohstoff\neingnag: {}\n").unwrap_err();
        assert!(
            e.message
                .starts_with("Unbekanntes Feld „eingnag“. Meinten Sie „eingang“?"),
            "{}",
            e.message
        );
        assert_eq!(e.position.unwrap().line, 5);
        assert_eq!(e.path.unwrap().to_string(), "produkte[0].eingnag");
    }

    #[test]
    fn missing_field() {
        let e = read("id: a\ntage: 1\nart: rohstoff\n").unwrap_err();
        assert_eq!(e.message, "Pflichtfeld „menge“ fehlt.");
        assert_eq!(e.position.unwrap().line, 1);
    }

    #[test]
    fn wrong_type_with_decimal_comma_hint() {
        let e = read("id: a\nmenge: 1,5\ntage: 1\nart: rohstoff\n").unwrap_err();
        assert!(
            e.message
                .starts_with("Erwartet wird eine Zahl, gefunden wurde der Text „1,5“."),
            "{}",
            e.message
        );
        assert!(e.message.contains("mit Punkt"));
        assert_eq!(e.position.unwrap().line, 2);
    }

    #[test]
    fn fractional_number_for_integer() {
        let e = read("id: a\nmenge: 1\ntage: 1.5\nart: rohstoff\n").unwrap_err();
        assert!(e.message.contains("eine ganze Zahl ab 0"), "{}", e.message);
        assert_eq!(e.position.unwrap().line, 3);
    }

    #[test]
    fn unknown_enum_value() {
        let e = read("id: a\nmenge: 1\ntage: 1\nart: rohstof\n").unwrap_err();
        assert!(
            e.message
                .starts_with("Unbekannter Wert „rohstof“. Meinten Sie „rohstoff“?"),
            "{}",
            e.message
        );
        assert_eq!(e.position.unwrap().line, 4);
    }

    #[test]
    fn error_inside_nested_map_points_to_value() {
        let e = read("id: a\nmenge: 1\ntage: 1\nart: rohstoff\neingang:\n  x: 1\n  y: viel\n")
            .unwrap_err();
        assert_eq!(e.position.unwrap().line, 7);
        assert_eq!(e.path.unwrap().to_string(), "produkte[0].eingang.y");
    }
}
