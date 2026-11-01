//! Explicit diagnostic only. Observe fresh output without changing it. Weak
//! identities survive one pass; they never keep a galley/atlas/render lease alive.
use eframe::egui::{self, epaint::Shape};
use std::{sync::Weak, time::Instant};

const SHAPES: usize = 512;
const PASSES: usize = 1024;
pub struct Census {
    pub stage: u8,
    previous: Vec<Weak<egui::Galley>>,
    current: Vec<Weak<egui::Galley>>,
    rows: Vec<[u64; 16]>,
    invalid: u64,
}
impl Census {
    pub fn new() -> Self {
        Self {
            stage: 0,
            previous: Vec::with_capacity(SHAPES),
            current: Vec::with_capacity(SHAPES),
            rows: Vec::with_capacity(PASSES),
            invalid: 0,
        }
    }
    fn visit(&mut self, shape: &Shape, row: &mut [u64; 16], depth: usize) {
        if depth > 32 {
            self.invalid += 1;
            return;
        }
        match shape {
            Shape::Vec(shapes) => {
                for shape in shapes {
                    self.visit(shape, row, depth + 1);
                }
            }
            Shape::Text(text) => {
                row[2] += 1;
                if self.current.len() == SHAPES {
                    self.invalid += 1;
                    return;
                }
                let identity = std::sync::Arc::as_ptr(&text.galley) as usize;
                row[3] += u64::from(
                    self.previous
                        .binary_search_by_key(&identity, |w| w.as_ptr() as usize)
                        .is_ok(),
                );
                self.current.push(std::sync::Arc::downgrade(&text.galley));
                for placed in &text.galley.rows {
                    row[5] += placed.row.visuals.mesh.vertices.len() as u64;
                    row[6] += placed.row.visuals.mesh.indices.len() as u64;
                }
            }
            _ => row[14] += 1,
        }
    }
    pub fn clear_identities(&mut self) {
        self.previous.clear();
        self.current.clear();
    }
    pub fn report(&self) -> String {
        format!(
            "{{\"schema\":1,\"invalid\":{},\"weak_identities\":{},\"rows\":{:?}}}",
            self.invalid,
            self.previous.len() + self.current.len(),
            self.rows
        )
    }
}
impl egui::plugin::Plugin for Census {
    fn debug_name(&self) -> &'static str {
        "bounded-text-work-diagnostic"
    }
    fn output_hook(&mut self, ctx: &egui::Context, output: &mut egui::FullOutput) {
        let at = Instant::now();
        if self.rows.len() == PASSES || ctx.viewport_id() != egui::ViewportId::ROOT {
            self.invalid += 1;
            return;
        }
        let mut row = [0; 16];
        row[0] = ctx.cumulative_pass_nr();
        row[1] = u64::from(self.stage);
        self.current.clear();
        for shape in &output.shapes {
            self.visit(&shape.shape, &mut row, 0);
        }
        self.current.sort_unstable_by_key(|w| w.as_ptr() as usize);
        self.current.dedup_by(|a, b| a.ptr_eq(b));
        row[4] = self.current.len() as u64;
        let (count, size, fill) = ctx.fonts(|f| {
            (
                f.num_galleys_in_cache(),
                f.font_image_size(),
                f.font_atlas_fill_ratio(),
            )
        });
        row[7] = count as u64;
        row[8] = size[0] as u64;
        row[9] = size[1] as u64;
        row[10] = (fill * 1_000_000.) as u64;
        for (id, deltas) in &output.textures_delta.set {
            for delta in deltas {
                if *id == egui::TextureId::default() {
                    row[if delta.pos.is_none() { 11 } else { 12 }] += 1;
                } else {
                    row[13] += 1;
                }
            }
        }
        std::mem::swap(&mut self.previous, &mut self.current);
        self.current.clear();
        row[15] = at.elapsed().as_nanos() as u64;
        self.rows.push(row);
    }
    fn on_exit(&mut self, _: &egui::Context) {
        self.clear_identities();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn pass(ctx: &egui::Context) -> egui::FullOutput {
        ctx.run_ui(Default::default(), |ui| {
            ui.label("fixed layout");
        })
    }
    #[test]
    fn framework_lookup_reuses_layout_and_rejects_changed_dpi_or_raster_options() {
        use egui::epaint::text::{Fonts, TextOptions, ViewportKey};
        let mut options = TextOptions::default();
        let mut fonts = Fonts::new(options, egui::FontDefinitions::default());
        let key = ViewportKey::default();
        let lookup = |fonts: &mut Fonts, scale| {
            fonts.with_pixels_per_point(scale).layout_no_wrap(
                "fixed layout".into(),
                egui::FontId::proportional(16.),
                egui::Color32::WHITE,
            )
        };
        fonts.begin_pass(options, key);
        let a = lookup(&mut fonts, 1.);
        fonts.begin_pass(options, key);
        let b = lookup(&mut fonts, 1.);
        assert!(std::sync::Arc::ptr_eq(&a, &b));
        let dpi = lookup(&mut fonts, 2.);
        assert!(!std::sync::Arc::ptr_eq(&a, &dpi));
        assert_eq!(dpi.pixels_per_point, 2.);
        options.font_hinting = !options.font_hinting;
        fonts.begin_pass(options, key);
        let changed = lookup(&mut fonts, 1.);
        assert!(!std::sync::Arc::ptr_eq(&a, &changed));
    }
    #[test]
    fn atlas_reset_at_the_same_size_rejects_an_old_layout() {
        use egui::epaint::text::{Fonts, TextOptions, ViewportKey};
        let options = TextOptions {
            max_texture_side: 1024,
            subpixel_binning: false,
            ..Default::default()
        };
        let mut fonts = Fonts::new(options, egui::FontDefinitions::default());
        let key = ViewportKey::default();
        fonts.begin_pass(options, key);
        let lookup = |fonts: &mut Fonts| {
            fonts.with_pixels_per_point(1.).layout_no_wrap(
                "anchor".into(),
                egui::FontId::proportional(16.),
                egui::Color32::WHITE,
            )
        };
        let old = lookup(&mut fonts);
        let fill = |fonts: &mut Fonts| {
            let letters: String = (33..=126).map(char::from).collect();
            for size in (16..=128).step_by(4) {
                fonts.with_pixels_per_point(1.).layout_no_wrap(
                    letters.clone(),
                    egui::FontId::proportional(size as f32),
                    egui::Color32::WHITE,
                );
                if fonts.font_atlas_fill_ratio() > 0.8 {
                    break;
                }
            }
        };
        fill(&mut fonts);
        assert!(fonts.font_atlas_fill_ratio() > 0.8);
        let size = fonts.font_image_size();
        fonts.begin_pass(options, key);
        fonts.with_pixels_per_point(1.).layout_no_wrap(
            "different prefix".into(),
            egui::FontId::proportional(16.),
            egui::Color32::WHITE,
        );
        let fresh = lookup(&mut fonts);
        fill(&mut fonts);
        assert_eq!(fonts.font_image_size(), size);
        assert!(!std::sync::Arc::ptr_eq(&old, &fresh));
        assert_ne!(
            old.rows[0].row.visuals.mesh.vertices[0].uv,
            fresh.rows[0].row.visuals.mesh.vertices[0].uv
        );
        // No output ever renders `old`; it remains only a witness. Size/DPI
        // cannot authorize old atlas coordinates after the framework reset.
    }
    #[test]
    fn survey_is_read_only_reuses_live_identities_and_releases_weak_references() {
        let a = egui::Context::default();
        let b = egui::Context::default();
        b.add_plugin(Census::new());
        for _ in 0..3 {
            let mut plain = pass(&a);
            let mut observed = pass(&b);
            assert_eq!(
                format!("{:?}", plain.shapes),
                format!("{:?}", observed.shapes)
            );
            assert_eq!(
                format!("{:?}", plain.textures_delta),
                format!("{:?}", observed.textures_delta)
            );
            plain.textures_delta.clear();
            observed.textures_delta.clear();
        }
        b.with_plugin::<Census, _>(|c| {
            assert_eq!(c.invalid, 0);
            assert_eq!(c.rows[2][2..4], [1, 1]);
            assert!(c.rows[2][5] > 0);
            c.clear_identities();
            assert!(c.previous.is_empty() && c.current.is_empty());
        });
    }
    #[test]
    fn survey_limits_and_weak_identity_cannot_retain_text_payload() {
        let ctx = egui::Context::default();
        ctx.add_plugin(Census::new());
        let mut frame = pass(&ctx);
        let galley = match &frame.shapes[0].shape {
            Shape::Text(text) => text.galley.clone(),
            _ => panic!("missing text"),
        };
        let strong = std::sync::Arc::strong_count(&galley);
        let mut bounded = Census::new();
        bounded.current.resize(SHAPES, Weak::new());
        bounded.visit(&frame.shapes[0].shape, &mut [0; 16], 0);
        assert_eq!(bounded.invalid, 1);
        assert_eq!(bounded.current.len(), SHAPES);
        ctx.with_plugin::<Census, _>(|c| {
            assert_eq!(std::sync::Arc::strong_count(&galley), strong);
            c.previous.clear();
            c.current.clear();
            assert_eq!(std::sync::Arc::strong_count(&galley), strong);
            c.rows.resize(PASSES, [0; 16]);
        });
        frame.textures_delta.clear();
        let mut last = pass(&ctx);
        last.textures_delta.clear();
        ctx.with_plugin::<Census, _>(|c| {
            assert_eq!(c.rows.len(), PASSES);
            assert_eq!(c.invalid, 1);
        });
    }
    #[test]
    fn identical_live_galley_does_not_authorize_old_geometry_or_color() {
        let ctx = egui::Context::default();
        ctx.add_plugin(Census::new());
        let mut shapes = vec![];
        for (x, color) in [(0., egui::Color32::WHITE), (10., egui::Color32::RED)] {
            let mut frame = ctx.run_ui(Default::default(), |ui| {
                let galley = ui.painter().layout_no_wrap(
                    "same".into(),
                    egui::FontId::proportional(16.),
                    egui::Color32::PLACEHOLDER,
                );
                ui.painter().galley(egui::Pos2::new(x, 0.), galley, color);
            });
            shapes.push(format!("{:?}", frame.shapes));
            frame.textures_delta.clear();
        }
        assert_ne!(shapes[0], shapes[1]);
        ctx.with_plugin::<Census, _>(|c| assert_eq!(c.rows[1][2..4], [1, 1]));
    }
    #[test]
    fn replacing_font_definitions_invalidates_layout_without_a_text_change() {
        let ctx = egui::Context::default();
        ctx.add_plugin(Census::new());
        let mut first = pass(&ctx);
        first.textures_delta.clear();
        let mut fonts = egui::FontDefinitions::default();
        fonts
            .families
            .get_mut(&egui::FontFamily::Proportional)
            .unwrap()
            .reverse();
        assert_ne!(fonts, egui::FontDefinitions::default());
        ctx.set_fonts(fonts);
        let mut next = pass(&ctx);
        next.textures_delta.clear();
        ctx.with_plugin::<Census, _>(|c| {
            assert_eq!(c.rows[1][2..4], [1, 0]);
            assert!(c.rows[1][11] > 0);
        });
    }
}
