use crate::geometry::Point;
use std::fmt;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BSpline {
    control_points: Vec<Point>,
    knots: Vec<f64>,
    degree: usize,
}

impl BSpline {
    #[inline]
    pub fn new(control_points: Vec<Point>, knots: Vec<f64>, degree: usize) -> Self {
        Self {
            control_points,
            knots,
            degree,
        }
    }

    #[inline]
    pub fn from_points(points: Vec<Point>, degree: usize) -> Self {
        let m = points.len();
        let mut knots = Vec::with_capacity(m + degree + 1);

        for _ in 0..=degree {
            knots.push(0.0);
        }
        for i in 1..(m - degree) {
            knots.push(i as f64 / (m - degree) as f64);
        }
        for _ in 0..=degree {
            knots.push(1.0);
        }

        Self {
            control_points: points,
            knots,
            degree,
        }
    }

    #[inline]
    pub fn control_points(&self) -> &[Point] {
        &self.control_points
    }

    #[inline]
    pub fn control_points_mut(&mut self) -> &mut [Point] {
        &mut self.control_points
    }

    #[inline]
    pub fn knots(&self) -> &[f64] {
        &self.knots
    }

    #[inline]
    pub fn degree(&self) -> usize {
        self.degree
    }

    #[inline]
    pub fn order(&self) -> usize {
        self.degree + 1
    }

    #[inline]
    pub fn is_valid(&self) -> bool {
        let n = self.control_points.len();
        if n < 2 {
            return false;
        }
        let expected_knots = n + self.degree + 1;
        self.knots.len() == expected_knots
    }

    #[inline]
    pub fn point_at(&self, t: f64) -> Point {
        self.evaluate_point(t)
    }

    #[inline]
    fn evaluate_point(&self, t: f64) -> Point {
        let t = t.clamp(0.0, 1.0);
        if t >= 1.0 {
            return *self.control_points.last().unwrap_or(&Point::origin());
        }
        let basis = self.compute_basis_functions(t);

        let mut x = 0.0;
        let mut y = 0.0;
        let mut z = 0.0;

        for (i, &b) in basis.iter().enumerate() {
            if b > 0.0 {
                x += self.control_points[i].x * b;
                y += self.control_points[i].y * b;
                z += self.control_points[i].z * b;
            }
        }

        Point::new(x, y, z)
    }

    fn compute_basis_functions(&self, t: f64) -> Vec<f64> {
        let n = self.control_points.len();
        let p = self.degree;
        let mut basis = vec![0.0; n];

        if p == 0 {
            for i in 0..n {
                if t >= self.knots[i] && t < self.knots[i + 1] {
                    basis[i] = 1.0;
                }
            }
            return basis;
        }

        let mut ndu = vec![vec![0.0; p + 1]; n + 1];
        for i in 0..=n {
            ndu[i][0] = if t >= self.knots[i] && t < self.knots[i + 1] { 1.0 } else { 0.0 };
        }

        for j in 1..=p {
            for i in 0..n {
                let mut saved = 0.0;
                let denom1 = self.knots[i + j] - self.knots[i];
                let denom2 = self.knots[i + j + 1] - self.knots[i + 1];

                if denom1 != 0.0 {
                    saved = ((t - self.knots[i]) / denom1) * ndu[i][j - 1];
                }
                if denom2 != 0.0 {
                    ndu[i][j] = ((self.knots[i + j + 1] - t) / denom2) * ndu[i + 1][j - 1] + saved;
                } else {
                    ndu[i][j] = saved;
                }
            }
        }

        for i in 0..n {
            basis[i] = ndu[i][p];
        }

        basis
    }

    #[inline]
    pub fn derivative(&self, _t: f64) -> Point {
        if self.degree == 0 {
            return Point::origin();
        }

        let mut dx = 0.0;
        let mut dy = 0.0;
        let mut dz = 0.0;
        
        for i in 0..self.control_points.len() - 1 {
            let factor = self.degree as f64 / (self.knots[i + self.degree + 1] - self.knots[i + 1]);
            let point_diff = Point::new(
                self.control_points[i + 1].x - self.control_points[i].x,
                self.control_points[i + 1].y - self.control_points[i].y,
                self.control_points[i + 1].z - self.control_points[i].z,
            );
            dx += point_diff.x * factor;
            dy += point_diff.y * factor;
            dz += point_diff.z * factor;
        }
        
        Point::new(dx, dy, dz)
    }
}

impl fmt::Display for BSpline {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "BSpline(degree: {}, control_points: {}, knots: {})",
            self.degree,
            self.control_points.len(),
            self.knots.len()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bspline_creation() {
        let points = vec![
            Point::new(0.0, 0.0, 0.0),
            Point::new(1.0, 1.0, 0.0),
            Point::new(2.0, 1.0, 0.0),
            Point::new(3.0, 0.0, 0.0),
        ];
        let spline = BSpline::from_points(points, 2);
        
        assert_eq!(spline.degree(), 2);
        assert!(spline.is_valid());
    }

    #[test]
    fn test_bspline_point_at() {
        let points = vec![
            Point::new(0.0, 0.0, 0.0),
            Point::new(1.0, 1.0, 0.0),
            Point::new(2.0, 1.0, 0.0),
            Point::new(3.0, 0.0, 0.0),
        ];
        let spline = BSpline::from_points(points, 2);
        
        let start = spline.point_at(0.0);
        assert!((start.x - 0.0).abs() < 1e-10);
        
        let end = spline.point_at(1.0);
        assert!((end.x - 3.0).abs() < 1e-10);
    }
}
