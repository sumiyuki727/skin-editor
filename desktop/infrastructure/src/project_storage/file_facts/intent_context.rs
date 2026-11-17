//! Fixed-index, allocation-free new projections; old codec costs remain unproven.
use super::*;
use proof::Encoder;
pub(super) type Workspace = [usize; 4096];
pub(super) const WORK_BYTES: usize =
    std::mem::size_of::<Workspace>() + 2 * std::mem::size_of::<Encoder>();
// Also charged by the original prepare_capture entry. Include the nested
// address hash and all fixed input/output digests through the prefix update.
pub(super) const PREFIX_WORK_BYTES: usize =
    2 * std::mem::size_of::<Sha256>() + 6 * std::mem::size_of::<[u8; 32]>();
const PREFIX: &[u8] = b"skin-editor/SaveIntent-Proof/v1\0";
fn encoder(domain: u16) -> Encoder {
    Encoder::scoped(PREFIX, domain)
}
fn redo_order(
    redo: &[(PersistentNodeId, PersistentNodeId)],
    work: &mut Workspace,
) -> Result<(), StorageError> {
    if redo.len() > MAX_FILE_CAPTURE_NODES {
        return Err(StorageError::Limit("intent redo"));
    }
    for (i, slot) in work[..redo.len()].iter_mut().enumerate() {
        *slot = i;
    }
    work[..redo.len()].sort_unstable_by_key(|i| redo[*i]);
    if work[..redo.len()]
        .windows(2)
        .any(|p| redo[p[0]].0 == redo[p[1]].0)
    {
        return Err(StorageError::Invalid("intent redo"));
    }
    Ok(())
}
pub(super) fn navigation(
    head: PersistentNodeId,
    alias: u64,
    branch: u64,
    redo: &[(PersistentNodeId, PersistentNodeId)],
    work: &mut Workspace,
) -> Result<[u8; 32], StorageError> {
    redo_order(redo, work)?;
    let mut h = encoder(1);
    h.id(head.get());
    h.number(alias);
    h.number(branch);
    h.number(redo.len() as u64);
    for i in &work[..redo.len()] {
        h.id(redo[*i].0.get());
        h.id(redo[*i].1.get());
    }
    h.finish()
}
pub(super) fn archive(
    metadata: &ArchiveMetadata,
    delta: Option<&ArchiveReceipt>,
    work: &mut Workspace,
) -> Result<[u8; 32], StorageError> {
    if metadata.tags.len() > 4096
        || metadata.extensions.len() > 1024
        || metadata.manual_marks.len() > MAX_FILE_FACTS_COMMITS
    {
        return Err(StorageError::Limit("intent metadata"));
    }
    let mut h = encoder(2);
    proof::metadata_receipts(&mut h, metadata, delta);
    for (i, slot) in work[..metadata.tags.len()].iter_mut().enumerate() {
        *slot = i;
    }
    work[..metadata.tags.len()]
        .sort_unstable_by_key(|i| (metadata.tags[*i].node, metadata.tags[*i].name.as_str()));
    h.number(metadata.tags.len() as u64);
    for i in &work[..metadata.tags.len()] {
        let t = &metadata.tags[*i];
        h.id(t.node.get());
        h.bytes(t.name.as_bytes());
        h.bytes(t.value.as_bytes());
    }
    for (i, slot) in work[..metadata.extensions.len()].iter_mut().enumerate() {
        *slot = i;
    }
    work[..metadata.extensions.len()]
        .sort_unstable_by_key(|i| metadata.extensions[*i].name.as_str());
    h.number(metadata.extensions.len() as u64);
    for i in &work[..metadata.extensions.len()] {
        let e = &metadata.extensions[*i];
        h.bytes(e.name.as_bytes());
        h.bytes(&e.bytes);
    }
    h.finish()
}
pub(super) fn prefix(
    previous: [u8; 32],
    info: &CapturedNodeInfo,
    checksum: [u8; 32],
    admissions: &[CapturedAdmission],
) -> [u8; 32] {
    let address = address_checksum(info);
    let mut h = Sha256::new();
    h.update(b"skin-editor.history-prefix.v1\0");
    h.update(previous);
    h.update(checksum);
    h.update(address);
    h.update(
        (admissions
            .iter()
            .filter(|a| a.first_node == info.id)
            .count() as u64)
            .to_be_bytes(),
    );
    for kind in ["asset", "component", "document", "layer", "output", "sound"] {
        for a in admissions {
            let (found, id) = object(a.object);
            if found == kind && a.first_node == info.id {
                h.update((kind.len() as u64).to_be_bytes());
                h.update(kind);
                h.update(id.to_be_bytes());
                h.update(a.first_node.get().to_be_bytes());
            }
        }
    }
    h.finalize().into()
}
pub(super) fn legacy(
    ticket: SaveTicket,
    m: &CaptureManifest,
    prefix: [u8; 32],
    work: &mut Workspace,
) -> Result<[u8; 32], StorageError> {
    redo_order(&m.redo_preferences, work)?;
    let mut h = Sha256::new();
    h.update(b"skin-editor.archive-request.v1\0");
    h.update(ticket.commit.get().to_be_bytes());
    if let Some(previous) = ticket.target.expected.commit {
        h.update([1]);
        h.update(previous.get().to_be_bytes());
    } else {
        h.update([0]);
    }
    let origin = save_name(ticket.origin);
    h.update((origin.len() as u64).to_be_bytes());
    h.update(origin);
    for id in [
        m.project_id.get(),
        m.identity.history.get(),
        m.identity.lineage.get(),
        m.identity.address_epoch.get(),
        m.root.get(),
        m.head.get(),
    ] {
        h.update(id.to_be_bytes());
    }
    h.update(prefix);
    for n in [m.nodes.len() as u64, m.next_alias, m.next_branch] {
        h.update(n.to_be_bytes());
    }
    h.update((m.redo_preferences.len() as u64).to_be_bytes());
    for i in &work[..m.redo_preferences.len()] {
        h.update(m.redo_preferences[*i].0.get().to_be_bytes());
        h.update(m.redo_preferences[*i].1.get().to_be_bytes());
    }
    Ok(h.finalize().into())
}
pub(super) fn file_context(
    cache: &Cache,
    opening: &RestoreHeader,
    observation: FileObservationId,
) -> Result<(FileIntentFacts, [[u8; 32]; MAX_FILE_FACTS_COMMITS]), StorageError> {
    let mut work = [0; 4096];
    let archive = archive(&cache.metadata, None, &mut work)?;
    let opening_navigation = navigation(
        opening.head,
        opening.next_alias,
        opening.next_branch,
        &opening.redo_preferences,
        &mut work,
    )?;
    let mut commits = [[0; 32]; MAX_FILE_FACTS_COMMITS];
    if cache.checkpoints.len() > commits.len() {
        return Err(StorageError::Limit("intent commits"));
    }
    for (i, c) in cache.checkpoints.iter().enumerate() {
        commits[i] = navigation(
            c.receipt.head,
            c.next_alias,
            c.next_branch,
            &c.redo,
            &mut work,
        )?;
    }
    let latest_navigation = commits[cache
        .checkpoints
        .len()
        .checked_sub(1)
        .ok_or(StorageError::Invalid("intent latest"))?];
    Ok((
        FileIntentFacts {
            version: FILE_SAVE_INTENT_VERSION,
            observation,
            archive,
            opening_navigation,
            latest_navigation,
        },
        commits,
    ))
}
