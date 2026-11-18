//! Actual observed immutable profile; never a desired write/migration policy.
use super::*;
use std::any::Any;
use std::mem::{align_of, size_of};
mod append;
mod coverage;
mod opening;
#[cfg(all(test, windows))]
pub(super) use opening::OPENING_QUERY_BYTES;

const ITEMS: usize = 256;
const TEXT_BYTES: usize = ITEMS * (256 + 64);
#[derive(Clone, Copy, Default)]
struct Entry {
    start: u32,
    name: u16,
    version: u8,
    required: bool,
}
struct Payload {
    origin: FileProfileOrigin,
    entries: [Entry; ITEMS],
    text: Vec<u8>,
    len: usize,
}
// Actual payload (including its text allocation) drops before the reservation.
struct Provider {
    payload: Box<Payload>,
    location: Option<opening::Location>,
    bytes: usize,
    _lease: ByteLease,
}
impl FileProfileSource for Provider {
    fn origin(&self) -> FileProfileOrigin {
        self.payload.origin
    }
    fn logical_bytes(&self) -> usize {
        self.bytes
    }
}
fn fixed_bytes() -> usize {
    4 * size_of::<Payload>()
        + 2 * size_of::<Provider>()
        + 2 * size_of::<std::sync::atomic::AtomicUsize>()
        + align_of::<Provider>()
        + opening::LOCATION_WORK_BYTES
}
const _: () = assert!(size_of::<Entry>() == 8);
fn fields(v: &Value) -> Result<(&str, &str, bool), FileFactsError> {
    let name = v["name"].as_str().ok_or(FileFactsError::ValidationFailed)?;
    let version = v["version"]
        .as_str()
        .ok_or(FileFactsError::ValidationFailed)?;
    let required = v["required"]
        .as_bool()
        .ok_or(FileFactsError::ValidationFailed)?;
    if name.is_empty() || name.len() > 256 || version.len() > 64 {
        return Err(FileFactsError::ValidationFailed);
    }
    Ok((name, version, required))
}
fn create(
    control: &Arc<ByteBudget>,
    origin: FileProfileOrigin,
    source: Option<&snapshot::Source>,
    features: &[Value],
    cancel: &CaptureCancellation,
    probe: Probe,
) -> Result<Arc<dyn FileProfileSource>, FileFactsError> {
    if features.len() > ITEMS {
        return Err(FileFactsError::Limit(FileFactsLimit::Control));
    }
    let mut total = 0usize;
    let mut previous = None;
    for f in features {
        check(cancel).map_err(storage)?;
        let (name, version, _) = fields(f)?;
        if previous.is_some_and(|p| p >= (name, version)) {
            return Err(FileFactsError::ValidationFailed);
        }
        previous = Some((name, version));
        total = total
            .checked_add(name.len() + version.len())
            .ok_or(FileFactsError::Limit(FileFactsLimit::Control))?;
    }
    if total > TEXT_BYTES {
        return Err(FileFactsError::Limit(FileFactsLimit::Control));
    }
    let bytes = fixed_bytes() + total;
    let lease = control
        .reserve(bytes)
        .ok_or(FileFactsError::Limit(FileFactsLimit::Control))?;
    let mut payload = Box::new(Payload {
        origin,
        entries: [Entry::default(); ITEMS],
        text: Vec::new(),
        len: features.len(),
    });
    payload
        .text
        .try_reserve_exact(total)
        .map_err(|_| FileFactsError::Limit(FileFactsLimit::Control))?;
    if payload.text.capacity() != total {
        return Err(FileFactsError::Limit(FileFactsLimit::Control));
    }
    for (i, f) in features.iter().enumerate() {
        check(cancel).map_err(storage)?;
        let (name, version, required) = fields(f)?;
        payload.entries[i] = Entry {
            start: payload.text.len() as u32,
            name: name.len() as u16,
            version: version.len() as u8,
            required,
        };
        payload.text.extend_from_slice(name.as_bytes());
        payload.text.extend_from_slice(version.as_bytes());
    }
    probe
        .step(Stage::ReadProfileSource, cancel)
        .map_err(storage)?;
    Ok(Arc::new(Provider {
        payload,
        location: source.and_then(opening::Location::observe),
        bytes,
        _lease: lease,
    }))
}
impl FileFactsReader {
    /// Joint full read-only restoration and immutable source publication. No
    /// source file handle survives publication; physical continuity is unproven.
    pub fn restore_profile(
        &mut self,
        source: &Path,
        request: FileProfileRestoreRequest,
        originals: &BlobStore,
        cancel: CaptureCancellation,
    ) -> Result<RestoreCandidate, FileFactsError> {
        self.read_profile(source, request, originals, cancel, false)
            .map(|(candidate, _)| candidate)
    }

