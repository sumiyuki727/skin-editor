//! Consumes a real frozen capture; equality remains partial experimental evidence.
use super::*;
use proof::Encoder;

type Roots = [Option<StateHash>; MAX_FILE_CAPTURE_NODES];
pub(super) trait CaptureStream {
    fn snapshot(&self) -> Result<CaptureSnapshot, CaptureError>;
    fn statistics(&self) -> CaptureStatistics;
    fn poll(&mut self, edges: usize) -> Result<CapturePoll, CaptureError>;
}
impl CaptureStream for HistoryCapture {
    fn snapshot(&self) -> Result<CaptureSnapshot, CaptureError> {
        HistoryCapture::snapshot(self)
    }
    fn statistics(&self) -> CaptureStatistics {
        HistoryCapture::statistics(self)
    }
    fn poll(&mut self, n: usize) -> Result<CapturePoll, CaptureError> {
        HistoryCapture::poll(self, n)
    }
}
impl CaptureStream for SavePreparationWork {
    fn snapshot(&self) -> Result<CaptureSnapshot, CaptureError> {
        Ok(SavePreparationWork::snapshot(self).clone())
    }
    fn statistics(&self) -> CaptureStatistics {
        SavePreparationWork::statistics(self)
    }
    fn poll(&mut self, n: usize) -> Result<CapturePoll, CaptureError> {
        SavePreparationWork::poll(self, n)
    }
}
pub struct FileCaptureCandidate {
    pub(super) summary: FileCaptureSummary,
    pub(super) reader_instance: u128,
    pub(super) prefix: [u8; 32],
    // Real capture pins exit before the new fixed-control reservation.
    pub(super) _snapshot: CaptureSnapshot,
    _lease: ByteLease,
}
impl std::fmt::Debug for FileCaptureCandidate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FileCaptureCandidate")
            .field("summary", &self.summary)
            .finish_non_exhaustive()
    }
}
impl FileCaptureCandidate {
    pub fn summary(&self) -> FileCaptureSummary {
        self.summary
    }
}
pub(super) fn retained_bytes() -> usize {
    2 * std::mem::size_of::<FileCaptureCandidate>()
}
pub(super) fn work_bytes() -> usize {
    std::mem::size_of::<Roots>()
        + 2 * std::mem::size_of::<Encoder>()
        + intent_context::PREFIX_WORK_BYTES
}
fn check_cancel(cancel: &CaptureCancellation) -> Result<(), FileFactsError> {
    if cancel.is_cancelled() {
        Err(FileFactsError::Cancelled)
    } else {
        Ok(())
    }
}
fn normalized_node(
    h: &mut Encoder,
    info: &CapturedNodeInfo,
    canonical: StateHash,
    limits: FileLimits,
) -> Result<[u8; 32], StorageError> {
    let generated;
    let record = if let Some(original) = &info.origin {
        original.as_ref()
    } else {
        let action = operation_bytes(&info.operation)?;
        generated = OriginalNodeRecord {
            record_version: 2,
            state: canonical.bytes(),
            checksum: node_checksum(info.id, info.parent, canonical, &action),
            action,
        };
        &generated
    };
    if record.record_version != 2 {
        return Err(StorageError::Unsupported("capture record"));
    }
    if operation(&record.action, limits.codec)?.as_ref() != info.operation.as_ref()
        || node_checksum(
            info.id,
            info.parent,
            StateHash::from_bytes(record.state),
            &record.action,
        ) != record.checksum
    {
        return Err(StorageError::Invalid("capture origin"));
    }
    h.id(info.id.get());
    h.optional_id(info.parent.map(PersistentNodeId::get));
    for value in [
        info.alias,
        info.coordinate.branch(),
        info.coordinate.ordinal(),
        info.depth,
        u64::from(record.record_version),
    ] {
        h.number(value);
    }
    h.bytes(&record.state);
    h.bytes(&record.action);
    h.bytes(&record.checksum);
    h.bytes(&canonical.bytes());
    Ok(record.checksum)
}
impl FileFactsReader {
    pub fn prepare_capture(
        &self,
        request: FileFactsRequest,
        mut capture: HistoryCapture,
        originals: &BlobStore,
        cancel: CaptureCancellation,
    ) -> Result<FileCaptureCandidate, FileFactsError> {
        self.prepare_stream(request, &mut capture, originals, cancel)
    }
    pub(super) fn prepare_stream(
        &self,
        request: FileFactsRequest,
        capture: &mut impl CaptureStream,
        originals: &BlobStore,
        cancel: CaptureCancellation,
    ) -> Result<FileCaptureCandidate, FileFactsError> {
        validate_request(request)?;
        check_cancel(&cancel)?;
        if let Some(token) = self.pending_cleanup() {
            return Err(FileFactsError::Cleanup { token });
        }
        if capture.statistics().nodes_emitted != 0 {
            return Err(FileFactsError::InvalidRequest);
        }
        let snapshot = capture
            .snapshot()
            .map_err(StorageError::from)
            .map_err(storage)?;
        let m = snapshot.manifest();
        if m.version != HISTORY_CAPTURE_VERSION {
            return Err(FileFactsError::UnsupportedVersion);
        }
        if snapshot.is_complete() || capture.statistics().nodes_emitted != 0 {
            return Err(FileFactsError::InvalidRequest);
        }
        if m.source.session_id != request.key.session
            || m.project_id != request.binding.project
            || request.binding.history != Some((&m.identity).into())
        {
            return Err(FileFactsError::TargetConflict);
        }
        if m.nodes.is_empty()
            || m.nodes.len() > self.limits.nodes
            || m.nodes.len() > MAX_FILE_CAPTURE_NODES
            || m.admissions.len() > self.limits.admissions
            || m.originals.len() > self.limits.originals
            || capture.statistics().retained_metadata_bytes > self.limits.metadata_bytes
        {
            return Err(FileFactsError::Limit(FileFactsLimit::Validation));
        }
        let lease = self
            .control
            .reserve(retained_bytes())
            .ok_or(FileFactsError::Limit(FileFactsLimit::Control))?;
        let work = self
            .control
            .reserve(work_bytes())
            .ok_or(FileFactsError::Limit(FileFactsLimit::Control))?;
        let mut roots: Roots = [None; MAX_FILE_CAPTURE_NODES];
        let mut encoded = 0usize;
        let mut emitted = 0usize;
        loop {
            check_cancel(&cancel)?;
            match capture
                .poll(64)
                .map_err(StorageError::from)
                .map_err(storage)?
            {
                CapturePoll::Pending => (),
                CapturePoll::Complete => break,
                CapturePoll::Node(node) => {
                    let index = m
                        .nodes
                        .iter()
                        .position(|n| n.id == node.info().id)
                        .ok_or(FileFactsError::ValidationFailed)?;
                    if roots[index].is_some() || node.info() != &m.nodes[index] {
                        return Err(FileFactsError::ValidationFailed);
                    }
                    let bundle = encode_project(node.project(), self.limits.file().codec)
                        .map_err(StorageError::from)
                        .map_err(storage)?;
                    for block in &bundle.blocks {
                        encoded = encoded
                            .checked_add(block.bytes.len())
                            .ok_or(FileFactsError::Limit(FileFactsLimit::Validation))?;
                    }
                    if encoded > self.limits.file().decoded_bytes {
                        return Err(FileFactsError::Limit(FileFactsLimit::Validation));
                    }
                    roots[index] = Some(bundle.root);
                    emitted += 1;
                }
            }
        }
        if !snapshot.is_complete() || emitted != m.nodes.len() {
            return Err(FileFactsError::ValidationFailed);
        }
        #[cfg(test)]
        self.probe.step(Stage::States, &cancel).map_err(storage)?;
        check_cancel(&cancel)?;
        if m.nodes.windows(2).any(|n| n[0].alias >= n[1].alias)
            || m.originals
                .windows(2)
                .any(|r| r[0].sha256() >= r[1].sha256())
        {
            return Err(FileFactsError::ValidationFailed);
        }
        let mut content = Encoder::new(1);
        for id in [
            m.project_id.get(),
            m.identity.history.get(),
            m.identity.lineage.get(),
            m.identity.address_epoch.get(),
            m.root.get(),
        ] {
            content.id(id);
        }
        content.number(m.nodes.len() as u64);
        let mut prefix = initial_prefix();
        for (i, info) in m.nodes.iter().enumerate() {
            check_cancel(&cancel)?;
            let checksum = normalized_node(
                &mut content,
                info,
                roots[i].ok_or(FileFactsError::ValidationFailed)?,
                self.limits.file(),
            )
            .map_err(storage)?;
            prefix = super::intent_context::prefix(prefix, info, checksum, &m.admissions);
        }
        content.number(m.admissions.len() as u64);
        for kind in ["asset", "component", "document", "layer", "output", "sound"] {
            let mut previous = 0;
            for admission in &m.admissions {
                let (found, id) = object(admission.object);
                if found == kind {
                    if id <= previous {
                        return Err(FileFactsError::ValidationFailed);
                    }
                    previous = id;
                    content.bytes(kind.as_bytes());
                    content.id(id);
                    content.id(admission.first_node.get());
                }
            }
        }
        let content = content.finish().map_err(storage)?;
        let mut raw = Encoder::new(2);
        raw.number(m.originals.len() as u64);
        let mut total = 0u64;
        for reference in &m.originals {
            check_cancel(&cancel)?;
            total = total
                .checked_add(reference.byte_len())
                .ok_or(FileFactsError::Limit(FileFactsLimit::Validation))?;
            if total > self.limits.original_bytes
                || reference.byte_len() > self.limits.original_bytes.min(65536)
            {
                return Err(FileFactsError::Limit(FileFactsLimit::Validation));
            }
            let bytes = originals
                .get(*reference)
                .ok_or(FileFactsError::ValidationFailed)?;
            if bytes.bytes().len() as u64 != reference.byte_len()
                || sha(bytes.bytes()) != reference.sha256()
            {
                return Err(FileFactsError::ValidationFailed);
            }
            raw.bytes(&reference.sha256());
            raw.number(reference.byte_len());
            #[cfg(test)]
            self.probe
                .step(Stage::Originals, &cancel)
                .map_err(storage)?;
        }
        let summary = FileCaptureSummary {
            version: FILE_CAPTURE_PROJECTION_VERSION,
            request,
            source: m.source,
            root: m.root,
            head: m.head,
            nodes: m.nodes.len(),
            admissions: m.admissions.len(),
            originals: m.originals.len(),
            original_bytes: total,
            encoded_block_bytes: encoded,
            projection: FileCaptureProjection {
                version: FILE_CAPTURE_PROJECTION_VERSION,
                content,
                originals: raw.finish().map_err(storage)?,
            },
        };
        #[cfg(test)]
        self.probe
            .step(Stage::BeforePublish, &cancel)
            .map_err(storage)?;
        check_cancel(&cancel)?;
        drop(work);
        Ok(FileCaptureCandidate {
            summary,
            reader_instance: self.instance,
            prefix,
            _snapshot: snapshot,
            _lease: lease,
        })
    }
}
impl InspectedFileFacts {
    pub fn compare_capture(
        &self,
        candidate: &FileCaptureCandidate,
        context: FileCaptureCompareContext,
    ) -> Result<FileCaptureComparison, FileFactsError> {
        if context.version != FILE_CAPTURE_PROJECTION_VERSION {
            return Err(FileFactsError::UnsupportedVersion);
        }
        let value = candidate.summary;
        let file = self.summary();
        if context.observation != file.observation
            || context.source != value.source
            || candidate.reader_instance != file.observation.instance
            || value.request != file.request
        {
            return Err(FileFactsError::StalePage);
        }
        if value.root != self.data.root {
            return Err(FileFactsError::TargetConflict);
        }
        Ok(FileCaptureComparison {
            version: FILE_CAPTURE_PROJECTION_VERSION,
            candidate: value,
            observation: file.observation,
            content_equal: value.projection.content == file.proof.content,
            originals_equal: value.projection.originals == file.proof.originals,
            selected_head_equal: value.head == file.selected_head,
            conclusion: FileCaptureConclusion::InsufficientForSaveReconciliation,
        })
    }
}
