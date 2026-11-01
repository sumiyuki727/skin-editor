//! Editor transform controls, in frame pixels; unrelated to game hit regions.
use super::*;

#[derive(Debug)]
pub(super) enum Handle {
    Move,
    Scale([f64; 2]),
    Rotate([f64; 2]),
}
#[derive(Debug)]
pub(super) struct Gesture {
    base: Affine2,
    mapping: Affine2,
    inverse: Affine2,
    ratio: [f64; 2],
    press: [f64; 2],
    handle: Handle,
}
impl Gesture {
    pub fn new(
        base: Affine2,
        mapping: Affine2,
        ratio: [f64; 2],
        press: [f64; 2],
        handle: Handle,
    ) -> Option<Self> {
        match handle {
            Handle::Scale(anchor) => {
                let axes = mapping.compose(base).ok()?.inverse().ok()?;
                let p = axes.point(press[0], press[1]).ok()?;
                let a = axes.point(anchor[0], anchor[1]).ok()?;
                if (p[0] - a[0]).abs() < 1e-6 || (p[1] - a[1]).abs() < 1e-6 {
                    return None;
                }
            }
            Handle::Rotate(center) if (press[0] - center[0]).hypot(press[1] - center[1]) < 1e-6 => {
                return None;
            }
            _ => {}
        }
        Some(Self {
            base,
            inverse: mapping.inverse().ok()?,
            mapping,
            ratio,
            press,
            handle,
        })
    }
    pub fn transform_at(
        &self,
        pointer: Pos2,
        canvas_origin: Pos2,
        preserve: bool,
    ) -> Option<Affine2> {
        // Keep the press origin after egui ends the drag on a release frame.
        // Pointer positions and the canvas origin use the same logical space.
        self.transform_with_aspect(self.pointer_delta(pointer, canvas_origin), preserve)
    }
    fn pointer_delta(&self, pointer: Pos2, canvas_origin: Pos2) -> Vec2 {
        let press = Vec2::new(
            (self.press[0] / self.ratio[0]) as f32,
            (self.press[1] / self.ratio[1]) as f32,
        );
        (pointer - canvas_origin) - press
    }
    pub fn frame_delta_at(&self, pointer: Pos2, origin: Pos2, preserve: bool) -> Option<Affine2> {
        self.frame_delta(self.pointer_delta(pointer, origin), preserve)
    }
    #[cfg(test)]
    pub fn transform(&self, delta: Vec2) -> Option<Affine2> {
        self.transform_with_aspect(delta, false)
    }
    fn transform_with_aspect(&self, delta: Vec2, preserve: bool) -> Option<Affine2> {
        let change = self.frame_delta(delta, preserve)?;
        if change == Affine2::IDENTITY {
            return Some(self.base);
        }
        self.inverse
            .compose(change)
            .ok()?
            .compose(self.mapping)
            .ok()?
            .compose(self.base)
            .ok()
    }
    fn frame_delta(&self, delta: Vec2, preserve: bool) -> Option<Affine2> {
        if delta == Vec2::ZERO {
            return Some(Affine2::IDENTITY);
        }
        let delta = [
            f64::from(delta.x) * self.ratio[0],
            f64::from(delta.y) * self.ratio[1],
        ];
        let around = |anchor: [f64; 2], a: f64, b: f64, c: f64, d: f64| {
            Affine2::new([
                a,
                b,
                c,
                d,
                anchor[0] - a * anchor[0] - c * anchor[1],
                anchor[1] - b * anchor[0] - d * anchor[1],
            ])
            .ok()
        };
        let change = match self.handle {
            Handle::Move => Affine2::new([1., 0., 0., 1., delta[0], delta[1]]).ok()?,
            Handle::Scale(anchor) => {
                // Scale along the selected node's own axes, including a rotated,
                // reflected or sheared parent. The opposite corner stays fixed.
                let axes = self.mapping.compose(self.base).ok()?;
                let inverse = axes.inverse().ok()?;
                let p = inverse.point(self.press[0], self.press[1]).ok()?;
                let a = inverse.point(anchor[0], anchor[1]).ok()?;
                let n = inverse
                    .point(self.press[0] + delta[0], self.press[1] + delta[1])
                    .ok()?;
                let old = [p[0] - a[0], p[1] - a[1]];
                let next = [n[0] - a[0], n[1] - a[1]];
                let (sx, sy) = if preserve {
                    let s =
                        (next[0] * old[0] + next[1] * old[1]) / (old[0] * old[0] + old[1] * old[1]);
                    (s, s)
                } else {
                    (next[0] / old[0], next[1] / old[1])
                };
                if sx.abs() < 1e-6 || sy.abs() < 1e-6 {
                    return None;
                }
                axes.compose(around(a, sx, 0., 0., sy)?)
                    .ok()?
                    .compose(inverse)
                    .ok()?
            }
            Handle::Rotate(center) => {
                let old = [self.press[0] - center[0], self.press[1] - center[1]];
                let next = [old[0] + delta[0], old[1] + delta[1]];
                if old[0].hypot(old[1]) < 1e-6 || next[0].hypot(next[1]) < 1e-6 {
                    return None;
                }
                let angle = next[1].atan2(next[0]) - old[1].atan2(old[0]);
                around(center, angle.cos(), angle.sin(), -angle.sin(), angle.cos())?
            }
        };
        Some(change)
    }
}

