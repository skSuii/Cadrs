use crate::geometry::Point;
use crate::geometry::Circle;
use std::fmt;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Arc {
    pub center: Point,
    pub radius: f64,
    pub start_angle: f64,
    pub end_angle: f64,
    pub is_counter_clockwise: bool,
}

impl Arc {
    #[inline]
    pub fn new(center: Point, radius: f64, start_angle: f64, end_angle: f64) -> Self {
        let is_ccw = Arc::calculate_direction(start_angle, end_angle);
        Self {
            center,
            radius,
            start_angle,
            end_angle,
            is_counter_clockwise: is_ccw,
        }
    }

    fn calculate_direction(start: f64, end: f64) -> bool {
        let mut start = start;
        let mut end = end;
        while start < 0.0 { start += 2.0 * std::f64::consts::PI; }
        while end < 0.0 { end += 2.0 * std::f64::consts::PI; }
        
        if end >= start {
            end - start <= std::f64::consts::PI
        } else {
            2.0 * std::f64::consts::PI - start + end <= std::f64::consts::PI
        }
    }

    #[inline]
    pub fn length(&self) -> f64 {
        self.radius * self.angle_span()
    }

    #[inline]
    pub fn angle_span(&self) -> f64 {
        let start = self.normalize_angle(self.start_angle);
        let end = self.normalize_angle(self.end_angle);
        
        if self.is_counter_clockwise {
            if end >= start {
                end - start
            } else {
                2.0 * std::f64::consts::PI - start + end
            }
        } else {
            if end <= start {
                start - end
            } else {
                start + 2.0 * std::f64::consts::PI - end
            }
        }
    }

    #[inline]
    pub fn normalize_angle(&self, angle: f64) -> f64 {
        let two_pi = 2.0 * std::f64::consts::PI;
        ((angle % two_pi) + two_pi) % two_pi
    }

    #[inline]
    pub fn start_point(&self) -> Point {
        self.point_at_angle(self.start_angle)
    }

    #[inline]
    pub fn end_point(&self) -> Point {
        self.point_at_angle(self.end_angle)
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
    pub fn point_at_parameter(&self, t: f64) -> Point {
        let angle_span = self.angle_span();
        let angle = if self.is_counter_clockwise {
            self.start_angle + t * angle_span
        } else {
            self.start_angle - t * angle_span
        };
        self.point_at_angle(angle)
    }

    #[inline]
    pub fn midpoint(&self) -> Point {
        self.point_at_parameter(0.5)
    }

    #[inline]
    pub fn is_valid(&self) -> bool {
        self.radius > 0.0 && self.start_angle != self.end_angle
    }

    #[inline]
    pub fn angle_from_center(&self, point: &Point) -> f64 {
        let dx = point.x - self.center.x;
        let dy = point.y - self.center.y;
        dy.atan2(dx)
    }

    #[inline]
    pub fn from_three_points(p1: Point, center: Point, p2: Point, is_ccw: bool) -> Self {
        let radius = p1.distance_to(&center);
        let start_angle = Arc::calculate_angle_from_points(&center, &p1);
        let end_angle = Arc::calculate_angle_from_points(&center, &p2);
        
        let two_pi = 2.0 * std::f64::consts::PI;
        let mut start = start_angle;
        let mut end = end_angle;
        while start < 0.0 { start += two_pi; }
        while end < 0.0 { end += two_pi; }
        
        let is_counter_clockwise = if is_ccw {
            if end >= start {
                end - start <= std::f64::consts::PI
            } else {
                two_pi - start + end <= std::f64::consts::PI
            }
        } else {
            if end <= start {
                start - end <= std::f64::consts::PI
            } else {
                start + two_pi - end <= std::f64::consts::PI
            }
        };

        Self {
            center,
            radius,
            start_angle,
            end_angle,
            is_counter_clockwise,
        }
    }

    fn calculate_angle_from_points(center: &Point, point: &Point) -> f64 {
        let dx = point.x - center.x;
        let dy = point.y - center.y;
        dy.atan2(dx)
    }

    #[inline]
    pub fn radius(&self) -> f64 {
        self.radius
    }

    #[inline]
    pub fn center(&self) -> Point {
        self.center
    }

    #[inline]
    pub fn start_angle(&self) -> f64 {
        self.start_angle
    }

    #[inline]
    pub fn end_angle(&self) -> f64 {
        self.end_angle
    }
}

impl From<Circle> for Arc {
    fn from(circle: Circle) -> Self {
        Self {
            center: circle.center,
            radius: circle.radius,
            start_angle: 0.0,
            end_angle: 2.0 * std::f64::consts::PI,
            is_counter_clockwise: true,
        }
    }
}

impl fmt::Display for Arc {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Arc(center: {}, radius: {}, start: {:.2}°, end: {:.2}°)",
            self.center,
            self.radius,
            self.start_angle.to_degrees(),
            self.end_angle.to_degrees()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_arc_creation() {
        let arc = Arc::new(Point::origin(), 5.0, 0.0, std::f64::consts::PI / 2.0);
        
        assert_eq!(arc.radius, 5.0);
        assert!(arc.is_valid());
    }

    #[test]
    fn test_arc_angle_span() {
        let arc = Arc::new(Point::origin(), 5.0, 0.0, std::f64::consts::PI / 2.0);
        
        assert!((arc.angle_span() - std::f64::consts::PI / 2.0).abs() < 1e-10);
    }

    #[test]
    fn test_arc_length() {
        let arc = Arc::new(Point::origin(), 1.0, 0.0, std::f64::consts::PI / 2.0);
        
        assert!((arc.length() - std::f64::consts::PI / 2.0).abs() < 1e-10);
    }
}
