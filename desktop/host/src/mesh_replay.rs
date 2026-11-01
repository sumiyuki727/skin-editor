//! Opt-in, same-output CPU replay. No output replacement, GPU upload, callback
//! execution or cross-pass rendering data. All scratch lives inside `capture`.
use eframe::egui::{
    self,
    epaint::{ClippedPrimitive, ClippedShape, Primitive, Shape, Vertex},
};
use std::{hint::black_box, mem::size_of, time::Instant};

const VERTICES: usize = 32768;
const INDICES: usize = 131072;
const NODES: usize = 2048;
const ATLAS_BYTES: usize = 4 * 1024 * 1024;
pub const ORDINALS: [u64; 4] = [1, 33, 65, 97];
pub const PLAN: [usize; 7] = [0, 1, 2, 3, 2, 1, 0]; // all/text/other/empty/other/text/all
const REPEATS: usize = 4;
fn admit_atlas(size: [usize; 2]) -> bool {
    size[0] >= 1024
        && size[1] >= 32
        && size[0]
            .checked_mul(size[1])
            .and_then(|v| v.checked_mul(4))
            .is_some_and(|v| v <= ATLAS_BYTES)
}

#[derive(Default)]
struct Counts {
    nodes: usize,
    texts: usize,
    vertices: usize,
    indices: usize,
}
fn admit(shape: &Shape, depth: usize, c: &mut Counts) -> Result<(), &'static str> {
    c.nodes += 1;
    if depth > 32 || c.nodes > NODES {
        return Err("source shape capacity");
    }
    match shape {
        Shape::Vec(children) => {
            for s in children {
                admit(s, depth + 1, c)?;
            }
        }
        Shape::Text(text) => {
            c.texts += 1;
            for row in &text.galley.rows {
                c.vertices += row.row.visuals.mesh.vertices.len();
                c.indices += row.row.visuals.mesh.indices.len();
            }
        }
        Shape::Mesh(m) => {
            c.vertices += m.vertices.len();
            c.indices += m.indices.len();
        }
        Shape::Path(path) => c.vertices += path.points.len(),
        _ => {}
    }
    if c.texts > 512 || c.vertices > VERTICES || c.indices > INDICES {
        return Err("source mesh capacity");
    }
    Ok(())
}
fn select(shape: &Shape, text: bool) -> Option<Shape> {
    match shape {
        Shape::Vec(children) => {
            let values: Vec<_> = children.iter().filter_map(|s| select(s, text)).collect();
            (!values.is_empty()).then_some(Shape::Vec(values))
        }
        Shape::Text(_) if text => Some(shape.clone()),
        Shape::Text(_) => None,
        _ if !text => Some(shape.clone()),
        _ => None,
    }
}
fn partition(source: &[ClippedShape]) -> Result<([Vec<ClippedShape>; 4], Counts), &'static str> {
    let mut counts = Counts::default();
    for s in source {
        admit(&s.shape, 0, &mut counts)?;
    }
    let selected = |text| {
        source
            .iter()
            .filter_map(|s| {
                select(&s.shape, text).map(|shape| ClippedShape {
                    clip_rect: s.clip_rect,
                    shape,
                    transform_after_tessellation: s.transform_after_tessellation,
                })
            })
            .collect()
    };
    Ok((
        [source.to_vec(), selected(true), selected(false), vec![]],
        counts,
    ))
}
fn counts(primitives: &[ClippedPrimitive]) -> Result<[usize; 3], &'static str> {
    let (mut v, mut i, mut cb) = (0, 0, 0);
    for p in primitives {
        match &p.primitive {
            Primitive::Mesh(m) => {
                v += m.vertices.len();
                i += m.indices.len();
            }
            Primitive::Callback(_) => cb += 1,
        }
        if v > VERTICES || i > INDICES || primitives.len() > NODES {
            return Err("tessellated mesh capacity");
        }
    }
    Ok([v, i, cb])
}
fn equal(a: &[ClippedPrimitive], b: &[ClippedPrimitive]) -> bool {
    a.len() == b.len()
        && a.iter().zip(b).all(|(a, b)| {
            a.clip_rect == b.clip_rect
                && match (&a.primitive, &b.primitive) {
                    (Primitive::Mesh(a), Primitive::Mesh(b)) => a == b,
                    (Primitive::Callback(a), Primitive::Callback(b)) => {
                        a.rect == b.rect && std::sync::Arc::ptr_eq(&a.callback, &b.callback)
                    }
                    _ => false,
                }
        })
}
struct Packed {
    vertices: Vec<Vertex>,
    indices: Vec<u32>,
}
impl Packed {
    fn new(v: usize, i: usize) -> Self {
        Self {
            vertices: Vec::with_capacity(v),
            indices: Vec::with_capacity(i),
        }
    }
    fn copy(&mut self, source: &[ClippedPrimitive]) -> Result<(), &'static str> {
        let [v, i, _] = counts(source)?;
        if v > self.vertices.capacity() || i > self.indices.capacity() {
            return Err("packing capacity");
        }
        self.vertices.clear();
        self.indices.clear();
        for p in source {
            if let Primitive::Mesh(m) = &p.primitive {
                self.vertices.extend_from_slice(&m.vertices);
                self.indices.extend_from_slice(&m.indices);
            }
        }
        black_box((&self.vertices, &self.indices));
        Ok(())
    }
}

