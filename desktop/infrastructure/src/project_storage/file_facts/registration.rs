//! Single-slot host ownership of a validated opening for read-only use.
use super::*;
mod session;
pub use session::RegisteredSessionOpening;

pub const FILE_TARGET_REGISTRY_VERSION: u16 = 1;

/// Conditional address of one registry state. Copying it does not retain the target.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FileTargetStamp {
    owner: u128,
    generation: u64,
    occupied: bool,
}

/// A registry is tied to one reader and its control pool. It may outlive that
/// reader; the original file and ancestor handles then remain held until release.
pub struct FileTargetRegistry {
    reader: u128,
    stamp: FileTargetStamp,
    active: Option<InspectedFileFacts>,
    // Drop the observation, including its original path/data reservations, first.
    _slot: ByteLease,
}

// One exclusive registry operation at a time. The path guards, boxed observation
// and original provider each keep their own reader reservations, including during
// replacement. Account for fixed slot moves and overlapping query/restore values.
pub(super) const SLOT_BYTES: usize = 3 * size_of::<FileTargetRegistry>()
    + 2 * (size_of::<FileTargetStamp>()
        + size_of::<SaveTicket>()
        + size_of::<FileFactsRequest>()
        + size_of::<FileProfileRestoreRequest>()
        + size_of::<FileProfileOrigin>()
        + size_of::<FileFactsSummary>());

impl FileTargetRegistry {
    #[cfg(all(test, windows))]
    pub(in crate::project_storage) fn set_generation_for_test(&mut self, value: u64) {
        self.stamp.generation = value;
    }
    pub fn new(reader: &FileFactsReader, version: u16) -> Result<Self, FileFactsError> {
        if version != FILE_TARGET_REGISTRY_VERSION {
            return Err(FileFactsError::UnsupportedVersion);
        }
        let slot = reader
            .control
            .reserve(SLOT_BYTES)
            .ok_or(FileFactsError::Limit(FileFactsLimit::Control))?;
        Ok(Self {
            reader: reader.instance,
            stamp: FileTargetStamp {
                owner: snapshot::identity()?,
                generation: 0,
                occupied: false,
            },
            active: None,
            _slot: slot,
        })
    }

    pub fn stamp(&self) -> FileTargetStamp {
        self.stamp
    }

    fn check_stamp(&self, stamp: FileTargetStamp) -> Result<(), FileFactsError> {
        if stamp == self.stamp {
            Ok(())
        } else {
            Err(FileFactsError::StalePage)
        }
    }

    /// Prepare the new opening while keeping the current slot wholly intact.
    /// Reopening the same physical file while its exclusive handle lives is Busy.
    pub fn bind(
        &mut self,
        reader: &mut FileFactsReader,
        expected: FileTargetStamp,
        source: &Path,
        request: FileProfileRestoreRequest,
        originals: &BlobStore,
        cancel: CaptureCancellation,
    ) -> Result<RestoreCandidate, FileFactsError> {
        self.check_stamp(expected)?;
        if self.reader != reader.instance {
            return Err(FileFactsError::StalePage);
        }
        let generation = self
            .stamp
            .generation
            .checked_add(1)
            .ok_or(FileFactsError::Limit(FileFactsLimit::Sequence))?;
        let (candidate, view) =
            reader.read_profile(source, request, originals, cancel.clone(), true)?;
        let view = view.ok_or(FileFactsError::ValidationFailed)?;
        check(&cancel).map_err(storage)?;
        self.active = Some(view);
        self.stamp.generation = generation;
        self.stamp.occupied = true;
        Ok(candidate)
    }

    /// No reservation or generation increment is needed to release the final generation.
    pub fn release(&mut self, expected: FileTargetStamp) -> Result<(), FileFactsError> {
        self.check_stamp(expected)?;
        if !self.stamp.occupied {
            return Err(FileFactsError::InvalidRequest);
        }
        self.active = None;
        self.stamp.occupied = false;
        Ok(())
    }

    /// Borrow the original validated opening for a genuine frozen capture. The
    /// observation retains its original request sequence, independent of the save ticket.
    /// Rebinding or releasing the registry requires the borrow to have ended.
    /// ```compile_fail,E0499
    /// use skin_editor_infrastructure::project_storage::{FileTargetRegistry, FileTargetStamp};
    /// use skin_editor_application::{CaptureCancellation, SaveCaptureCursor, SaveTicket};
    /// fn release_while_held(r: &mut FileTargetRegistry, s: FileTargetStamp, q: &SaveCaptureCursor<'_>, t: SaveTicket) {
    ///     let view = r.opening_for_capture(s, q, t, &CaptureCancellation::default()).unwrap();
    ///     r.release(s).unwrap();
    ///     let _ = view.preflight();
    /// }
    /// ```
    pub fn opening_for_capture<'a>(
        &'a mut self,
        stamp: FileTargetStamp,
        cursor: &SaveCaptureCursor<'_>,
        ticket: SaveTicket,
        cancel: &CaptureCancellation,
    ) -> Result<&'a InspectedFileFacts, FileFactsError> {
        self.check_stamp(stamp)?;
        let view = self.active.as_ref().ok_or(FileFactsError::InvalidRequest)?;
        if cancel.is_cancelled() || cursor.is_cancelled() {
            return Err(FileFactsError::Cancelled);
        }
        if ticket != cursor.ticket() {
            return Err(FileFactsError::StalePage);
        }
        let snapshot = cursor.snapshot();
        let summary = &view.data.summary;
        save::capture_request(ticket, snapshot, summary.request.selection)?;
        let origin = profile::captured_origin(snapshot, ticket, self.reader)
            .ok_or(FileFactsError::TargetConflict)?;
        view.check_opening_origin(&origin)?;
        Ok(view)
    }
}

impl InspectedFileFacts {
    pub(super) fn check_opening_origin(
        &self,
        origin: &FileProfileOrigin,
    ) -> Result<(), FileFactsError> {
        let summary = &self.data.summary;
        if origin.observation == summary.observation
            && origin.file == summary.request
            && origin.root == self.data.root
            && origin.opened_head == summary.selected_head
            && origin.source_sha256 == self._source.digest
        {
            Ok(())
        } else {
            Err(FileFactsError::TargetConflict)
        }
    }
}
