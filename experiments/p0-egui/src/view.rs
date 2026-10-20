//! Small experimental UI helpers, kept separate from read-only fixture data.
use eframe::egui;
use std::time::Duration;

pub const LOADING_PERIOD: Duration = Duration::from_nanos(16_666_667);
pub fn wake_loading(ctx: &egui::Context) {
    // A burst of completions must not request immediate settling frames.
    ctx.request_repaint_after(LOADING_PERIOD);
}

pub fn show_resource_list<R>(
    ui: &mut egui::Ui,
    id: &str,
    empty: bool,
    contents: impl FnOnce(&mut egui::Ui) -> R,
) -> Option<egui::scroll_area::ScrollAreaOutput<R>> {
    if empty {
        ui.label("No items");
        return None;
    }
    Some(egui::ScrollArea::vertical().id_salt(id).show(ui, contents))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn completion_burst_does_not_request_immediate_or_settling_frames() {
        let ctx = egui::Context::default();
        for _ in 0..5 {
            let _ = ctx.run_ui(Default::default(), |_| {});
        }
        let output = ctx.run_ui(Default::default(), |_| {
            for _ in 0..100 {
                wake_loading(&ctx);
            }
        });
        assert!(output.viewport_output[&egui::ViewportId::ROOT].repaint_delay >= LOADING_PERIOD);
        let settled = ctx.run_ui(Default::default(), |_| {});
        assert!(
            settled.viewport_output[&egui::ViewportId::ROOT].repaint_delay
                >= Duration::from_secs(1)
        );
    }

    #[test]
    fn removing_list_contents_stops_requesting_animation_frames() {
        for remaining in [(0, 0), (12, 0), (0, 50)] {
            let ctx = egui::Context::default();
            let mut trailing_repaints = 0;
            let mut empty_animation_frames = 0;
            for frame in 0..300 {
                let (assets, layers) = if frame < 60 { (12, 50) } else { remaining };
                let output = ctx.run_ui(
                    egui::RawInput {
                        screen_rect: Some(egui::Rect::from_min_size(
                            egui::Pos2::ZERO,
                            egui::vec2(1100., 720.),
                        )),
                        time: Some(f64::from(frame) / 60.),
                        ..Default::default()
                    },
                    |root| {
                        egui::Panel::top("scene").show(root, |ui| {
                            ui.label("P0 load test");
                        });
                        egui::Panel::left("library")
                            .default_size(180.)
                            .show(root, |ui| {
                                ui.heading("Asset library");
                                ui.label("100 originals / 12 thumbnails");
                                show_resource_list(ui, "assets", assets == 0, |ui| {
                                    for number in 0..assets {
                                        ui.allocate_space(egui::vec2(100., 40.));
                                        ui.label(format!("Asset {number}"));
                                    }
                                });
                            });
                        egui::Panel::right("layers")
                            .default_size(210.)
                            .show(root, |ui| {
                                ui.heading("Layers / parameters");
                                ui.label("Document 9 / edit 300");
                                ui.label("CPU decoded: 0 MiB");
                                ui.label("Texture payload: 0 MiB");
                                show_resource_list(ui, "layers", layers == 0, |ui| {
                                    for number in 0..layers {
                                        ui.label(format!("Layer {number}"));
                                    }
                                });
                            });
                        egui::CentralPanel::default().show(root, |ui| {
                            ui.allocate_space(ui.available_size());
                        });
                    },
                );
                if frame >= 180
                    && output.viewport_output[&egui::ViewportId::ROOT].repaint_delay
                        < Duration::from_secs(1)
                {
                    trailing_repaints += 1;
                }
                if remaining == (0, 0)
                    && (61..90).contains(&frame)
                    && output.viewport_output[&egui::ViewportId::ROOT].repaint_delay
                        < Duration::from_secs(1)
                {
                    empty_animation_frames += 1;
                }
            }
            assert_eq!(
                trailing_repaints, 0,
                "Lists {remaining:?} kept requesting frames long after contents were removed"
            );
            assert_eq!(
                empty_animation_frames, 0,
                "Empty resource panes should show a static placeholder, not animate obsolete scrollbars"
            );
        }
    }

    #[test]
    fn queued_texture_upload_shares_decoded_pixels() {
        use std::sync::Arc;
        let ctx = egui::Context::default();
        let decoded = Arc::new(egui::ColorImage::filled([2, 2], egui::Color32::RED));
        let texture = ctx.load_texture(
            "shared-source",
            decoded.clone(),
            egui::TextureOptions::LINEAR,
        );
        let output = ctx.run_ui(Default::default(), |ui| {
            ui.image((texture.id(), egui::vec2(2., 2.)));
        });
        let upload = &output
            .textures_delta
            .set
            .get(&texture.id())
            .expect("source upload")[0]
            .image;
        let egui::ImageData::Color(pixels) = upload;
        assert!(
            Arc::ptr_eq(pixels, &decoded),
            "Uploading a shared source must not clone its pixel plane"
        );
    }
}
