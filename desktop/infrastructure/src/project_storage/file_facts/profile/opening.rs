use super::*;

/// Both values belong to one short-lived observation of the already-bound source.
/// Keep only the name digest and numeric fields after releasing the original handle.
pub(super) struct Location {
    name_digest: [u8; 32],
    id: Option<(u64, u64)>,
}

impl Location {
    #[cfg(windows)]
    pub(super) fn observe(source: &snapshot::Source) -> Option<Self> {
        use std::os::windows::ffi::OsStrExt;
        let mut hash = Sha256::new();
        hash.update(b"skin-editor:opening-name:utf16le:v1\0");
        // BoundPath validated at most 1024 UTF-16 units. Preserve the supplied spelling.
        for unit in source.bound.path.as_os_str().encode_wide() {
            hash.update(unit.to_le_bytes());
        }
        let id = winapi_util::file::information(&source.file)
            .ok()
            .map(|info| (info.volume_serial_number(), info.file_index()))
            .filter(|&(volume, index)| volume != 0 && index != 0);
        Some(Self {
            name_digest: hash.finalize().into(),
            id,
        })
    }
    #[cfg(not(windows))]
    pub(super) fn observe(_: &snapshot::Source) -> Option<Self> {
        None
    }
}

// Fixed native information, path iterator, hash state and overlapping return values.
#[cfg(windows)]
const _: () = assert!(size_of::<winapi_util::file::Information>() <= 128);
pub(super) const LOCATION_WORK_BYTES: usize =
    512 + 2 * (size_of::<Sha256>() + size_of::<Option<Location>>());
pub(in crate::project_storage::file_facts) const OPENING_QUERY_BYTES: usize = LOCATION_WORK_BYTES
    + 2 * (size_of::<FileOpeningTargetComparison>()
        + size_of::<FileOpeningTargetContext>()
        + size_of::<FileSaveCompareContext>()
        + size_of::<FileFactsRequest>()
        + size_of::<SaveTicket>());

impl FileFactsReader {
    /// Compare two observations without moving the owner's capture traversal.
    /// No path or caller-provided file identity can be substituted for the held observation.
    pub fn compare_opening_target(
        &self,
        cursor: &SaveCaptureCursor<'_>,
        view: &InspectedFileFacts,
        context: FileOpeningTargetContext,
        cancel: &CaptureCancellation,
    ) -> Result<FileOpeningTargetComparison, FileFactsError> {
        check(cancel).map_err(storage)?;
        if context.version != FILE_OPENING_TARGET_VERSION {
            return Err(FileFactsError::UnsupportedVersion);
        }
        let _workspace = self
            .control
            .reserve(OPENING_QUERY_BYTES)
            .ok_or(FileFactsError::Limit(FileFactsLimit::Control))?;
        let observed = &view.data.summary;
        if context.observed_request != observed.request {
            return Err(FileFactsError::StalePage);
        }
        let ticket = cursor.ticket();
        let snapshot = cursor.snapshot();
        let request = save::capture_request(ticket, snapshot, observed.request.selection)?;
        view.check_save_context(
            self.instance,
            request,
            ticket,
            snapshot.manifest().root,
            FileSaveCompareContext {
                version: FILE_SAVE_INTENT_VERSION,
                observation: context.observation,
                ticket: context.ticket,
            },
        )?;
        let facts = scoped_provider(snapshot, ticket, self.instance).and_then(|p| {
            let origin = p.origin();
            if origin.file.selection != observed.request.selection
                || origin.observation.sequence >= observed.observation.sequence
            {
                return None;
            }
            let initial = p.location.as_ref()?;
            let current = Location::observe(&view._source)?;
            Some(FileOpeningTargetFacts {
                origin,
                name_digest_equal: initial.name_digest == current.name_digest,
                file_id_fields_equal: initial.id.zip(current.id).map(|(a, b)| a == b),
                source_digest_equal: origin.source_sha256 == view._source.digest,
                expected_equal: origin.file.binding.expected == observed.request.binding.expected,
                selected_head_equal: origin.opened_head == observed.selected_head,
            })
        });
        check(cancel).map_err(storage)?;
        Ok(FileOpeningTargetComparison {
            context,
            facts,
            assurance: FileFactsAssurance::ExperimentalValidation,
            conclusion: FileCaptureConclusion::InsufficientForSaveReconciliation,
        })
    }
}
