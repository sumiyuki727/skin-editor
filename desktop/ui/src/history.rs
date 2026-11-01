//! Bounded history navigation. Queries describe one head for this UI pass;
//! navigation is applied only after its controls have been presented.
use super::*;

impl EditorUi {
    pub(super) fn history_panel(&mut self, ui: &mut egui::Ui, session: &mut ProjectSession) {
        let info = session.project_info();
        let mut navigation = None;
        ui.heading("历史");
        ui.label(format!(
            "{} · r{}",
            info.state.head.short_address(),
            info.state.revision
        ));
        ui.push_id(
            ("history", info.session_id.get(), info.state.head.get()),
            |ui| {
                ui.label("当前节点的重做分支");
                match session.history_children(info.session_id, info.state.head, 0, 16) {
                    Ok(page) => {
                        ui.weak(format!("本页 {} 项", page.entries.len()));
                        if page.entries.is_empty() {
                            ui.weak("当前节点没有重做分支");
                        } else {
                            let height =
                                (ui.available_height() * 0.4).max(ui.spacing().interact_size.y);
                            egui::ScrollArea::vertical()
                                .id_salt("children")
                                .max_height(height)
                                .show(ui, |ui| {
                                    for node in page.entries {
                                        if ui
                                            .push_id(("child", node.id.get()), |ui| {
                                                ui.button(node.id.short_address())
                                            })
                                            .inner
                                            .clicked()
                                        {
                                            navigation = Some(HistoryAction::Redo {
                                                child: Some(node.id),
                                            });
                                        }
                                    }
                                });
                        }
                        if page.next_offset.is_some() {
                            ui.weak("还有更多子分支，当前仅显示前16项");
                        }
                    }
                    Err(e) => {
                        ui.colored_label(Color32::LIGHT_RED, format!("重做分支查询失败：{e:?}"));
                    }
                }
                ui.separator();
                ui.label("当前与祖先节点");
                match session.history_ancestors(info.session_id, info.state.head, 12) {
                    Ok(page) => {
                        let height = ui.available_height().max(ui.spacing().interact_size.y);
                        egui::ScrollArea::vertical()
                            .id_salt("ancestors")
                            .max_height(height)
                            .show(ui, |ui| {
                                for node in page.entries {
                                    if ui
                                        .push_id(("ancestor", node.id.get()), |ui| {
                                            ui.small_button(format!(
                                                "{}  {}",
                                                node.id.short_address(),
                                                node.coordinate.short_address()
                                            ))
                                        })
                                        .inner
                                        .clicked()
                                    {
                                        navigation =
                                            Some(HistoryAction::Checkout { node: node.id });
                                    }
                                }
                                if page.next_ancestor.is_some() {
                                    ui.weak("仅显示最近12项；可继续撤销查看更早节点");
                                }
                            });
                    }
                    Err(e) => {
                        ui.colored_label(Color32::LIGHT_RED, format!("祖先查询失败：{e:?}"));
                    }
                }
            },
        );
        if let Some(action) = navigation {
            self.navigation(session, action);
            ui.ctx().request_repaint();
        }
    }
}
