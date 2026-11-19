//! Bounded JSON with duplicate-key and integer-precision rejection.
use serde::de::{DeserializeSeed, Error, MapAccess, SeqAccess, Visitor};
use serde_json::{Map, Number, Value};
use std::borrow::Cow;
use std::fmt;

use super::{CodecError, FormatLimits};
#[cfg(test)]
mod compact_page_tests;

pub(crate) fn parse(raw: &[u8], limits: FormatLimits) -> Result<Value, CodecError> {
    parse_with::<false>(raw, limits)
}

/// Small history pages keep their completed arrays and text until the end of
/// the checkpoint. Discard spare capacity before transferring those buffers.
pub(crate) fn parse_compact_page(raw: &[u8], limits: FormatLimits) -> Result<Value, CodecError> {
    parse_with::<true>(raw, limits)
}

fn parse_with<const COMPACT: bool>(raw: &[u8], limits: FormatLimits) -> Result<Value, CodecError> {
    if raw.len() > limits.block_bytes {
        return Err(CodecError::ResourceLimit("block_bytes"));
    }
    // Check integer *lexemes* before serde can convert an overflowing integer
    // to f64. Quoted content is left to serde's UTF-8/escape validation.
    let mut index = 0;
    let mut quoted = false;
    while index < raw.len() {
        match raw[index] {
            b'\\' if quoted => index += 1,
            b'"' => quoted = !quoted,
            b'-' | b'0'..=b'9' if !quoted => {
                let start = index;
                while index < raw.len()
                    && matches!(raw[index], b'-' | b'+' | b'.' | b'e' | b'E' | b'0'..=b'9')
                {
                    index += 1;
                }
                let token = &raw[start..index];
                if !token.iter().any(|b| matches!(b, b'.' | b'e' | b'E')) {
                    let number = std::str::from_utf8(token)
                        .ok()
                        .and_then(|s| s.parse::<i128>().ok())
                        .ok_or(CodecError::InvalidJson)?;
                    if number <= -(1_i128 << 53) || number >= (1_i128 << 53) {
                        return Err(CodecError::InvalidJson);
                    }
                }
                continue;
            }
            _ => {}
        }
        index += 1;
    }
    let mut remaining = limits.json_values;
    let mut limit_hit = false;
    let mut reader = serde_json::Deserializer::from_slice(raw);
    let value = Seed::<COMPACT> {
        depth: 0,
        max_depth: limits.json_depth,
        remaining: &mut remaining,
        limit_hit: &mut limit_hit,
    }
    .deserialize(&mut reader)
    .map_err(|_| {
        if limit_hit {
            CodecError::ResourceLimit("json_structure")
        } else {
            CodecError::InvalidJson
        }
    })?;
    reader.end().map_err(|_| CodecError::InvalidJson)?;
    Ok(value)
}

struct Seed<'a, const COMPACT: bool> {
    depth: usize,
    max_depth: usize,
    remaining: &'a mut usize,
    limit_hit: &'a mut bool,
}
impl<'de, const COMPACT: bool> DeserializeSeed<'de> for Seed<'_, COMPACT> {
    type Value = Value;
    fn deserialize<D: serde::Deserializer<'de>>(self, reader: D) -> Result<Value, D::Error> {
        if self.depth > self.max_depth || *self.remaining == 0 {
            *self.limit_hit = true;
            return Err(D::Error::custom("JSON admission limit"));
        }
        *self.remaining -= 1;
        reader.deserialize_any(self)
    }
}
fn owned_text<const COMPACT: bool>(value: String) -> String {
    if COMPACT && value.capacity() != value.len() {
        value.into_boxed_str().into_string()
    } else {
        value
    }
}

// Only an unescaped key may borrow the input across next_value_seed. The
// other callback borrows the parser's reusable scratch for this call only.
struct CompactKey;

impl<'de> DeserializeSeed<'de> for CompactKey {
    type Value = Cow<'de, str>;

    fn deserialize<D: serde::Deserializer<'de>>(self, reader: D) -> Result<Self::Value, D::Error> {
        reader.deserialize_str(self)
    }
}

impl<'de> Visitor<'de> for CompactKey {
    type Value = Cow<'de, str>;

    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("a JSON object key")
    }

    fn visit_borrowed_str<E: Error>(self, key: &'de str) -> Result<Self::Value, E> {
        Ok(Cow::Borrowed(key))
    }

    fn visit_str<E: Error>(self, key: &str) -> Result<Self::Value, E> {
        Ok(Cow::Owned(owned_text::<true>(key.to_owned())))
    }
}

// The child has already consumed its JSON node. Any future sibling must use
// at least one of the remaining nodes, even if this array is still open
// while its descendants are being decoded.
fn push_compact_array(values: &mut Vec<Value>, child: Value, remaining: usize) {
    if values.len() == values.capacity() {
        let additional = values.len().max(1).min(remaining.saturating_add(1));
        values.reserve_exact(additional);
    }
    values.push(child);
}

impl<'de, const COMPACT: bool> Visitor<'de> for Seed<'_, COMPACT> {
    type Value = Value;
    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("bounded JSON")
    }
    fn visit_unit<E: Error>(self) -> Result<Value, E> {
        Ok(Value::Null)
    }
    fn visit_bool<E: Error>(self, value: bool) -> Result<Value, E> {
        Ok(Value::Bool(value))
    }
    fn visit_i64<E: Error>(self, value: i64) -> Result<Value, E> {
        Ok(Value::from(value))
    }
    fn visit_u64<E: Error>(self, value: u64) -> Result<Value, E> {
        Ok(Value::from(value))
    }
    fn visit_f64<E: Error>(self, value: f64) -> Result<Value, E> {
        Number::from_f64(value)
            .map(Value::Number)
            .ok_or_else(|| E::custom("nonfinite number"))
    }
    fn visit_str<E: Error>(self, value: &str) -> Result<Value, E> {
        self.visit_string(value.to_owned())
    }
    fn visit_string<E: Error>(self, value: String) -> Result<Value, E> {
        Ok(Value::String(owned_text::<COMPACT>(value)))
    }
    fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Value, A::Error> {
        let mut result = Vec::new();
        while let Some(value) = seq.next_element_seed(Seed::<COMPACT> {
            depth: self.depth + 1,
            max_depth: self.max_depth,
            remaining: self.remaining,
            limit_hit: self.limit_hit,
        })? {
            if COMPACT {
                push_compact_array(&mut result, value, *self.remaining);
            } else {
                result.push(value);
            }
        }
        if COMPACT && result.capacity() != result.len() {
            result = result.into_boxed_slice().into_vec();
        }
        Ok(Value::Array(result))
    }
    fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<Value, A::Error> {
        let mut result = Map::new();
        loop {
            let key = if COMPACT {
                map.next_key_seed(CompactKey)?
            } else {
                map.next_key::<String>()?.map(Cow::Owned)
            };
            let Some(key) = key else { break };
            if result.contains_key(key.as_ref()) {
                return Err(A::Error::custom("duplicate JSON key"));
            }
            let value = map.next_value_seed(Seed::<COMPACT> {
                depth: self.depth + 1,
                max_depth: self.max_depth,
                remaining: self.remaining,
                limit_hit: self.limit_hit,
            })?;
            result.insert(owned_text::<COMPACT>(key.into_owned()), value);
        }
        Ok(Value::Object(result))
    }
}
