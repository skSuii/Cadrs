use crate::geometry::Point;
use crate::math::Vector2;
use std::fmt;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Circle {
    pub center: Point,
    pub radius: f64,
}

impl Circle {
    #[inline]
    pub fn new(center: Point, radius: f64) -> Self {
        Self { center, radius }
    }

    #[inline]
    pub fn new2d(center: Point, radius: f64) -> Self {
        Self { center, radius }
    }

    #[inline]
    pub fn diameter(&self) -> f64 {
        self.radius * 2.0
    }

    #[inline]
    pub fn circumference(&self) -> f64 {
        2.0 * std::f64::consts::PI * self.radius
    }

    #[inline]
    pub fn area(&self) -> f64 {
        std::f64::consts::PI * self.radius * self.radius
    }

    #[inline]
    pub fn point_at_angle(&self, angle: f64) -> Point {
        Point::new(
            self.center.x + self.radius * angle.cos(),
            self.center.y + self.radius * angle.sin(),
            self.center.z,
        )
    }

    #[inline]
    pub fn tangent_at_point(&self, p: &Point) -> Vector2 {
        let to_point = p.to_vector2() - self.center.to_vector2();
        Vector2::new(-to_point.y, to_point.x).normalize()
    }

    #[inline]
    pub fn contains_point(&self, p: &Point) -> bool {
        self.center.distance_to(p) <= self.radius + 1e-10
    }

    #[inline]
    pub fn normalize_angle(&self, angle: f64) -> f64 {
        let two_pi = 2.0 * std::f64::consts::PI;
        ((angle % two_pi) + two_pi) % two_pi
    }

    #[inline]
    pub fn angle_from_center(&self, p: &Point) -> f64 {
        let to_point = p.to_vector2() - self.center.to_vector2();
        to_point.y.atan2(to_point.x)
    }

    #[inline]
    pub fn radius(&self) -> f64 {
        self.radius
    }

    #[inline]
    pub fn center(&self) -> Point {
        self.center
    }
}

impl fmt::Display for Circle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Circle(center: {}, radius: {})", self.center, self.radius)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_circle_creation() {
        let center = Point::new(0.0, 0.0, 0.0);
        let circle = Circle::new(center, 5.0);
        
        assert_eq!(circle.radius, 5.0);
        assert!((circle.diameter() - 10.0).abs() < 1e-10);
    }

    #[test]
    fn test_circle_area() {
        let circle = Circle::new(Point::origin(), 1.0);
        assert!((circle.area() - std::f64::consts::PI).abs() < 1e-10);
    }

    #[test]
    fn test_circle_point_at_angle() {
        let circle = Circle::new(Point::origin(), 1.0);
        let p = circle.point_at_angle(0.0);
        
        assert!((p.x - 1.0).abs() < 1e-10);
        assert!((p.y - 0.0).abs() < 1e-10);
    }
}
