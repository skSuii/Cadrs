use crate::geometry::Point;
use crate::math::Vector2;
use std::fmt;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Line {
    pub start: Point,
    pub end: Point,
}

impl Line {
    #[inline]
    pub fn new(start: Point, end: Point) -> Self {
        Self { start, end }
    }

    #[inline]
    pub fn new2d(start: Point, end: Point) -> Self {
        Self { start, end }
    }

    #[inline]
    pub fn direction(&self) -> Vector2 {
        (self.end.to_vector2() - self.start.to_vector2()).normalize()
    }

    #[inline]
    pub fn length(&self) -> f64 {
        self.start.distance_to(&self.end)
    }

    #[inline]
    pub fn midpoint(&self) -> Point {
        Point::new(
            (self.start.x + self.end.x) / 2.0,
            (self.start.y + self.end.y) / 2.0,
            (self.start.z + self.end.z) / 2.0,
        )
    }

    #[inline]
    pub fn point_at_parameter(&self, t: f64) -> Point {
        Point::new(
            self.start.x + (self.end.x - self.start.x) * t,
            self.start.y + (self.end.y - self.start.y) * t,
            self.start.z + (self.end.z - self.start.z) * t,
        )
    }

    #[inline]
    pub fn is_horizontal(&self) -> bool {
        (self.end.y - self.start.y).abs() < 1e-10
    }

    #[inline]
    pub fn is_vertical(&self) -> bool {
        (self.end.x - self.start.x).abs() < 1e-10
    }

    #[inline]
    pub fn closest_point(&self, p: &Point) -> Point {
        let d = self.end.to_vector2() - self.start.to_vector2();
        let to_point = p.to_vector2() - self.start.to_vector2();
        let len_sq = d.dot(&d);
        if len_sq < 1e-30 {
            return self.start;
        }
        let t = (to_point.dot(&d) / len_sq).clamp(0.0, 1.0);
        self.point_at_parameter(t)
    }

    #[inline]
    pub fn distance_to_point(&self, p: &Point) -> f64 {
        p.distance_to(&self.closest_point(p))
    }

    #[inline]
    pub fn start_point(&self) -> Point {
        self.start
    }

    #[inline]
    pub fn end_point(&self) -> Point {
        self.end
    }
}

impl fmt::Display for Line {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Line({} -> {})", self.start, self.end)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::validation;

    #[test]
    fn test_line_creation() {
        let start = Point::new(0.0, 0.0, 0.0);
        let end = Point::new(1.0, 1.0, 0.0);
        let line = Line::new(start, end);
        
        assert_eq!(line.start, start);
        assert_eq!(line.end, end);
    }

    #[test]
    fn test_line_length() {
        let start = Point::new(0.0, 0.0, 0.0);
        let end = Point::new(3.0, 4.0, 0.0);
        let line = Line::new(start, end);
        
        assert!((line.length() - 5.0).abs() < 1e-10);
    }

