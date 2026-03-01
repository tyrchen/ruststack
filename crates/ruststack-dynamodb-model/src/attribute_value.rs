//! DynamoDB `AttributeValue` type with custom serialization.
//!
//! `AttributeValue` is a tagged union where exactly one variant is present.
//! The JSON wire format uses single-key objects like `{"S": "hello"}`.

use std::collections::HashMap;
use std::fmt;

use serde::de::{self, MapAccess, Visitor};
use serde::ser::SerializeMap;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// DynamoDB attribute value.
///
/// Represented as a tagged union where exactly one variant is present.
/// Numbers are always string-encoded to preserve arbitrary precision.
#[derive(Debug, Clone, PartialEq)]
pub enum AttributeValue {
    /// String value.
    S(String),
    /// Number value (string-encoded for arbitrary precision).
    N(String),
    /// Binary value (base64-encoded in JSON).
    B(bytes::Bytes),
    /// String Set.
    Ss(Vec<String>),
    /// Number Set (string-encoded).
    Ns(Vec<String>),
    /// Binary Set (base64-encoded in JSON).
    Bs(Vec<bytes::Bytes>),
    /// Boolean value.
    Bool(bool),
    /// Null value.
    Null(bool),
    /// List of attribute values.
    L(Vec<AttributeValue>),
    /// Map of attribute values.
    M(HashMap<String, AttributeValue>),
}

impl AttributeValue {
    /// Returns `true` if this is a string value.
    #[must_use]
    pub fn is_s(&self) -> bool {
        matches!(self, Self::S(_))
    }

    /// Returns `true` if this is a number value.
    #[must_use]
    pub fn is_n(&self) -> bool {
        matches!(self, Self::N(_))
    }

    /// Returns `true` if this is a binary value.
    #[must_use]
    pub fn is_b(&self) -> bool {
        matches!(self, Self::B(_))
    }

    /// Returns `true` if this is a boolean value.
    #[must_use]
    pub fn is_bool(&self) -> bool {
        matches!(self, Self::Bool(_))
    }

    /// Returns `true` if this is a null value.
    #[must_use]
    pub fn is_null(&self) -> bool {
        matches!(self, Self::Null(true))
    }

    /// Returns `true` if this is a list value.
    #[must_use]
    pub fn is_l(&self) -> bool {
        matches!(self, Self::L(_))
    }

    /// Returns `true` if this is a map value.
    #[must_use]
    pub fn is_m(&self) -> bool {
        matches!(self, Self::M(_))
    }

    /// Returns the string value if this is an `S` variant.
    #[must_use]
    pub fn as_s(&self) -> Option<&str> {
        match self {
            Self::S(s) => Some(s),
            _ => None,
        }
    }

    /// Returns the number string if this is an `N` variant.
    #[must_use]
    pub fn as_n(&self) -> Option<&str> {
        match self {
            Self::N(n) => Some(n),
            _ => None,
        }
    }

    /// Returns the map if this is an `M` variant.
    #[must_use]
    pub fn as_m(&self) -> Option<&HashMap<String, AttributeValue>> {
        match self {
            Self::M(m) => Some(m),
            _ => None,
        }
    }

    /// Returns the list if this is an `L` variant.
    #[must_use]
    pub fn as_l(&self) -> Option<&[AttributeValue]> {
        match self {
            Self::L(l) => Some(l),
            _ => None,
        }
    }

    /// Returns the boolean if this is a `Bool` variant.
    #[must_use]
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Self::Bool(b) => Some(*b),
            _ => None,
        }
    }

    /// Returns the DynamoDB type descriptor string (e.g., "S", "N", "BOOL").
    #[must_use]
    pub fn type_descriptor(&self) -> &'static str {
        match self {
            Self::S(_) => "S",
            Self::N(_) => "N",
            Self::B(_) => "B",
            Self::Ss(_) => "SS",
            Self::Ns(_) => "NS",
            Self::Bs(_) => "BS",
            Self::Bool(_) => "BOOL",
            Self::Null(_) => "NULL",
            Self::L(_) => "L",
            Self::M(_) => "M",
        }
    }
}

