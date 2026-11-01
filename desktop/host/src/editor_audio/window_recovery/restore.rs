//! Local single-owner escrow. Not a project loader or a command write port.
use super::*;
use eframe::egui_wgpu::wgpu;

pub(super) struct Slot<T> {
    value: Option<T>,
}
impl<T> Slot<T> {
    pub fn new(value: T) -> Self {
        Self { value: Some(value) }
    }
    pub fn vacant() -> Self {
        Self { value: None }
    }
    pub fn take(&mut self) -> Option<Loan<'_, T>> {
        let value = self.value.take()?;
        Some(Loan {
            slot: self,
            value: Some(value),
        })
    }
    pub fn get(&self) -> Option<&T> {
        self.value.as_ref()
    }
}
pub(super) struct Loan<'a, T> {
    slot: &'a mut Slot<T>,
    value: Option<T>,
}
impl<T> Loan<'_, T> {
    pub fn get_mut(&mut self) -> &mut T {
        self.value.as_mut().expect("live local loan")
    }
    pub fn finish(&mut self) -> T {
        self.value.take().expect("live local loan")
    }
}
impl<T> Drop for Loan<'_, T> {
    fn drop(&mut self) {
        if let Some(value) = self.value.take() {
            self.slot.value = Some(value);
        }
    }
}
pub(super) enum Owner {
    Seed(Box<Seed>),
    Editor(Box<Editor>),
}
pub(super) struct State {
    pub owner: Option<Owner>,
    pub error: Option<String>,
    pub held_blocked: bool,
    pub prepared: bool,
    pub observer_closed: bool,
    cleanup: Option<Cleanup>,
    framework_retired: bool,
}
impl State {
    pub fn seed(seed: Seed) -> Self {
        Self {
            owner: Some(Owner::Seed(Box::new(seed))),
            error: None,
            held_blocked: false,
            prepared: false,
            observer_closed: true,
            cleanup: None,
            framework_retired: true,
        }
    }
    pub fn snapshot(&self) -> ProjectSnapshot {
        match self.owner.as_ref().expect("retained owner") {
            Owner::Seed(s) => s.session.snapshot(),
            Owner::Editor(e) => e.session.snapshot(),
        }
    }
    /// Check before creating another native backend, not just in its factory.
    pub fn retry_ready(&self) -> bool {
        self.framework_retired && self.runtime_retired()
    }
    pub fn runtime_retired(&self) -> bool {
        self.error.is_none() && self.cleanup.is_none() && matches!(self.owner, Some(Owner::Seed(_)))
    }
    /// Call only after observing the enclosing framework objects' retirement
    /// and dropping its Instance. Local runtime cleanup is not that receipt.
    pub fn framework_retired(&mut self) -> Result<(), String> {
        if !self.runtime_retired() {
            return Err("runtime cleanup incomplete".into());
        }
        self.framework_retired = true;
        Ok(())
    }
    pub fn open_backend(&self) -> Result<backend::Backend, String> {
        if !self.retry_ready() {
            return Err(
                "native cleanup must complete before creating a replacement backend".into(),
            );
        }
        backend::Backend::new()
    }
    pub fn retry_cleanup(&mut self) -> Result<(), String> {
        let Some(cleanup) = &mut self.cleanup else {
            return self
                .runtime_retired()
                .then_some(())
                .ok_or("no cleanup owner".into());
        };
        let Some(Owner::Editor(editor)) = &mut self.owner else {
            return Err("cleanup lost its runtime owner".into());
        };
        let result = cleanup.gate.run(|| {
            Cleanup::step(
                editor,
                &cleanup.notice,
                &cleanup.audio,
                &mut cleanup.observer,
                &mut cleanup.held,
                &mut self.observer_closed,
                &mut self.held_blocked,
            )
        });
        if let Err(failure) = result {
            self.error = Some(failure.message().into());
            return Err(failure.message().into());
        }
        let Some(Owner::Editor(editor)) = self.owner.take() else {
            unreachable!("retiring editor");
        };
        let Editor {
            session,
            ui,
            images,
            preview,
            host,
            importer,
            ..
        } = *editor;
        drop(importer);
        let gpu = match host.into_rebuild_seed() {
            Ok(gpu) => gpu,
            Err(_) => unreachable!("zero targets and joined worker checked without new work"),
        };
        self.owner = Some(Owner::Seed(Box::new(Seed {
            session,
            ui,
            images,
            preview,
            gpu,
        })));
        self.cleanup.take();
        self.error = None;
        Ok(())
    }
    pub(super) fn settle(&mut self, once: bool) {
        let end = Instant::now() + Duration::from_secs(5);
        while self.retry_cleanup().is_err() {
            if once
                || Instant::now() >= end
                || self.cleanup.as_ref().is_none_or(|c| c.gate.terminal())
            {
                break;
            }
            std::thread::yield_now();
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq)]
enum Failure {
    Pending(&'static str),
    Terminal(&'static str),
}
impl Failure {
    fn message(self) -> &'static str {
        match self {
            Self::Pending(m) | Self::Terminal(m) => m,
        }
    }
}
#[derive(Default)]
struct Gate(Option<Failure>);
impl Gate {
    fn terminal(&self) -> bool {
        matches!(self.0, Some(Failure::Terminal(_)))
    }
    fn run(&mut self, work: impl FnOnce() -> Result<(), Failure>) -> Result<(), Failure> {
        if let Some(failure @ Failure::Terminal(_)) = self.0 {
            return Err(failure);
        }
        let result = work();
        self.0 = result.err();
        result
    }
}
/// Retains every cleanup capability across a failed attempt. Pending is not a
/// release receipt; a failed join is sticky even if called again.
struct Cleanup {
    notice: Arc<AtomicU8>,
    audio: Option<Rc<RefCell<EditorAudio>>>,
    observer: Option<Observations>,
    held: Option<egui::FullOutput>,
    gate: Gate,
}
impl Cleanup {
    fn step(
        editor: &mut Editor,
        notice: &AtomicU8,
        audio: &Option<Rc<RefCell<EditorAudio>>>,
        observer: &mut Option<Observations>,
        held: &mut Option<egui::FullOutput>,
        observer_closed: &mut bool,
        held_blocked: &mut bool,
    ) -> Result<(), Failure> {
        if !editor.interrupt_for_fault() {
            return Err(Failure::Terminal(
                "failed initialization UI draft cancellation refused",
            ));
        }
        editor.host.loss_signal().report();
        if let Some(audio) = audio
            && let Some(native) = &audio.borrow().native
        {
            native.fault_signal().report();
        }
        *observer_closed = observer.as_mut().is_none_or(Observations::close);
        if !*observer_closed {
            return Err(Failure::Terminal(
                "failed initialization observer did not join",
            ));
        }
        observer.take();
        if let Some(audio) = audio {
            editor.ui.refresh_audio(&editor.session);
            let a = audio.borrow();
            if a.stop_pending {
                return Err(Failure::Pending(
                    "failed initialization audio endpoint return pending",
                ));
            }
            if a.close_failed || a.native.is_some() || a.client.is_some() {
                return Err(Failure::Terminal(
                    "failed initialization audio cleanup refused",
                ));
            }
        }
        editor.importer.close();
        if !editor.importer.closed() {
            return Err(Failure::Terminal(
                "failed initialization importer did not join",
            ));
        }
        editor.host.device.destroy();
        let _ = editor.host.device.poll(wgpu::PollType::Poll);
        match notice.load(Ordering::Acquire) {
            0 => return Err(Failure::Pending("framework device notice pending")),
            1 => (),
            _ => return Err(Failure::Terminal("unexpected device notice")),
        }
        let _ = editor.host.poll();
        if held.is_some() {
            *held_blocked |= editor.host.live_targets() > 0;
            held.take();
        }
        if !editor.host.preparation_closed() || editor.host.live_targets() != 0 {
            return Err(Failure::Pending(
                "failed initialization runtime still retiring",
            ));
        }
        Ok(())
    }
}
/// Returning an error or unwinding returns the same owned session. A cleanup
/// failure keeps the entire Editor in the slot and blocks another attempt.
pub(super) struct Attempt<'a> {
    loan: Loan<'a, State>,
    notice: Arc<AtomicU8>,
    audio: Option<Rc<RefCell<EditorAudio>>>,
    pub observer: Option<Observations>,
    pub held: Option<egui::FullOutput>,
    pub cleanup_once: bool,
}
impl<'a> Attempt<'a> {
    pub fn begin(
        slot: &'a mut Slot<State>,
        ctx: &egui::Context,
        state: &eframe::egui_wgpu::RenderState,
        notice: Arc<AtomicU8>,
    ) -> Result<Self, String> {
        let mut loan = slot.take().ok_or("project handoff already consumed")?;
        let value = loan.get_mut();
        if !value.retry_ready() {
            return Err("previous initialization still owns runtime resources".into());
        }
        let Some(Owner::Seed(seed)) = value.owner.take() else {
            return Err("missing project handoff".into());
        };
        value.framework_retired = false;
        match seed.restore(ctx, state) {
            Ok(editor) => value.owner = Some(Owner::Editor(Box::new(editor))),
            Err((error, seed)) => {
                value.owner = Some(Owner::Seed(seed));
                return Err(error);
            }
        }
        Ok(Self {
            loan,
            notice,
            audio: None,
            observer: None,
            held: None,
            cleanup_once: false,
        })
    }
    pub fn fresh(slot: &'a mut Slot<State>, editor: Editor, notice: Arc<AtomicU8>) -> Self {
        slot.value = Some(State {
            owner: Some(Owner::Editor(Box::new(editor))),
            error: None,
            held_blocked: false,
            prepared: false,
            observer_closed: true,
            cleanup: None,
            framework_retired: false,
        });
        Self {
            loan: slot.take().expect("new local owner"),
            notice,
            audio: None,
            observer: None,
            held: None,
            cleanup_once: false,
        }
    }
    pub fn editor(&mut self) -> &mut Editor {
        match self
            .loan
            .get_mut()
            .owner
            .as_mut()
            .expect("initializing owner")
        {
            Owner::Editor(e) => e,
            _ => unreachable!("editor was constructed before this borrow"),
        }
    }
    pub fn attach_audio(&mut self, audio: Rc<RefCell<EditorAudio>>) {
        self.audio = Some(audio);
    }
    pub fn prepared(&mut self) {
        self.loan.get_mut().prepared = true;
    }
    pub fn commit(mut self) -> Editor {
        // The observer is transferred explicitly by the caller before publish.
        let value = self.loan.finish();
        let Some(Owner::Editor(editor)) = value.owner else {
            unreachable!("ready editor");
        };
        *editor
    }
}
impl Drop for Attempt<'_> {
    fn drop(&mut self) {
        if self
            .loan
            .value
            .as_ref()
            .is_some_and(|s| matches!(s.owner, Some(Owner::Editor(_))))
        {
            let state = self.loan.get_mut();
            state.cleanup = Some(Cleanup {
                notice: self.notice.clone(),
                audio: self.audio.take(),
                observer: self.observer.take(),
                held: self.held.take(),
                gate: Gate::default(),
            });
            state.settle(self.cleanup_once);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn pending_cleanup_keeps_history_and_inflight_ownership_until_release_receipt() {
        struct Lease(Arc<std::sync::atomic::AtomicUsize>);
        impl Drop for Lease {
            fn drop(&mut self) {
                self.0.fetch_add(1, Ordering::AcqRel);
            }
        }
        let released = Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let lease = Arc::new(Lease(released.clone()));
        let worker_lease = lease.clone();
        let (release, gate) = mpsc::sync_channel(0);
        let worker = std::thread::spawn(move || {
            gate.recv().unwrap();
            drop(worker_lease);
        });
        let session = project();
        let identity = &*session as *const ProjectSession;
        let before = init_failure::Stamp::read(&session).unwrap();
        let mut slot = Slot::new((session, lease));
        let mut gate = Gate::default();
        for _ in 0..3 {
            let mut loan = slot.take().unwrap();
            assert_eq!(
                gate.run(|| Err(Failure::Pending("inflight"))),
                Err(Failure::Pending("inflight"))
            );
            assert_eq!(
                init_failure::Stamp::read(&loan.get_mut().0).unwrap(),
                before
            );
            assert_eq!(released.load(Ordering::Acquire), 0);
        }
        assert!(!gate.terminal());
        release.send(()).unwrap();
        worker.join().unwrap();
        assert_eq!(released.load(Ordering::Acquire), 0); // retained cleanup owner
        assert!(gate.run(|| Ok(())).is_ok());
        assert!(gate.0.is_none());
        let mut loan = slot.take().unwrap();
        let (same, lease) = loan.finish();
        drop(loan);
        assert_eq!(&*same as *const ProjectSession, identity);
        assert_eq!(init_failure::Stamp::read(&same).unwrap(), before);
        drop(lease);
        assert_eq!(released.load(Ordering::Acquire), 1);
    }
    #[test]
    fn failed_join_is_sticky_and_cannot_be_cleared_by_a_successful_noop_retry() {
        let session = project();
        let before = init_failure::Stamp::read(&session).unwrap();
        let slot = Slot::new(session);
        let mut gate = Gate::default();
        let failure = Failure::Terminal("join failed");
        assert_eq!(gate.run(|| Err(failure)), Err(failure));
        let mut work = 0;
        for _ in 0..3 {
            assert_eq!(
                gate.run(|| {
                    work += 1;
                    Ok(())
                }),
                Err(failure)
            );
        }
        assert_eq!(work, 0);
        assert!(gate.terminal());
        assert_eq!(
            init_failure::Stamp::read(slot.get().unwrap()).unwrap(),
            before
        );
    }
    fn project() -> Box<ProjectSession> {
        let mut s = ProjectSession::new(1, 9).unwrap();
        for (n, label) in [(1, b"branch A".to_vec()), (3, b"branch B".to_vec())] {
            let info = s.project_info();
            s.submit(CommandRequest {
                api_version: API_VERSION,
                session_id: info.session_id,
                request_id: RequestId::new(n).unwrap(),
                expected: info.state,
                commands: vec![Command::InsertConfigRecord {
                    index: 0,
                    record: ConfigRecord::new(label, ConfigInterpretation::Opaque).unwrap(),
                }],
            })
            .unwrap();
            let info = s.project_info();
            s.navigate(HistoryRequest {
                api_version: API_VERSION,
                session_id: info.session_id,
                request_id: RequestId::new(n + 1).unwrap(),
                expected: info.state,
                action: HistoryAction::Undo,
            })
            .unwrap();
        }
        Box::new(s)
    }
    #[test]
    fn failed_loan_and_retry_preserve_the_same_session_and_both_branch_contents() {
        let session = project();
        let identity = &*session as *const ProjectSession;
        let before = init_failure::Stamp::read(&session).unwrap();
        let mut slot = Slot::new(session);
        {
            let mut loan = slot.take().unwrap();
            assert_eq!(init_failure::Stamp::read(loan.get_mut()).unwrap(), before);
        }
        assert_eq!(&**slot.get().unwrap() as *const ProjectSession, identity);
        assert_eq!(
            init_failure::Stamp::read(slot.get().unwrap()).unwrap(),
            before
        );
        let mut retry = slot.take().unwrap();
        let owner = retry.finish();
        drop(retry);
        assert!(slot.get().is_none());
        assert_eq!(&*owner as *const ProjectSession, identity);
    }
    #[test]
    fn unwinding_before_publish_returns_history_without_creating_another_node() {
        let session = project();
        let before = init_failure::Stamp::read(&session).unwrap();
        let mut slot = Slot::new(session);
        let failure = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let _consumed = slot.take().unwrap();
            panic!("controlled initialization unwind");
        }));
        assert!(failure.is_err());
        assert_eq!(
            init_failure::Stamp::read(slot.get().unwrap()).unwrap(),
            before
        );
    }
    #[test]
    fn cancelled_initialization_keeps_an_inflight_lease_until_its_actual_owner_exits() {
        struct Lease(Arc<std::sync::atomic::AtomicUsize>);
        impl Drop for Lease {
            fn drop(&mut self) {
                self.0.fetch_add(1, Ordering::AcqRel);
            }
        }
        let released = Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let resource = Arc::new(Lease(released.clone()));
        let worker_lease = resource.clone();
        let (release, gate) = mpsc::sync_channel(0);
        let worker = std::thread::spawn(move || {
            gate.recv().unwrap();
            drop(worker_lease);
        });
        let mut slot = Slot::new((project(), resource));
        {
            let _cancelled = slot.take().unwrap();
        }
        assert_eq!(released.load(Ordering::Acquire), 0);
        drop(slot);
        assert_eq!(released.load(Ordering::Acquire), 0);
        release.send(()).unwrap();
        worker.join().unwrap();
        assert_eq!(released.load(Ordering::Acquire), 1);
    }
}
