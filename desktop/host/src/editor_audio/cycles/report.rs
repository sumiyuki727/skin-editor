use super::*;
impl Round {
    pub(super) fn record(&self, index: usize, usage: [usize; 5], post_drop: bool) -> String {
        let s = self
            .audio
            .borrow()
            .native
            .as_ref()
            .map_or(self.stats, |n| n.stats());
        let b = self.counters.background.stats();
        let probes = self
            .probes
            .complete
            .iter()
            .map(|p| {
                format!(
                    "[{},{},{},{},{},{},{},{},{},{}]",
                    p.id,
                    usize::from(p.lane == Channel::Audition),
                    p.voice,
                    p.upper_ms,
                    p.joined_ms,
                    p.first.callback_nanos,
                    p.first.callback_ordinal,
                    p.first.offset_frames,
                    p.first.callback_frames,
                    p.measured
                )
            })
            .collect::<Vec<_>>()
            .join(",");
        format!(
            concat!(
                "{{\"schema\":1,\"index\":{},\"session\":\"{}\",\"target\":{},\"shape\":[100,10,500,16],\"counts\":{:?},\"images_published\":{},",
                "\"playing_us\":{},\"native_play_frames\":{},\"background_start\":{},\"background_end\":{},\"ready_ms\":{},\"close_ms\":{},",
                "\"probes\":[{probes}],\"cpu_frame_ms\":{:?},\"resources_peak\":{:?},\"resources_final\":{usage:?},\"post_drop_confirmed\":{post_drop},",
                "\"workers_joined\":{:?},\"pending_graphics_at_close\":{},\"voices_at_close\":{:?},\"peak_voices\":{},\"view_valid\":{},\"geometry\":{:?},",
                "\"callbacks\":{},\"missing_frames\":{},\"render_errors\":{},\"observation_overflows\":{},\"device_fault\":{},\"endpoint_returns\":{},",
                "\"callback_resource_releases\":{},\"endpoint_returns_in_render\":{},\"endpoint_returns_outside_render\":{},",
                "\"background_read_bytes\":{},\"background_raw_peak\":{},\"background_raw_final\":{},\"background_fault\":{},",
                "\"source_uploads\":{},\"source_allocations\":{},\"staging_final_bytes\":{},\"max_trigger_lateness_us\":{}}}"
            ),
            index,
            self.session,
            self.target,
            self.counts,
            self.images_published,
            self.playing_us,
            self.playing_frames,
            self.bg_start,
            self.bg_end,
            self.ready_ms,
            self.close_ms,
            self.frame_ms,
            self.peaks,
            self.workers,
            self.pending_at_close,
            self.voices_at_close,
            self.max_voices,
            self.view_valid,
            self.geometry.unwrap_or([0.; 3]),
            s.callbacks,
            s.missing_frames,
            s.render_errors,
            s.observation_overflows,
            s.device_fault,
            s.endpoint_returns,
            s.callback_resource_releases,
            s.endpoint_returns_in_render,
            s.endpoint_returns_outside_render,
            b.read_bytes,
            b.raw_peak_bytes,
            b.raw_bytes,
            b.fault,
            self.editor.host.source_usage().0,
            self.editor.host.source_allocations(),
            self.editor.host.staging_usage().bytes,
            self.max_late_us,
            probes = probes,
            usage = usage,
            post_drop = post_drop
        )
    }
    pub(super) fn summary(&self, index: usize) -> String {
        format!(
            "{{\"index\":{index},\"session\":\"{}\",\"target\":{},\"resources_after_drop\":[0,0,0],\"workers_joined\":{:?},\"endpoint_returns\":{},\"pending_at_close\":{},\"playing_us\":{},\"close_ms\":{}}}",
            self.session,
            self.target,
            self.workers,
            self.stats.endpoint_returns,
            self.pending_at_close,
            self.playing_us,
            self.close_ms
        )
    }
}
impl Run {
    pub(super) fn report(&self) -> String {
        format!(
            concat!(
                "{{\"schema\":1,\"scope\":\"current-separated-full-resource-cycles\",\"smoke\":{},\"completed\":{},\"rounds\":{},",
                "\"expected_rounds\":{},\"rate\":{},\"channels\":{},\"period_frames\":{},\"production_policy\":{},\"gpu_memory_usage_hint\":{},\"reusable_staging\":false,",
                "\"physical_input_tested\":false,\"physical_inputs\":{},\"controlled_focus\":true,\"adapter_utf8\":{:?},",
                "\"cycles\":[{}],\"retained_summary_capacity_bytes\":{},\"closed_baseline_ms\":{},\"closed_settle_ms\":{},",
                "\"final_settle_ms\":{},\"final_observe_ms\":{},\"closed_editor_owners\":{},\"all_post_drop_bytes_zero\":{}}}"
            ),
            self.mode.smoke,
            self.completed,
            self.rows.len(),
            self.mode.rounds(),
            self.format.0,
            self.format.1,
            self.period,
            self.product,
            self.product && crate::product_policy::Policy::CURRENT.memory_usage,
            self.physical_inputs,
            self.adapter.as_bytes(),
            self.rows.join(","),
            self.rows.capacity() * std::mem::size_of::<String>()
                + self.rows.iter().map(|s| s.capacity()).sum::<usize>(),
            self.mode.baseline().as_millis(),
            self.mode.settle().as_millis(),
            self.mode.final_settle().as_millis(),
            self.mode.final_observe().as_millis(),
            self.active.is_none(),
            self.active.is_none() && self.retired.as_ref().is_none_or(|o| o.bytes() == [0; 3])
        )
    }
}