pub struct Replay {
    pub stage: u8,
    seen: u64,
    last_pass: Option<u64>,
    rows: Vec<[u64; 17]>,
    samples: Vec<[u64; 11]>,
    invalid: u64,
    scratch_active: bool,
    error: Option<&'static str>,
}
impl Replay {
    pub fn new() -> Self {
        Self {
            stage: 0,
            seen: 0,
            last_pass: None,
            rows: Vec::with_capacity(112),
            samples: Vec::with_capacity(4),
            invalid: 0,
            scratch_active: false,
            error: None,
        }
    }
    fn capture(
        &mut self,
        ctx: &egui::Context,
        output: &egui::FullOutput,
        pass: u64,
    ) -> Result<(), &'static str> {
        let total = Instant::now();
        let (groups, source) = partition(&output.shapes)?;
        let partition_ns = total.elapsed().as_nanos() as u64;
        let at = Instant::now();
        let (size, options) = ctx.fonts(|f| (f.font_image_size(), *f.options()));
        if !admit_atlas(size) {
            return Err("atlas snapshot capacity");
        }
        // Public constructor regenerates ONLY built-in disc metadata. The real
        // glyphs/UVs are from this output. Every result must match Context's
        // actual atlas path exactly; no guessed UV table is accepted.
        let atlas = egui::epaint::TextureAtlas::new(size, options);
        let tess_options = ctx.tessellation_options(|o| *o);
        let ppp = output.pixels_per_point;
        if !ppp.is_finite() || ppp <= 0. {
            return Err("invalid output scale");
        }
        let atlas_ns = at.elapsed().as_nanos() as u64;
        let mut warm = Packed::new(VERTICES, INDICES);
        for repeat in 0..REPEATS {
            for (position, &kind) in PLAN.iter().enumerate() {
                let at = Instant::now();
                let input_core = groups[kind].clone();
                let clone_core = at.elapsed().as_nanos() as u64;
                let at = Instant::now();
                let input_api = groups[kind].clone();
                let clone_api = at.elapsed().as_nanos() as u64;
                let run_core = |input| {
                    let at = Instant::now();
                    let value = egui::epaint::Tessellator::new(
                        ppp,
                        tess_options,
                        size,
                        atlas.prepared_discs(),
                    )
                    .tessellate_shapes(input);
                    (value, at.elapsed().as_nanos() as u64)
                };
                let run_api = |input| {
                    let at = Instant::now();
                    let value = ctx.tessellate(input, ppp);
                    (value, at.elapsed().as_nanos() as u64)
                };
                let ((core, core_ns), (api, api_ns)) = if repeat % 2 == 0 {
                    (run_core(input_core), run_api(input_api))
                } else {
                    let api = run_api(input_api);
                    (run_core(input_core), api)
                };
                if !equal(&core, &api) {
                    return Err("core/framework output mismatch");
                }
                let [v, i, cb] = counts(&api)?;
                let run_cold = || {
                    let at = Instant::now();
                    let mut cold = Packed::new(v, i);
                    cold.copy(&api)?;
                    Ok::<_, &'static str>((cold, at.elapsed().as_nanos() as u64))
                };
                let mut run_warm = || {
                    let at = Instant::now();
                    warm.copy(&api)?;
                    Ok::<_, &'static str>(at.elapsed().as_nanos() as u64)
                };
                let ((cold, cold_ns), warm_ns) = if repeat % 2 == 0 {
                    (run_cold()?, run_warm()?)
                } else {
                    let ns = run_warm()?;
                    (run_cold()?, ns)
                };
                if warm.vertices != cold.vertices || warm.indices != cold.indices {
                    return Err("packing mismatch");
                }
                black_box((&core, &api));
                let primitive_count = api.len() as u64;
                let at = Instant::now();
                drop(cold);
                drop(api);
                drop(core);
                let release_ns = at.elapsed().as_nanos() as u64;
                self.rows.push([
                    pass,
                    self.samples.len() as u64,
                    repeat as u64,
                    position as u64,
                    kind as u64,
                    clone_core,
                    core_ns,
                    clone_api,
                    api_ns,
                    cold_ns,
                    warm_ns,
                    v as u64,
                    i as u64,
                    primitive_count,
                    cb as u64,
                    (v * size_of::<Vertex>() + i * size_of::<u32>()) as u64,
                    release_ns,
                ]);
            }
        }
        if ctx.cumulative_pass_nr() != pass
            || ctx.fonts(|f| (f.font_image_size(), *f.options())) != (size, options)
            || ctx.tessellation_options(|o| *o) != tess_options
        {
            return Err("context changed during replay");
        }
        drop(warm);
        drop(groups);
        drop(atlas);
        self.samples.push([
            pass,
            self.seen,
            output.shapes.len() as u64,
            partition_ns,
            atlas_ns,
            total.elapsed().as_nanos() as u64,
            u64::from(ppp.to_bits()),
            size[0] as u64,
            size[1] as u64,
            source.nodes as u64,
            source.texts as u64,
        ]);
        Ok(())
    }
    pub fn report(&self) -> String {
        format!(
            "{{\"schema\":1,\"vertex_stride\":{},\"invalid\":{},\"scratch_active\":{},\"error\":{},\"samples\":{:?},\"rows\":{:?}}}",
            size_of::<Vertex>(),
            self.invalid,
            self.scratch_active,
            self.error
                .map_or_else(|| "null".into(), |e| format!("{e:?}")),
            self.samples,
            self.rows
        )
    }
}