/// Geometry for editor controls only. Union descendants in the selected node's
/// local space BEFORE applying its world transform; a world AABB loses rotation.
pub(super) fn geometry(project: &SkinProject, selection: Selection) -> Option<(Bounds, Affine2)> {
    let mut bounds: Option<Bounds> = None;
    let mut add = |size, transform| -> Option<()> {
        let b = Bounds::transformed(size, transform).ok()?;
        bounds = Some(bounds.map_or(b, |old| old.union(b)));
        Some(())
    };
    let world = match selection {
        Selection::Layer(document, layer) => {
            let tree = project.document(document)?.layers();
            let mut stack = vec![(layer, Affine2::IDENTITY)];
            while let Some((id, relative)) = stack.pop() {
                let node = tree.node(id)?;
                if let LayerKind::Image { asset } = node.kind {
                    let AssetKind::RasterImage(size) = project.assets().get(asset)?.kind() else {
                        return None;
                    };
                    add(*size, relative)?;
                }
                for child in node.children() {
                    stack.push((*child, relative.compose(tree.node(*child)?.local).ok()?));
                }
            }
            project
                .document_content_transform(document)
                .ok()?
                .compose(tree.world_transform(layer)?)
                .ok()?
        }
        Selection::Component(component) => {
            let tree = project.components();
            let mut stack = vec![(component, Affine2::IDENTITY)];
            while let Some((id, relative)) = stack.pop() {
                let node = tree.node(id)?;
                if let ComponentKind::Element {
                    document,
                    document_to_component,
                } = node.kind
                {
                    add(
                        project.document(document)?.canvas(),
                        relative.compose(document_to_component).ok()?,
                    )?;
                }
                for child in node.children() {
                    stack.push((*child, relative.compose(tree.node(*child)?.local).ok()?));
                }
            }
            tree.world_transform(component)?
        }
    };
    Some((bounds?, world))
}

