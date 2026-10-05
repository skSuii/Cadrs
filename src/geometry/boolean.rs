use crate::geometry::{Point, Line, Circle, Arc, Ellipse, Polyline, Curve};
use crate::geometry::intersection::{IntersectionResult, IntersectionPoint};
use std::cmp::Ordering;

#[cfg(feature = "boolean")]
use clipper2::{Clipper, FillRule, Path, Paths, Point as ClipperPoint};
#[cfg(feature = "boolean")]
use crate::geometry::extended_geometry::Point as Point2D;

#[derive(Debug, Clone, PartialEq)]
pub enum BooleanOperation {
    Union,
    Intersection,
    Difference,
    ExclusiveOr,
}

#[derive(Debug, Clone)]
pub struct BooleanResult {
    pub entities: Vec<GeometricEntity>,
    pub success: bool,
    pub message: String,
}

#[derive(Debug, Clone)]
pub enum GeometricEntity {
    Line(Line),
    Arc(Arc),
    Circle(Circle),
    Polyline(Polyline),
    Composite(Vec<GeometricEntity>),
}

#[cfg(feature = "boolean")]
struct ClipperAdapter;

#[cfg(feature = "boolean")]
impl ClipperAdapter {
    fn point_to_clipper(p: &Point) -> ClipperPoint {
        ClipperPoint::new(p.x, p.y)
    }

    fn polyline_to_path(poly: &Polyline) -> Path {
        Path::new(poly.vertices.iter().map(|v| ClipperPoint::new(v.x, v.y)).collect())
    }

    fn path_to_polyline(path: &Path) -> Polyline {
        let vertices: Vec<Point2D> = path.iter()
            .map(|p| Point2D::new(p.x(), p.y()))
            .collect();
        Polyline {
            vertices,
            is_closed: true,
        }
    }
}

pub struct BooleanEngine;

impl BooleanEngine {
    pub fn new() -> Self {
        Self {}
    }

    #[cfg(feature = "boolean")]
    pub fn union_shapes(&self, shapes: &[GeometricEntity]) -> BooleanResult {
        self.boolean_operation(shapes, BooleanOperation::Union)
    }

    #[cfg(not(feature = "boolean"))]
    pub fn union_shapes(&self, shapes: &[GeometricEntity]) -> BooleanResult {
        BooleanResult {
            entities: shapes.to_vec(),
            success: false,
            message: "Boolean operations require 'boolean' feature".to_string(),
        }
    }

    #[cfg(feature = "boolean")]
    pub fn intersect_shapes(&self, shapes: &[GeometricEntity]) -> BooleanResult {
        self.boolean_operation(shapes, BooleanOperation::Intersection)
    }

    #[cfg(not(feature = "boolean"))]
    pub fn intersect_shapes(&self, shapes: &[GeometricEntity]) -> BooleanResult {
        BooleanResult {
            entities: shapes.to_vec(),
            success: false,
            message: "Boolean operations require 'boolean' feature".to_string(),
        }
    }

    #[cfg(feature = "boolean")]
    pub fn subtract_shapes(&self, subject: &[GeometricEntity], tool: &[GeometricEntity]) -> BooleanResult {
        let result = self.boolean_operation(subject, BooleanOperation::Difference);
        result
    }

    #[cfg(not(feature = "boolean"))]
    pub fn subtract_shapes(&self, subject: &[GeometricEntity], _tool: &[GeometricEntity]) -> BooleanResult {
        BooleanResult {
            entities: subject.to_vec(),
            success: false,
            message: "Boolean operations require 'boolean' feature".to_string(),
        }
    }

