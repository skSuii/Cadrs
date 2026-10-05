use crate::geometry::Point;
use std::fmt;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Ellipse {
    pub center: Point,
    pub semi_major: f64,
    pub semi_minor: f64,
    pub rotation: f64,
}

impl Ellipse {
    #[inline]
    pub fn new(center: Point, semi_major: f64, semi_minor: f64, rotation: f64) -> Self {
        Self {
            center,
            semi_major,
            semi_minor,
            rotation,
        }
    }

    #[inline]
    pub fn eccentricity(&self) -> f64 {
        if self.semi_major == 0.0 {
            0.0
        } else {
            (1.0 - (self.semi_minor / self.semi_major).powi(2)).sqrt()
        }
    }

    #[inline]
    pub fn area(&self) -> f64 {
        std::f64::consts::PI * self.semi_major * self.semi_minor
    }

    #[inline]
    pub fn circumference_approx(&self) -> f64 {
        let a = self.semi_major;
        let b = self.semi_minor;
        std::f64::consts::PI * (3.0 * (a + b) - ((3.0 * a + b) * (a + 3.0 * b)).sqrt())
    }

    #[inline]
    pub fn point_at_parameter(&self, t: f64) -> Point {
        let angle = t * 2.0 * std::f64::consts::PI;
        let cos_angle = angle.cos();
        let sin_angle = angle.sin();
        let cos_rot = self.rotation.cos();
        let sin_rot = self.rotation.sin();

        let local_x = self.semi_major * cos_angle;
        let local_y = self.semi_minor * sin_angle;

        Point::new(
            self.center.x + local_x * cos_rot - local_y * sin_rot,
            self.center.y + local_x * sin_rot + local_y * cos_rot,
            self.center.z,
        )
    }

    #[inline]
    pub fn contains_point(&self, p: &Point) -> bool {
        let dx = p.x - self.center.x;
        let dy = p.y - self.center.y;
        let cos_rot = self.rotation.cos();
        let sin_rot = self.rotation.sin();

        let local_x = dx * cos_rot + dy * sin_rot;
        let local_y = -dx * sin_rot + dy * cos_rot;

        ((local_x / self.semi_major).powi(2) + (local_y / self.semi_minor).powi(2)) <= 1.0 + 1e-10
    }

    #[inline]
    pub fn major_axis(&self) -> f64 {
        self.semi_major * 2.0
    }

    #[inline]
    pub fn minor_axis(&self) -> f64 {
        self.semi_minor * 2.0
    }

    #[inline]
    pub fn center(&self) -> Point {
        self.center
    }
}

impl fmt::Display for Ellipse {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Ellipse(center: {}, semi_major: {}, semi_minor: {}, rotation: {:.2}°)",
            self.center,
            self.semi_major,
            self.semi_minor,
            self.rotation.to_degrees()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ellipse_creation() {
        let ellipse = Ellipse::new(Point::origin(), 5.0, 3.0, 0.0);
        
        assert_eq!(ellipse.semi_major, 5.0);
        assert_eq!(ellipse.semi_minor, 3.0);
    }

    #[test]
    fn test_ellipse_eccentricity() {
        let ellipse = Ellipse::new(Point::origin(), 5.0, 3.0, 0.0);
        let ecc = ellipse.eccentricity();
        
        assert!(ecc > 0.0 && ecc < 1.0);
    }

    #[test]
    fn test_ellipse_area() {
        let ellipse = Ellipse::new(Point::origin(), 5.0, 3.0, 0.0);
        let area = ellipse.area();
        
        assert!((area - 15.0 * std::f64::consts::PI).abs() < 1e-10);
    }
}
