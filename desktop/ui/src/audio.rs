use super::*;
/// Cache only passive numeric readouts. Preparation, errors, controls and the
/// live AudioEditorState keep their existing immediate refresh semantics.
#[derive(Default)]
pub(super) struct CounterLabels {
    previous: Option<(AudioOutputObservation, bool, bool)>,
    at: f64,
    values: [String; 2],
}
impl CounterLabels {
    fn update(
        &mut self,
        now: f64,
        output: &AudioOutputObservation,
        frozen: bool,
        freezing: bool,
    ) -> bool {
        let changed = self.previous.as_ref().is_none_or(|(old, a, b)| {
            *a != frozen
                || *b != freezing
                || old.candidate != output.candidate
                || old.rate != output.rate
                || old.channels != output.channels
                || old.background_frames != output.background_frames
                || old.missing_frames != output.missing_frames
                || output.callbacks < old.callbacks
                || output.copied_background < old.copied_background
                || output.generated_background < old.generated_background
        });
        if !changed && now.is_finite() && now >= self.at && now - self.at < 0.1 {
            return false;
        }
        self.previous = Some((output.clone(), frozen, freezing));
        self.at = now;
        self.values[0] = format!(
            "候选音频 · {} Hz / {} 声道 · 回调 {} · 欠载 {} 帧",
            output.rate, output.channels, output.callbacks, output.missing_frames
        );
        self.values[1] = if output.background_frames == 0 {
            "背景音频尚未载入；可试听音效".into()
        } else {
            format!(
                "背景复制 {} / {} 帧 · 生成 {} 帧",
                output.copied_background, output.background_frames, output.generated_background
            )
        };
        true
    }
}
impl EditorUi {
    pub fn scene_is_frozen(&self) -> bool {
        self.paused
            || self.audio_state.enabled && (self.audio_state.frozen || self.audio_state.freezing)
    }
    pub fn open_audio_editor(&mut self) {
        self.audio_window = true;
    }
    pub fn set_audio_port(&mut self, port: Box<dyn AudioEditorPort>) {
        self.audio_port = port;
        self.audio_sync_pending = true;
    }
    pub(super) fn synchronize_audio(&mut self, session: &ProjectSession) {
        self.audio_sync_error = session
            .synchronize_audio(self.audio_port.as_mut())
            .err()
            .map(|e| format!("音频同步未接纳：{e:?}；旧来源已门控，等待重试"));
        self.audio_sync_pending = self.audio_sync_error.is_some();
    }
    pub fn refresh_audio(&mut self, session: &ProjectSession) {
        if self.audio_sync_pending {
            self.synchronize_audio(session);
        }
        self.audio_state = self.audio_port.poll();
        if !self.audio_state.enabled && self.audio_state.frozen && self.audio_state.error.is_some()
        {
            // Re-enabling a failed device must not inherit the old play intent.
            self.paused = true;
        }
        if self.audio_sync_pending {
            self.audio_state.error = self.audio_sync_error.clone();
        }
    }
    pub fn audio_needs_poll(&self) -> bool {
        self.audio_sync_pending
            || self.audio_state.enabled
                && (self.audio_state.pending_controls
                    || self
                        .audio_state
                        .asset_audition
                        .as_ref()
                        .is_some_and(|s| s.preparation == AudioPreparationState::Pending)
                    || self.audio_state.freezing
                    || self.audio_state.voices.iter().any(|n| *n > 0)
                    || self
                        .audio_state
                        .components
                        .iter()
                        .any(|s| s.preparation == AudioPreparationState::Pending))
    }
    pub fn audio_command(&mut self, session: &ProjectSession, action: AudioEditorAction) {
        let accepted = match session.control_audio(self.audio_port.as_mut(), action) {
            Ok(()) => true,
            Err(e) => {
                self.message = format!("音频: {e:?}");
                false
            }
        };
        if accepted
            && action == AudioEditorAction::Enable
            && let Err(e) = session.control_audio(
                self.audio_port.as_mut(),
                AudioEditorAction::SceneRunning(!self.paused),
            )
        {
            self.message = format!("音频门控: {e:?}");
        }
        self.refresh_audio(session);
    }
    pub fn create_audio_component(
        &mut self,
        session: &mut ProjectSession,
        asset: AssetId,
        slot: String,
    ) -> Option<AudioComponentId> {
        let slot = match SoundSlotKey::new(slot) {
            Ok(s) => s,
            Err(e) => {
                self.message = format!("{e:?}");
                return None;
            }
        };
        let id = AudioComponentId::new(self.object_id()).unwrap();
        self.submit(
            session,
            vec![Command::CreateAudioComponent {
                component: AudioComponent::new(id, slot, asset),
            }],
        )
        .then_some(id)
    }
    fn source_label(source: SkinAudioSource) -> String {
        format!(
            "{} · r{} · nav{}{}",
            source.state.head.short_address(),
            source.state.revision,
            source.navigation_revision,
            source.draft.map_or_else(String::new, |d| format!(
                " · 草稿{}:{}",
                d.id.get(),
                d.revision
            ))
        )
    }
    pub(super) fn audio_panel(
        &mut self,
        ctx: &egui::Context,
        session: &mut ProjectSession,
        output: &mut UiOutput,
        observer: &mut Option<&mut dyn FnMut(UiPhase)>,
    ) {
        if !self.audio_window {
            return;
        }
        let mut open = true;
        egui::Window::new("音效编辑 · 与图片独立")
            .open(&mut open)
            .fade_in(false)
            .default_size(Vec2::new(540.0, 640.0))
            .default_pos(Pos2::new(270.0, 90.0))
            .show(ctx, |ui| {
                phase(observer, UiPhase::AudioDevice);
                ui.horizontal(|ui| {
                    if ui
                        .button(if self.audio_state.enabled {
                            "关闭音频设备"
                        } else {
                            "启用音频设备"
                        })
                        .clicked()
                    {
                        self.audio_command(
                            session,
                            if self.audio_state.enabled {
                                AudioEditorAction::Disable
                            } else {
                                AudioEditorAction::Enable
                            },
                        );
                    }
                    if ui.button("停止试听").clicked() {
                        self.audio_command(session, AudioEditorAction::StopAudition);
                    }
                    ui.label(if !self.audio_state.enabled {
                        "设备未启用"
                    } else if self.audio_state.freezing {
                        "音频冻结确认中"
                    } else if self.audio_state.frozen {
                        "场景音频已冻结 · 仍可试听"
                    } else {
                        "场景音频运行"
                    });
                });
                let mut volume = self.audio_state.monitor_volume;
                if ui
                    .add(egui::Slider::new(&mut volume, 0..=1000).text("监听总音量（不影响导出）"))
                    .changed()
                {
                    self.audio_command(session, AudioEditorAction::MonitorVolume(volume));
                }
                phase(observer, UiPhase::AudioStatus);
                if let Some(error) = &self.audio_state.error {
                    ui.colored_label(Color32::LIGHT_RED, error);
                }
                if let Some(s) = &self.audio_state.output {
                    self.audio_labels.update(
                        ui.input(|i| i.time),
                        s,
                        self.audio_state.frozen,
                        self.audio_state.freezing,
                    );
                    ui.label(&self.audio_labels.values[0]);
                    ui.label(&self.audio_labels.values[1]);
                } else {
                    self.audio_labels = CounterLabels::default();
                }
                if let Some((source, old)) = self.audio_state.last_audition {
                    ui.label(format!(
                        "上次触发{}：{}",
                        if old { "（旧版本）" } else { "" },
                        Self::source_label(source)
                    ));
                }
                ui.label(
                    "组件准备不会自动播放；素材试听会在准备完成后播放一次。支持短 PCM16 WAV。",
                );
                if let Some(s) = &self.audio_state.asset_audition {
                    ui.label(format!(
                        "素材试听 {}：{:?} · {}",
                        s.asset.get(),
                        s.preparation,
                        Self::source_label(s.source)
                    ));
                }
                phase(observer, UiPhase::AudioLibrary);
                egui::ScrollArea::vertical()
                    .max_height(560.0)
                    .show(ui, |ui| {
                        ui.heading("音频素材库");
                        if self.pending_imports > 0 {
                            ui.label(format!(
                                "待完成／登记导入：{}；活动草稿结束后登记",
                                self.pending_imports
                            ));
                        }
                        ui.text_edit_singleline(&mut self.audio_path);
                        ui.checkbox(
                            &mut self.audio_original,
                            "仅保留原始字节（其他编码／不解码）",
                        );
                        ui.horizontal(|ui| {
                            if ui.button("导入音频").clicked() && !self.audio_path.trim().is_empty()
                            {
                                output
                                    .audio_imports
                                    .push((self.audio_path.clone(), self.audio_original));
                            }
                            if ui.button("添加自制 WAV 示例").clicked() {
                                output.sample_sound = true;
                            }
                        });
                        let project = Self::project(session);
                        if self
                            .audio_asset
                            .is_some_and(|id| project.assets().get(id).is_none())
                        {
                            self.audio_asset = None;
                        }
                        if self
                            .audio_component
                            .is_some_and(|id| project.audio_component(id).is_none())
                        {
                            self.audio_component = None;
                        }
                        for asset in project
                            .assets()
                            .iter()
                            .filter(|a| matches!(a.kind(), AssetKind::Audio(_) | AssetKind::Opaque))
                        {
                            ui.horizontal(|ui| {
                                ui.selectable_value(
                                    &mut self.audio_asset,
                                    Some(asset.id()),
                                    asset.name(),
                                );
                                match asset.kind() {
                                    AssetKind::Audio(m) => {
                                        ui.label(format!(
                                            "{} Hz · {} ch · {:.3}s",
                                            m.sample_rate(),
                                            m.channels(),
                                            m.frames() as f64 / f64::from(m.sample_rate())
                                        ));
                                    }
                                    _ => {
                                        ui.label("仅原始保留 · 无解码能力");
                                    }
                                }
                            });
                        }
                        if let Some(asset) = self.audio_asset {
                            ui.label("用途键是编辑器身份，不推断 stable 文件名");
                            ui.text_edit_singleline(&mut self.audio_slot);
                            ui.horizontal(|ui| {
                                if ui.button("试听原始素材").clicked() {
                                    self.audio_command(
                                        session,
                                        AudioEditorAction::AuditionAsset(asset),
                                    );
                                }
                                if ui.button("创建音效组件").clicked() {
                                    self.audio_component = self.create_audio_component(
                                        session,
                                        asset,
                                        self.audio_slot.clone(),
                                    );
                                }
                                if ui.button("绑定到选中音效").clicked()
                                    && let Some(component) = self.audio_component
                                {
                                    self.submit(
                                        session,
                                        vec![Command::SetAudioAsset { component, asset }],
                                    );
                                }
                                if ui.button("删除素材").clicked() {
                                    self.submit(session, vec![Command::RemoveAsset { asset }]);
                                }
                            });
                            ui.label("原始试听不创建组件；同一素材可以被多个音效复用。");
                        }
                        phase(observer, UiPhase::AudioComponents);
                        ui.separator();
                        ui.heading("音效组件");
                        let project = Self::project(session);
                        for component in project.audio_components() {
                            if ui
                                .selectable_label(
                                    self.audio_component == Some(component.id()),
                                    format!(
                                        "{} · #{}",
                                        component.slot().as_str(),
                                        component.id().get()
                                    ),
                                )
                                .clicked()
                            {
                                self.cancel(session);
                                self.audio_component = Some(component.id());
                                self.audio_asset = Some(component.asset());
                            }
                        }
                        phase(observer, UiPhase::AudioProperties);
                        if let Some(component) = self
                            .audio_component
                            .and_then(|id| project.audio_component(id))
                        {
                            let id = component.id();
                            let mut gain = component.gain().get();
                            let mut visible = component.visible();
                            if ui
                                .checkbox(&mut visible, "可见／启用（隐藏保留静音占位计划）")
                                .changed()
                            {
                                self.submit(
                                    session,
                                    vec![Command::SetAudioProperties {
                                        component: id,
                                        gain: component.gain(),
                                        visible,
                                    }],
                                );
                            }
                            let response =
                                ui.add(egui::Slider::new(&mut gain, 0.0..=1.0).text("增益"));
                            ui.label("试听中的增益实时生效；拖动中 Esc 还原本次调整。");
                            self.parameter(
                                session,
                                response,
                                vec![Command::SetAudioProperties {
                                    component: id,
                                    gain: Gain::new(gain).unwrap(),
                                    visible,
                                }],
                            );
                            phase(observer, UiPhase::AudioReadiness);
                            let current = session.skin_audio_source();
                            if self.audio_state.enabled && self.audio_state.source != Some(current)
                            {
                                ui.label("音频来源同步中");
                            }
                            if let Some(s) = self
                                .audio_state
                                .components
                                .iter()
                                .find(|s| s.component == id)
                            {
                                ui.label(match &s.preparation {
                                    AudioPreparationState::Pending => "准备中".into(),
                                    AudioPreparationState::Ready => {
                                        if visible {
                                            "就绪".into()
                                        } else {
                                            "就绪 · 静音".into()
                                        }
                                    }
                                    AudioPreparationState::Failed(error) => {
                                        format!("准备失败：{error}；原始字节仍保留")
                                    }
                                });
                                if let Some(effective) = s.effective {
                                    ui.label(format!(
                                        "{}：{}",
                                        if effective != current {
                                            "有效旧版本"
                                        } else {
                                            "有效版本"
                                        },
                                        Self::source_label(effective)
                                    ));
                                } else {
                                    ui.label("尚无可用版本");
                                }
                            } else {
                                ui.label(if self.audio_state.enabled {
                                    "等待音频投影"
                                } else {
                                    "启用设备后准备"
                                });
                            }
                            phase(observer, UiPhase::AudioActions);
                            ui.horizontal(|ui| {
                                if ui.button("显式试听音效").clicked() {
                                    self.audio_command(session, AudioEditorAction::Audition(id));
                                }
                                if ui.button("删除音效").clicked() {
                                    self.submit(
                                        session,
                                        vec![Command::RemoveAudioComponent { component: id }],
                                    );
                                }
                                if Self::draft_state(session).is_some()
                                    && ui.button("取消增益草稿").clicked()
                                {
                                    self.cancel(session);
                                }
                            });
                        }
                        phase(observer, UiPhase::AudioActions);
                        if ui.button("重试准备").clicked()
                            && let Some(id) = self.audio_component
                        {
                            self.audio_command(session, AudioEditorAction::Retry(id));
                        }
                    });
                phase(observer, UiPhase::AudioPanel);
            });
        if !open {
            self.cancel(session);
            self.audio_window = false;
        }
    }
}
#[cfg(test)]
mod counter_tests {
    use super::*;
    fn observation() -> AudioOutputObservation {
        AudioOutputObservation {
            candidate: true,
            rate: 48000,
            channels: 2,
            callbacks: 1,
            missing_frames: 0,
            generated_background: 2048,
            copied_background: 0,
            background_frames: 480000,
            pcm_bytes: 65536,
        }
    }
    #[test]
    fn passive_readouts_are_bounded_while_live_observation_keeps_advancing() {
        let mut cache = CounterLabels::default();
        let mut output = observation();
        assert!(cache.update(1.0, &output, false, false));
        let text = cache.values.clone();
        output.callbacks = 2;
        output.copied_background = 480;
        output.generated_background += 480;
        assert!(!cache.update(1.05, &output, false, false));
        assert_eq!(cache.values, text);
        assert_eq!(output.copied_background, 480);
        assert!(cache.update(1.101, &output, false, false));
        assert_ne!(cache.values, text);
    }
    #[test]
    fn underrun_freeze_reset_format_and_clock_changes_are_never_hidden_by_cache() {
        for case in 0..7 {
            let mut cache = CounterLabels::default();
            let mut output = observation();
            output.callbacks = 9;
            output.copied_background = 960;
            output.generated_background = 4096;
            cache.update(1.0, &output, false, false);
            match case {
                0 => output.missing_frames = 1,
                1 => output.rate = 44100,
                2 => output.callbacks = 0,
                3 => output.copied_background = 0,
                4 => output.background_frames = 0,
                _ => {}
            }
            assert!(cache.update(
                if case == 6 { 0.5 } else { 1.01 },
                &output,
                case == 5,
                false
            ));
        }
    }
}