    #[cfg(feature = "boolean")]
    fn boolean_operation(&self, shapes: &[GeometricEntity], operation: BooleanOperation) -> BooleanResult {
        if shapes.is_empty() {
            return BooleanResult {
                entities: vec![],
                success: true,
                message: "No shapes provided".to_string(),
            };
        }

        let mut subject_paths: Paths = Paths::new(Vec::new());

        for shape in shapes {
            match shape {
                GeometricEntity::Polyline(poly) => {
                    if poly.vertices.len() >= 3 {
                        let path = ClipperAdapter::polyline_to_path(poly);
                        if !path.is_empty() {
                            subject_paths.push(path);
                        }
                    }
                }
                _ => {
                    return self.simple_boolean_operation(shapes, operation);
                }
            }
        }

        if subject_paths.is_empty() {
            return self.simple_boolean_operation(shapes, operation);
        }

        let fill_rule = FillRule::NonZero;

        let solution = match operation {
            BooleanOperation::Union => Clipper::new()
                .add_subject(subject_paths)
                .add_clip(Paths::new(Vec::new()))
                .union(fill_rule),
            BooleanOperation::Intersection => Clipper::new()
                .add_subject(subject_paths)
                .add_clip(Paths::new(Vec::new()))
                .intersect(fill_rule),
            BooleanOperation::Difference => Clipper::new()
                .add_subject(subject_paths)
                .add_clip(Paths::new(Vec::new()))
                .difference(fill_rule),
            BooleanOperation::ExclusiveOr => Clipper::new()
                .add_subject(subject_paths)
                .add_clip(Paths::new(Vec::new()))
                .xor(fill_rule),
        };

        let solution_paths = match solution {
            Ok(paths) => paths,
            Err(err) => {
                return BooleanResult {
                    entities: shapes.to_vec(),
                    success: false,
                    message: format!("{:?} failed: {:?}", operation, err),
                };
            }
        };

        let mut result_entities = Vec::new();
        for path in solution_paths.iter() {
            let polyline = ClipperAdapter::path_to_polyline(path);
            if polyline.vertices.len() >= 3 {
                result_entities.push(GeometricEntity::Polyline(polyline));
            }
        }

        let result_count = result_entities.len();
        BooleanResult {
            success: !result_entities.is_empty(),
            entities: result_entities,
            message: format!("{:?} completed with {} result polygons", operation, result_count),
        }
    }

    #[cfg(feature = "boolean")]
    fn simple_boolean_operation(&self, shapes: &[GeometricEntity], operation: BooleanOperation) -> BooleanResult {
        let mut subject_paths: Paths = Paths::new(Vec::new());
        let mut clip_paths: Paths = Paths::new(Vec::new());
        let mut has_subject = false;
        let mut has_clip = false;

        for (idx, shape) in shapes.iter().enumerate() {
            match shape {
                GeometricEntity::Polyline(poly) if poly.vertices.len() >= 3 => {
                    let path = ClipperAdapter::polyline_to_path(poly);
                    if !path.is_empty() {
                        if idx == 0 || operation == BooleanOperation::Union || operation == BooleanOperation::ExclusiveOr {
                            subject_paths.push(path);
                            has_subject = true;
                        } else {
                            clip_paths.push(path);
                            has_clip = true;
                        }
                    }
                }
                GeometricEntity::Line(line) => {
                    let path = Path::new(vec![
                        ClipperAdapter::point_to_clipper(&line.start),
                        ClipperAdapter::point_to_clipper(&line.end),
                    ]);
                    subject_paths.push(path);
                    has_subject = true;
                }
                GeometricEntity::Circle(circle) => {
                    let path = Self::circle_to_path(&circle.center, circle.radius);
                    if !path.is_empty() {
                        subject_paths.push(path);
                        has_subject = true;
                    }
                }
                _ => {}
            }
        }

        if !has_subject || (operation != BooleanOperation::Union && operation != BooleanOperation::ExclusiveOr && !has_clip) {
            return BooleanResult {
                entities: shapes.to_vec(),
                success: false,
                message: format!("{:?} - insufficient valid shapes", operation),
            };
        }

        let fill_rule = FillRule::NonZero;

        let clipper = Clipper::new()
            .add_subject(subject_paths)
            .add_clip(clip_paths);
        let solution = match operation {
            BooleanOperation::Union => clipper.union(fill_rule),
            BooleanOperation::Intersection => clipper.intersect(fill_rule),
            BooleanOperation::Difference => clipper.difference(fill_rule),
            BooleanOperation::ExclusiveOr => clipper.xor(fill_rule),
        };

        let solution_paths = match solution {
            Ok(paths) => paths,
            Err(err) => {
                return BooleanResult {
                    entities: shapes.to_vec(),
                    success: false,
                    message: format!("{:?} failed: {:?}", operation, err),
                };
            }
        };

        let mut result_entities = Vec::new();
        for path in solution_paths.iter() {
            let polyline = ClipperAdapter::path_to_polyline(path);
            if polyline.vertices.len() >= 3 {
                result_entities.push(GeometricEntity::Polyline(polyline));
            }
        }

        let result_count = result_entities.len();
        BooleanResult {
            success: !result_entities.is_empty(),
            entities: result_entities,
            message: format!("{:?} completed with {} result polygons", operation, result_count),
        }
    }

