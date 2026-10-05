pub mod point;
pub use point::Point;

pub mod line;
pub use line::Line;

pub mod circle;
pub use circle::Circle;

pub mod arc;
pub use arc::Arc;

pub mod ellipse;
pub use ellipse::Ellipse;

pub mod curve;
pub use curve::Curve;

pub mod algorithms;
pub use algorithms::*;

pub mod intersection;
pub use intersection::IntersectionResult;

#[cfg(feature = "boolean")]
pub mod boolean;

pub mod extended_geometry;
pub use extended_geometry::{Polyline, EllipseArc, SplineFittedPolyline};

pub mod bspline;
pub use bspline::BSpline;

pub mod nurbs;
pub use nurbs::NURBS;

pub type Point2D = Point;
pub type Line2D = Line;
pub type Circle2D = Circle;
pub type Arc2D = Arc;
pub type Ellipse2D = Ellipse;

pub use crate::math::Vector2;
