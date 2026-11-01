//! Temporary visual selection. No component creation, reparenting or project writer.
use super::*;

pub const MAX_SELECTED: usize = 256;
pub(super) type Stamp = (
    HistorySessionId,
    ExpectedState,
    Option<DraftState>,
    Option<PreviewTarget>,
);

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct Geometry {
    pub low: [f64; 2],
    pub high: [f64; 2],
    pub world: Affine2,
}
impl Geometry {
    pub fn single(project: &SkinProject, item: Selection, target: PreviewTarget) -> Option<Self> {
        let (bounds, mut world) = gestures::geometry(project, item)?;
        if let Selection::Layer(d, l) = item
            && matches!(target, PreviewTarget::Components | PreviewTarget::Workspace)
        {
            world = project
                .document_workspace_transform(d)
                .ok()?
                .compose(project.document(d)?.layers().world_transform(l)?)
                .ok()?;
        }
        Some(Self {
            low: bounds.min(),
            high: bounds.max(),
            world,
        })
    }
    fn points(self) -> [[f64; 2]; 4] {
        [
            [self.low[0], self.low[1]],
            [self.high[0], self.low[1]],
            [self.high[0], self.high[1]],
            [self.low[0], self.high[1]],
        ]
    }
    pub fn controls(self, view: PresentedView, canvas: Rect) -> Option<gestures::ControlBox> {
        let map = view.view.compose(self.world).ok()?;
        let mut corners = [Pos2::ZERO; 4];
        for (i, p) in self.points().into_iter().enumerate() {
            let p = map.point(p[0], p[1]).ok()?;
            corners[i] = canvas.min
                + Vec2::new(
                    p[0] as f32 / view.frame.source.context.size[0] as f32 * canvas.width(),
                    p[1] as f32 / view.frame.source.context.size[1] as f32 * canvas.height(),
                );
            if !corners[i].x.is_finite() || !corners[i].y.is_finite() {
                return None;
            }
        }
        Some(gestures::ControlBox { corners })
    }
    pub fn joint(
        project: &SkinProject,
        items: &[Selection],
        target: PreviewTarget,
    ) -> Option<Self> {
        if items.len() == 1 {
            return Self::single(project, items[0], target);
        }
        let basis = items
            .iter()
            .find_map(|item| Self::single(project, *item, target))?
            .world;
        let (world, inverse) = basis
            .inverse()
            .map_or((Affine2::IDENTITY, Affine2::IDENTITY), |inv| (basis, inv));
        let mut low = [f64::INFINITY; 2];
        let mut high = [f64::NEG_INFINITY; 2];
        let mut any = false;
        for item in items {
            // Empty groups contribute no bounds, but remain transform targets.
            let Some(g) = Self::single(project, *item, target) else {
                continue;
            };
            let map = inverse.compose(g.world).ok()?;
            for p in g.points() {
                let p = map.point(p[0], p[1]).ok()?;
                for i in 0..2 {
                    low[i] = low[i].min(p[i]);
                    high[i] = high[i].max(p[i]);
                }
            }
            any = true;
        }
        any.then_some(Self { low, high, world })
    }
}

#[derive(Clone, Debug)]
pub(super) struct Frame {
    pub stamp: Stamp,
    pub geometry: Geometry,
}

/// Resolve selected ancestors once per item. Work is bounded by selection
/// count and ancestor walks, rather than pairwise walks for every selected pair.
pub(super) fn effective(project: &SkinProject, items: &[Selection]) -> Vec<Selection> {
    let selected: std::collections::BTreeSet<_> = items.iter().copied().collect();
    items
        .iter()
        .copied()
        .filter(|item| {
            let mut component = match *item {
                Selection::Layer(d, l) => {
                    let Some(doc) = project.document(d) else {
                        return false;
                    };
                    let mut parent = doc.layers().parent(l).flatten();
                    while let Some(p) = parent {
                        if selected.contains(&Selection::Layer(d, p)) {
                            return false;
                        }
                        parent = doc.layers().parent(p).flatten();
                    }
                    project.components().member(d)
                }
                Selection::Component(c) => project.components().parent(c).flatten(),
            };
            while let Some(c) = component {
                if selected.contains(&Selection::Component(c)) {
                    return false;
                }
                component = project.components().parent(c).flatten();
            }
            true
        })
        .collect()
}
fn parent(project: &SkinProject, item: Selection, target: PreviewTarget) -> Option<Affine2> {
    match item {
        Selection::Layer(d, l) => {
            let tree = project.document(d)?.layers();
            let world = tree
                .parent(l)?
                .map_or(Some(Affine2::IDENTITY), |p| tree.world_transform(p))?;
            let prefix = if matches!(target, PreviewTarget::Components | PreviewTarget::Workspace) {
                project.document_workspace_transform(d).ok()?
            } else {
                project.document_content_transform(d).ok()?
            };
            prefix.compose(world).ok()
        }
        Selection::Component(c) => project
            .components()
            .parent(c)?
            .map_or(Some(Affine2::IDENTITY), |p| {
                project.components().world_transform(p)
            }),
    }
}