    #[test]
    fn test_line_midpoint() {
        let start = Point::new(0.0, 0.0, 0.0);
        let end = Point::new(2.0, 2.0, 0.0);
        let line = Line::new(start, end);
        let midpoint = line.midpoint();
        
        assert!((midpoint.x - 1.0).abs() < 1e-10);
        assert!((midpoint.y - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_line_direction() {
        let start = Point::new(0.0, 0.0, 0.0);
        let end = Point::new(3.0, 4.0, 0.0);
        let line = Line::new(start, end);
        let direction = line.direction();
        
        assert!((direction.magnitude() - 1.0).abs() < 1e-10);
        assert!((direction.x - 0.6).abs() < 1e-10);
        assert!((direction.y - 0.8).abs() < 1e-10);
    }

    #[test]
    fn test_line_point_at_parameter() {
        let start = Point::new(0.0, 0.0, 0.0);
        let end = Point::new(10.0, 10.0, 0.0);
        let line = Line::new(start, end);
        
        let t0 = line.point_at_parameter(0.0);
        assert_eq!(t0, start);
        
        let t1 = line.point_at_parameter(1.0);
        assert_eq!(t1, end);
        
        let t05 = line.point_at_parameter(0.5);
        assert!((t05.x - 5.0).abs() < 1e-10);
        assert!((t05.y - 5.0).abs() < 1e-10);
    }

    #[test]
    fn test_line_is_horizontal() {
        let horizontal = Line::new(Point::new(0.0, 0.0, 0.0), Point::new(10.0, 0.0, 0.0));
        let not_horizontal = Line::new(Point::new(0.0, 0.0, 0.0), Point::new(10.0, 1.0, 0.0));
        
        assert!(horizontal.is_horizontal());
        assert!(!not_horizontal.is_horizontal());
    }

    #[test]
    fn test_line_is_vertical() {
        let vertical = Line::new(Point::new(0.0, 0.0, 0.0), Point::new(0.0, 10.0, 0.0));
        let not_vertical = Line::new(Point::new(0.0, 0.0, 0.0), Point::new(1.0, 10.0, 0.0));
        
        assert!(vertical.is_vertical());
        assert!(!not_vertical.is_vertical());
    }

    #[test]
    fn test_line_closest_point() {
        let line = Line::new(Point::new(0.0, 0.0, 0.0), Point::new(10.0, 0.0, 0.0));
        
        let point_on_line = line.closest_point(&Point::new(5.0, 0.0, 0.0));
        assert!((point_on_line.x - 5.0).abs() < 1e-10);
        assert!((point_on_line.y - 0.0).abs() < 1e-10);
        
        let point_below = line.closest_point(&Point::new(5.0, -5.0, 0.0));
        assert!((point_below.x - 5.0).abs() < 1e-10);
        assert!((point_below.y - 0.0).abs() < 1e-10);
    }

    #[test]
    fn test_line_distance_to_point() {
        let line = Line::new(Point::new(0.0, 0.0, 0.0), Point::new(10.0, 0.0, 0.0));
        
        let distance = line.distance_to_point(&Point::new(5.0, 5.0, 0.0));
        assert!((distance - 5.0).abs() < 1e-10);
        
        let distance_on_line = line.distance_to_point(&Point::new(5.0, 0.0, 0.0));
        assert!((distance_on_line - 0.0).abs() < 1e-10);
    }

    #[test]
    fn test_line_display() {
        let line = Line::new(Point::new(1.0, 2.0, 0.0), Point::new(3.0, 4.0, 0.0));
        let display = format!("{}", line);
        assert!(display.contains("Line"));
        assert!(display.contains("1"));
        assert!(display.contains("2"));
        assert!(display.contains("3"));
        assert!(display.contains("4"));
    }

    #[test]
    fn test_line_clone() {
        let line1 = Line::new(Point::new(1.0, 2.0, 0.0), Point::new(3.0, 4.0, 0.0));
        let line2 = line1;
        assert_eq!(line1.start, line2.start);
        assert_eq!(line1.end, line2.end);
    }

    #[test]
    fn test_line_zero_length() {
        let start = Point::new(5.0, 5.0, 0.0);
        let end = Point::new(5.0, 5.0, 0.0);
        let line = Line::new(start, end);
        
        assert!((line.length() - 0.0).abs() < 1e-10);
    }

    #[test]
    fn test_validation_positive() {
        assert!(validation::positive(1.0, "test").is_ok());
        assert!(validation::positive(0.0, "test").is_err());
        assert!(validation::positive(-1.0, "test").is_err());
    }

    #[test]
    fn test_validation_in_range() {
        assert!(validation::in_range(5.0, 0.0, 10.0, "test").is_ok());
        assert!(validation::in_range(-1.0, 0.0, 10.0, "test").is_err());
    }

    #[test]
    fn test_validation_non_negative() {
        assert!(validation::non_negative(0.0, "test").is_ok());
        assert!(validation::non_negative(1.0, "test").is_ok());
        assert!(validation::non_negative(-1.0, "test").is_err());
    }

    #[test]
    fn test_validation_scale_factor() {
        assert!(validation::scale_factor(1.0, "test").is_ok());
        assert!(validation::scale_factor(0.0, "test").is_err());
        assert!(validation::scale_factor(-1.0, "test").is_err());
        assert!(validation::scale_factor(f64::INFINITY, "test").is_err());
    }

    #[test]
    fn test_validation_coordinate() {
        assert!(validation::coordinate(1.0, "test").is_ok());
        assert!(validation::coordinate(f64::NAN, "test").is_err());
        assert!(validation::coordinate(f64::INFINITY, "test").is_err());
    }
}
