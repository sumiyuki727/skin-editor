use super::*;
impl Run {
    pub(super) fn report(&self) -> String {
        let s = self
            .audio
            .borrow()
            .native
            .as_ref()
            .map_or(self.stats, |n| n.stats());
        let native = format!(
            concat!(
                "{{\"callbacks\":{},\"frames\":{},\"missing_frames\":{},\"render_errors\":{},\"observation_overflows\":{},",
                "\"device_fault\":{},\"nonzero_samples\":{},\"endpoint_returns\":{},\"callback_resource_releases\":{},\"endpoint_returns_in_render\":{},\"endpoint_returns_outside_render\":{}}}"
            ),
            s.callbacks,
            s.frames,
            s.missing_frames,
            s.render_errors,
            s.observation_overflows,
            s.device_fault,
            s.nonzero_samples,
            s.endpoint_returns,
            s.callback_resource_releases,
            s.endpoint_returns_in_render,
            s.endpoint_returns_outside_render
        );
        let v = self.source.stats();
        let background = format!(
            "{{\"frames\":{},\"read_bytes\":{},\"seeks\":{},\"discarded_frames\":{},\"raw_peak_bytes\":{},\"raw_final_bytes\":{},\"fault\":{},\"start\":{},\"end\":{},\"continuous\":true}}",
            u64::from(self.format.0) * u64::from(crate::stream_fixture::SECONDS),
            v.read_bytes,
            v.seeks,
            v.discarded_frames,
            v.raw_peak_bytes,
            v.raw_bytes,
            v.fault,
            self.bg_start,
            self.bg_end
        );
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
        let traces=self.editor.host.traces.iter().map(|t|format!("{{\"started_ms\":{},\"published_ms\":{},\"gpu_done_ms\":{},\"decode_ms\":{},\"upload_ms\":{},\"reused\":{}}}",
            t.started_ms,t.published_ms.map_or("null".into(),|v|v.to_string()),t.gpu_done_ms.map_or("null".into(),|v|v.to_string()),t.decode_ms,t.gpu.upload_ms,t.gpu.reused))
            .collect::<Vec<_>>().join(",");
        let st = self.editor.host.staging_usage();
        format!(
            concat!(
                "{{\"schema\":1,\"scope\":\"current-separated-streamed-standard-N\",\"completed\":{},\"seconds\":300,\"playing_us\":{},",
                "\"shape\":[100,10,500,16],\"counts\":{:?},\"normal_auto_controls\":true,\"physical_input_tested\":false,\"physical_inputs\":{},",
                "\"controlled_focus\":true,\"production_policy\":{},\"gpu_memory_usage_hint\":{},\"reusable_staging\":false,\"adapter_utf8\":{:?},\"rate\":{},\"channels\":{},\"period_frames\":{},",
                "\"geometry\":{:?},\"view_valid\":{},\"native_play_frames\":{},\"native\":{native},\"background\":{background},",
                "\"probes\":[{probes}],\"edit_activation_ms\":{:?},\"switch_activation_ms\":{:?},\"cpu_frame_ms\":{:?},",
                "\"preparation_trace\":[{traces}],\"trace_overflow\":{},\"publications\":{},\"allocations\":{},\"max_voices\":{},",
                "\"resources_peak\":{:?},\"resources_final\":{:?},\"staging_final_bytes\":{},\"source_allocations\":{},\"source_uploads\":{},",
                "\"controls\":{:?},\"hot_continuity\":{:?},\"max_trigger_lateness_us\":{},\"max_supply_gap_micros\":{},\"gui_passes\":{}}}"
            ),
            self.completed,
            self.playing_us,
            self.plan.counts,
            self.physical_inputs,
            self.product,
            self.product && crate::product_policy::Policy::CURRENT.memory_usage,
            self.adapter.as_bytes(),
            self.format.0,
            self.format.1,
            self.period,
            self.geometry.unwrap_or([0.; 3]),
            self.view_valid,
            self.playing_frames,
            self.edits_ms,
            self.switches_ms,
            self.frame_ms,
            self.editor.host.trace_overflow,
            self.editor.host.publications,
            self.editor.host.allocations,
            self.max_voices,
            self.peaks,
            self.final_usage,
            st.bytes,
            self.editor.host.source_allocations(),
            self.editor.host.source_usage().0,
            self.controls,
            self.continuity,
            self.plan.max_late_us,
            self.max_supply_gap,
            self.editor.ui_passes,
            native = native,
            background = background,
            probes = probes,
            traces = traces
        )
    }
}