#[derive(Debug)]
struct Member {
    item: Selection,
    base: Affine2,
    parent: Affine2,
    inverse: Affine2,
}
#[derive(Debug)]
pub(super) struct EditGesture {
    control: gestures::Gesture,
    members: Vec<Member>,
    pub before: Geometry,
    pub after: Geometry,
}
impl EditGesture {
    pub fn new(
        project: &SkinProject,
        items: &[Selection],
        geometry: Geometry,
        view: PresentedView,
        ratio: [f64; 2],
        press: [f64; 2],
        handle: gestures::Handle,
    ) -> Option<Self> {
        if items.is_empty() || items.len() > MAX_SELECTED {
            return None;
        }
        let mut members = Vec::with_capacity(items.len());
        for item in effective(project, items) {
            let mapping = view
                .view
                .compose(parent(project, item, view.frame.source.context.target)?)
                .ok()?;
            members.push(Member {
                item,
                base: EditorUi::transform(project, item)?,
                inverse: mapping.inverse().ok()?,
                parent: mapping,
            });
        }
        Some(Self {
            control: gestures::Gesture::new(geometry.world, view.view, ratio, press, handle)?,
            members,
            before: geometry,
            after: geometry,
        })
    }
    /// All values are prepared before application receives the atomic replace.
    /// Pure calculation cannot advance the remembered frame on a failed replace.
    pub fn values(
        &self,
        pointer: Pos2,
        origin: Pos2,
        preserve: bool,
    ) -> Option<(Vec<(Selection, Affine2)>, Geometry)> {
        let delta = self.control.frame_delta_at(pointer, origin, preserve)?;
        let mut values = Vec::with_capacity(self.members.len());
        for m in &self.members {
            let transform = if delta == Affine2::IDENTITY {
                m.base
            } else {
                m.inverse
                    .compose(delta)
                    .ok()?
                    .compose(m.parent)
                    .ok()?
                    .compose(m.base)
                    .ok()?
            };
            values.push((m.item, transform));
        }
        let mut geometry = self.before;
        geometry.world = self.control.transform_at(pointer, origin, preserve)?;
        Some((values, geometry))
    }
}

impl EditorUi {
    pub fn selection(&self) -> Option<Selection> {
        self.selected.last().copied()
    }
    pub fn selections(&self) -> &[Selection] {
        &self.selected
    }
    /// UI selection input, not a project edit. Same path as Ctrl/Command click.
    pub fn toggle_selection(&mut self, session: &mut ProjectSession, item: Selection) {
        self.pick(session, item, true);
    }
    pub fn select(&mut self, session: &mut ProjectSession, item: Option<Selection>) {
        if self.interaction && item.is_some() {
            return;
        }
        self.cancel_owned(session);
        self.selected.clear();
        self.selected.extend(item);
        self.selection_frame = None;
    }
    pub(super) fn pick(&mut self, session: &mut ProjectSession, item: Selection, additive: bool) {
        if self.interaction {
            return;
        }
        if Self::transform(&Self::project(session), item).is_none() {
            self.message = "选中对象不存在".into();
            return;
        }
        self.cancel_owned(session);
        if !additive {
            self.selected.clear();
        }
        if let Some(i) = self.selected.iter().position(|s| *s == item) {
            self.selected.remove(i);
        } else if self.selected.len() < MAX_SELECTED {
            self.selected.push(item);
        } else {
            self.message = format!("最多选择{MAX_SELECTED}项");
        }
        self.selection_frame = None;
        let layer_document = self.selected.first().and_then(|first| match first {
            Selection::Layer(d, _)
                if self
                    .selected
                    .iter()
                    .all(|s| matches!(s,Selection::Layer(e,_) if d==e)) =>
            {
                Some(*d)
            }
            _ => None,
        });
        self.target =
            Some(layer_document.map_or(PreviewTarget::Workspace, PreviewTarget::Document));
    }
    fn selection_stamp(&self, session: &ProjectSession) -> Stamp {
        let info = session.project_info();
        (
            info.session_id,
            info.state,
            Self::draft_state(session),
            self.target,
        )
    }
    pub(super) fn selection_geometry(
        &mut self,
        session: &ProjectSession,
        project: &SkinProject,
    ) -> Option<Geometry> {
        if self.interaction {
            return None;
        }
        let stamp = self.selection_stamp(session);
        if let Some(frame) = &self.selection_frame
            && frame.stamp == stamp
        {
            return Some(frame.geometry);
        }
        let items = effective(project, &self.selected);
        let geometry = Geometry::joint(project, &items, self.target?)?;
        self.selection_frame = Some(Frame { stamp, geometry });
        Some(geometry)
    }
    pub(super) fn remember_frame(&mut self, session: &ProjectSession, geometry: Geometry) {
        self.selection_frame = Some(Frame {
            stamp: self.selection_stamp(session),
            geometry,
        });
    }
}