#[derive(Clone, Copy, Debug)]
pub(super) struct ControlBox {
    /// Local TL, TR, BR, BL mapped to canvas logical points; not a hit region.
    pub corners: [Pos2; 4],
}
impl From<Rect> for ControlBox {
    fn from(r: Rect) -> Self {
        Self {
            corners: [
                r.left_top(),
                r.right_top(),
                r.right_bottom(),
                r.left_bottom(),
            ],
        }
    }
}
impl ControlBox {
    pub fn contains(self, p: Pos2) -> bool {
        let mut positive = false;
        let mut negative = false;
        for i in 0..4 {
            let edge = self.corners[(i + 1) % 4] - self.corners[i];
            let v = p - self.corners[i];
            let cross = edge.x * v.y - edge.y * v.x;
            positive |= cross > 1e-4;
            negative |= cross < -1e-4;
        }
        !(positive && negative) && Rect::from_points(&self.corners).contains(p)
    }
    pub fn center(self) -> Pos2 {
        self.corners[0].lerp(self.corners[2], 0.5)
    }
    pub fn top(self) -> Pos2 {
        self.corners[0].lerp(self.corners[1], 0.5)
    }
}
pub(super) fn hit(
    bounds: Option<ControlBox>,
    canvas: Rect,
    press: Pos2,
    to_frame: impl Fn(Pos2) -> [f64; 2],
) -> Handle {
    if let Some(b) = bounds {
        for i in 0..4 {
            let (corner, anchor) = (b.corners[i], b.corners[(i + 2) % 4]);
            if corner.distance(press) <= 9. {
                return Handle::Scale(to_frame(anchor));
            }
        }
        if rotation(b, canvas).distance(press) <= 10. {
            return Handle::Rotate(to_frame(b.center()));
        }
    }
    Handle::Move
}
pub(super) fn rotation(bounds: impl Into<ControlBox>, canvas: Rect) -> Pos2 {
    // A component can fill its document canvas. Keep the control hittable even
    // when there is no margin above its bounds; never borrow the scene toolbar.
    let bounds = bounds.into();
    let top = bounds.top();
    let direction = (top - bounds.center()).normalized();
    let p = top + direction * 24.;
    let inside = canvas.shrink(10.);
    Pos2::new(
        p.x.clamp(inside.left(), inside.right()),
        p.y.clamp(inside.top(), inside.bottom()),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    fn near(a: [f64; 2], b: [f64; 2]) {
        for i in 0..2 {
            assert!((a[i] - b[i]).abs() < 1e-8, "{a:?} != {b:?}");
        }
    }
    #[test]
    fn corner_scaling_keeps_the_opposite_anchor_under_a_transformed_parent() {
        let parent = Affine2::new([0., 2., -3., 0., 40., 50.]).unwrap();
        let base = Affine2::new([1., 0., 0., 1., 5., 7.]).unwrap();
        let anchor = parent.point(5., 7.).unwrap();
        let press = parent.point(15., 17.).unwrap();
        let g = Gesture::new(base, parent, [2., 2.], press, Handle::Scale(anchor)).unwrap();
        assert_eq!(g.transform(Vec2::ZERO), Some(base));
        let after = g.transform(Vec2::new(-15., 10.)).unwrap();
        near(
            parent.compose(after).unwrap().point(0., 0.).unwrap(),
            anchor,
        );
        near(
            parent.compose(after).unwrap().point(10., 10.).unwrap(),
            [press[0] - 30., press[1] + 20.],
        );
    }
    #[test]
    fn rotation_preserves_center_and_unstable_zero_vectors_are_rejected() {
        let g = Gesture::new(
            Affine2::IDENTITY,
            Affine2::IDENTITY,
            [1., 1.],
            [10., 0.],
            Handle::Rotate([0., 0.]),
        )
        .unwrap();
        let after = g.transform(Vec2::new(-10., 10.)).unwrap();
        near(after.point(0., 0.).unwrap(), [0., 0.]);
        near(after.point(10., 0.).unwrap(), [0., 10.]);
        assert!(g.transform(Vec2::new(-10., 0.)).is_none());
        assert!(
            Gesture::new(
                Affine2::IDENTITY,
                Affine2::IDENTITY,
                [1., 1.],
                [0., 0.],
                Handle::Scale([0., 0.])
            )
            .is_none()
        );
    }
    #[test]
    fn proportional_scaling_preserves_local_ratio_under_reflection_and_shear() {
        for parent in [
            Affine2::IDENTITY,
            Affine2::new([2., 0.3, 1., -3., 40., 50.]).unwrap(),
        ] {
            let a = std::f64::consts::FRAC_PI_4;
            let base = Affine2::new([a.cos(), a.sin(), -a.sin(), a.cos(), 5., 7.]).unwrap();
            let axes = parent.compose(base).unwrap();
            let anchor = axes.point(4., 7.).unwrap();
            let press = axes.point(14., 27.).unwrap();
            let next = axes.point(24., 37.).unwrap();
            let g = Gesture::new(base, parent, [1., 1.], press, Handle::Scale(anchor)).unwrap();
            let delta = Vec2::new((next[0] - press[0]) as f32, (next[1] - press[1]) as f32);
            let free = g.transform(delta).unwrap();
            let free_relative = base
                .inverse()
                .unwrap()
                .compose(free)
                .unwrap()
                .coefficients();
            assert!((free_relative[0] - 2.).abs() < 1e-6 && (free_relative[3] - 1.5).abs() < 1e-6);
            let locked = g.transform_with_aspect(delta, true).unwrap();
            let relative = base
                .inverse()
                .unwrap()
                .compose(locked)
                .unwrap()
                .coefficients();
            assert!((relative[0] - 1.6).abs() < 1e-6 && (relative[3] - 1.6).abs() < 1e-6);
            let kept = parent.compose(locked).unwrap().point(4., 7.).unwrap();
            assert!((kept[0] - anchor[0]).abs() < 1e-6 && (kept[1] - anchor[1]).abs() < 1e-6);
        }
    }
}