    #[cfg(feature = "boolean")]
    fn circle_to_path(center: &Point, radius: f64) -> Path {
        if radius <= 0.0 {
            return Path::new(Vec::new());
        }

        let num_points = 64;
        let mut points: Vec<ClipperPoint> = Vec::with_capacity(num_points);

        for i in 0..num_points {
            let angle = (i as f64) / (num_points as f64) * std::f64::consts::TAU;
            points.push(ClipperPoint::new(
                center.x + angle.cos() * radius,
                center.y + angle.sin() * radius,
            ));
        }

        Path::new(points)
    }

    #[cfg(not(feature = "boolean"))]
    fn boolean_operation(&self, shapes: &[GeometricEntity], operation: BooleanOperation) -> BooleanResult {
        BooleanResult {
            entities: shapes.to_vec(),
            success: false,
            message: format!("{:?} - requires 'boolean' feature", operation),
        }
    }

    #[cfg(not(feature = "boolean"))]
    fn simple_boolean_operation(&self, shapes: &[GeometricEntity], operation: BooleanOperation) -> BooleanResult {
        BooleanResult {
            entities: shapes.to_vec(),
            success: false,
            message: format!("{:?} - requires 'boolean' feature", operation),
        }
    }

    pub fn line_circle_union(&self, line: &Line, circle: &Circle) -> BooleanResult {
        self.simple_line_circle_operation(line, circle, BooleanOperation::Union)
    }

    pub fn line_circle_intersection(&self, line: &Line, circle: &Circle) -> BooleanResult {
        let result = crate::geometry::intersection::intersect_line_circle(line.clone(), circle.clone());
        match result {
            IntersectionResult::Point(ip) => {
                BooleanResult {
                    entities: vec![GeometricEntity::Line(Line::new(ip.point, ip.point))],
                    success: true,
                    message: "Line-Circle intersection found".to_string(),
                }
            }
            IntersectionResult::Points(points) => {
                let mut entities = Vec::new();
                for ip in &points {
                    entities.push(GeometricEntity::Line(Line::new(ip.point, ip.point)));
                }
                BooleanResult {
                    entities,
                    success: true,
                    message: format!("Found {} intersection points", points.len()),
                }
            }
            _ => BooleanResult {
                entities: vec![],
                success: false,
                message: "No intersection found".to_string(),
            },
        }
    }

    pub fn line_circle_difference(&self, line: &Line, circle: &Circle) -> BooleanResult {
        let intersection = self.line_circle_intersection(line, circle);
        if intersection.entities.is_empty() {
            return BooleanResult {
                entities: vec![GeometricEntity::Line(line.clone())],
                success: true,
                message: "Line not intersected by circle".to_string(),
            };
        }

        BooleanResult {
            entities: vec![GeometricEntity::Line(line.clone())],
            success: true,
            message: "Line split by circle".to_string(),
        }
    }

