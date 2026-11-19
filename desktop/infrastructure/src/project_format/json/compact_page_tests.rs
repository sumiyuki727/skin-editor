use super::*;

// Measure only Rust-visible array slots and text buffers in a completed DOM.
fn count(value: &Value, slots: &mut usize, text_bytes: &mut usize) {
    match value {
        Value::Array(rows) => {
            assert_eq!(rows.capacity(), rows.len());
            *slots += rows.capacity();
            for row in rows {
                count(row, slots, text_bytes);
            }
        }
        Value::Object(map) => {
            for (key, child) in map {
                assert_eq!(key.capacity(), key.len());
                *text_bytes += key.capacity();
                count(child, slots, text_bytes);
            }
        }
        Value::String(s) => {
            assert_eq!(s.capacity(), s.len());
            *text_bytes += s.capacity();
        }
        _ => {}
    }
}

#[test]
fn compact_page_retains_exact_nested_slots_and_decoded_text_including_keys() {
    let raw = r#"{"\u0041":[[],["雪","\uD83D\uDE03","\u0000\r\n"],{"":"", "é":"e\u0301"}],"k":[true,false,null,0,1]}"#.as_bytes();
    let limits = FormatLimits::default();
    let value = parse_compact_page(raw, limits).unwrap();
    assert_eq!(value, parse(raw, limits).unwrap());
    let (mut slots, mut bytes) = (0, 0);
    count(&value, &mut slots, &mut bytes);
    assert_eq!(slots, 3 + 3 + 5);
    assert_eq!(bytes, "A雪😃\0\r\née\u{301}k".len());
    assert!(bytes <= raw.len());

    // Force incoming owned text with spare capacity (including empty text).
    for contents in ["", "a", "雪"] {
        let mut s = String::with_capacity(128);
        s.push_str(contents);
        let text = owned_text::<true>(s);
        assert_eq!(text, contents);
        assert_eq!(text.capacity(), contents.len());
    }
}

#[test]
fn compact_page_counts_every_array_edge_at_the_node_limit() {
    let limits = FormatLimits {
        json_values: 8192,
        ..FormatLimits::default()
    };
    let mut raw = String::from("[");
    raw.push_str(&"0,".repeat(limits.json_values - 2));
    raw.push_str("0]");
    let value = parse_compact_page(raw.as_bytes(), limits).unwrap();
    let (mut slots, mut bytes) = (0, 0);
    count(&value, &mut slots, &mut bytes);
    assert_eq!((slots, bytes), (limits.json_values - 1, 0));
    assert_eq!(
        parse_compact_page(
            raw.as_bytes(),
            FormatLimits {
                json_values: limits.json_values - 1,
                ..limits
            }
        ),
        Err(CodecError::ResourceLimit("json_structure"))
    );
}

#[test]
fn compact_page_reports_the_same_first_json_error_as_ordinary_decode() {
    use CodecError::{InvalidJson, ResourceLimit};
    let limits = FormatLimits {
        json_depth: 2,
        json_values: 8,
        ..FormatLimits::default()
    };
    for (raw, expected) in [
        (br#"{"a":[],"\u0061":[[[null]]]}"#.as_slice(), InvalidJson),
        (b"[[[0]]]", ResourceLimit("json_structure")),
        (b"[0,1,2,3,4,5,6,7]", ResourceLimit("json_structure")),
        (b"[[[0]]],9007199254740992", InvalidJson),
        (b"9007199254740992", InvalidJson),
        (b"[\"\xff\"]", InvalidJson),
        (br#"["\uD800"]"#, InvalidJson),
        (b"[] true", InvalidJson),
        (b"[0,]", InvalidJson),
    ] {
        assert_eq!(parse(raw, limits), Err(expected.clone()));
        assert_eq!(parse_compact_page(raw, limits), Err(expected));
    }
    let tiny = FormatLimits {
        block_bytes: 1,
        ..limits
    };
    assert_eq!(
        parse_compact_page(b"[]", tiny),
        Err(ResourceLimit("block_bytes"))
    );
}
