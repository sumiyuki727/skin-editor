use super::*;
#[cfg(test)]
mod tests;

pub(crate) const SORTED_INDEX_BLOCKS: usize = 8192;

type Rows<'a, const N: usize> = [Option<&'a StateBlock>; N];
type Visits<const N: usize> = [bool; N];

// The slots hold only borrowed pointers and visit flags. Allow both array
// initializers and destinations, the view, and fixed lookup/hash temporaries.
// Closure discovery and the decoded model/JSON have independent owners.
pub(crate) const SORTED_INDEX_WORK_BYTES: usize = 2
    * (size_of::<Rows<'static, SORTED_INDEX_BLOCKS>>() + size_of::<Visits<SORTED_INDEX_BLOCKS>>())
    + 2 * (size_of::<Index<'static, 'static>>() + size_of::<Sha256>() + size_of::<StateHash>())
    + size_of::<[usize; 64]>();

pub(super) enum Index<'s, 'a> {
    Mapped(BTreeMap<StateHash, (&'a StateBlock, bool)>),
    Sorted {
        rows: &'s [Option<&'a StateBlock>],
        visited: &'s mut [bool],
    },
}

fn find(rows: &[Option<&StateBlock>], hash: StateHash) -> Option<usize> {
    rows.binary_search_by_key(&hash, |r| r.expect("filled index").hash)
        .ok()
}

impl<'a> Index<'_, 'a> {
    pub(super) fn visit(&mut self, hash: StateHash) -> Option<&'a StateBlock> {
        match self {
            Self::Mapped(map) => {
                let (block, visited) = map.get_mut(&hash)?;
                *visited = true;
                Some(*block)
            }
            Self::Sorted { rows, visited } => {
                let i = find(rows, hash)?;
                visited[i] = true;
                rows[i]
            }
        }
    }
    pub(super) fn contains(&self, hash: StateHash) -> bool {
        match self {
            Self::Mapped(map) => map.contains_key(&hash),
            Self::Sorted { rows, .. } => find(rows, hash).is_some(),
        }
    }
    pub(super) fn all_used(&self) -> bool {
        match self {
            Self::Mapped(map) => map.values().all(|(_, used)| *used),
            Self::Sorted { visited, .. } => visited.iter().all(|used| *used),
        }
    }
}

/// The storage closure yields distinct blocks in hash order. Large standalone
/// archives retain the general codec limits and still borrow their raw bytes.
pub(crate) fn decode_sorted_blocks<'a>(
    root: StateHash,
    blocks: impl ExactSizeIterator<Item = &'a StateBlock>,
    limits: FormatLimits,
) -> Result<SkinProject, CodecError> {
    let limits = limits.validate()?;
    let count = blocks.len();
    if count > limits.blocks {
        return Err(CodecError::ResourceLimit("blocks"));
    }
    if count <= 128 {
        decode_fixed::<128>(root, blocks, limits)
    } else if count <= SORTED_INDEX_BLOCKS {
        decode_fixed::<SORTED_INDEX_BLOCKS>(root, blocks, limits)
    } else {
        decode(root, blocks, limits)
    }
}

fn decode_fixed<'a, const N: usize>(
    root: StateHash,
    mut blocks: impl ExactSizeIterator<Item = &'a StateBlock>,
    limits: FormatLimits,
) -> Result<SkinProject, CodecError> {
    let mut slots: Rows<'a, N> = [None; N];
    let count = blocks.len();
    let rows = &mut slots[..count];
    let mut total = 0;
    let mut previous = None;
    for slot in rows.iter_mut() {
        let block = blocks.next().ok_or(CodecError::MissingBlock)?;
        validate_block(block, &mut total, limits)?;
        if previous == Some(block.hash) {
            return Err(CodecError::DuplicateBlock);
        }
        if previous.is_some_and(|h| h > block.hash) {
            return Err(CodecError::ReferenceMismatch);
        }
        previous = Some(block.hash);
        *slot = Some(block);
    }
    if blocks.next().is_some() {
        return Err(CodecError::ResourceLimit("blocks"));
    }
    drop(blocks);
    let mut visited: Visits<N> = [false; N];
    decode_index(
        root,
        Index::Sorted {
            rows,
            visited: &mut visited[..count],
        },
        limits,
    )
}