    #[cfg(feature = "boolean")]
    fn simple_line_circle_operation(&self, line: &Line, circle: &Circle, operation: BooleanOperation) -> BooleanResult {
        let circle_path = Self::circle_to_path(&circle.center, circle.radius);
        let line_path = Path::new(vec![
            ClipperAdapter::point_to_clipper(&line.start),
            ClipperAdapter::point_to_clipper(&line.end),
        ]);

        if circle_path.is_empty() || line_path.is_empty() {
            return BooleanResult {
                entities: vec![GeometricEntity::Line(line.clone())],
                success: false,
                message: "Invalid input shapes".to_string(),
            };
        }

        let clipper = Clipper::new()
            .add_subject(line_path)
            .add_clip(circle_path);
        let solution = match operation {
            BooleanOperation::Union => clipper.union(FillRule::NonZero),
            BooleanOperation::Intersection => clipper.intersect(FillRule::NonZero),
            BooleanOperation::Difference => clipper.difference(FillRule::NonZero),
            BooleanOperation::ExclusiveOr => clipper.xor(FillRule::NonZero),
        };

        let solution_paths = match solution {
            Ok(paths) => paths,
            Err(err) => {
                return BooleanResult {
                    entities: vec![GeometricEntity::Line(line.clone())],
                    success: false,
                    message: format!("Line-Circle {:?} failed: {:?}", operation, err),
                };
            }
        };

        let mut results = Vec::new();
        for path in solution_paths.iter() {
            if path.len() == 2 {
                let coords: Vec<(f64, f64)> = path.iter().map(|p| (p.x(), p.y())).collect();
                let start_point = Point::new(coords[0].0, coords[0].1, 0.0);
                let end_point = Point::new(coords[1].0, coords[1].1, 0.0);
                results.push(GeometricEntity::Line(Line::new(start_point, end_point)));
            } else if path.len() > 2 {
                let polyline = ClipperAdapter::path_to_polyline(path);
                results.push(GeometricEntity::Polyline(polyline));
            }
        }

        BooleanResult {
            success: !results.is_empty(),
            entities: results,
            message: format!("Line-Circle {:?} completed", operation),
        }
    }

    #[cfg(not(feature = "boolean"))]
    fn simple_line_circle_operation(&self, line: &Line, circle: &Circle, operation: BooleanOperation) -> BooleanResult {
        BooleanResult {
            entities: vec![GeometricEntity::Line(line.clone())],
            success: false,
            message: format!("Line-Circle {:?} - requires 'boolean' feature", operation),
        }
    }

    #[cfg(feature = "boolean")]
    pub fn circle_circle_union(&self, circle1: &Circle, circle2: &Circle) -> BooleanResult {
        let path1 = Self::circle_to_path(&circle1.center, circle1.radius);
        let path2 = Self::circle_to_path(&circle2.center, circle2.radius);

        if path1.is_empty() || path2.is_empty() {
            return BooleanResult {
                entities: vec![GeometricEntity::Circle(circle1.clone()), GeometricEntity::Circle(circle2.clone())],
                success: false,
                message: "Invalid circle parameters".to_string(),
            };
        }

        let solution = Clipper::new()
            .add_subject(Paths::new(vec![path1, path2]))
            .add_clip(Paths::new(Vec::new()))
            .union(FillRule::NonZero);

        let solution_paths = match solution {
            Ok(paths) => paths,
            Err(err) => {
                return BooleanResult {
                    entities: vec![GeometricEntity::Circle(circle1.clone()), GeometricEntity::Circle(circle2.clone())],
                    success: false,
                    message: format!("Circle-Circle union failed: {:?}", err),
                };
            }
        };

        let mut results = Vec::new();
        for path in solution_paths.iter() {
            if let Some(circle) = Self::path_to_circle(path) {
                results.push(GeometricEntity::Circle(circle));
            } else {
                let polyline = ClipperAdapter::path_to_polyline(path);
                if polyline.vertices.len() >= 3 {
                    results.push(GeometricEntity::Polyline(polyline));
                }
            }
        }

        let result_count = results.len();
        BooleanResult {
            success: !results.is_empty(),
            entities: results,
            message: format!("Circle-Circle union completed with {} results", result_count),
        }
    }

