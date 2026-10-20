use p0_preview_contract::{Error, Probe, Scene, Tool};

#[test]
fn paused_material_edit_changes_pixels_without_advancing_simulation() {
    let mut probe = Probe::new(0);
    probe.pause(true, 123).unwrap();
    let frozen = probe.observe();
    let before = probe.render_request().unwrap().render();
    assert!(probe.present(&before));
    probe.replace_material(0, [10, 20, 30]).unwrap();
    probe.tick(60_000).unwrap();
    let after = probe.render_request().unwrap().render();
    assert_ne!(before.rgb(), after.rgb());
    assert_eq!(probe.observe().simulation_ms, frozen.simulation_ms);
    assert_eq!(probe.observe().audio_cursor_ms, frozen.audio_cursor_ms);
    // The moving marker's position and phase are retained through a material swap.
    let marker = (8 * 32 + 123 % 32) * 3;
    assert_eq!(&after.rgb()[marker..marker + 3], &[255, 255, 255]);
}

#[test]
fn resume_without_any_paused_ticks_does_not_catch_up() {
    let mut probe = Probe::new(10);
    probe.pause(true, 110).unwrap();
    probe.pause(false, 60_110).unwrap();
    probe.tick(60_120).unwrap();
    assert_eq!(probe.observe().simulation_ms, 110);
    assert_eq!(probe.observe().audio_cursor_ms, 110);
}

#[test]
fn paused_inputs_are_discarded_instead_of_replayed_on_resume() {
    let mut probe = Probe::new(0);
    probe.select_tool(Tool::Interact);
    assert!(probe.game_press());
    probe.pause(true, 1).unwrap();
    for _ in 0..100 {
        assert!(!probe.game_press());
    }
    probe.pause(false, 5_000).unwrap();
    assert_eq!(probe.observe().accepted_game_inputs, 1);
    assert!(probe.game_press());
    assert_eq!(probe.observe().accepted_game_inputs, 2);
}

#[test]
fn edit_tool_and_focus_prevent_game_interaction() {
    let mut probe = Probe::new(0);
    assert!(!probe.game_press());
    probe.select_tool(Tool::Interact);
    probe.set_focus(false);
    assert!(!probe.game_press());
    probe.set_focus(true);
    assert!(probe.game_press());
    probe.select_tool(Tool::Edit);
    assert!(!probe.game_press());
}

#[test]
fn outdated_render_cannot_overwrite_a_new_material_or_scene() {
    let mut probe = Probe::new(0);
    let old = probe.render_request().unwrap().render();
    probe.replace_material(0, [1, 2, 3]).unwrap();
    let intermediate = probe.render_request().unwrap().render();
    probe.select_scene(Scene::Second);
    assert!(!probe.present(&old));
    assert!(!probe.present(&intermediate));
    let current = probe.render_request().unwrap().render();
    assert!(probe.present(&current));
}

#[test]
fn paused_idle_does_not_request_repeated_frames() {
    let mut probe = Probe::new(0);
    probe.pause(true, 10).unwrap();
    let frame = probe.render_request().unwrap().render();
    assert!(probe.present(&frame));
    for time in 11..1_000 {
        probe.tick(time).unwrap();
        assert!(probe.render_request().is_none());
    }
    assert!(!probe.present(&frame));
}

#[test]
fn scene_control_works_while_paused_without_a_content_edit() {
    let mut probe = Probe::new(0);
    probe.pause(true, 50).unwrap();
    let before = probe.render_request().unwrap().render();
    probe.select_scene(Scene::Second);
    let after = probe.render_request().unwrap().render();
    assert_ne!(before.rgb(), after.rgb());
    assert!(probe.observe().paused);
    assert_eq!(probe.observe().simulation_ms, 50);
    assert_eq!(probe.observe().content_revision, 0);
}

#[test]
fn stale_content_command_is_rejected_without_state_or_frame_change() {
    let mut probe = Probe::new(0);
    probe.replace_material(0, [1, 2, 3]).unwrap();
    let before = probe.observe();
    let frame = probe.render_request().unwrap().render();
    assert_eq!(probe.replace_material(0, [5, 6, 7]), Err(Error::Conflict));
    assert_eq!(probe.observe(), before);
    assert!(probe.present(&frame));
}

#[test]
fn no_change_does_not_invalidate_a_presented_frame() {
    let mut probe = Probe::new(0);
    let frame = probe.render_request().unwrap().render();
    assert!(probe.present(&frame));
    assert_eq!(probe.replace_material(0, [40, 80, 120]), Ok(false));
    assert!(probe.render_request().is_none());
    assert_eq!(probe.observe().content_revision, 0);
}

#[test]
fn backwards_wall_time_is_rejected_atomically() {
    let mut probe = Probe::new(100);
    assert_eq!(probe.pause(true, 90), Err(Error::NonMonotonicTime));
    assert!(!probe.observe().paused);
    probe.tick(110).unwrap();
    assert_eq!(probe.observe().simulation_ms, 10);
}
