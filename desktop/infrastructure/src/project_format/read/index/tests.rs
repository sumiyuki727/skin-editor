use super::*;

fn project(assets: u128) -> SkinProject {
    let mut input = ProjectDefinition::empty(ProjectId::new(17).unwrap());
    for i in 1..=assets {
        input.assets.push(
            Asset::new(
                AssetId::new(i).unwrap(),
                format!("source-{i}"),
                BlobRef::new([3; 32], 0),
                AssetKind::Opaque,
            )
            .unwrap(),
        );
    }
    SkinProject::from_definition(input).unwrap()
}

fn sorted(bundle: &StateBundle, limits: FormatLimits) -> Result<SkinProject, CodecError> {
    let mut refs: Vec<_> = bundle.blocks.iter().collect();
    refs.sort_unstable_by_key(|b| b.hash);
    decode_sorted_blocks(bundle.root, refs.into_iter(), limits)
}

#[test]
fn borrowed_and_owned_inputs_share_validation_and_preserve_source_bytes() {
    for assets in [0, 130] {
        let limits = FormatLimits::default();
        let input = project(assets);
        let bundle = encode_project(&input, limits).unwrap();
        assert_eq!(bundle.blocks.len() > 128, assets > 0);
        let before = bundle.clone();
        assert_eq!(sorted(&bundle, limits), Ok(input.clone()));
        assert_eq!(decode_project(&bundle, limits), Ok(input));
        assert_eq!(bundle, before);
        let both = |candidate: &StateBundle, limits, error: CodecError| {
            assert_eq!(sorted(candidate, limits).err(), Some(error.clone()));
            assert_eq!(decode_project(candidate, limits).err(), Some(error));
        };
        let raw_bytes = bundle.blocks.iter().map(|b| b.bytes.len()).sum::<usize>();
        assert!(
            sorted(
                &bundle,
                FormatLimits {
                    total_bytes: raw_bytes,
                    ..limits
                }
            )
            .is_ok()
        );
        both(
            &bundle,
            FormatLimits {
                total_bytes: raw_bytes - 1,
                ..limits
            },
            CodecError::ResourceLimit("bytes"),
        );
        both(
            &bundle,
            FormatLimits {
                decoded_bytes: 1,
                ..limits
            },
            CodecError::ResourceLimit("decoded_bytes"),
        );
        both(
            &bundle,
            FormatLimits {
                json_depth: 65,
                ..limits
            },
            CodecError::InvalidLimits,
        );
        both(
            &bundle,
            FormatLimits {
                blocks: bundle.blocks.len() - 1,
                ..limits
            },
            CodecError::ResourceLimit("blocks"),
        );
        let mut wrong = bundle.clone();
        wrong.blocks[0].bytes.push(0);
        both(&wrong, limits, CodecError::DigestMismatch);
        let mut duplicate = bundle.clone();
        duplicate.blocks.push(bundle.blocks[0].clone());
        both(&duplicate, limits, CodecError::DuplicateBlock);
        let mut missing = bundle.clone();
        missing.blocks.retain(|b| b.hash == bundle.root);
        both(&missing, limits, CodecError::MissingBlock);
        let mut unused = bundle;
        let bytes = b"unreferenced input".to_vec();
        unused.blocks.push(StateBlock {
            hash: hash(&bytes),
            bytes,
        });
        both(&unused, limits, CodecError::UnusedBlock);
    }
}

#[test]
fn fixed_index_maximum_and_larger_standalone_archives_keep_distinct_limits() {
    let limits = FormatLimits::default();
    let mut bundle = encode_project(&project(0), limits).unwrap();
    for i in bundle.blocks.len()..=SORTED_INDEX_BLOCKS {
        let bytes = (i as u64).to_be_bytes().to_vec();
        bundle.blocks.push(StateBlock {
            hash: hash(&bytes),
            bytes,
        });
    }
    let small = FormatLimits {
        blocks: SORTED_INDEX_BLOCKS,
        ..limits
    };
    assert_eq!(
        sorted(&bundle, small).err(),
        Some(CodecError::ResourceLimit("blocks"))
    );
    // An unused block is diagnosed only after decoding the reachable project.
    assert_eq!(sorted(&bundle, limits).err(), Some(CodecError::UnusedBlock));
    bundle.blocks.pop();
    assert_eq!(sorted(&bundle, small).err(), Some(CodecError::UnusedBlock));
    let mut refs: Vec<_> = bundle.blocks.iter().collect();
    refs.sort_unstable_by_key(|b| b.hash);
    refs.swap(1, 2);
    assert_eq!(
        decode_sorted_blocks(bundle.root, refs.into_iter(), small).err(),
        Some(CodecError::ReferenceMismatch)
    );
}