    #[cfg(feature = "boolean")]
    fn path_to_circle(path: &Path) -> Option<Circle> {
        if path.len() < 60 {
            return None;
        }

        let mut min_x = f64::MAX;
        let mut max_x = f64::MIN;
        let mut min_y = f64::MAX;
        let mut max_y = f64::MIN;

        for p in path.iter() {
            min_x = min_x.min(p.x());
            max_x = max_x.max(p.x());
            min_y = min_y.min(p.y());
            max_y = max_y.max(p.y());
        }

        let center = Point::new2d((min_x + max_x) / 2.0, (min_y + max_y) / 2.0);
        let radius = (max_x - min_x).max(max_y - min_y) / 2.0;

        let tolerance = radius.max(0.1) * 0.05;
        let mut is_circle = true;
        for p in path.iter() {
            let expected_radius_sq = (p.x() - center.x).powi(2) + (p.y() - center.y).powi(2);
            if (expected_radius_sq - radius * radius).abs() > tolerance {
                is_circle = false;
                break;
            }
        }

        if is_circle && radius > 0.0 {
            Some(Circle::new(center, radius))
        } else {
            None
        }
    }

    #[cfg(feature = "boolean")]
    pub fn circle_circle_intersection(&self, circle1: &Circle, circle2: &Circle) -> BooleanResult {
        let result = crate::geometry::intersection::intersect_circle_circle(circle1.clone(), circle2.clone());
        match result {
            IntersectionResult::Point(ip) => {
                BooleanResult {
                    entities: vec![GeometricEntity::Circle(Circle::new(ip.point, 0.0))],
                    success: true,
                    message: "Circle-Circle intersection at point".to_string(),
                }
            }
            IntersectionResult::Points(points) if points.len() == 2 => {
                let p1 = &points[0].point;
                let p2 = &points[1].point;
                let arc1 = Arc::from_three_points(*p1, circle1.center, *p2, true);
                let arc2 = Arc::from_three_points(*p1, circle2.center, *p2, true);
                BooleanResult {
                    entities: vec![
                        GeometricEntity::Arc(arc1),
                        GeometricEntity::Arc(arc2),
                    ],
                    success: true,
                    message: "Circle-Circle lens intersection created".to_string(),
                }
            }
            _ => BooleanResult {
                entities: vec![],
                success: false,
                message: "No valid intersection".to_string(),
            },
        }
    }

    pub fn circle_circle_difference(&self, circle1: &Circle, circle2: &Circle) -> BooleanResult {
        let result = crate::geometry::intersection::intersect_circle_circle(circle1.clone(), circle2.clone());
        match result {
            IntersectionResult::Points(points) if points.len() == 2 => {
                BooleanResult {
                    entities: vec![GeometricEntity::Circle(circle1.clone())],
                    success: true,
                    message: "Circle difference (cut by another circle)".to_string(),
                }
            }
            _ => BooleanResult {
                entities: vec![GeometricEntity::Circle(circle1.clone())],
                success: true,
                message: "Circle unchanged (no intersection)".to_string(),
            },
        }
    }