    // Both paths attach the very same authenticated origin. Only registration
    // moves that observation and its original handles out of this read.
    pub(super) fn read_profile(
        &mut self,
        source: &Path,
        request: FileProfileRestoreRequest,
        originals: &BlobStore,
        cancel: CaptureCancellation,
        retain: bool,
    ) -> Result<(RestoreCandidate, Option<InspectedFileFacts>), FileFactsError> {
        if request.version != FILE_PROFILE_VERSION {
            return Err(FileFactsError::UnsupportedVersion);
        }
        let (candidate, retained, source, lease) = {
            let (data, restoration, source, lease) =
                self.read_observation(source, request.file, cancel.clone(), Some(originals))?;
            let Restoration {
                mut candidate,
                features,
            } = restoration.ok_or(FileFactsError::ValidationFailed)?;
            let origin = FileProfileOrigin {
                version: FILE_PROFILE_VERSION,
                file: request.file,
                observation: data.summary.observation,
                root: data.root,
                opened_head: data.summary.selected_head,
                features_root: data.metadata.latest_features_root(),
                source_sha256: source.digest,
            };
            #[cfg(test)]
            let probe = self.probe;
            #[cfg(not(test))]
            let probe = Probe::default();
            let profile = create(
                &self.control,
                origin,
                Some(&source),
                &features,
                &cancel,
                probe,
            )?;
            candidate
                .attach_file_profile(FILE_PROFILE_VERSION, profile)
                .map_err(|_| FileFactsError::ValidationFailed)?;
            check(&cancel).map_err(storage)?;
            drop(features);
            let retained = if retain { Some(Box::new(data)) } else { None };
            (candidate, retained, source, lease)
        };
        let view = if let Some(data) = retained {
            Some(InspectedFileFacts {
                data,
                _source: source,
                _lease: lease,
            })
        } else {
            drop(source);
            drop(lease);
            None
        };
        Ok((candidate, view))
    }
}
fn provider(snapshot: &CaptureSnapshot) -> Option<&Provider> {
    let source = snapshot.file_profile(FILE_PROFILE_VERSION).ok()??;
    (source as &dyn Any).downcast_ref::<Provider>()
}
fn scoped_provider(
    snapshot: &CaptureSnapshot,
    ticket: SaveTicket,
    reader: u128,
) -> Option<&Provider> {
    let p = provider(snapshot)?;
    let o = p.origin();
    let m = snapshot.manifest();
    if o.version != FILE_PROFILE_VERSION
        || o.observation.instance != reader
        || o.file.binding != ticket.target
        || o.file.key.session != ticket.source.session_id
        || o.root != m.root
        || o.file.binding.history != Some((&m.identity).into())
        || o.file.binding.project != m.project_id
    {
        return None;
    }
    Some(p)
}
pub(super) fn captured_origin(
    snapshot: &CaptureSnapshot,
    ticket: SaveTicket,
    reader: u128,
) -> Option<FileProfileOrigin> {
    scoped_provider(snapshot, ticket, reader).map(FileProfileSource::origin)
}
pub(super) fn captured(
    snapshot: &CaptureSnapshot,
    ticket: SaveTicket,
    reader: u128,
    work: &mut intent_context::Workspace,
    cancel: &CaptureCancellation,
    probe: Probe,
) -> Result<Option<FileCapturedMetadataFacts>, FileFactsError> {
    let Some(p) = scoped_provider(snapshot, ticket, reader) else {
        return Ok(None);
    };
    encode(
        &snapshot.manifest().archive_metadata,
        &p.payload,
        work,
        cancel,
        probe,
    )
    .map(Some)
}
fn encode(
    meta: &ArchiveMetadata,
    source: &Payload,
    work: &mut intent_context::Workspace,
    cancel: &CaptureCancellation,
    probe: Probe,
) -> Result<FileCapturedMetadataFacts, FileFactsError> {
    let (metadata, required_features) = encode_metadata(meta, source, work, cancel, None)?;
    probe
        .step(Stage::ReadCapturedMetadata, cancel)
        .map_err(storage)?;
    Ok(FileCapturedMetadataFacts {
        origin: source.origin,
        proof_version: FILE_COORD_PROOF_VERSION,
        metadata,
        features: source.len,
        required_features,
    })
}

