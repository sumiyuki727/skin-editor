use p0_preview_contract::{Probe, Tool};

fn main() {
    let mut probe = Probe::new(0);
    probe.select_tool(Tool::Interact);
    probe.pause(true, 123).expect("monotonic clock");
    let before = probe.render_request().expect("initial frame").render();
    assert!(probe.present(&before));
    probe
        .replace_material(0, [180, 50, 90])
        .expect("current revision");
    let after = probe
        .render_request()
        .expect("content invalidation")
        .render();
    let changed = before.rgb() != after.rgb();
    let stale_rejected = !probe.present(&before);
    assert!(probe.present(&after));
    let input_discarded = !probe.game_press();
    probe.pause(false, 60_123).expect("resume after idle");
    probe.tick(60_133).expect("first resumed tick");
    let result = probe.observe();
    assert!(changed && stale_rejected && input_discarded);
    assert_eq!(result.simulation_ms, 133);
    println!(
        "{{\"probe\":\"synthetic-preview-contract\",\"changed_pixels_while_paused\":{},\"stale_frame_rejected\":{},\"paused_input_discarded\":{},\"simulation_after_resume_ms\":{},\"logical_audio_cursor_ms\":{},\"ui_gpu_audio_backend_tested\":false}}",
        changed, stale_rejected, input_discarded, result.simulation_ms, result.audio_cursor_ms
    );
}