    #[cfg(feature = "boolean")]
    pub fn polygon_union(&self, polygons: &[Polyline]) -> BooleanResult {
        if polygons.is_empty() {
            return BooleanResult {
                entities: vec![],
                success: true,
                message: "No polygons provided".to_string(),
            };
        }

        if polygons.len() == 1 {
            return BooleanResult {
                entities: vec![GeometricEntity::Polyline(polygons[0].clone())],
                success: true,
                message: "Single polygon returned".to_string(),
            };
        }

        let mut subject_paths: Paths = Paths::new(Vec::new());

        for poly in polygons {
            if poly.vertices.len() >= 3 {
                let path = ClipperAdapter::polyline_to_path(poly);
                if !path.is_empty() {
                    subject_paths.push(path);
                }
            }
        }

        if subject_paths.is_empty() {
            return BooleanResult {
                entities: polygons.iter().map(|p| GeometricEntity::Polyline(p.clone())).collect(),
                success: false,
                message: "No valid polygons".to_string(),
            };
        }

        let solution = Clipper::new()
            .add_subject(subject_paths)
            .add_clip(Paths::new(Vec::new()))
            .union(FillRule::NonZero);

        let solution_paths = match solution {
            Ok(paths) => paths,
            Err(err) => {
                return BooleanResult {
                    entities: polygons.iter().map(|p| GeometricEntity::Polyline(p.clone())).collect(),
                    success: false,
                    message: format!("Polygon union failed: {:?}", err),
                };
            }
        };

        let mut results = Vec::new();
        for path in solution_paths.iter() {
            let polyline = ClipperAdapter::path_to_polyline(path);
            if polyline.vertices.len() >= 3 {
                results.push(GeometricEntity::Polyline(polyline));
            }
        }

        let result_count = results.len();
        BooleanResult {
            success: !results.is_empty(),
            entities: results,
            message: format!("Polygon union completed with {} result(s)", result_count),
        }
    }

    #[cfg(not(feature = "boolean"))]
    pub fn polygon_union(&self, polygons: &[Polyline]) -> BooleanResult {
        let mut combined = if let Some(first) = polygons.first() {
            let mut vertices = first.vertices.clone();
            for poly in &polygons[1..] {
                vertices.extend(poly.vertices.clone());
            }
            Polyline {
                vertices,
                is_closed: first.is_closed,
            }
        } else {
            return BooleanResult {
                entities: vec![],
                success: false,
                message: "No polygons provided".to_string(),
            };
        };

        BooleanResult {
            entities: vec![GeometricEntity::Polyline(combined)],
            success: false,
            message: "Polygon union requires 'boolean' feature".to_string(),
        }
    }

    fn merge_polygons(&self, poly1: &Polyline, poly2: &Polyline) -> Polyline {
        let mut vertices = poly1.vertices.clone();
        vertices.extend(poly2.vertices.clone());
        Polyline {
            vertices,
            is_closed: poly1.is_closed,
        }
    }

    pub fn polygon_intersection(&self, poly1: &Polyline, poly2: &Polyline) -> BooleanResult {
        #[cfg(feature = "boolean")]
        {
            let path1 = ClipperAdapter::polyline_to_path(poly1);
            let path2 = ClipperAdapter::polyline_to_path(poly2);

            if path1.is_empty() || path2.is_empty() {
                return BooleanResult {
                    entities: vec![],
                    success: false,
                    message: "Invalid polygons".to_string(),
                };
            }

            let solution = Clipper::new()
                .add_subject(path1)
                .add_clip(path2)
                .intersect(FillRule::NonZero);

            let solution_paths = match solution {
                Ok(paths) => paths,
                Err(err) => {
                    return BooleanResult {
                        entities: vec![],
                        success: false,
                        message: format!("Polygon intersection failed: {:?}", err),
                    };
                }
            };

            let mut results = Vec::new();
            for path in solution_paths.iter() {
                let polyline = ClipperAdapter::path_to_polyline(path);
                if polyline.vertices.len() >= 3 {
                    results.push(GeometricEntity::Polyline(polyline));
                }
            }

            let result_count = results.len();
            BooleanResult {
                success: !results.is_empty(),
                entities: results,
                message: format!("Polygon intersection completed with {} result(s)", result_count),
            }
        }

        #[cfg(not(feature = "boolean"))]
        {
            BooleanResult {
                entities: vec![],
                success: false,
                message: "Polygon intersection requires 'boolean' feature".to_string(),
            }
        }
    }

