use super::*;

/// Counts from one ledger, never score, ACC or an implied from-start result.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct FrameCounts {
    pub hits: usize,
    pub misses: usize,
    pub inspection_skipped: usize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FrameKey {
    source: RunSource,
    at: MapTime,
    publication: u64,
}
impl FrameKey {
    pub const fn source(self) -> RunSource {
        self.source
    }
    pub const fn at(self) -> MapTime {
        self.at
    }
    pub const fn publication(self) -> u64 {
        self.publication
    }
    /// The range is checked when read, before any output is changed.
    pub const fn cursor(self, index: usize) -> FrameCursor {
        FrameCursor { frame: self, index }
    }
}

/// Fixed-size metadata only. Object data requires a borrow of its run.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FrameHeader {
    pub source: RunSource,
    pub at: MapTime,
    pub evidence: Evidence,
    pub profile_version: u32,
    pub statistics_origin: StatisticsOrigin,
    pub counts: FrameCounts,
    pub(super) publication: u64,
    pub(super) len: usize,
}
impl FrameHeader {
    pub const fn key(self) -> FrameKey {
        FrameKey {
            source: self.source,
            at: self.at,
            publication: self.publication,
        }
    }
    pub const fn len(self) -> usize {
        self.len
    }
    pub const fn is_empty(self) -> bool {
        self.len == 0
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FrameCursor {
    frame: FrameKey,
    index: usize,
}
impl FrameCursor {
    pub const fn frame(self) -> FrameKey {
        self.frame
    }
    pub const fn index(self) -> usize {
        self.index
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FramePage {
    pub frame: FrameKey,
    pub written: usize,
    pub next: Option<FrameCursor>,
    pub end: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FrameReadError {
    StaleSource,
    StaleFrame,
    InvalidPageSize,
    InvalidPosition,
    OutputCapacity { required: usize, available: usize },
}

/// A complete section borrowed from its owner, including during Pending/Failed.
/// It cannot outlive or be used across a mutable operation on that owner:
/// ```compile_fail
/// use skin_editor_gameplay::{SyntheticRun, MapTime};
/// fn mutate(run: &mut SyntheticRun) {
///     let source = run.source();
///     let frame = run.ready_frame(source).unwrap().unwrap();
///     run.seal_through(source, MapTime::from_micros(1)).unwrap();
///     let _ = frame.object(0);
/// }
/// ```
pub struct ReadyFrame<'a> {
    run: &'a SyntheticRun,
    header: FrameHeader,
}
impl core::fmt::Debug for ReadyFrame<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("ReadyFrame")
            .field("header", &self.header)
            .finish()
    }
}
impl ReadyFrame<'_> {
    pub const fn header(&self) -> FrameHeader {
        self.header
    }
    pub const fn len(&self) -> usize {
        self.header.len
    }
    pub const fn is_empty(&self) -> bool {
        self.header.len == 0
    }
    pub fn object(&self, index: usize) -> Option<CircleFrame> {
        self.run.frame_object(index, self.header.at)
    }
    pub fn read_page(
        &self,
        cursor: FrameCursor,
        limit: usize,
        output: &mut [Option<CircleFrame>],
    ) -> Result<FramePage, FrameReadError> {
        // This borrow pins the complete section, so it cannot disappear.
        self.run
            .read_frame_page(cursor, limit, output)?
            .ok_or(FrameReadError::StaleFrame)
    }
}

impl SyntheticRun {
    pub fn ready_frame(&self, source: RunSource) -> Result<Option<ReadyFrame<'_>>, FrameReadError> {
        self.check_read_source(source)?;
        Ok(self.ready.map(|header| ReadyFrame { run: self, header }))
    }
    /// Current atomically committed prefix, not necessarily a complete frame.
    pub fn committed_judgement(
        &self,
        source: RunSource,
        index: usize,
    ) -> Result<Option<JudgementRecord>, FrameReadError> {
        self.check_read_source(source)?;
        Ok(self.judgements.get(index).copied().flatten())
    }
    /// No allocation or run mutation. Errors leave the caller's output intact.
    /// A missing complete section is distinct from stale source/frame errors.
    pub fn read_frame_page(
        &self,
        cursor: FrameCursor,
        limit: usize,
        output: &mut [Option<CircleFrame>],
    ) -> Result<Option<FramePage>, FrameReadError> {
        self.check_read_source(cursor.frame.source)?;
        let Some(header) = self.ready else {
            return Ok(None);
        };
        if cursor.frame != header.key() {
            return Err(FrameReadError::StaleFrame);
        }
        if limit == 0 || limit > MAX_FRAME_PAGE {
            return Err(FrameReadError::InvalidPageSize);
        }
        let remaining = header
            .len
            .checked_sub(cursor.index)
            .ok_or(FrameReadError::InvalidPosition)?;
        let count = limit.min(remaining);
        let next = cursor
            .index
            .checked_add(count)
            .ok_or(FrameReadError::InvalidPosition)?;
        if output.len() < count {
            return Err(FrameReadError::OutputCapacity {
                required: count,
                available: output.len(),
            });
        }
        for (offset, slot) in output.iter_mut().take(count).enumerate() {
            *slot = self.frame_object(cursor.index + offset, header.at);
        }
        Ok(Some(FramePage {
            frame: header.key(),
            written: count,
            end: next == header.len,
            next: (next < header.len).then_some(header.key().cursor(next)),
        }))
    }
    fn check_read_source(&self, source: RunSource) -> Result<(), FrameReadError> {
        if source == self.source {
            Ok(())
        } else {
            Err(FrameReadError::StaleSource)
        }
    }
    fn frame_object(&self, index: usize, at: MapTime) -> Option<CircleFrame> {
        let circle = self.map.circle(index)?;
        let judgement = self.judgements[index].filter(|record| record.at <= at);
        let skipped = self.inspection_skipped[index];
        Some(CircleFrame {
            object: circle.id,
            phase: if at < circle.visible {
                Phase::Future
            } else if at >= circle.retire || skipped {
                Phase::Retired
            } else if judgement.is_some() {
                Phase::ResolvedVisual
            } else {
                Phase::VisibleUnjudged
            },
            judgement,
            unavailable: skipped.then_some(UnavailableReason::InspectionSkipped),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Circle, InputEpoch, MapRevision, PhysicalSlot, Point};

    fn fixture() -> SyntheticRun {
        let source = RunSource {
            run: RunId::new(110_001).unwrap(),
            transport: TransportEpoch::new(1).unwrap(),
            input: InputEpoch::new(1).unwrap(),
            revision: MapRevision::new(1),
        };
        let map = SyntheticMap::try_new(
            source.revision,
            &[Circle::new(
                ObjectId::new(5),
                MapTime::from_micros(0),
                Point::new(0, 0).unwrap(),
            )
            .unwrap()],
        )
        .unwrap();
        SyntheticRun::new(source, map, MapTime::from_micros(-200_001), Limits::tiny()).unwrap()
    }
    fn advance(run: &mut SyntheticRun, at: i64) {
        let at = MapTime::from_micros(at);
        run.seal_through(run.source(), at).unwrap();
        let mut r = run.advance(run.source(), at, WorkBudget::one()).unwrap();
        for _ in 0..32 {
            match r {
                AdvanceReport::Complete { .. } => return,
                AdvanceReport::Pending { cursor, .. } => {
                    r = run.continue_advance(cursor, WorkBudget::one()).unwrap()
                }
            }
        }
        panic!("unit fixture bound");
    }

    #[test]
    fn missing_sections_and_wrong_publication_are_non_mutating_read_results() {
        let mut run = fixture();
        let cursor = FrameKey {
            source: run.source(),
            at: MapTime::from_micros(-1),
            publication: 1,
        }
        .cursor(0);
        let before = run.observation();
        assert_eq!(run.read_frame_page(cursor, 0, &mut []), Ok(None));
        assert_eq!(run.observation(), before);
        advance(&mut run, -1);
        let key = run.ready.unwrap().key();
        let mut wrong = key;
        wrong.publication = wrong.publication.checked_add(1).unwrap();
        assert_eq!(
            run.read_frame_page(wrong.cursor(usize::MAX), 0, &mut []),
            Err(FrameReadError::StaleFrame)
        );
        wrong = key;
        wrong.at = MapTime::from_micros(0);
        assert_eq!(
            run.read_frame_page(wrong.cursor(0), 1, &mut [None]),
            Err(FrameReadError::StaleFrame)
        );
    }

    #[test]
    fn publication_exhaustion_retains_the_committed_prefix_and_old_complete_section() {
        let mut run = fixture();
        advance(&mut run, -1);
        let old = run.ready.unwrap();
        run.next_publication = None;
        run.submit_inputs(
            run.source(),
            &[InputEvent {
                sequence: InputSequence::new(1).unwrap(),
                at: MapTime::from_micros(0),
                slot: PhysicalSlot::PrimaryOne,
                edge: InputEdge::Press(Point::new(0, 0).unwrap()),
            }],
        )
        .unwrap();
        run.seal_through(run.source(), MapTime::from_micros(0))
            .unwrap();
        let mut r = run.advance(run.source(), MapTime::from_micros(0), WorkBudget::one());
        for _ in 0..32 {
            match r {
                Ok(AdvanceReport::Pending { cursor, .. }) => {
                    r = run.continue_advance(cursor, WorkBudget::one())
                }
                _ => break,
            }
        }
        assert_eq!(r, Err(RunError::CounterExhausted));
        let state = run.observation();
        assert!(matches!(state.status, RunStatus::Failed(_)));
        assert_eq!(state.ready_frame, Some(old));
        assert_eq!(state.committed_counts.hits, 1);
        assert_eq!(state.pending_events, 2);
        assert_eq!(state.armed_slots, [false; 4]);
        assert_eq!(
            run.ready_frame(run.source())
                .unwrap()
                .unwrap()
                .object(0)
                .unwrap()
                .judgement,
            None
        );
        assert_eq!(run.resume(run.source()), Err(RunError::Faulted));
        assert_eq!(run.next_publication, None);
        assert_eq!(run.observation(), state);
    }

    #[test]
    fn final_publication_is_usable_without_wrap_or_duplicate_allocation_on_same_time() {
        let mut run = fixture();
        run.next_publication = Some(u64::MAX);
        advance(&mut run, -200_000);
        assert_eq!(run.ready.unwrap().key().publication(), u64::MAX);
        assert_eq!(run.next_publication, None);
        advance(&mut run, -200_000);
        assert_eq!(run.ready.unwrap().key().publication(), u64::MAX);
        run.request_freeze(run.source(), MapTime::from_micros(-200_000))
            .unwrap();
        assert!(matches!(
            run.settle_freeze(run.source(), WorkBudget::one()).unwrap(),
            FreezeReport::Frozen { .. }
        ));
    }
}
