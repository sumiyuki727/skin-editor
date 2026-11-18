//! Experimental healthy-file observations, never save or activation authority.
use super::*;
use crate::byte_budget::{ByteBudget, ByteLease};
mod capture;
mod historical_content;
mod historical_metadata;
mod historical_originals;
mod historical_save;
mod intent_context;
mod original_query;
mod preflight;
mod profile;
mod proof;
mod save;
mod snapshot;
pub use capture::FileCaptureCandidate;
pub use save::FileSaveCandidate;

#[derive(Clone, Copy, Debug)]
pub struct FileFactsLimits {
    pub file_bytes: u64,
    pub scratch_bytes: u64,
    pub control_bytes: usize,
    pub nodes: usize,
    pub admissions: usize,
    pub originals: usize,
    pub original_bytes: u64,
    pub metadata_bytes: usize,
    pub commits: usize,
}
impl Default for FileFactsLimits {
    fn default() -> Self {
        Self {
            file_bytes: 32 * 1024 * 1024,
            scratch_bytes: 32 * 1024 * 1024,
            control_bytes: 512 * 1024,
            nodes: 64,
            admissions: 256,
            originals: 8,
            original_bytes: 256 * 1024,
            metadata_bytes: 1024 * 1024,
            commits: 128,
        }
    }
}
impl FileFactsLimits {
    fn valid(self) -> bool {
        let max = Self::default();
        [self.file_bytes, self.scratch_bytes, self.original_bytes]
            .into_iter()
            .zip([max.file_bytes, max.scratch_bytes, max.original_bytes])
            .all(|(n, m)| n > 0 && n <= m)
            && [
                self.control_bytes,
                self.nodes,
                self.admissions,
                self.originals,
                self.metadata_bytes,
                self.commits,
            ]
            .into_iter()
            .zip([
                max.control_bytes,
                max.nodes,
                max.admissions,
                max.originals,
                max.metadata_bytes,
                max.commits,
            ])
            .all(|(n, m)| n > 0 && n <= m)
    }
    fn file(self) -> FileLimits {
        FileLimits {
            file_bytes: self.file_bytes,
            metadata_bytes: self.metadata_bytes,
            blocks: 8192,
            originals: self.originals,
            original_bytes: self.original_bytes,
            decoded_bytes: 8 * 1024 * 1024,
            restore: RestoreLimits {
                nodes: self.nodes,
                object_ids: self.admissions,
                metadata_bytes: 16 * 1024 * 1024,
                original_bytes: self.original_bytes,
            },
            codec: FormatLimits {
                block_bytes: 256 * 1024,
                total_bytes: 1024 * 1024,
                decoded_bytes: 2 * 1024 * 1024,
                blocks: 8192,
                items: 8192,
                json_values: 8192,
                json_depth: 32,
            },
        }
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FileFactsUsage {
    pub managed_bytes: usize,
    pub managed_peak_bytes: usize,
    pub scratch_reserved_bytes: u64,
    pub cleanup_pending: Option<u128>,
}
struct Data {
    summary: FileFactsSummary,
    root: PersistentNodeId,
    preflight: FileReadPreflight,
    commits: [Option<FileCommitFacts>; MAX_FILE_FACTS_COMMITS],
    intent: FileIntentFacts,
    commit_navigation: [[u8; 32]; MAX_FILE_FACTS_COMMITS],
    historical: historical_content::HistoricalContentData,
    historical_originals: historical_originals::HistoricalOriginalsData,
    metadata: historical_metadata::MetadataData,
}
// Context workspace exits before the Data producer is constructed.
const _: () = assert!(
    2 * std::mem::size_of::<Data>()
        >= intent_context::WORK_BYTES
            + proof::CONTENT_WORK_BYTES
            + 2 * std::mem::size_of::<historical_content::HistoricalContentData>()
            + 2 * std::mem::size_of::<historical_originals::HistoricalOriginalsData>()
            + 2 * std::mem::size_of::<historical_metadata::MetadataData>()
            + std::mem::size_of::<(FileIntentFacts, [[u8; 32]; MAX_FILE_FACTS_COMMITS])>()
            + std::mem::size_of::<FileFactsSummary>()
            + std::mem::size_of::<FileReadPreflight>()
            + 2 * std::mem::size_of::<Restoration>()
);
const INSPECT_WORK_BYTES: usize =
    historical_originals::WORK_BYTES + historical_metadata::WORK_BYTES;
/// No conversion to a writer, restore candidate or application save authority.
pub struct InspectedFileFacts {
    data: Box<Data>,
    _source: snapshot::Source,
    _lease: ByteLease,
}
struct Restoration {
    candidate: RestoreCandidate,
    features: Vec<Value>,
}
impl FileFactsPort for InspectedFileFacts {
    fn summary(&self) -> FileFactsSummary {
        self.data.summary
    }
    fn commits(&self, request: FileFactsPageRequest) -> Result<FileFactsPage, FileFactsError> {
        if request.version != FILE_FACTS_PORT_VERSION {
            return Err(FileFactsError::UnsupportedVersion);
        }
        let summary = self.summary();
        if request.observation != summary.observation
            || request.key != summary.request.key
            || request.binding != summary.request.binding
        {
            return Err(FileFactsError::StalePage);
        }
        if request.limit == 0
            || request.limit > MAX_FILE_FACTS_PAGE
            || request.offset > summary.commits
        {
            return Err(FileFactsError::InvalidPage);
        }
        let end = request
            .offset
            .saturating_add(request.limit)
            .min(summary.commits);
        let mut page = FileFactsPage {
            observation: summary.observation,
            offset: request.offset,
            entries: [None; MAX_FILE_FACTS_PAGE],
            len: end - request.offset,
            next: (end < summary.commits).then_some(end),
        };
        page.entries[..page.len].copy_from_slice(&self.data.commits[request.offset..end]);
        Ok(page)
    }
}
impl InspectedFileFacts {
    pub fn preflight(&self) -> FileReadPreflight {
        self.data.preflight
    }
    pub fn intent_facts(&self) -> FileIntentFacts {
        self.data.intent
    }
}
pub struct FileFactsReader {
    _root_lock: snapshot::RootLock,
    // Residual handles/paths exit before the root's managed reservation.
    pending: Option<snapshot::Scratch>,
    root: snapshot::BoundPath,
    control: Arc<ByteBudget>,
    limits: FileFactsLimits,
    instance: u128,
    sequence: u64,
    #[cfg(test)]
    fail_cleanup: bool,
    #[cfg(test)]
    probe: Probe,
}
fn storage(error: StorageError) -> FileFactsError {
    match error {
        StorageError::Busy => FileFactsError::Busy,
        StorageError::Cancelled
        | StorageError::Capture(CaptureError::Cancelled)
        | StorageError::Restore(RestoreError::Cancelled) => FileFactsError::Cancelled,
        StorageError::Limit(_) | StorageError::Restore(RestoreError::Limit(_)) => {
            FileFactsError::Limit(FileFactsLimit::Validation)
        }
        StorageError::Unsupported(_) | StorageError::JournalPresent => FileFactsError::Unsupported,
        StorageError::ExternalChange | StorageError::Conflict => FileFactsError::TargetConflict,
        StorageError::Io => FileFactsError::Io,
        _ => FileFactsError::ValidationFailed,
    }
}
fn validate_request(request: FileFactsRequest) -> Result<(), FileFactsError> {
    if request.version != FILE_FACTS_PORT_VERSION
        || request.binding.version != SAVE_COORDINATION_VERSION
    {
        return Err(FileFactsError::UnsupportedVersion);
    }
    if request.key.sequence == 0
        || request.binding.key.generation == 0
        || request.binding.phase != SaveTargetPhase::ExistingVerified
        || request.binding.history.is_none()
        || request.binding.expected.commit.is_none()
        || !matches!(request.binding.expected.generation, Some(1..))
    {
        return Err(FileFactsError::InvalidRequest);
    }
    Ok(())
}
impl FileFactsReader {
    pub fn open(scratch_root: &Path, limits: FileFactsLimits) -> Result<Self, FileFactsError> {
        if !limits.valid() {
            return Err(FileFactsError::InvalidLimits);
        }
        let control = ByteBudget::new(limits.control_bytes);
        let root = snapshot::BoundPath::bind(scratch_root, true, false, &control)?;
        snapshot::validate_path(
            &root
                .path
                .join("file-facts-00000000000000000000000000000000.scratch"),
        )?;
        snapshot::root_empty(&root.path, false)?;
        let root_lock = snapshot::RootLock::create(&root.path)?;
        let instance = snapshot::identity()?;
        Ok(Self {
            _root_lock: root_lock,
            root,
            control,
            limits,
            instance,
            sequence: 0,
            pending: None,
            #[cfg(test)]
            fail_cleanup: false,
            #[cfg(test)]
            probe: Probe::default(),
        })
    }
    pub fn usage(&self) -> FileFactsUsage {
        FileFactsUsage {
            managed_bytes: self.control.used(),
            managed_peak_bytes: self.control.peak(),
            scratch_reserved_bytes: self.pending.as_ref().map_or(0, |s| s.bytes),
            cleanup_pending: self.pending_cleanup(),
        }
    }
    pub fn pending_cleanup(&self) -> Option<u128> {
        self.pending.as_ref().map(|s| s.token)
    }
    pub fn retry_cleanup(&mut self) -> Result<(), FileFactsError> {
        if let Some(scratch) = &self.pending {
            #[cfg(test)]
            if self.fail_cleanup {
                return Err(FileFactsError::Cleanup {
                    token: scratch.token,
                });
            }
            scratch.cleanup()?;
        }
        self.pending = None;
        Ok(())
    }
    pub fn inspect(
        &mut self,
        source: &Path,
        request: FileFactsRequest,
        cancel: CaptureCancellation,
    ) -> Result<InspectedFileFacts, FileFactsError> {
        let (data, restoration, source, lease) =
            self.read_observation(source, request, cancel, None)?;
        debug_assert!(restoration.is_none());
        Ok(InspectedFileFacts {
            data: Box::new(data),
            _source: source,
            _lease: lease,
        })
    }
    fn read_observation(
        &mut self,
        source: &Path,
        request: FileFactsRequest,
        cancel: CaptureCancellation,
        restore_store: Option<&BlobStore>,
    ) -> Result<(Data, Option<Restoration>, snapshot::Source, ByteLease), FileFactsError> {
        validate_request(request)?;
        if cancel.is_cancelled() {
            return Err(FileFactsError::Cancelled);
        }
        if let Some(token) = self.pending_cleanup() {
            return Err(FileFactsError::Cleanup { token });
        }
        self.root.verify()?;
        snapshot::root_empty(&self.root.path, true)?;
        let sequence = self
            .sequence
            .checked_add(1)
            .ok_or(FileFactsError::Limit(FileFactsLimit::Sequence))?;
        self.sequence = sequence;
        let observation = FileObservationId {
            instance: self.instance,
            sequence,
        };
        let mut source =
            snapshot::Source::bind(source, self.limits.file_bytes, &self.control, &cancel)?;
        if source.bytes > self.limits.scratch_bytes {
            return Err(FileFactsError::Limit(FileFactsLimit::Scratch));
        }
        // Account producer and retained boxed data simultaneously. Old validator
        // heap/JSON/model allocations explicitly remain outside this new wrapper proof.
        let lease = self
            .control
            .reserve(2 * std::mem::size_of::<Data>())
            .ok_or(FileFactsError::Limit(FileFactsLimit::Control))?;
        // Covers only the new fixed traversal workspace, before validate's
        // frame/roots are constructed; the old full validator stays unproven.
        let work_lease = self
            .control
            .reserve(INSPECT_WORK_BYTES)
            .ok_or(FileFactsError::Limit(FileFactsLimit::Control))?;
        let (scratch, mut file) = snapshot::Scratch::create(&self.root.path, source.bytes)?;
        self.pending = Some(scratch);
        let result = (|| {
            self.pending
                .as_mut()
                .expect("owned scratch")
                .bind_identity()?;
            source.copy(&mut file, &cancel)?;
            let path = &self.pending.as_ref().expect("owned scratch").path;
            #[cfg(test)]
            self.probe.step(Stage::CopyMain, &cancel).map_err(storage)?;
            let mut conn =
                Connection::open_with_flags(path, rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY)
                    .map_err(StorageError::from)
                    .map_err(storage)?;
            let data = self.validate(&mut conn, request, observation, &cancel, restore_store)?;
            conn.close().map_err(|_| FileFactsError::Io)?;
            snapshot::healthy(path)?;
            if snapshot::hash(&mut file, source.bytes, &cancel)? != source.digest {
                return Err(FileFactsError::TargetConflict);
            }
            source.verify(&cancel)?;
            Ok(data)
        })();
        drop(file);
        // Failed validation does not make scratch bytes or cleanup ownership vanish.
        self.retry_cleanup()?;
        let (data, restoration) = result?;
        // validate and every borrowed workspace have actually exited.
        drop(work_lease);
        Ok((data, restoration, source, lease))
    }
    fn validate(
        &self,
        conn: &mut Connection,
        request: FileFactsRequest,
        observation: FileObservationId,
        cancel: &CaptureCancellation,
        restore_store: Option<&BlobStore>,
    ) -> Result<(Data, Option<Restoration>), FileFactsError> {
        let limits = self.limits.file();
        configure(conn, limits, cancel).map_err(storage)?;
        let preflight = preflight::scan(conn, self.limits, observation, cancel)?;
        for (table, limit) in [
            ("commits", self.limits.commits),
            ("nodes", self.limits.nodes),
            ("blobs", self.limits.originals),
            ("object_ids", self.limits.admissions),
        ] {
            let count = conn
                .query_row(&format!("SELECT count(*) FROM {table}"), [], |r| {
                    r.get::<_, i64>(0)
                })
                .map_err(StorageError::from)
                .map_err(storage)?;
            if count < 0 || count as u64 > limit as u64 {
                return Err(FileFactsError::Limit(if table == "commits" {
                    FileFactsLimit::Commits
                } else {
                    FileFactsLimit::Validation
                }));
            }
        }
        let total = self.limits.original_bytes as usize;
        let private_store;
        let store = if let Some(store) = restore_store {
            store
        } else {
            private_store = BlobStore::with_limits(total, total.min(64 * 1024))
                .map_err(|_| FileFactsError::InvalidLimits)?;
            &private_store
        };
        #[cfg(test)]
        let probe = self.probe;
        #[cfg(not(test))]
        let probe = Probe::default();
        let mut metadata = historical_metadata::MetadataData::empty();
        let (cache, candidate) = {
            let mut collector = historical_metadata::Collector::new(&mut metadata);
            read::full_observed(
                conn,
                store,
                limits,
                request.selection,
                cancel,
                probe,
                Some(&mut collector),
            )
            .map_err(storage)?
        };
        if cache.header.project != request.binding.project
            || request.binding.history != Some((&cache.header.identity).into())
            || cache.metadata.last_commit.as_ref().map(|r| r.commit)
                != request.binding.expected.commit
            || cache.metadata.last_commit.as_ref().map(|r| r.generation)
                != request.binding.expected.generation
        {
            return Err(FileFactsError::TargetConflict);
        }
        if request.selection == RestoreSelection::Manual && cache.metadata.manual.is_none() {
            return Err(FileFactsError::NeedsExplicitRecovery);
        }
        let summary = FileFactsSummary {
            request,
            observation,
            assurance: FileFactsAssurance::ExperimentalValidation,
            files: cache.metadata.as_ref().into(),
            selected_head: candidate.header().head,
            nodes: cache.nodes.len(),
            admissions: cache.admissions.len(),
            originals: cache.originals.len(),
            original_bytes: cache.originals.iter().map(|b| b.byte_len()).sum(),
            commits: cache.checkpoints.len(),
            proof: proof::build(
                &cache,
                candidate.header(),
                request.selection,
                limits,
                cancel,
            )
            .map_err(storage)?,
        };
        let (intent, commit_navigation) =
            intent_context::file_context(&cache, candidate.header(), observation)
                .map_err(storage)?;
        if !metadata.matches_latest(&cache, summary.proof.metadata) {
            return Err(FileFactsError::ValidationFailed);
        }
        let historical = historical_content::build(&cache, cancel, probe).map_err(storage)?;
        let mut result = Data {
            summary,
            root: cache.header.root,
            preflight,
            commits: [None; MAX_FILE_FACTS_COMMITS],
            intent,
            commit_navigation,
            historical,
            historical_originals: historical_originals::HistoricalOriginalsData::empty(),
            metadata,
        };
        let mut original_roots = [StateHash::from_bytes([0; 32]); MAX_FILE_FACTS_COMMITS];
        for (index, p) in cache.checkpoints.iter().enumerate() {
            let parsed = read::checkpoint(&cache, p.navigation, limits).map_err(storage)?;
            original_roots[index] = parsed.original_root();
            result.commits[index] = Some(FileCommitFacts {
                fact: (&p.receipt).into(),
                previous: p.previous,
                legacy_request: parsed.request,
                navigation: p.navigation.bytes(),
                prefix: p.prefix,
                nodes: p.node_count,
                next_alias: p.next_alias,
                next_branch: p.next_branch,
            });
        }
        historical_originals::validate(
            &cache,
            &original_roots[..cache.checkpoints.len()],
            cancel,
            probe,
            &mut result.historical_originals,
        )
        .map_err(storage)?;
        let last = result.historical_originals.points[cache.checkpoints.len() - 1];
        if last.digest != summary.proof.originals
            || last.mask.count_ones() as usize != summary.originals
            || last.bytes != summary.original_bytes
        {
            return Err(FileFactsError::ValidationFailed);
        }
        let restoration = if restore_store.is_some() {
            Some(Restoration {
                candidate,
                features: cache.features,
            })
        } else {
            drop(candidate);
            drop(cache);
            debug_assert_eq!(store.usage().map_err(|_| FileFactsError::Io)?.bytes, 0);
            None
        };
        Ok((result, restoration))
    }
}
impl Drop for FileFactsReader {
    fn drop(&mut self) {
        let _ = self.retry_cleanup();
    }
}
#[cfg(test)]
pub(super) mod test_support {
    use super::*;
    #[cfg(windows)]
    pub fn single_node_content(
        header: [u128; 5],
        node: &CapturedNodeInfo,
        canonical: StateHash,
    ) -> [u8; 32] {
        let mut h = proof::Encoder::new(1);
        for id in header {
            h.id(id);
        }
        h.number(1);
        proof::content_node(&mut h, node, canonical).unwrap();
        h.number(0);
        h.finish().unwrap()
    }
    #[cfg(windows)]
    pub fn release_work_observed(
        reader: &FileFactsReader,
        work: SavePreparationWork,
        observe: impl FnOnce(),
    ) {
        save::release_work_observed(reader, work, observe);
    }
    #[cfg(windows)]
    pub fn save_controls() -> (usize, usize) {
        (save::extra_bytes(), intent_context::WORK_BYTES)
    }
    #[cfg(windows)]
    pub fn hold_cursor_margin(reader: &FileFactsReader, short_by: usize) -> ByteLease {
        let required = save::CURSOR_WORK_BYTES + capture::retained_bytes() + capture::work_bytes();
        reader
            .control
            .reserve(reader.limits.control_bytes - reader.control.used() - required + short_by)
            .unwrap()
    }
    #[cfg(windows)]
    pub fn intent_archive(
        m: &ArchiveMetadata,
        delta: Option<&ArchiveReceipt>,
    ) -> Result<[u8; 32], StorageError> {
        intent_context::archive(m, delta, &mut [0; 4096])
    }
    #[cfg(windows)]
    pub fn intent_navigation(
        head: PersistentNodeId,
        a: u64,
        b: u64,
        redo: &[(PersistentNodeId, PersistentNodeId)],
    ) -> Result<[u8; 32], StorageError> {
        intent_context::navigation(head, a, b, redo, &mut [0; 4096])
    }
    #[cfg(windows)]
    pub fn intent_prediction(m: &ArchiveMetadata, ticket: SaveTicket) -> Option<ArchiveReceipt> {
        save::predict(m, ticket)
    }
    #[cfg(windows)]
    pub fn commit_match(
        q: FileSaveIntentSummary,
        c: FileCommitFacts,
        nav: [u8; 32],
        archive: [u8; 32],
        latest: Option<SaveCommitFact>,
    ) -> FileSaveCommitEvidence {
        save::match_commit(q, c, nav, archive, latest)
    }
    #[cfg(windows)]
    pub fn capture_controls() -> (usize, usize) {
        (capture::retained_bytes(), capture::work_bytes())
    }
    #[cfg(windows)]
    pub fn control_observation(reader: &FileFactsReader) -> crate::byte_budget::ByteObservation {
        reader.control.observation()
    }
    pub fn preflight(
        conn: &Connection,
        limits: FileFactsLimits,
    ) -> Result<FileReadPreflight, FileFactsError> {
        super::preflight::scan(
            conn,
            limits,
            FileObservationId {
                instance: 1,
                sequence: 1,
            },
            &CaptureCancellation::default(),
        )
    }
    pub fn preflight_cancel(
        conn: &Connection,
        cancel: &CaptureCancellation,
    ) -> Result<FileReadPreflight, FileFactsError> {
        super::preflight::scan(
            conn,
            FileFactsLimits::default(),
            FileObservationId {
                instance: 1,
                sequence: 1,
            },
            cancel,
        )
    }
    #[cfg(windows)]
    pub fn controls() -> usize {
        aggregate_controls(1)
    }
    #[cfg(windows)]
    pub fn inspect_controls(count: usize) -> usize {
        aggregate_controls(count) + INSPECT_WORK_BYTES
    }
    #[cfg(windows)]
    pub fn inspection_work_bytes() -> usize {
        INSPECT_WORK_BYTES
    }
    #[cfg(windows)]
    pub fn metadata_layout() -> (usize, usize) {
        (
            std::mem::size_of::<historical_metadata::Point>(),
            historical_metadata::WORK_BYTES,
        )
    }
    #[cfg(windows)]
    pub fn profile_controls(text: usize) -> usize {
        profile::source_bytes(text)
    }
    #[cfg(windows)]
    pub fn profile_work_scope(
        reader: &FileFactsReader,
        work: &SavePreparationWork,
        ticket: SaveTicket,
    ) -> Option<FileCapturedMetadataFacts> {
        profile::captured(
            work.snapshot(),
            ticket,
            reader.instance,
            &mut [0; 4096],
            &CaptureCancellation::default(),
            Probe::default(),
        )
        .unwrap()
    }
    #[cfg(windows)]
    pub fn aggregate_controls(count: usize) -> usize {
        snapshot::PATH_CHARGE + count * (snapshot::PATH_CHARGE + 2 * std::mem::size_of::<Data>())
    }
    #[cfg(windows)]
    pub fn cleanup(reader: &mut FileFactsReader, fail: bool) {
        reader.fail_cleanup = fail;
    }
    #[cfg(windows)]
    pub fn probe(reader: &mut FileFactsReader, probe: Probe) {
        reader.probe = probe;
    }
    #[cfg(windows)]
    pub fn sequence(reader: &mut FileFactsReader, value: u64) {
        reader.sequence = value;
    }
    #[cfg(windows)]
    pub fn path(path: &Path) -> Result<(), FileFactsError> {
        snapshot::validate_path(path)
    }
    pub fn framing(domain: u16, fields: &[Vec<u8>]) -> [u8; 32] {
        let mut encoder = proof::Encoder::new(domain);
        for field in fields {
            encoder.bytes(field);
        }
        encoder.finish().unwrap()
    }
    pub fn metadata(metadata: &ArchiveMetadata) -> [u8; 32] {
        proof::metadata(metadata, &write::features()).unwrap()
    }
}