fn encode_metadata(
    meta: &ArchiveMetadata,
    source: &Payload,
    work: &mut intent_context::Workspace,
    cancel: &CaptureCancellation,
    append: Option<&ArchiveReceipt>,
) -> Result<([u8; 32], usize), FileFactsError> {
    if meta.tags.len() > work.len()
        || meta.extensions.len() > 1024
        || meta.manual_marks.len() > MAX_FILE_FACTS_COMMITS
    {
        return Err(FileFactsError::Limit(FileFactsLimit::Control));
    }
    check(cancel).map_err(storage)?;
    let mut h = proof::Encoder::new(3);
    proof::metadata_receipts(&mut h, meta, append);
    for (i, slot) in work[..meta.tags.len()].iter_mut().enumerate() {
        *slot = i;
    }
    work[..meta.tags.len()]
        .sort_unstable_by_key(|i| (meta.tags[*i].node, meta.tags[*i].name.as_str()));
    h.number(meta.tags.len() as u64);
    for i in &work[..meta.tags.len()] {
        check(cancel).map_err(storage)?;
        let t = &meta.tags[*i];
        proof::metadata_tag(&mut h, t.node.get(), &t.name, &t.value);
    }
    for (i, slot) in work[..meta.extensions.len()].iter_mut().enumerate() {
        *slot = i;
    }
    work[..meta.extensions.len()].sort_unstable_by_key(|i| meta.extensions[*i].name.as_str());
    h.number(meta.extensions.len() as u64);
    for i in &work[..meta.extensions.len()] {
        check(cancel).map_err(storage)?;
        let e = &meta.extensions[*i];
        proof::metadata_extension(&mut h, &e.name, &e.bytes);
    }
    h.number(source.len as u64);
    let mut required = 0;
    for e in &source.entries[..source.len] {
        check(cancel).map_err(storage)?;
        let start = e.start as usize;
        let mid = start + usize::from(e.name);
        h.bytes(&source.text[start..mid]);
        h.bytes(&source.text[mid..mid + usize::from(e.version)]);
        h.number(u64::from(e.required));
        required += usize::from(e.required);
    }
    Ok((h.finish().map_err(storage)?, required))
}
impl InspectedFileFacts {
    /// Both append and coverage queries require the original file and a fresh
    /// ticket ID. The callers independently check metadata and write capacity.
    fn original_metadata_baseline(
        &self,
        q: &FileSaveCandidate,
        context: FileCapturedMetadataContext,
    ) -> Result<Option<(FileCapturedMetadataFacts, FileHistoricalMetadataFacts)>, FileFactsError>
    {
        let before = self.compare_captured_metadata(
            q,
            FileCapturedMetadataContext {
                version: FILE_PROFILE_VERSION,
                ..context
            },
        )?;
        let target = context.ticket.target;
        if Some(context.commit) != target.expected.commit {
            return Err(FileFactsError::InvalidRequest);
        }
        let (Some(captured), Some(parent)) = (before.captured, before.selected) else {
            return Ok(None);
        };
        let summary = self.summary();
        if context.file.binding != target
            || summary.files.latest != Some(parent.commit.fact)
            || Some(parent.commit.fact.generation) != target.expected.generation
            || captured.origin.features_root != parent.features_root
            || captured.origin.source_sha256 != self._source.digest
            || self.data.commits[..summary.commits]
                .iter()
                .flatten()
                .any(|c| c.fact.commit == context.ticket.commit)
        {
            return Ok(None);
        }
        Ok(Some((captured, parent)))
    }

    pub fn compare_captured_metadata(
        &self,
        q: &FileSaveCandidate,
        context: FileCapturedMetadataContext,
    ) -> Result<FileCapturedMetadataComparison, FileFactsError> {
        if context.version != FILE_PROFILE_VERSION {
            return Err(FileFactsError::UnsupportedVersion);
        }
        self.compare_save(
            q,
            FileSaveCompareContext {
                version: FILE_SAVE_INTENT_VERSION,
                observation: context.observation,
                ticket: context.ticket,
            },
        )?;
        let selected = self
            .historical_metadata(FileHistoricalMetadataRequest {
                version: FILE_HISTORICAL_METADATA_VERSION,
                observation: context.observation,
                file: context.file,
                commit: context.commit,
            })?
            .facts;
        let captured = q.captured_metadata();
        Ok(FileCapturedMetadataComparison {
            version: FILE_PROFILE_VERSION,
            context,
            captured,
            selected,
            metadata_equal: captured
                .zip(selected)
                .map(|(a, b)| a.metadata == b.metadata),
            assurance: FileFactsAssurance::ExperimentalValidation,
            conclusion: FileCaptureConclusion::InsufficientForSaveReconciliation,
        })
    }
}

#[cfg(all(test, windows))]
pub(super) fn source_bytes(text: usize) -> usize {
    fixed_bytes() + text
}
#[cfg(test)]
mod tests;