impl egui::plugin::Plugin for Replay {
    fn debug_name(&self) -> &'static str {
        "same-output-mesh-replay"
    }
    fn output_hook(&mut self, ctx: &egui::Context, output: &mut egui::FullOutput) {
        if self.stage != 4 || self.samples.len() == ORDINALS.len() || self.invalid != 0 {
            return;
        }
        let pass = ctx.cumulative_pass_nr();
        if self.last_pass.is_some_and(|p| pass <= p) || ctx.viewport_id() != egui::ViewportId::ROOT
        {
            self.invalid += 1;
            return;
        }
        self.last_pass = Some(pass);
        self.seen += 1;
        if !ORDINALS.contains(&self.seen) {
            return;
        }
        self.scratch_active = true;
        if let Err(error) = self.capture(ctx, output, pass) {
            self.invalid += 1;
            self.error = Some(error);
        }
        self.scratch_active = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn frame(ctx: &egui::Context) -> egui::FullOutput {
        ctx.run_ui(Default::default(), |ui| {
            ui.label("fresh text 123");
            ui.painter()
                .circle_filled(egui::Pos2::new(60., 60.), 4., egui::Color32::RED);
            ui.painter().rect_filled(
                egui::Rect::from_min_size(egui::Pos2::new(20., 80.), egui::vec2(40., 20.)),
                3.,
                egui::Color32::BLUE,
            );
        })
    }
    #[test]
    fn partition_preserves_nested_clip_transform_and_relative_order() {
        let ctx = egui::Context::default();
        let mut output = frame(&ctx);
        let text = output.shapes[0].shape.clone();
        let other = output.shapes[1].shape.clone();
        let clip = egui::Rect::from_min_max(egui::pos2(4., 5.), egui::pos2(90., 100.));
        let mut shape = ClippedShape::new(
            clip,
            Shape::Vec(vec![
                other.clone(),
                Shape::Vec(vec![text.clone(), other.clone()]),
                text.clone(),
            ]),
        );
        shape.transform_after_tessellation = egui::emath::TSTransform::new(egui::vec2(2., 3.), 1.5);
        let (groups, c) = partition(std::slice::from_ref(&shape)).unwrap();
        assert_eq!(groups[0], [shape.clone()]);
        assert_eq!(c.texts, 2);
        assert_eq!(groups[1][0].clip_rect, clip);
        assert_eq!(
            groups[1][0].transform_after_tessellation,
            shape.transform_after_tessellation
        );
        assert_eq!(
            groups[1][0].shape,
            Shape::Vec(vec![Shape::Vec(vec![text.clone()]), text])
        );
        assert_eq!(
            groups[2][0].shape,
            Shape::Vec(vec![other.clone(), Shape::Vec(vec![other])])
        );
        output.textures_delta.clear();
    }
    #[test]
    fn same_output_replay_matches_framework_and_keeps_shapes_and_texture_updates() {
        let plain = egui::Context::default();
        let observed = egui::Context::default();
        observed.add_plugin(Replay::new());
        observed.with_plugin::<Replay, _>(|r| r.stage = 4);
        for _ in 0..100 {
            let mut a = frame(&plain);
            let mut b = frame(&observed);
            assert_eq!(a.shapes, b.shapes);
            assert_eq!(
                format!("{:?}", a.textures_delta),
                format!("{:?}", b.textures_delta)
            );
            a.textures_delta.clear();
            b.textures_delta.clear();
        }
        observed.with_plugin::<Replay, _>(|r| {
            assert_eq!(r.invalid, 0, "{:?}", r.error);
            assert_eq!(r.samples.iter().map(|r| r[1]).collect::<Vec<_>>(), ORDINALS);
            assert_eq!(r.rows.len(), 112);
            assert!(!r.scratch_active);
            for sample in r.rows.chunks_exact(28) {
                for repeat in sample.chunks_exact(7) {
                    assert_eq!(
                        repeat.iter().map(|r| r[4] as usize).collect::<Vec<_>>(),
                        PLAN
                    );
                    assert_eq!(repeat[0][11..16], repeat[6][11..16]);
                    assert_eq!(repeat[3][11..16], [0; 5]);
                }
            }
        });
    }
    #[test]
    fn packing_keeps_mesh_local_indices_and_reuses_only_capacity() {
        let a = egui::epaint::Mesh {
            vertices: vec![Vertex::default(); 3],
            indices: vec![0, 1, 2],
            ..Default::default()
        };
        let mut b = a.clone();
        b.vertices[0].pos = egui::pos2(9., 10.);
        let source = vec![
            ClippedPrimitive {
                clip_rect: egui::Rect::EVERYTHING,
                primitive: Primitive::Mesh(a.clone()),
            },
            ClippedPrimitive {
                clip_rect: egui::Rect::EVERYTHING,
                primitive: Primitive::Mesh(b.clone()),
            },
        ];
        let mut packed = Packed::new(6, 6);
        let capacities = (packed.vertices.capacity(), packed.indices.capacity());
        packed.copy(&source).unwrap();
        assert_eq!(packed.indices, [0, 1, 2, 0, 1, 2]);
        assert_eq!(&packed.vertices[..3], &a.vertices);
        assert_eq!(&packed.vertices[3..], &b.vertices);
        packed.copy(&[]).unwrap();
        assert!(packed.vertices.is_empty() && packed.indices.is_empty());
        assert_eq!(
            (packed.vertices.capacity(), packed.indices.capacity()),
            capacities
        );
        let mut small = Packed::new(0, 0);
        assert!(small.copy(&source).is_err());
        assert_eq!(small.vertices.capacity(), 0);
    }
    #[test]
    fn admission_rejects_deep_or_large_sources_and_atlas_before_copy() {
        assert!(admit_atlas([1024, 1024]));
        for size in [[512, 32], [8192, 512], [usize::MAX, usize::MAX]] {
            assert!(!admit_atlas(size));
        }
        let mut s = Shape::Noop;
        for _ in 0..34 {
            s = Shape::Vec(vec![s]);
        }
        assert!(partition(&[ClippedShape::new(egui::Rect::EVERYTHING, s)]).is_err());
        let source = vec![ClippedShape::new(egui::Rect::EVERYTHING, Shape::Noop); NODES + 1];
        assert!(partition(&source).is_err());
    }
    #[test]
    fn callbacks_are_forwarded_without_invocation_or_retention() {
        let ctx = egui::Context::default();
        let mut output = frame(&ctx);
        let marker = std::sync::Arc::new(std::sync::atomic::AtomicU64::new(0));
        output.shapes.push(ClippedShape::new(
            egui::Rect::EVERYTHING,
            Shape::Callback(egui::PaintCallback {
                rect: egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(10., 10.)),
                callback: marker.clone(),
            }),
        ));
        let strong = std::sync::Arc::strong_count(&marker);
        let mut replay = Replay::new();
        replay.seen = 1;
        replay
            .capture(&ctx, &output, ctx.cumulative_pass_nr())
            .unwrap();
        assert_eq!(std::sync::Arc::strong_count(&marker), strong);
        assert_eq!(marker.load(std::sync::atomic::Ordering::Relaxed), 0);
        assert_eq!(replay.rows[0][14], 1);
        output.textures_delta.clear();
    }
    #[test]
    fn fractional_scale_clip_opacity_and_transform_match_fresh_framework_output() {
        for scale in [1., 1.5, 2.] {
            let ctx = egui::Context::default();
            ctx.set_pixels_per_point(scale);
            let mut prime = frame(&ctx);
            prime.textures_delta.clear();
            let mut output = frame(&ctx);
            for s in &mut output.shapes {
                s.clip_rect = egui::Rect::from_min_max(egui::pos2(3., 4.), egui::pos2(90., 95.));
                s.transform_after_tessellation =
                    egui::emath::TSTransform::new(egui::vec2(1.25, 2.5), 1.1);
                if let Shape::Text(text) = &mut s.shape {
                    text.opacity_factor = 0.6;
                    text.angle = 0.12;
                }
            }
            let before = output.shapes.clone();
            let mut replay = Replay::new();
            replay.seen = 1;
            replay
                .capture(&ctx, &output, ctx.cumulative_pass_nr())
                .unwrap();
            assert_eq!(output.shapes, before);
            assert_eq!(replay.rows.len(), 28);
            output.textures_delta.clear();
        }
    }
}
