//! Shared composition plan. Linear-light, isolated groups, nearest sampling,
//! pixel centres, transparent document canvas clipping; no stable rules.
#![forbid(unsafe_code)]
mod gpu;
mod pixels;
mod upload;
pub use gpu::{GpuComposer, GpuImage, GpuPreparation};
pub use pixels::RasterPixels;
use skin_editor_domain::{
    Affine2, BlobRef, ComponentKind, DocumentId, LayerId, LayerKind, PixelSize, SkinProject,
};
use std::collections::BTreeMap;
use std::sync::Arc;
pub use upload::{MAX_STAGING_BYTES, SourceUpload, StagingUsage};

pub const MAX_DEPTH: usize = 32;
pub const MAX_PIXELS: u64 = 1024 * 1024;
pub const MAX_OPS: usize = 65_536;
#[derive(Clone, Debug)]
pub struct Raster {
    pub size: PixelSize,
    pub pixels: RasterPixels,
}
pub type Images = BTreeMap<BlobRef, Arc<Raster>>;
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Target {
    Document(DocumentId),
    Components,
    Workspace,
}
#[derive(Clone, Debug)]
pub enum Op {
    Push(f32),
    Pop,
    Image {
        blob: BlobRef,
        inverse: Affine2,
        opacity: f32,
        clip_inverse: Affine2,
        canvas: PixelSize,
    },
}
#[derive(Clone, Debug)]
pub struct Plan {
    pub(crate) size: PixelSize,
    pub(crate) ops: Vec<Op>,
    pub view: Affine2,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RenderError(pub String);
fn error(text: &str) -> RenderError {
    RenderError(text.into())
}
fn math(_: skin_editor_domain::ModelError) -> RenderError {
    error("invalid render geometry")
}

impl Plan {
    pub fn build(
        project: &SkinProject,
        target: Target,
        size: PixelSize,
    ) -> Result<Self, RenderError> {
        if u64::from(size.width()) * u64::from(size.height()) > MAX_PIXELS {
            return Err(error("preview pixel budget exceeded"));
        }
        let canvas = match target {
            Target::Document(id) => project
                .document(id)
                .ok_or_else(|| error("missing document"))?
                .canvas(),
            Target::Components | Target::Workspace => PixelSize::new(512, 512).map_err(math)?,
        };
        let scale = (f64::from(size.width()) / f64::from(canvas.width()))
            .min(f64::from(size.height()) / f64::from(canvas.height()));
        let view = Affine2::new([
            scale,
            0.0,
            0.0,
            scale,
            (f64::from(size.width()) - f64::from(canvas.width()) * scale) / 2.0,
            (f64::from(size.height()) - f64::from(canvas.height()) * scale) / 2.0,
        ])
        .map_err(math)?;
        let mut plan = Self {
            size,
            ops: vec![],
            view,
        };
        match target {
            Target::Document(id) => plan.document(
                project,
                id,
                project.document_content_transform(id).map_err(math)?,
                Affine2::IDENTITY,
            )?,
            Target::Components | Target::Workspace => {
                for node in project.components().iter() {
                    if let ComponentKind::Element {
                        document,
                        document_to_component: map,
                    } = node.kind
                    {
                        let world = project
                            .components()
                            .world_transform(node.id)
                            .ok_or_else(|| error("missing component transform"))?;
                        plan.document(project, document, world.compose(map).map_err(math)?, map)?;
                    }
                }
                if matches!(target, Target::Workspace) {
                    for doc in project
                        .documents()
                        .filter(|d| project.components().member(d.id()).is_none())
                    {
                        plan.document(project, doc.id(), Affine2::IDENTITY, Affine2::IDENTITY)?;
                    }
                }
            }
        }
        Ok(plan)
    }
    fn document(
        &mut self,
        project: &SkinProject,
        id: DocumentId,
        content: Affine2,
        clip: Affine2,
    ) -> Result<(), RenderError> {
        let doc = project
            .document(id)
            .ok_or_else(|| error("missing document"))?;
        if !doc.visible() {
            return Ok(());
        }
        let prefix = self.view.compose(content).map_err(math)?;
        let clip_inverse = self
            .view
            .compose(clip)
            .map_err(math)?
            .inverse()
            .map_err(math)?;
        enum Visit {
            Node(LayerId, usize),
            Pop,
        }
        let mut stack: Vec<_> = doc
            .layers()
            .roots()
            .iter()
            .rev()
            .map(|id| Visit::Node(*id, 0))
            .collect();
        while let Some(visit) = stack.pop() {
            if self.ops.len() >= MAX_OPS {
                return Err(error("composition operation budget exceeded"));
            }
            match visit {
                Visit::Pop => self.ops.push(Op::Pop),
                Visit::Node(id, depth) => {
                    let node = doc
                        .layers()
                        .node(id)
                        .ok_or_else(|| error("missing layer"))?;
                    if !node.visible {
                        continue;
                    }
                    match &node.kind {
                        LayerKind::Group { children } => {
                            if depth >= MAX_DEPTH {
                                return Err(error(
                                    "render group depth exceeds 32; model is preserved",
                                ));
                            }
                            self.ops.push(Op::Push(node.opacity.get()));
                            stack.push(Visit::Pop);
                            stack.extend(
                                children.iter().rev().map(|id| Visit::Node(*id, depth + 1)),
                            );
                        }
                        LayerKind::Image { asset } => {
                            let asset = project
                                .assets()
                                .get(*asset)
                                .ok_or_else(|| error("missing asset"))?;
                            let transform = prefix
                                .compose(
                                    doc.layers()
                                        .world_transform(id)
                                        .ok_or_else(|| error("missing layer transform"))?,
                                )
                                .map_err(math)?;
                            // A zero-area transform contributes no pixels on both backends.
                            if let Ok(inverse) = transform.inverse() {
                                self.ops.push(Op::Image {
                                    blob: asset.blob(),
                                    inverse,
                                    opacity: node.opacity.get(),
                                    clip_inverse,
                                    canvas: doc.canvas(),
                                });
                            }
                        }
                    }
                }
            }
        }
        Ok(())
    }
    pub fn sources(&self) -> Vec<BlobRef> {
        let mut result: Vec<_> = self
            .ops
            .iter()
            .filter_map(|op| match op {
                Op::Image { blob, .. } => Some(*blob),
                _ => None,
            })
            .collect();
        result.sort();
        result.dedup();
        result
    }
}
fn linear(c: f32) -> f32 {
    if c <= 0.04045 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}
fn gamma(c: f32) -> f32 {
    if c <= 0.0031308 {
        c * 12.92
    } else {
        1.055 * c.powf(1.0 / 2.4) - 0.055
    }
}
fn over(back: [f32; 4], front: [f32; 4]) -> [f32; 4] {
    std::array::from_fn(|i| front[i] + back[i] * (1.0 - front[3]))
}
fn inside(p: [f64; 2], size: PixelSize) -> bool {
    p[0] >= 0.0 && p[1] >= 0.0 && p[0] < f64::from(size.width()) && p[1] < f64::from(size.height())
}
pub fn compose(plan: &Plan, images: &Images, background: bool) -> Result<Vec<u8>, RenderError> {
    for blob in plan.sources() {
        let image = images
            .get(&blob)
            .ok_or_else(|| error("missing image bytes"))?;
        if image.pixels.len() as u64
            != u64::from(image.size.width()) * u64::from(image.size.height()) * 4
        {
            return Err(error("invalid RGBA length"));
        }
    }
    let lut: [f32; 256] = std::array::from_fn(|i| linear(i as f32 / 255.0));
    let mut output = vec![0; plan.size.width() as usize * plan.size.height() as usize * 4];
    for y in 0..plan.size.height() {
        for x in 0..plan.size.width() {
            let p = [f64::from(x) + 0.5, f64::from(y) + 0.5];
            let mut color = [0.0; 4];
            let mut stack = Vec::new();
            for op in &plan.ops {
                match op {
                    Op::Push(opacity) => {
                        stack.push((color, *opacity));
                        color = [0.0; 4];
                    }
                    Op::Pop => {
                        let (back, opacity) =
                            stack.pop().ok_or_else(|| error("invalid group plan"))?;
                        color = over(back, color.map(|v| v * opacity));
                    }
                    Op::Image {
                        blob,
                        inverse,
                        opacity,
                        clip_inverse,
                        canvas,
                    } => {
                        if !inside(clip_inverse.point(p[0], p[1]).map_err(math)?, *canvas) {
                            continue;
                        }
                        let image = &images[blob];
                        let at = inverse.point(p[0], p[1]).map_err(math)?;
                        if inside(at, image.size) {
                            let i = (at[1].floor() as usize * image.size.width() as usize
                                + at[0].floor() as usize)
                                * 4;
                            let pixel = &image.pixels[i..i + 4];
                            let alpha = pixel[3] as f32 / 255.0 * opacity;
                            color = over(
                                color,
                                [
                                    lut[pixel[0] as usize] * alpha,
                                    lut[pixel[1] as usize] * alpha,
                                    lut[pixel[2] as usize] * alpha,
                                    alpha,
                                ],
                            );
                        }
                    }
                }
            }
            if background {
                let grey = if (x / 16 + y / 16) % 2 == 0 {
                    0.035
                } else {
                    0.055
                };
                color = over([grey, grey, grey, 1.0], color);
            }
            let i = (y as usize * plan.size.width() as usize + x as usize) * 4;
            for c in 0..3 {
                output[i + c] = if color[3] > 0.0 {
                    (gamma((color[c] / color[3]).clamp(0.0, 1.0)) * 255.0).round() as u8
                } else {
                    0
                };
            }
            output[i + 3] = (color[3] * 255.0).round() as u8;
        }
    }
    Ok(output)
}
