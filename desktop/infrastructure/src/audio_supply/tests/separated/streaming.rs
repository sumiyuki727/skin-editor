use super::*;
use crate::background_stream::{BufferedBackground, tests::Generated};
use std::{
    io::{Read, Seek, SeekFrom},
    sync::mpsc::{Receiver, SyncSender, sync_channel},
};
const GUARD: Duration = Duration::from_secs(5);
fn prime(core: &mut SupplyCore, out: &mut SupplyOutput) {
    let until = std::time::Instant::now() + GUARD;
    loop {
        out.pump(core, 0).unwrap();
        if core.status().background_primed {
            break;
        }
        assert!(std::time::Instant::now() < until);
        std::thread::yield_now();
    }
}
fn streamed() -> Setup {
    let pool = PcmBudget::default();
    let source = BufferedBackground::open(Generated::new(48000 * 600, 1), &pool).unwrap();
    let (core, owner, cb) = SupplyCore::new_separated(source, 48000, 1, pool.clone()).unwrap();
    (
        pool,
        BlobStore::default(),
        core,
        SupplyOutput::Separated(Box::new(owner)),
        cb,
    )
}
fn seek(
    core: &mut SupplyCore,
    out: &mut SupplyOutput,
    store: &BlobStore,
    frame: u64,
) -> Option<SupplyReceipt> {
    let mark = core.output.invalidate_scene().unwrap();
    apply_output(core, out, store, Control::Seek { frame, mark })
}
#[test]
fn streamed_seek_waits_for_owner_and_copy_ack_preserves_audition_and_uses_requested_anchor() {
    let (pool, store, mut core, mut out, mut cb) = streamed();
    ready(&mut core, origin(1), &pool);
    prime(&mut core, &mut out);
    start(&mut core, &mut out, &store);
    trigger(&mut core, &mut out, &store, Channel::Audition);
    trigger(&mut core, &mut out, &store, Channel::Scene);
    cb.render(&mut [0.0; 32]).unwrap();
    out.pump(&mut core, 0).unwrap();
    assert!(seek(&mut core, &mut out, &store, 12345).is_none());
    assert!(core.finish_freeze().unwrap().is_none());
    let mut values = [0.0; 32];
    cb.render(&mut values).unwrap();
    assert_eq!(values, [0.125; 32]);
    assert!(core.finish_freeze().unwrap().is_none());
    out.pump(&mut core, 0).unwrap();
    assert!(matches!(
        core.finish_freeze().unwrap().unwrap().result,
        Ok(SupplyOutcome::Located {
            frame: 12345,
            previous_copy_anchor: 32
        })
    ));
    assert_eq!(core.controller.runtime().scene_frame(), 12345);
    assert_eq!(core.status().background_position, 12345);
    prime(&mut core, &mut out);
    start(&mut core, &mut out, &store);
    cb.render(&mut values).unwrap();
    assert_eq!(values[0], Generated::sample(12345) + 0.125);
    assert_eq!(cb.render(&mut values).unwrap().missing_frames, 0);
    assert!(seek(&mut core, &mut out, &store, 7).is_none());
    cb.render(&mut values).unwrap();
    out.pump(&mut core, 0).unwrap();
    assert!(matches!(
        core.finish_freeze().unwrap().unwrap().result,
        Ok(SupplyOutcome::Located { frame: 7, .. })
    ));
    prime(&mut core, &mut out);
    start(&mut core, &mut out, &store);
    cb.render(&mut values).unwrap();
    assert_eq!(values[0], Generated::sample(7) + 0.125);
    finish(pool, core, out, cb);
}
#[test]
fn busy_second_seek_merges_fence_without_changing_first_location_or_locking_completion() {
    let (pool, store, mut core, mut out, mut cb) = streamed();
    prime(&mut core, &mut out);
    start(&mut core, &mut out, &store);
    cb.render(&mut [0.0; 32]).unwrap();
    assert!(seek(&mut core, &mut out, &store, 999).is_none());
    assert!(matches!(
        seek(&mut core, &mut out, &store, 444).unwrap().result,
        Err(SupplyError::Busy)
    ));
    cb.render(&mut [0.0; 32]).unwrap();
    out.pump(&mut core, 0).unwrap();
    assert!(matches!(
        core.finish_freeze().unwrap().unwrap().result,
        Ok(SupplyOutcome::Located { frame: 999, .. })
    ));
    prime(&mut core, &mut out);
    start(&mut core, &mut out, &store);
    let mut values = [0.0; 32];
    cb.render(&mut values).unwrap();
    assert_eq!(values[0], Generated::sample(999));
    finish(pool, core, out, cb);
}
struct Gate {
    reader: Generated,
    reads: usize,
    entered: Option<SyncSender<()>>,
    release: Receiver<()>,
}
impl Read for Gate {
    fn read(&mut self, b: &mut [u8]) -> std::io::Result<usize> {
        // Metadata reads are <=18 bytes, worker data reads are larger.
        if b.len() > 18 {
            self.reads += 1;
            if self.reads == 1 {
                self.entered.take().unwrap().send(()).unwrap();
                self.release.recv().unwrap();
            }
        }
        self.reader.read(b)
    }
}
impl Seek for Gate {
    fn seek(&mut self, p: SeekFrom) -> std::io::Result<u64> {
        self.reader.seek(p)
    }
}
#[test]
fn blocked_io_is_not_in_callback_and_unprimed_resume_is_explicit_busy_then_retryable() {
    let pool = PcmBudget::default();
    let (tx, entered) = sync_channel(0);
    let (release, rx) = sync_channel(0);
    let stream = BufferedBackground::open(
        Gate {
            reader: Generated::new(48000, 1),
            reads: 0,
            entered: Some(tx),
            release: rx,
        },
        &pool,
    )
    .unwrap();
    entered.recv_timeout(GUARD).unwrap();
    let (mut core, owner, mut cb) =
        SupplyCore::new_separated(stream, 48000, 1, pool.clone()).unwrap();
    let mut out = SupplyOutput::Separated(Box::new(owner));
    let store = BlobStore::default();
    ready(&mut core, origin(1), &pool);
    assert!(matches!(
        scene(&mut core, &mut out, &store, true).unwrap().result,
        Err(SupplyError::Busy)
    ));
    assert!(core.frozen);
    trigger(&mut core, &mut out, &store, Channel::Audition);
    let scope = crate::callback_audit::Scope::enter();
    let mut values = [0.0; 128];
    for _ in 0..4 {
        let report = cb.render(&mut values).unwrap();
        assert_eq!(report.missing_frames, 0);
        assert_eq!(values, [0.125; 128]);
    }
    assert_eq!(scope.finish(), 0);
    release.send(()).unwrap();
    prime(&mut core, &mut out);
    start(&mut core, &mut out, &store);
    cb.render(&mut values).unwrap();
    assert_eq!(values[0], Generated::sample(0) + 0.125);
    finish(pool, core, out, cb);
}
#[test]
fn streamed_hot_selection_and_history_change_do_not_seek_or_rewind_background() {
    let (pool, store, mut core, mut out, mut cb) = streamed();
    prime(&mut core, &mut out);
    start(&mut core, &mut out, &store);
    let mut expected = 0;
    for i in 0..12 {
        let mark = core.output.invalidate_skin(true, true).unwrap();
        let mut source = origin(i + 1);
        source.navigation_revision = i + 1;
        apply_output(
            &mut core,
            &mut out,
            &store,
            Control::Select(Projection {
                origin: source,
                plans: Vec::new(),
                cancelled: Vec::new(),
                mark,
            }),
        )
        .unwrap()
        .result
        .unwrap();
        prime(&mut core, &mut out);
        let mut values = [0.0; 480];
        let report = cb.render(&mut values).unwrap();
        assert_eq!(report.missing_frames, 0);
        assert_eq!(values[0], Generated::sample(expected));
        expected += 480;
    }
    finish(pool, core, out, cb);
}
#[test]
fn invalid_client_location_does_not_invalidate_any_lane_or_consume_control_credit() {
    let pool = PcmBudget::default();
    let source = BufferedBackground::open(Generated::new(48000, 1), &pool).unwrap();
    let (mut client, cb) =
        AudioSupplyClient::spawn_streamed(source, 48000, 1, pool.clone(), BlobStore::default())
            .unwrap();
    let before = client.output.mark();
    let credits = client.credits.load(Ordering::Acquire);
    assert_eq!(
        client.seek_background(48001),
        Err(SupplyError::InvalidPosition)
    );
    assert_eq!(client.output.mark().skin, before.skin);
    assert_eq!(client.output.mark().background, before.background);
    assert_eq!(client.credits.load(Ordering::Acquire), credits);
    client.close();
    client.finish_separated(cb.stop()).unwrap();
    assert_eq!(pool.usage().bytes, 0);
}