impl Eq for AttributeValue {}

impl std::hash::Hash for AttributeValue {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        core::mem::discriminant(self).hash(state);
        match self {
            Self::S(s) => s.hash(state),
            Self::N(n) => n.hash(state),
            Self::B(b) => b.hash(state),
            Self::Bool(b) | Self::Null(b) => b.hash(state),
            Self::Ss(v) | Self::Ns(v) => v.hash(state),
            Self::Bs(v) => {
                for b in v {
                    b.hash(state);
                }
            }
            Self::L(v) => v.hash(state),
            Self::M(m) => {
                // Deterministic hash for maps: sort keys.
                let mut pairs: Vec<_> = m.iter().collect();
                pairs.sort_by_key(|(k, _)| *k);
                for (k, v) in pairs {
                    k.hash(state);
                    v.hash(state);
                }
            }
        }
    }
}

impl fmt::Display for AttributeValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::S(s) => write!(f, "{{S: {s}}}"),
            Self::N(n) => write!(f, "{{N: {n}}}"),
            Self::B(b) => write!(f, "{{B: {} bytes}}", b.len()),
            Self::Ss(v) => write!(f, "{{SS: {v:?}}}"),
            Self::Ns(v) => write!(f, "{{NS: {v:?}}}"),
            Self::Bs(v) => write!(f, "{{BS: {} items}}", v.len()),
            Self::Bool(b) => write!(f, "{{BOOL: {b}}}"),
            Self::Null(b) => write!(f, "{{NULL: {b}}}"),
            Self::L(v) => write!(f, "{{L: {} items}}", v.len()),
            Self::M(m) => write!(f, "{{M: {} keys}}", m.len()),
        }
    }
}

impl Serialize for AttributeValue {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(Some(1))?;
        match self {
            Self::S(s) => map.serialize_entry("S", s)?,
            Self::N(n) => map.serialize_entry("N", n)?,
            Self::B(b) => {
                use base64::Engine;
                let encoded = base64::engine::general_purpose::STANDARD.encode(b);
                map.serialize_entry("B", &encoded)?;
            }
            Self::Ss(v) => map.serialize_entry("SS", v)?,
            Self::Ns(v) => map.serialize_entry("NS", v)?,
            Self::Bs(v) => {
                use base64::Engine;
                let encoded: Vec<String> = v
                    .iter()
                    .map(|b| base64::engine::general_purpose::STANDARD.encode(b))
                    .collect();
                map.serialize_entry("BS", &encoded)?;
            }
            Self::Bool(b) => map.serialize_entry("BOOL", b)?,
            Self::Null(b) => map.serialize_entry("NULL", b)?,
            Self::L(list) => map.serialize_entry("L", list)?,
            Self::M(m) => map.serialize_entry("M", m)?,
        }
        map.end()
    }
}

impl<'de> Deserialize<'de> for AttributeValue {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserializer.deserialize_map(AttributeValueVisitor)
    }
}

struct AttributeValueVisitor;

