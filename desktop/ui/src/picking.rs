//! Editor rectangle picking, separate from game GUI hit regions and alpha masks.
use super::*;

fn inside(p: [f64; 2], size: PixelSize) -> bool {
    p[0] >= 0. && p[1] >= 0. && p[0] < f64::from(size.width()) && p[1] < f64::from(size.height())
}
fn document(
    project: &SkinProject,
    id: DocumentId,
    content: Affine2,
    clip: Affine2,
    point: [f64; 2],
) -> Option<Selection> {
    let doc = project.document(id)?;
    if !doc.visible()
        || !inside(
            clip.inverse().ok()?.point(point[0], point[1]).ok()?,
            doc.canvas(),
        )
    {
        return None;
    }
    let tree = doc.layers();
    let mut hit = None;
    let mut stack: Vec<_> = tree.roots().iter().rev().copied().collect();
    while let Some(id) = stack.pop() {
        let node = tree.node(id)?;
        if !node.visible || node.opacity.get() == 0. {
            continue;
        }
        match &node.kind {
            LayerKind::Group { children } => stack.extend(children.iter().rev().copied()),
            LayerKind::Image { asset } => {
                let AssetKind::RasterImage(size) = project.assets().get(*asset)?.kind() else {
                    continue;
                };
                if let Ok(inverse) = content.compose(tree.world_transform(id)?).ok()?.inverse()
                    && let Ok(at) = inverse.point(point[0], point[1])
                    && inside(at, *size)
                {
                    hit = Some(Selection::Layer(doc.id(), id));
                }
            }
        }
    }
    hit
}
pub(super) fn layer_at(
    project: &SkinProject,
    view: PresentedView,
    canvas: Rect,
    pointer: Pos2,
) -> Option<Selection> {
    if !canvas.contains(pointer) {
        return None;
    }
    let frame = view.frame.source.context;
    let p = [
        (pointer.x - canvas.left()) as f64 / canvas.width() as f64 * frame.size[0] as f64,
        (pointer.y - canvas.top()) as f64 / canvas.height() as f64 * frame.size[1] as f64,
    ];
    let p = view.view.inverse().ok()?.point(p[0], p[1]).ok()?;
    match frame.target {
        PreviewTarget::Document(d) => document(
            project,
            d,
            project.document_content_transform(d).ok()?,
            Affine2::IDENTITY,
            p,
        ),
        PreviewTarget::Components | PreviewTarget::Workspace => {
            let mut hit = None;
            for node in project.components().iter() {
                if let ComponentKind::Element {
                    document: d,
                    document_to_component: map,
                } = node.kind
                {
                    let content = project
                        .components()
                        .world_transform(node.id)?
                        .compose(map)
                        .ok()?;
                    hit = document(project, d, content, map, p).or(hit);
                }
            }
            if frame.target == PreviewTarget::Workspace {
                for doc in project
                    .documents()
                    .filter(|d| project.components().member(d.id()).is_none())
                {
                    hit = document(project, doc.id(), Affine2::IDENTITY, Affine2::IDENTITY, p)
                        .or(hit);
                }
            }
            hit
        }
    }
}
