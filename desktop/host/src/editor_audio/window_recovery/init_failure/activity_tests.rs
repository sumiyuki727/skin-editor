use super::endpoint_tests::rig_with_ui;
use super::*;

fn settle(
    a: &Rc<RefCell<EditorAudio>>,
    callback: &mut skin_editor_infrastructure::separated_native::ControlledCallback,
) {
    let end = Instant::now() + Duration::from_secs(5);
    loop {
        // A barrier can itself await a copy acknowledgement. Drive the real
        // software endpoint between bounded receipt reads instead of blocking
        // the only actor capable of producing that acknowledgement.
        callback.render(&mut [0.0; 128], 100);
        {
            let mut a = a.borrow_mut();
            if let Some(r) = a
                .client
                .as_mut()
                .unwrap()
                .wait_receipt(Duration::from_millis(2))
                .unwrap()
            {
                r.result.as_ref().unwrap();
                a.receive(r.id, &r.result);
            }
        }
        let state = a.borrow_mut().poll();
        if state.components.len() == 1
            && state.components[0].preparation == AudioPreparationState::Ready
            && !state.pending_controls
            && !state.freezing
        {
            return;
        }
        assert!(Instant::now() < end, "controlled audio completion deadline");
    }
}
#[test]
fn active_audio_draft_fault_or_ui_interrupt_invalidates_old_voices_without_committing() {
    for native_fault in [false, true] {
        let (mut session, a, mut callback, mut gate, pool, mut ui) = rig_with_ui();
        let shared = Rc::new(RefCell::new(a));
        ui.set_audio_port(Box::new(Shared(shared.clone())));
        let before = Stamp::read(&session).unwrap();
        let expected = session.project_info().state;
        assert!(ui.draft(&mut session, DraftAction::Begin { expected }));
        let d = session
            .draft_info(session.project_info().session_id)
            .unwrap()
            .unwrap()
            .state;
        assert!(ui.draft(
            &mut session,
            DraftAction::Replace {
                expected: d,
                commands: vec![Command::SetAudioProperties {
                    component: sound::component(),
                    gain: Gain::new(0.5).unwrap(),
                    visible: true,
                }]
            }
        ));
        settle(&shared, &mut callback);
        ui.refresh_audio(&session);
        shared
            .borrow_mut()
            .request(AudioEditorAction::Audition(sound::component()))
            .unwrap();
        settle(&shared, &mut callback);
        let mut samples = [0.0; 128];
        callback.render(&mut samples, 200);
        assert!(samples.iter().any(|v| v.abs() > 0.0001));
        if native_fault {
            shared
                .borrow()
                .native
                .as_ref()
                .unwrap()
                .fault_signal()
                .report();
            samples.fill(1.0);
            callback.render(&mut samples, 300);
            assert_eq!(samples, [0.0; 128]); // before another UI or project query
        }
        assert!(ui.interrupt_for_fault(&mut session));
        assert!(ui.interrupt_for_fault(&mut session));
        ui.refresh_audio(&session);
        assert!(
            session
                .draft_info(session.project_info().session_id)
                .unwrap()
                .is_none()
        );
        assert!(before.same_committed(&Stamp::read(&session).unwrap()));
        samples.fill(1.0);
        callback.render(&mut samples, 400);
        assert_eq!(samples, [0.0; 128]);
        if !native_fault {
            settle(&shared, &mut callback);
            assert_eq!(shared.borrow_mut().poll().voices, [0, 0]);
            shared
                .borrow_mut()
                .request(AudioEditorAction::SceneRunning(true))
                .unwrap();
            settle(&shared, &mut callback);
            for _ in 0..4 {
                samples.fill(1.0);
                callback.render(&mut samples, 500);
                assert_eq!(samples, [0.0; 128]);
            }
            shared
                .borrow_mut()
                .request(AudioEditorAction::Audition(sound::component()))
                .unwrap();
            settle(&shared, &mut callback);
            callback.render(&mut samples, 600);
            assert!(samples.iter().any(|v| v.abs() > 0.0001));
        } else {
            assert!(
                shared
                    .borrow_mut()
                    .client
                    .as_mut()
                    .unwrap()
                    .trigger_selected(Channel::Scene, sound::component())
                    .is_err()
            );
            assert!(shared.borrow().stop_pending);
        }
        assert!(shared.borrow_mut().close().is_err());
        let retained = pool.usage().bytes;
        assert!(retained > 0);
        drop(callback);
        assert_eq!(pool.usage().bytes, retained);
        gate.deliver().unwrap();
        shared.borrow_mut().close().unwrap();
        assert_eq!(pool.usage().bytes, 0);
        assert!(before.same_committed(&Stamp::read(&session).unwrap()));
        assert_eq!(
            Stamp::read(&session).unwrap().requests() - before.requests(),
            3
        );
    }
}