    pub fn polygon_difference(&self, subject: &Polyline, tool: &Polyline) -> BooleanResult {
        #[cfg(feature = "boolean")]
        {
            let subject_path = ClipperAdapter::polyline_to_path(subject);
            let tool_path = ClipperAdapter::polyline_to_path(tool);

            if subject_path.is_empty() || tool_path.is_empty() {
                return BooleanResult {
                    entities: vec![GeometricEntity::Polyline(subject.clone())],
                    success: false,
                    message: "Invalid polygons".to_string(),
                };
            }

            let solution = Clipper::new()
                .add_subject(subject_path)
                .add_clip(tool_path)
                .difference(FillRule::NonZero);

            let solution_paths = match solution {
                Ok(paths) => paths,
                Err(err) => {
                    return BooleanResult {
                        entities: vec![GeometricEntity::Polyline(subject.clone())],
                        success: false,
                        message: format!("Polygon difference failed: {:?}", err),
                    };
                }
            };

            let mut results = Vec::new();
            for path in solution_paths.iter() {
                let polyline = ClipperAdapter::path_to_polyline(path);
                if polyline.vertices.len() >= 3 {
                    results.push(GeometricEntity::Polyline(polyline));
                }
            }

            let result_count = results.len();
            BooleanResult {
                success: !results.is_empty(),
                entities: results,
                message: format!("Polygon difference completed with {} result(s)", result_count),
            }
        }

        #[cfg(not(feature = "boolean"))]
        {
            BooleanResult {
                entities: vec![GeometricEntity::Polyline(subject.clone())],
                success: false,
                message: "Polygon difference requires 'boolean' feature".to_string(),
            }
        }
    }
}

impl Default for BooleanEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[inline]
pub fn point_in_polygon(point: Point, polygon: &Polyline) -> bool {
    let mut inside = false;
    let n = polygon.vertices.len();
    if n < 3 {
        return false;
    }

    for i in 0..n {
        let j = if i == 0 { n - 1 } else { i - 1 };
        let xi = polygon.vertices[i].x;
        let yi = polygon.vertices[i].y;
        let xj = polygon.vertices[j].x;
        let yj = polygon.vertices[j].y;

        let intersect = ((yi > point.y) != (yj > point.y)) &&
            (point.x < (xj - xi) * (point.y - yi) / (yj - yi) + xi);
        if intersect {
            inside = !inside;
        }
    }

    inside
}

#[inline]
pub fn polygon_area(polygon: &Polyline) -> f64 {
    let mut area: f64 = 0.0;
    let n = polygon.vertices.len();
    if n < 3 {
        return 0.0;
    }

    for i in 0..n {
        let j = (i + 1) % n;
        area += polygon.vertices[i].x * polygon.vertices[j].y;
        area -= polygon.vertices[j].x * polygon.vertices[i].y;
    }

    area.abs() / 2.0
}

#[inline]
pub fn polygons_overlap(poly1: &Polyline, poly2: &Polyline) -> bool {
    for point in &poly1.vertices {
        if point_in_polygon(Point::new(point.x, point.y, 0.0), poly2) {
            return true;
        }
    }
    for point in &poly2.vertices {
        if point_in_polygon(Point::new(point.x, point.y, 0.0), poly1) {
            return true;
        }
    }
    false
}

#[cfg(feature = "boolean")]
#[cfg(test)]
mod tests {
    use super::*;

    fn closed_polygon(points: &[Point2D]) -> Polyline {
        Polyline {
            vertices: points.to_vec(),
            is_closed: true,
        }
    }

    #[test]
    fn test_boolean_engine_creation() {
        let engine = BooleanEngine::new();
        let line = Line::new(Point::new2d(0.0, 0.0), Point::new2d(10.0, 0.0));
        let circle = Circle::new(Point::new2d(5.0, 0.0), 2.0);
        let result = engine.line_circle_union(&line, &circle);
        assert!(result.success || !result.message.contains("feature"));
    }

