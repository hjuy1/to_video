//! A 2d point type.

/// A 2d point.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Point<T> {
    /// x-coordinate.
    pub x: T,
    /// y-coordinate.
    pub y: T,
}

impl<T> Point<T> {
    /// Construct a point at (x, y).
    pub fn new(x: T, y: T) -> Point<T> {
        Point::<T> { x, y }
    }
}

/// A fixed rotation. This struct exists solely to cache the values of `sin(theta)` and `cos(theta)` when
/// applying a fixed rotation to multiple points.
#[derive(Debug, Copy, Clone, PartialEq)]
pub(crate) struct Rotation {
    sin_theta: f64,
    cos_theta: f64,
}

impl Rotation {
    /// A rotation of `theta` radians.
    pub(crate) fn new(theta: f64) -> Rotation {
        let (sin_theta, cos_theta) = theta.sin_cos();
        Rotation {
            sin_theta,
            cos_theta,
        }
    }
}

impl Point<f64> {
    /// Rotates a point.
    pub(crate) fn rotate(&self, rotation: Rotation) -> Point<f64> {
        let x = self.x * rotation.cos_theta + self.y * rotation.sin_theta;
        let y = self.y * rotation.cos_theta - self.x * rotation.sin_theta;
        Point::new(x, y)
    }

    /// Inverts a rotation.
    pub(crate) fn invert_rotation(&self, rotation: Rotation) -> Point<f64> {
        let x = self.x * rotation.cos_theta - self.y * rotation.sin_theta;
        let y = self.y * rotation.cos_theta + self.x * rotation.sin_theta;
        Point::new(x, y)
    }
}

/// A line of the form Ax + By + C = 0.
#[derive(Debug, Copy, Clone, PartialEq)]
pub(crate) struct Line {
    a: f64,
    b: f64,
    c: f64,
}

impl Line {
    /// Returns the `Line` that passes through p and q.
    pub fn from_points(p: Point<f64>, q: Point<f64>) -> Line {
        let a = p.y - q.y;
        let b = q.x - p.x;
        let c = p.x * q.y - q.x * p.y;
        Line { a, b, c }
    }

    /// Computes the shortest distance from this line to the given point.
    pub fn distance_from_point(&self, point: Point<f64>) -> f64 {
        let Line { a, b, c } = self;
        (a * point.x + b * point.y + c).abs() / (a.powf(2.0) + b.powf(2.)).sqrt()
    }
}
