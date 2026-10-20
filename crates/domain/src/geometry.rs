use crate::ModelError;

/// Pixel extents, independent of DPI, scene coordinates and output naming.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PixelSize {
    width: u32,
    height: u32,
}

impl PixelSize {
    pub fn new(width: u32, height: u32) -> Result<Self, ModelError> {
        if width == 0 || height == 0 {
            return Err(ModelError::InvalidValue("pixel_size"));
        }
        u64::from(width)
            .checked_mul(u64::from(height))
            .and_then(|pixels| pixels.checked_mul(4))
            .ok_or(ModelError::NumericOverflow)?;
        Ok(Self { width, height })
    }

    pub const fn width(self) -> u32 {
        self.width
    }

    pub const fn height(self) -> u32 {
        self.height
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Opacity(f32);

impl Opacity {
    pub const OPAQUE: Self = Self(1.0);

    pub fn new(value: f32) -> Result<Self, ModelError> {
        if !value.is_finite() || !(0.0..=1.0).contains(&value) {
            return Err(ModelError::InvalidValue("opacity"));
        }
        Ok(Self(value))
    }

    pub const fn get(self) -> f32 {
        self.0
    }
}

/// Column-vector matrix [a c tx; b d ty; 0 0 1]. No TRS decomposition.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Affine2([f64; 6]);

impl Affine2 {
    pub const IDENTITY: Self = Self([1.0, 0.0, 0.0, 1.0, 0.0, 0.0]);

    pub fn new(coefficients: [f64; 6]) -> Result<Self, ModelError> {
        if coefficients.iter().any(|value| !value.is_finite()) {
            return Err(ModelError::InvalidValue("affine_coefficients"));
        }
        Ok(Self(coefficients))
    }

    pub const fn coefficients(self) -> [f64; 6] {
        self.0
    }

    /// self * local: apply local first, then self.
    pub fn compose(self, local: Self) -> Result<Self, ModelError> {
        let [a, b, c, d, tx, ty] = self.0;
        let [e, f, g, h, ux, uy] = local.0;
        Self::new([
            a * e + c * f,
            b * e + d * f,
            a * g + c * h,
            b * g + d * h,
            a * ux + c * uy + tx,
            b * ux + d * uy + ty,
        ])
        .map_err(|_| ModelError::NumericOverflow)
    }

    pub fn inverse(self) -> Result<Self, ModelError> {
        let [a, b, c, d, tx, ty] = self.0;
        let determinant = a * d - b * c;
        if determinant == 0.0 || !determinant.is_finite() {
            return Err(ModelError::NotInvertible);
        }
        Self::new([
            d / determinant,
            -b / determinant,
            -c / determinant,
            a / determinant,
            (c * ty - d * tx) / determinant,
            (b * tx - a * ty) / determinant,
        ])
        .map_err(|_| ModelError::NotInvertible)
    }

    /// Geometry for a future reparent command; does not edit any project.
    pub fn rebased(world: Self, new_parent_world: Self) -> Result<Self, ModelError> {
        new_parent_world.inverse()?.compose(world)
    }

    pub fn point(self, x: f64, y: f64) -> Result<[f64; 2], ModelError> {
        let [a, b, c, d, tx, ty] = self.0;
        let point = [a * x + c * y + tx, b * x + d * y + ty];
        if point.iter().any(|value| !value.is_finite()) {
            return Err(ModelError::NumericOverflow);
        }
        Ok(point)
    }
}

/// Geometric bounds only: no alpha tests, GUI hit region or gameplay judgment.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Bounds {
    min: [f64; 2],
    max: [f64; 2],
}

impl Bounds {
    pub fn transformed(size: PixelSize, transform: Affine2) -> Result<Self, ModelError> {
        let w = f64::from(size.width);
        let h = f64::from(size.height);
        let points = [
            transform.point(0.0, 0.0)?,
            transform.point(w, 0.0)?,
            transform.point(0.0, h)?,
            transform.point(w, h)?,
        ];
        let mut result = Self {
            min: points[0],
            max: points[0],
        };
        for point in points {
            for (axis, value) in point.into_iter().enumerate() {
                result.min[axis] = result.min[axis].min(value);
                result.max[axis] = result.max[axis].max(value);
            }
        }
        Ok(result)
    }

    pub fn union(self, other: Self) -> Self {
        Self {
            min: [self.min[0].min(other.min[0]), self.min[1].min(other.min[1])],
            max: [self.max[0].max(other.max[0]), self.max[1].max(other.max[1])],
        }
    }

    pub const fn min(self) -> [f64; 2] {
        self.min
    }

    pub const fn max(self) -> [f64; 2] {
        self.max
    }
}