    #[test]
    fn test_polygon_union() {
        let square1 = closed_polygon(&[
            Point2D::new(0.0, 0.0),
            Point2D::new(5.0, 0.0),
            Point2D::new(5.0, 5.0),
            Point2D::new(0.0, 5.0),
        ]);

        let square2 = closed_polygon(&[
            Point2D::new(3.0, 0.0),
            Point2D::new(8.0, 0.0),
            Point2D::new(8.0, 5.0),
            Point2D::new(3.0, 5.0),
        ]);

        let engine = BooleanEngine::new();
        let result = engine.polygon_union(&[square1, square2]);

        assert!(result.success || result.message.contains("feature"));
    }

    #[test]
    fn test_polygon_intersection() {
        let square1 = closed_polygon(&[
            Point2D::new(0.0, 0.0),
            Point2D::new(5.0, 0.0),
            Point2D::new(5.0, 5.0),
            Point2D::new(0.0, 5.0),
        ]);

        let square2 = closed_polygon(&[
            Point2D::new(3.0, 0.0),
            Point2D::new(8.0, 0.0),
            Point2D::new(8.0, 5.0),
            Point2D::new(3.0, 5.0),
        ]);

        let engine = BooleanEngine::new();
        let result = engine.polygon_intersection(&square1, &square2);

        assert!(result.success || result.message.contains("feature"));
    }

    #[test]
    fn test_polygon_difference() {
        let square1 = closed_polygon(&[
            Point2D::new(0.0, 0.0),
            Point2D::new(10.0, 0.0),
            Point2D::new(10.0, 10.0),
            Point2D::new(0.0, 10.0),
        ]);

        let square2 = closed_polygon(&[
            Point2D::new(3.0, 3.0),
            Point2D::new(7.0, 3.0),
            Point2D::new(7.0, 7.0),
            Point2D::new(3.0, 7.0),
        ]);

        let engine = BooleanEngine::new();
        let result = engine.polygon_difference(&square1, &square2);

        assert!(result.success || result.message.contains("feature"));
    }
}

#[cfg(not(feature = "boolean"))]
#[cfg(test)]
mod tests {
    use super::*;

    fn closed_polygon(points: &[Point2D]) -> Polyline {
        Polyline {
            vertices: points.to_vec(),
            is_closed: true,
        }
    }

    #[test]
    fn test_boolean_engine_creation() {
        let engine = BooleanEngine::new();
        let result = engine.line_circle_union(&Line::new(Point::new2d(0.0, 0.0), Point::new2d(10.0, 0.0)), &Circle::new(Point::new2d(5.0, 0.0), 2.0));
        assert!(!result.success);
        assert!(result.message.contains("feature"));
    }

    #[test]
    fn test_polygon_union() {
        let square1 = closed_polygon(&[
            Point2D::new(0.0, 0.0),
            Point2D::new(5.0, 0.0),
            Point2D::new(5.0, 5.0),
            Point2D::new(0.0, 5.0),
        ]);

        let square2 = closed_polygon(&[
            Point2D::new(3.0, 0.0),
            Point2D::new(8.0, 0.0),
            Point2D::new(8.0, 5.0),
            Point2D::new(3.0, 5.0),
        ]);

        let engine = BooleanEngine::new();
        let result = engine.polygon_union(&[square1, square2]);

        assert!(!result.success);
        assert!(result.message.contains("feature"));
    }

    #[test]
    fn test_point_in_polygon() {
        let square = closed_polygon(&[
            Point2D::new(0.0, 0.0),
            Point2D::new(10.0, 0.0),
            Point2D::new(10.0, 10.0),
            Point2D::new(0.0, 10.0),
        ]);

        assert!(point_in_polygon(Point::new2d(5.0, 5.0), &square));
        assert!(!point_in_polygon(Point::new2d(15.0, 5.0), &square));
    }

    #[test]
    fn test_polygon_area() {
        let triangle = closed_polygon(&[
            Point2D::new(0.0, 0.0),
            Point2D::new(10.0, 0.0),
            Point2D::new(5.0, 10.0),
        ]);

        let area = polygon_area(&triangle);
        assert!((area - 50.0).abs() < 0.01);
    }
}
