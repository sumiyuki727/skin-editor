use super::*;

#[test]
fn plain_keys_borrow_the_input_and_escaped_keys_move_their_decoded_buffer() {
    let raw = "\"plain雪\"".as_bytes();
    let mut reader = serde_json::Deserializer::from_slice(raw);
    let key = CompactKey.deserialize(&mut reader).unwrap();
    reader.end().unwrap();
    let Cow::Borrowed(key) = key else {
        panic!("expected input slice")
    };
    assert_eq!(key, "plain雪");
    assert_eq!(key.as_ptr(), raw[1..].as_ptr());

    let escaped = br#""plain\u96ea\u0000""#;
    let mut reader = serde_json::Deserializer::from_slice(escaped);
    let key = CompactKey.deserialize(&mut reader).unwrap();
    reader.end().unwrap();
    let Cow::Owned(ref s) = key else {
        panic!("expected owned decoded key")
    };
    assert_eq!(s, "plain雪\0");
    assert_eq!(s.capacity(), s.len());
    let ptr = s.as_ptr();
    let inserted_key = key.into_owned();
    assert_eq!(inserted_key.as_ptr(), ptr);
}

#[test]
fn pending_keys_survive_deeper_values_and_duplicate_rejection_precedes_the_value() {
    let outer = r"\u004b".repeat(2048);
    let decoded = "K".repeat(2048);
    let mut value = format!(r#""{}""#, r"\u96ea".repeat(8192));
    for i in 0..30 {
        let key = if i % 2 == 0 { r"k\u0000" } else { "plain" };
        value = format!(r#"{{"{key}":{value}}}"#);
    }
    let raw = format!(r#"{{"{outer}":{value}}}"#);
    let limits = FormatLimits {
        json_depth: 32,
        json_values: 8192,
        block_bytes: 256 * 1024,
        ..FormatLimits::default()
    };
    let root = parse_compact_page(raw.as_bytes(), limits).unwrap();
    assert_eq!(root, parse(raw.as_bytes(), limits).unwrap());
    let mut leaf = &root[&decoded];
    for i in (0..30).rev() {
        leaf = &leaf[if i % 2 == 0 { "k\0" } else { "plain" }];
    }
    assert_eq!(leaf.as_str().unwrap(), "雪".repeat(8192));
    let (mut slots, mut bytes) = (0, 0);
    count(&root, &mut slots, &mut bytes);
    assert_eq!(bytes, decoded.len() + 15 * (2 + 5) + 8192 * "雪".len());

    let tight = FormatLimits {
        json_values: 2,
        ..limits
    };
    for (second, error) in [
        (&decoded[..], CodecError::InvalidJson),
        ("unique", CodecError::ResourceLimit("json_structure")),
    ] {
        let raw = format!(r#"{{"{outer}":null,"{second}":[0]}}"#);
        assert_eq!(parse(raw.as_bytes(), tight), Err(error.clone()));
        assert_eq!(parse_compact_page(raw.as_bytes(), tight), Err(error));
    }
}

#[test]
fn slice_string_decoder_keeps_one_scratch_across_large_short_and_bad_tokens() {
    // Use the same slice decoder with a caller-owned scratch so its capacity
    // is observable without instrumenting the allocator or serde_json.
    use serde_json::de::{Read, SliceRead};
    const B: usize = 256 * 1024;
    let prefix = "a".repeat(140_000);
    let suffix = "b".repeat(B - prefix.len() - 4);
    let raw = format!(r#""{prefix}\n{suffix}""#);
    assert_eq!(raw.len(), B);
    let limits = FormatLimits {
        block_bytes: B,
        ..FormatLimits::default()
    };
    assert_eq!(
        parse_compact_page(raw.as_bytes(), limits)
            .unwrap()
            .as_str()
            .unwrap()
            .len(),
        B - 3
    );
    assert_eq!(
        parse_compact_page(
            raw.as_bytes(),
            FormatLimits {
                block_bytes: B - 1,
                ..limits
            }
        ),
        Err(CodecError::ResourceLimit("block_bytes"))
    );
    let mut scratch = Vec::new();
    let mut source = SliceRead::new(raw.as_bytes());
    assert_eq!(source.next().unwrap(), Some(b'"'));
    {
        let decoded = source.parse_str(&mut scratch).unwrap();
        assert_eq!(&*decoded, format!("{prefix}\n{suffix}"));
    }
    let high = scratch.capacity();
    assert!(high > B && high <= 2 * B);
    for (raw, expected) in [
        (br#""short""#.as_slice(), Some("short")),
        (br#""\uD83D\uDE03""#, Some("😃")),
        (br#""abc\n\uD800""#, None),
    ] {
        scratch.clear();
        let mut source = SliceRead::new(raw);
        assert_eq!(source.next().unwrap(), Some(b'"'));
        assert_eq!(source.parse_str(&mut scratch).ok().as_deref(), expected);
        assert_eq!(scratch.capacity(), high);
    }
}

#[test]
fn long_float_scratch_and_escaped_text_share_the_reader_on_success_and_error() {
    let zeros = "0".repeat(4096);
    let integer = format!("1{zeros}e-4096");
    let fraction = format!("0.{zeros}1234567890123456789012345e4097");
    let underflow = format!("9{zeros}e-9999999999");
    let overflow = format!("9{zeros}e9999999999");
    let limits = FormatLimits {
        json_values: 8192,
        block_bytes: 256 * 1024,
        ..FormatLimits::default()
    };
    let raw = format!(r#"["\n",{integer},"k",{fraction},{underflow},"\u96ea"]"#);
    let v = parse_compact_page(raw.as_bytes(), limits).unwrap();
    assert_eq!(v, parse(raw.as_bytes(), limits).unwrap());
    assert_eq!(v[1].as_f64(), Some(1.0));
    assert_eq!(v[3].as_f64(), Some(1.2345678901234567));
    assert_eq!(v[4].as_f64(), Some(0.0));
    assert_eq!(v[5], "雪");
    let raw = format!(r#"["\n",{fraction},{overflow}]"#);
    assert_eq!(parse(raw.as_bytes(), limits), Err(CodecError::InvalidJson));
    assert_eq!(
        parse_compact_page(raw.as_bytes(), limits),
        Err(CodecError::InvalidJson)
    );
}
