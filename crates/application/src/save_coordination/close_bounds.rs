use super::*;
use crate::{CommandError, CommandRequest, DraftAction, DraftRequest, RequestId};

struct Ids(u128);
impl HistoryIdentitySource for Ids {
    fn next_identity(&mut self) -> Result<u128, crate::IdentityError> {
        self.0 += 1;
        Ok(self.0)
    }
}
fn request(s: &ProjectSession) -> CloseRequest {
    CloseRequest {
        api_version: API_VERSION,
        key: s.close_status(1).unwrap().next_key.unwrap(),
        source: s.save_source(),
        purpose: ClosePurpose::Exit,
    }
}
fn qualified() -> (ProjectSession, SaveProtocolPort) {
    let (old, _port, _) = tests::setup();
    let config = old
        .save_coordination
        .as_ref()
        .unwrap()
        .resident
        .config
        .unwrap();
    let mut s = ProjectSession::new(7, 70).unwrap();
    let authority = synthetic_authority(&s, false);
    let port = s
        .install_save_protocol(config, SaveCandidateLimits::default(), authority)
        .unwrap();
    (s, port)
}
#[test]
fn final_save_and_close_maximum_are_usable_once_without_wrap_or_lost_cleanup() {
    let (mut s, _port) = qualified();
    let old_command = CommandRequest {
        api_version: API_VERSION,
        session_id: s.session_id,
        request_id: RequestId::new(1).unwrap(),
        expected: s.state,
        commands: Default::default(),
    };
    let old_result = s.submit(old_command.clone()).unwrap();
    let old_navigation = crate::HistoryRequest {
        api_version: API_VERSION,
        session_id: s.session_id,
        request_id: RequestId::new(2).unwrap(),
        expected: s.state,
        action: crate::HistoryAction::Undo,
    };
    let old_navigation_result = s.navigate(old_navigation.clone()).unwrap();
    let c = s.save_coordination.as_mut().unwrap();
    c.resident.next_save = u64::MAX;
    c.close.next_close = Some(u64::MAX);
    let r = request(&s);
    let t = s.request_close(r).unwrap().ticket();
    s.choose_close(t, CloseChoice::Manual).unwrap();
    let mut ids = Ids(1000);
    s.advance_close(t, &mut ids).unwrap();
    let child = s.close_status(1).unwrap().active.unwrap().child.unwrap();
    assert_eq!(child.key.sequence, u64::MAX);
    assert_eq!(child.parent_close, Some(t.key));
    assert!(s.close_status(1).unwrap().next_key.is_none());
    assert!(s.save_coordination_status(1).unwrap().next_key.is_none());
    assert_eq!(s.request_close(r), Ok(CloseAdmission::Existing(t)));
    let consumed = ids.0;
    s.cancel_save(child).unwrap();
    assert_eq!(s.acknowledge_save(child), Ok(SaveAcknowledgement::Released));
    assert_eq!(
        s.retry_close(t, child, &mut ids),
        Err(SaveCoordinationError::Limit(SaveLimit::SaveIdentity))
    );
    for _ in 0..8 {
        s.advance_close(t, &mut ids).unwrap();
    }
    assert_eq!(ids.0, consumed);
    s.cancel_close(t).unwrap();
    s.acknowledge_close(t).unwrap();
    assert_eq!(s.request_close(r), Err(SaveCoordinationError::Stale));
    assert_eq!(s.acknowledge_close(t), Err(SaveCoordinationError::Stale));
    assert_eq!(s.acknowledge_save(child), Err(SaveCoordinationError::Stale));
    assert_eq!(
        s.submit(CommandRequest {
            api_version: API_VERSION,
            session_id: s.session_id,
            request_id: RequestId::new(3).unwrap(),
            expected: s.state,
            commands: Default::default(),
        }),
        Err(CommandError::ResourceLimit("save_control_identity"))
    );
    assert_eq!(s.submit(old_command.clone()), Ok(old_result));
    assert_eq!(s.navigate(old_navigation), Ok(old_navigation_result));
    let mut conflict = old_command;
    conflict.expected.revision += 1;
    assert_eq!(s.submit(conflict), Err(CommandError::RequestIdReused));
    assert_eq!(s.receipts.len(), 2);
    assert_eq!(s.history_len(), 1);
}
#[test]
fn last_close_identity_keeps_real_draft_cancel_and_final_capture_available() {
    for finish_before_cancel in [false, true] {
        let (mut s, _port) = qualified();
        let begin = DraftRequest {
            api_version: API_VERSION,
            session_id: s.session_id,
            request_id: RequestId::new(1).unwrap(),
            action: DraftAction::Begin { expected: s.state },
        };
        let old_begin = s.edit_draft(begin.clone()).unwrap();
        let draft = s.draft.as_ref().unwrap().state;
        s.save_coordination.as_mut().unwrap().close.next_close = Some(u64::MAX);
        let r = request(&s);
        let close = s.request_close(r).unwrap().ticket();
        let permit = s.close_finish_permit().unwrap();
        s.choose_close(close, CloseChoice::Manual).unwrap();
        let cancel = DraftRequest {
            api_version: API_VERSION,
            session_id: s.session_id,
            request_id: RequestId::new(2).unwrap(),
            action: DraftAction::Cancel { expected: draft },
        };
        if finish_before_cancel {
            s.finish_close_draft(permit, cancel).unwrap();
            s.advance_close(close, &mut Ids(1000)).unwrap();
            let child = s.close_status(1).unwrap().active.unwrap().child.unwrap();
            s.cancel_close(close).unwrap();
            s.acknowledge_save(child).unwrap();
        } else {
            s.cancel_close(close).unwrap();
            s.edit_draft(cancel).unwrap();
        }
        s.acknowledge_close(close).unwrap();
        assert_eq!(s.edit_draft(begin), Ok(old_begin));
        assert!(s.draft.is_none());
        assert_eq!(s.receipts.len(), 2);
        assert_eq!(s.history_len(), 1);
    }
}
#[test]
fn close_only_installation_reserves_the_same_profile_without_a_work_port() {
    let mut s = ProjectSession::new(7, 70).unwrap();
    assert_eq!(
        s.install_close_control(SaveControlProfile {
            version: 1,
            available_bytes: SAVE_CONTROL_BYTES - 1
        }),
        Err(SaveCoordinationError::Limit(SaveLimit::Control))
    );
    assert!(s.save_coordination.is_none());
    s.install_close_control(SaveControlProfile::default())
        .unwrap();
    assert_eq!(
        s.save_coordination_status(1).unwrap().availability,
        SaveAvailability::Unavailable
    );
    assert_eq!(
        s.save_coordination_status(1)
            .unwrap()
            .reserved_control_bytes,
        SAVE_CONTROL_BYTES
    );
    assert_eq!(
        s.install_close_control(SaveControlProfile::default()),
        Err(SaveCoordinationError::AlreadyInstalled)
    );
}