impl<'de> Visitor<'de> for AttributeValueVisitor {
    type Value = AttributeValue;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("a DynamoDB AttributeValue object with exactly one type key")
    }

    fn visit_map<M: MapAccess<'de>>(self, mut map: M) -> Result<Self::Value, M::Error> {
        let Some(key) = map.next_key::<String>()? else {
            return Err(de::Error::custom(
                "AttributeValue must have exactly one key",
            ));
        };

        let value = match key.as_str() {
            "S" => AttributeValue::S(map.next_value()?),
            "N" => AttributeValue::N(map.next_value()?),
            "B" => {
                use base64::Engine;
                let encoded: String = map.next_value()?;
                let decoded = base64::engine::general_purpose::STANDARD
                    .decode(&encoded)
                    .map_err(de::Error::custom)?;
                AttributeValue::B(bytes::Bytes::from(decoded))
            }
            "SS" => AttributeValue::Ss(map.next_value()?),
            "NS" => AttributeValue::Ns(map.next_value()?),
            "BS" => {
                use base64::Engine;
                let encoded: Vec<String> = map.next_value()?;
                let decoded: Result<Vec<bytes::Bytes>, _> = encoded
                    .iter()
                    .map(|e| {
                        base64::engine::general_purpose::STANDARD
                            .decode(e)
                            .map(bytes::Bytes::from)
                    })
                    .collect();
                AttributeValue::Bs(decoded.map_err(de::Error::custom)?)
            }
            "BOOL" => AttributeValue::Bool(map.next_value()?),
            "NULL" => AttributeValue::Null(map.next_value()?),
            "L" => AttributeValue::L(map.next_value()?),
            "M" => AttributeValue::M(map.next_value()?),
            other => {
                return Err(de::Error::unknown_field(
                    other,
                    &["S", "N", "B", "SS", "NS", "BS", "BOOL", "NULL", "L", "M"],
                ));
            }
        };

        Ok(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_should_serialize_string_value() {
        let val = AttributeValue::S("hello".to_owned());
        let json = serde_json::to_string(&val).unwrap();
        assert_eq!(json, r#"{"S":"hello"}"#);
    }

    #[test]
    fn test_should_serialize_number_value() {
        let val = AttributeValue::N("42".to_owned());
        let json = serde_json::to_string(&val).unwrap();
        assert_eq!(json, r#"{"N":"42"}"#);
    }

    #[test]
    fn test_should_serialize_bool_value() {
        let val = AttributeValue::Bool(true);
        let json = serde_json::to_string(&val).unwrap();
        assert_eq!(json, r#"{"BOOL":true}"#);
    }

    #[test]
    fn test_should_serialize_null_value() {
        let val = AttributeValue::Null(true);
        let json = serde_json::to_string(&val).unwrap();
        assert_eq!(json, r#"{"NULL":true}"#);
    }

    #[test]
    fn test_should_serialize_list_value() {
        let val = AttributeValue::L(vec![
            AttributeValue::S("a".to_owned()),
            AttributeValue::N("1".to_owned()),
        ]);
        let json = serde_json::to_string(&val).unwrap();
        assert_eq!(json, r#"{"L":[{"S":"a"},{"N":"1"}]}"#);
    }

    #[test]
    fn test_should_roundtrip_map_value() {
        let mut m = HashMap::new();
        m.insert("key".to_owned(), AttributeValue::S("value".to_owned()));
        let val = AttributeValue::M(m);
        let json = serde_json::to_string(&val).unwrap();
        let deserialized: AttributeValue = serde_json::from_str(&json).unwrap();
        assert_eq!(val, deserialized);
    }

    #[test]
    fn test_should_roundtrip_binary_value() {
        let val = AttributeValue::B(bytes::Bytes::from_static(b"test data"));
        let json = serde_json::to_string(&val).unwrap();
        let deserialized: AttributeValue = serde_json::from_str(&json).unwrap();
        assert_eq!(val, deserialized);
    }

    #[test]
    fn test_should_deserialize_number_set() {
        let json = r#"{"NS":["1","2","3"]}"#;
        let val: AttributeValue = serde_json::from_str(json).unwrap();
        assert!(matches!(val, AttributeValue::Ns(ref v) if v.len() == 3));
    }

    #[test]
    fn test_should_deserialize_string_set() {
        let json = r#"{"SS":["a","b"]}"#;
        let val: AttributeValue = serde_json::from_str(json).unwrap();
        assert!(matches!(val, AttributeValue::Ss(ref v) if v.len() == 2));
    }
}
