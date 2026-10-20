//! Pure, bounded composition experiment. No UI, GPU, filesystem or project mutation.
//! Pixel edges use integer coordinates; sample at output pixel centers. Source
//! RGBA8 is straight-alpha sRGB. Work in linear premultiplied RGBA, source-over.
//! Groups isolate their children before applying group opacity exactly once.

#[derive(Clone, Debug)]
pub struct Image {
    pub size: [u32; 2],
    pub rgba: Vec<[u8; 4]>,
}
#[derive(Clone, Copy, Debug)]
pub struct Affine(pub [f64; 6]);
impl Affine {
    pub const IDENTITY: Self = Self([1., 0., 0., 1., 0., 0.]);
    /// self * child, column vectors. Does not decompose TRS or discard shear.
    pub fn then(self, child: Self) -> Self {
        let [a, b, c, d, x, y] = self.0;
        let [e, f, g, h, u, v] = child.0;
        Self([
            a * e + c * f,
            b * e + d * f,
            a * g + c * h,
            b * g + d * h,
            a * u + c * v + x,
            b * u + d * v + y,
        ])
    }
    pub fn inverse(self) -> Result<Self, String> {
        let [a, b, c, d, x, y] = self.0;
        let det = a * d - b * c;
        if self.0.iter().any(|n| !n.is_finite() || n.abs() > 1e6) || det.abs() < 1e-10 {
            return Err("Nonfinite, excessive or singular transform".into());
        }
        let inverse = Self([
            d / det,
            -b / det,
            -c / det,
            a / det,
            (c * y - d * x) / det,
            (b * x - a * y) / det,
        ]);
        if inverse.0.iter().any(|n| !n.is_finite() || n.abs() > 1e6) {
            return Err("Inverse transform exceeds experiment precision range".into());
        }
        Ok(inverse)
    }
    fn point(self, x: f64, y: f64) -> [f64; 2] {
        let [a, b, c, d, u, v] = self.0;
        [a * x + c * y + u, b * x + d * y + v]
    }
}
#[derive(Clone, Copy, Debug)]
pub enum Sampling {
    Nearest,
    Bilinear,
}
#[derive(Clone, Debug)]
pub enum Node {
    Layer {
        asset: usize,
        transform: Affine,
        opacity: f64,
        sampling: Sampling,
    },
    Group {
        transform: Affine,
        opacity: f64,
        children: Vec<Node>,
    },
}
#[derive(Clone, Debug)]
pub enum Op {
    Begin,
    End(f64),
    Draw {
        asset: usize,
        inverse: Affine,
        opacity: f64,
        sampling: Sampling,
    },
}
#[derive(Clone, Debug)]
pub struct Plan<'a> {
    size: [u32; 2],
    ops: Vec<Op>,
    assets: &'a [Image],
}
/// Explicit experiment budgets, not product file-format limits.
#[derive(Clone, Copy)]
pub struct Limits {
    max_side: u32,
    max_asset_bytes: usize,
}
impl Limits {
    pub const SMALL: Self = Self {
        max_side: 256,
        max_asset_bytes: 32 * 1024 * 1024,
    };
    pub const STANDARD: Self = Self {
        max_side: 1024,
        max_asset_bytes: 40 * 1024 * 1024,
    };
}
impl Plan<'_> {
    pub fn size(&self) -> [u32; 2] {
        self.size
    }
    pub fn ops(&self) -> &[Op] {
        &self.ops
    }
    pub fn assets(&self) -> &[Image] {
        self.assets
    }
}
pub fn compile<'a>(
    size: [u32; 2],
    assets: &'a [Image],
    nodes: &[Node],
) -> Result<Plan<'a>, String> {
    compile_with_limits(size, assets, nodes, Limits::SMALL)
}
pub fn compile_with_limits<'a>(
    size: [u32; 2],
    assets: &'a [Image],
    nodes: &[Node],
    limits: Limits,
) -> Result<Plan<'a>, String> {
    if size.contains(&0) || size.iter().any(|&n| n > limits.max_side) || assets.len() > 128 {
        return Err("Canvas or asset count outside bounded experiment".into());
    }
    let mut asset_bytes = 0usize;
    for image in assets {
        if image.size.contains(&0)
            || image.size.iter().any(|&n| n > limits.max_side)
            || image.rgba.len() != image.size[0] as usize * image.size[1] as usize
        {
            return Err("Invalid asset pixels or dimensions".into());
        }
        asset_bytes = asset_bytes
            .checked_add(image.rgba.len() * 4)
            .ok_or("Asset byte overflow")?;
        if asset_bytes > limits.max_asset_bytes {
            return Err("Active source byte budget exceeded".into());
        }
    }
    fn walk(
        nodes: &[Node],
        parent: Affine,
        depth: usize,
        count: &mut usize,
        assets: usize,
        ops: &mut Vec<Op>,
    ) -> Result<(), String> {
        if depth > 8 {
            return Err("Group depth exceeds eight".into());
        }
        for node in nodes {
            *count += 1;
            if *count > 128 {
                return Err("Node limit exceeded".into());
            }
            let (transform, opacity) = match node {
                Node::Layer {
                    transform, opacity, ..
                }
                | Node::Group {
                    transform, opacity, ..
                } => (*transform, *opacity),
            };
            if !opacity.is_finite() || !(0.0..=1.0).contains(&opacity) {
                return Err("Invalid opacity".into());
            }
            transform.inverse()?;
            let world = parent.then(transform);
            let inverse = world.inverse()?;
            match node {
                Node::Layer {
                    asset, sampling, ..
                } => {
                    if *asset >= assets {
                        return Err("Missing asset".into());
                    }
                    ops.push(Op::Draw {
                        asset: *asset,
                        inverse,
                        opacity,
                        sampling: *sampling,
                    });
                }
                Node::Group { children, .. } => {
                    ops.push(Op::Begin);
                    walk(children, world, depth + 1, count, assets, ops)?;
                    ops.push(Op::End(opacity));
                }
            }
        }
        Ok(())
    }
    let mut ops = Vec::new();
    walk(nodes, Affine::IDENTITY, 0, &mut 0, assets.len(), &mut ops)?;
    Ok(Plan { size, ops, assets })
}
pub fn linear(n: f64) -> f64 {
    if n <= 0.04045 {
        n / 12.92
    } else {
        ((n + 0.055) / 1.055).powf(2.4)
    }
}
fn srgb(n: f64) -> f64 {
    if n <= 0.0031308 {
        12.92 * n
    } else {
        1.055 * n.powf(1.0 / 2.4) - 0.055
    }
}
fn pixel(image: &Image, x: i32, y: i32) -> [f64; 4] {
    let x = x.clamp(0, image.size[0] as i32 - 1) as usize;
    let y = y.clamp(0, image.size[1] as i32 - 1) as usize;
    let p = image.rgba[y * image.size[0] as usize + x];
    let a = f64::from(p[3]) / 255.;
    [
        linear(f64::from(p[0]) / 255.) * a,
        linear(f64::from(p[1]) / 255.) * a,
        linear(f64::from(p[2]) / 255.) * a,
        a,
    ]
}
fn sample(image: &Image, p: [f64; 2], sampling: Sampling) -> [f64; 4] {
    // Shared CPU/GPU half-open boundary tie rule, in source pixel units.
    let [x, y] = p.map(|v| {
        if (v - v.round()).abs() <= 0.0001 {
            v.round()
        } else {
            v
        }
    });
    if x < 0. || y < 0. || x >= f64::from(image.size[0]) || y >= f64::from(image.size[1]) {
        return [0.; 4];
    }
    if matches!(sampling, Sampling::Nearest) {
        return pixel(image, x.floor() as i32, y.floor() as i32);
    }
    let (x, y) = (x - 0.5, y - 0.5);
    let (ix, iy) = (x.floor() as i32, y.floor() as i32);
    let (fx, fy) = (x - x.floor(), y - y.floor());
    let p = [
        pixel(image, ix, iy),
        pixel(image, ix + 1, iy),
        pixel(image, ix, iy + 1),
        pixel(image, ix + 1, iy + 1),
    ];
    std::array::from_fn(|c| {
        (p[0][c] * (1. - fx) + p[1][c] * fx) * (1. - fy) + (p[2][c] * (1. - fx) + p[3][c] * fx) * fy
    })
}
fn over(dst: [f64; 4], src: [f64; 4]) -> [f64; 4] {
    std::array::from_fn(|i| src[i] + dst[i] * (1. - src[3]))
}
fn encode(v: [f64; 4]) -> [u8; 4] {
    let alpha = (v[3].clamp(0., 1.) * 255.).round() as u8;
    if alpha == 0 {
        return [0; 4];
    }
    let mut result = [0; 4];
    for c in 0..3 {
        result[c] = (srgb((v[c] / v[3]).clamp(0., 1.)) * 255.).round() as u8;
    }
    result[3] = alpha;
    result
}
pub fn render(plan: &Plan<'_>) -> Image {
    let assets = plan.assets;
    let mut rgba = Vec::with_capacity(plan.size[0] as usize * plan.size[1] as usize);
    for y in 0..plan.size[1] {
        for x in 0..plan.size[0] {
            let mut stack = [[0.; 4]; 9];
            let mut depth = 0;
            for op in &plan.ops {
                match op {
                    Op::Begin => {
                        depth += 1;
                        stack[depth] = [0.; 4];
                    }
                    Op::End(opacity) => {
                        let src = stack[depth].map(|n| n * opacity);
                        depth -= 1;
                        stack[depth] = over(stack[depth], src);
                    }
                    Op::Draw {
                        asset,
                        inverse,
                        opacity,
                        sampling,
                    } => {
                        let src = sample(
                            &assets[*asset],
                            inverse.point(f64::from(x) + 0.5, f64::from(y) + 0.5),
                            *sampling,
                        )
                        .map(|n| n * opacity);
                        stack[depth] = over(stack[depth], src);
                    }
                }
            }
            rgba.push(encode(stack[0]));
        }
    }
    Image {
        size: plan.size,
        rgba,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn solid(p: [u8; 4]) -> Image {
        Image {
            size: [1, 1],
            rgba: vec![p],
        }
    }
    fn layer(asset: usize) -> Node {
        Node::Layer {
            asset,
            transform: Affine::IDENTITY,
            opacity: 1.,
            sampling: Sampling::Nearest,
        }
    }
    fn image(assets: &[Image], nodes: &[Node], size: [u32; 2]) -> Image {
        render(&compile(size, assets, nodes).unwrap())
    }
    #[test]
    fn group_opacity_is_applied_after_overlap() {
        let assets = [solid([255, 0, 0, 255]), solid([0, 0, 255, 255])];
        let nodes = [Node::Group {
            transform: Affine::IDENTITY,
            opacity: 0.5,
            children: vec![layer(0), layer(1)],
        }];
        assert_eq!(image(&assets, &nodes, [1, 1]).rgba, vec![[0, 0, 255, 128]]);
    }
    #[test]
    fn nested_groups_multiply_alpha_once_and_preserve_order() {
        let assets = [solid([255, 0, 0, 255])];
        let inner = Node::Group {
            transform: Affine::IDENTITY,
            opacity: 0.5,
            children: vec![layer(0)],
        };
        let outer = Node::Group {
            transform: Affine::IDENTITY,
            opacity: 0.5,
            children: vec![inner],
        };
        assert_eq!(image(&assets, &[outer], [1, 1]).rgba[0], [255, 0, 0, 64]);
    }
    #[test]
    fn linear_light_source_over_has_known_result() {
        let assets = [solid([255, 255, 255, 255]), solid([0, 0, 0, 128])];
        let p = image(&assets, &[layer(0), layer(1)], [1, 1]).rgba[0];
        assert_eq!(p, [187, 187, 187, 255]);
    }
    #[test]
    fn rotation_nonuniform_scale_and_clipping() {
        let assets = [Image {
            size: [2, 1],
            rgba: vec![[255, 0, 0, 255], [0, 0, 255, 255]],
        }];
        let n = Node::Layer {
            asset: 0,
            transform: Affine([0., 1., -1., 0., 3., 0.]),
            opacity: 1.,
            sampling: Sampling::Nearest,
        };
        let p = image(&assets, &[n], [4, 3]);
        assert_eq!(p.rgba[2], [255, 0, 0, 255]);
        assert_eq!(p.rgba[6], [0, 0, 255, 255]);
        assert_eq!(p.rgba.iter().filter(|p| p[3] > 0).count(), 2);
        let n = Node::Layer {
            asset: 0,
            transform: Affine([2., 0., 0., 3., -1., 0.]),
            opacity: 1.,
            sampling: Sampling::Nearest,
        };
        assert_eq!(
            image(&assets, &[n], [3, 3])
                .rgba
                .iter()
                .filter(|p| p[3] > 0)
                .count(),
            9
        );
    }
    #[test]
    fn transparent_colored_pixels_do_not_create_bilinear_halos() {
        let assets = [Image {
            size: [2, 1],
            rgba: vec![[255, 0, 0, 0], [0, 0, 255, 255]],
        }];
        let n = Node::Layer {
            asset: 0,
            transform: Affine([2., 0., 0., 1., 0., 0.]),
            opacity: 1.,
            sampling: Sampling::Bilinear,
        };
        let p = image(&assets, &[n], [4, 1]);
        assert_eq!(p.rgba[0], [0; 4]);
        assert_eq!(p.rgba[1], [0, 0, 255, 64]);
        assert_eq!(p.rgba[2], [0, 0, 255, 191]);
    }
    #[test]
    fn affine_roundtrip_preserves_shear_and_reflection() {
        let m = Affine([1.2, 0.6, -0.3, -2.1, 7., -4.]).then(Affine([0., 1., -1., 0., 3., 9.]));
        let p = m
            .inverse()
            .unwrap()
            .point(m.point(5., 8.)[0], m.point(5., 8.)[1]);
        assert!((p[0] - 5.).abs() < 1e-10 && (p[1] - 8.).abs() < 1e-10);
    }
    #[test]
    fn invalid_plans_fail_without_modifying_sources() {
        let a = [solid([9, 8, 7, 6])];
        assert!(compile([1, 1], &a, &[layer(1)]).is_err());
        assert!(compile([0, 1], &a, &[]).is_err());
        let n = Node::Layer {
            asset: 0,
            transform: Affine([0.; 6]),
            opacity: 1.,
            sampling: Sampling::Nearest,
        };
        assert!(compile([1, 1], &a, &[n]).is_err());
        let n = Node::Group {
            transform: Affine::IDENTITY,
            opacity: f64::NAN,
            children: vec![],
        };
        assert!(compile([1, 1], &a, &[n]).is_err());
        assert_eq!(a[0].rgba, vec![[9, 8, 7, 6]]);
    }
    #[test]
    fn bounded_plan_rejects_excessive_depth_nodes_and_malformed_pixels() {
        let assets = [solid([255; 4])];
        let mut n = layer(0);
        for _ in 0..9 {
            n = Node::Group {
                transform: Affine::IDENTITY,
                opacity: 1.,
                children: vec![n],
            };
        }
        assert!(compile([1, 1], &assets, &[n]).is_err());
        assert!(compile([1, 1], &assets, &vec![layer(0); 129]).is_err());
        assert!(compile([257, 1], &assets, &[]).is_err());
        let bad = Image {
            size: [2, 2],
            rgba: vec![[255; 4]],
        };
        assert!(compile([1, 1], &[bad], &[]).is_err());
        assert!(Affine([f64::NAN, 0., 0., 1., 0., 0.]).inverse().is_err());
    }
    #[test]
    fn near_integer_boundary_uses_shared_half_open_tie_rule() {
        let assets = [solid([255, 0, 0, 255])];
        let node = Node::Layer {
            asset: 0,
            transform: Affine([1., 0., 0., 1., 0.500001, 0.]),
            opacity: 1.,
            sampling: Sampling::Nearest,
        };
        assert_eq!(
            image(&assets, &[node], [2, 1]).rgba,
            vec![[255, 0, 0, 255], [0, 0, 0, 0]]
        );
    }
    #[test]
    fn scale_limits_are_explicit_and_bound_total_source_bytes() {
        let a = [solid([255; 4]), solid([0; 4])];
        assert!(compile([1024, 1024], &a, &[]).is_err());
        assert!(compile_with_limits([1024, 1024], &a, &[], Limits::STANDARD).is_ok());
        assert!(compile_with_limits([1025, 1], &a, &[], Limits::STANDARD).is_err());
        let budget = Limits {
            max_side: 1024,
            max_asset_bytes: 4,
        };
        assert!(compile_with_limits([1, 1], &a, &[], budget).is_err());
        assert!(compile_with_limits([1, 1], &a[..1], &[layer(0)], budget).is_ok());
        assert_eq!(a[0].rgba, vec![[255; 4]]);
    }
}
