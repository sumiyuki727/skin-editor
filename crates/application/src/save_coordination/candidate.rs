//! Separately reserved metadata payload; never charged as a control summary.
//! Under pinned Rust 1.95, fresh non-ZST buffers with try_reserve_exact retain
//! the requested capacity (raw_vec/mod.rs, grow_exact/set_ptr_and_cap at commit
//! 59807616e1fa2540724bfbac14d7976d7e4a3860). Every retained capacity is checked.
//! System allocator overhead/rounding is not part of this managed layout proof.
use super::*;
use crate::{ArchiveExtension, ArchiveTag};
use core::mem::size_of;

pub const SAVE_CANDIDATE_LOGICAL_BYTES: usize = 1024 * 1024;
pub const SAVE_CANDIDATE_ALLOCATION_BYTES: usize =
    SAVE_CANDIDATE_LOGICAL_BYTES + budget::arc_bytes::<ArchiveMetadata>();

impl Default for SaveCandidateLimits {
    fn default() -> Self {
        Self {
            logical_bytes: SAVE_CANDIDATE_LOGICAL_BYTES,
            allocation_bytes: SAVE_CANDIDATE_ALLOCATION_BYTES,
        }
    }
}

pub(super) struct PreparedCandidate {
    pub metadata: Arc<ArchiveMetadata>,
    pub cost: SaveCandidateCost,
    pub fact: SaveCommitFact,
}

fn limited() -> SaveDiagnostic {
    SaveDiagnostic::new(SaveDiagnosticCode::RequiresWriteCapacity)
}
fn add(sum: &mut usize, count: usize, size: usize) -> Result<(), SaveDiagnostic> {
    *sum = sum
        .checked_add(count.checked_mul(size).ok_or_else(limited)?)
        .ok_or_else(limited)?;
    Ok(())
}

fn buffer<T>(count: usize) -> Result<Vec<T>, SaveDiagnostic> {
    let mut value = Vec::new();
    value
        .try_reserve_exact(count)
        .map_err(|_| SaveDiagnostic::new(SaveDiagnosticCode::AllocationFailed))?;
    if value.capacity() != count {
        return Err(SaveDiagnostic::new(SaveDiagnosticCode::CapacityProof));
    }
    Ok(value)
}
fn bytes(value: &[u8]) -> Result<Vec<u8>, SaveDiagnostic> {
    let mut result = buffer(value.len())?;
    result.extend_from_slice(value);
    Ok(result)
}
fn string(value: &str) -> Result<String, SaveDiagnostic> {
    // Already validated UTF-8; transferring the buffer does not allocate.
    Ok(String::from_utf8(bytes(value.as_bytes())?).expect("existing UTF-8"))
}

pub(super) fn prepare(
    original: &ArchiveMetadata,
    ticket: SaveTicket,
    limits: SaveCandidateLimits,
) -> Result<PreparedCandidate, SaveDiagnostic> {
    let generation = ticket
        .target
        .expected
        .generation
        .unwrap_or(0)
        .checked_add(1)
        .ok_or_else(limited)?;
    let mark = usize::from(ticket.origin == SaveOrigin::Manual);
    let mark_count = original
        .manual_marks
        .len()
        .checked_add(mark)
        .ok_or_else(limited)?;
    let logical = original
        .logical_bytes()
        .map_err(|_| limited())?
        .checked_add(mark * 64)
        .ok_or_else(limited)?;
    let mut allocated = budget::arc_bytes::<ArchiveMetadata>();
    add(
        &mut allocated,
        mark_count,
        size_of::<(CommitId, crate::PersistentNodeId)>(),
    )?;
    add(&mut allocated, original.tags.len(), size_of::<ArchiveTag>())?;
    add(
        &mut allocated,
        original.extensions.len(),
        size_of::<ArchiveExtension>(),
    )?;
    for tag in &original.tags {
        add(&mut allocated, tag.name.len(), 1)?;
        add(&mut allocated, tag.value.len(), 1)?;
    }
    for extension in &original.extensions {
        add(&mut allocated, extension.name.len(), 1)?;
        add(&mut allocated, extension.bytes.len(), 1)?;
    }
    // Entire candidate reservation precedes its FIRST allocation. Old metadata
    // is immutable, separately retained, and cannot lend its reservation here.
    if logical > limits.logical_bytes
        || logical > SAVE_CANDIDATE_LOGICAL_BYTES
        || allocated > limits.allocation_bytes
        || allocated > SAVE_CANDIDATE_ALLOCATION_BYTES
    {
        return Err(limited());
    }
    let fact = SaveCommitFact {
        commit: ticket.commit,
        generation,
        head: ticket.head,
        origin: ticket.origin,
    };
    let mut marks = buffer(mark_count)?;
    marks.extend_from_slice(&original.manual_marks);
    if mark != 0 {
        marks.push((ticket.commit, ticket.head));
    }
    let mut tags = buffer(original.tags.len())?;
    for tag in &original.tags {
        tags.push(ArchiveTag {
            node: tag.node,
            name: string(&tag.name)?,
            value: string(&tag.value)?,
        });
    }
    let mut extensions = buffer(original.extensions.len())?;
    for extension in &original.extensions {
        extensions.push(ArchiveExtension {
            name: string(&extension.name)?,
            bytes: bytes(&extension.bytes)?,
        });
    }
    let metadata = ArchiveMetadata {
        last_commit: Some(fact.receipt()),
        manual: if mark != 0 {
            Some(fact.receipt())
        } else {
            original.manual.clone()
        },
        protected: Some(fact.receipt()),
        manual_marks: marks,
        tags,
        extensions,
    };
    // All capacities were fixed before filling; no push can trigger growth.
    let mut actual = budget::arc_bytes::<ArchiveMetadata>();
    add(
        &mut actual,
        metadata.manual_marks.capacity(),
        size_of::<(CommitId, crate::PersistentNodeId)>(),
    )?;
    add(
        &mut actual,
        metadata.tags.capacity(),
        size_of::<ArchiveTag>(),
    )?;
    add(
        &mut actual,
        metadata.extensions.capacity(),
        size_of::<ArchiveExtension>(),
    )?;
    for tag in &metadata.tags {
        add(&mut actual, tag.name.capacity(), 1)?;
        add(&mut actual, tag.value.capacity(), 1)?;
    }
    for extension in &metadata.extensions {
        add(&mut actual, extension.name.capacity(), 1)?;
        add(&mut actual, extension.bytes.capacity(), 1)?;
    }
    if actual != allocated {
        return Err(SaveDiagnostic::new(SaveDiagnosticCode::CapacityProof));
    }
    Ok(PreparedCandidate {
        metadata: Arc::new(metadata),
        cost: SaveCandidateCost {
            logical_bytes: logical,
            allocation_bytes: allocated,
        },
        fact,
    })
}

const _: () = {
    assert!(size_of::<(CommitId, crate::PersistentNodeId)>() <= 64);
    assert!(size_of::<ArchiveTag>() <= 64);
    assert!(size_of::<ArchiveExtension>() <= 64);
};
