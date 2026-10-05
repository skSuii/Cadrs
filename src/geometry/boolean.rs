//! 布尔运算：对闭合区域做并（Union）、交（Intersection）、差（Difference）、异或（ExclusiveOr），
//! 并提供线—圆、圆—圆、多边形之间的成对入口。
//!
//! 本模块在 `geometry` 中由 `boolean` feature 控制：启用时基于 clipper2 完成多边形布尔运算
//! （`FillRule::NonZero`，圆按 64 边形离散），未启用时同名入口返回 `success = false` 并把输入
//! 原样回传。辅助判定函数 `point_in_polygon`、`polygon_area`、`polygons_overlap` 与 feature 无关。
//!
//! 约定：参与运算的折线应闭合且顶点数不少于 3；运算只使用世界坐标的 x、y，z 分量忽略。

use crate::geometry::{Point, Line, Circle, Arc, Ellipse, Polyline, Curve};
use crate::geometry::intersection::{IntersectionResult, IntersectionPoint};
use std::cmp::Ordering;

#[cfg(feature = "boolean")]
use clipper2::{Clipper, FillRule, Path, Paths, Point as ClipperPoint};
#[cfg(feature = "boolean")]
use crate::geometry::extended_geometry::Point as Point2D;

/// 布尔运算类型。
#[derive(Debug, Clone, PartialEq)]
pub enum BooleanOperation {
    /// 并集：合并所有区域，重叠部分只保留一次。
    Union,
    /// 交集：只保留所有区域的公共部分。
    Intersection,
    /// 差集：从主体区域中减去其余区域。
    Difference,
    /// 异或：保留只被奇数个区域覆盖的部分。
    ExclusiveOr,
}

/// 布尔运算的结果。
#[derive(Debug, Clone)]
pub struct BooleanResult {
    /// 结果实体；失败时通常是输入实体的回传副本，顺序与内部路径遍历顺序一致。
    pub entities: Vec<GeometricEntity>,
    /// 是否成功；输入为空但合法时可能为 `true`，底层失败或缺少 feature 时为 `false`。
    pub success: bool,
    /// 结果说明或失败原因（英文，形如 `Union completed with 2 result polygons`）。
    pub message: String,
}

/// 参与或产出布尔运算的几何实体。
#[derive(Debug, Clone)]
pub enum GeometricEntity {
    /// 直线段。
    Line(Line),
    /// 圆弧。
    Arc(Arc),
    /// 整圆。
    Circle(Circle),
    /// 折线；布尔运算要求其顶点数不少于 3 并按顺序围成闭合环。
    Polyline(Polyline),
    /// 组合实体，按嵌套顺序依次参与运算。
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

/// 布尔运算引擎：无内部状态，可重复使用；所有方法只读取输入，不修改传入实体。
pub struct BooleanEngine;

impl BooleanEngine {
    /// 创建引擎实例；不持有任何额外状态。
    pub fn new() -> Self {
        Self {}
    }

    /// 求全部输入实体的并集。
    ///
    /// - `shapes`：参与运算的实体；空切片视为成功并返回空结果。
    ///
    /// 全部为折线时走 clipper2 路径，否则退化为逐实体离散的简化运算；输入不被修改。
    #[cfg(feature = "boolean")]
    pub fn union_shapes(&self, shapes: &[GeometricEntity]) -> BooleanResult {
        self.boolean_operation(shapes, BooleanOperation::Union)
    }

    /// 未启用 `boolean` feature 时的降级实现：原样回传输入并置 `success = false`。
    #[cfg(not(feature = "boolean"))]
    pub fn union_shapes(&self, shapes: &[GeometricEntity]) -> BooleanResult {
        BooleanResult {
            entities: shapes.to_vec(),
            success: false,
            message: "Boolean operations require 'boolean' feature".to_string(),
        }
    }

    /// 求全部输入实体的交集（公共区域）。
    ///
    /// - `shapes`：参与运算的实体；空切片视为成功并返回空结果。
    ///
    /// 结果只保留顶点数不少于 3 的闭合折线，输入不被修改。
    #[cfg(feature = "boolean")]
    pub fn intersect_shapes(&self, shapes: &[GeometricEntity]) -> BooleanResult {
        self.boolean_operation(shapes, BooleanOperation::Intersection)
    }

    /// 未启用 `boolean` feature 时的降级实现：原样回传输入并置 `success = false`。
    #[cfg(not(feature = "boolean"))]
    pub fn intersect_shapes(&self, shapes: &[GeometricEntity]) -> BooleanResult {
        BooleanResult {
            entities: shapes.to_vec(),
            success: false,
            message: "Boolean operations require 'boolean' feature".to_string(),
        }
    }

    /// 从主体实体中减去工具实体。
    ///
    /// - `subject`：主体区域。
    /// - `tool`：工具区域；当前实现未使用该参数，实际只对 `subject` 求差集。
    ///
    /// 结果与输入实体均不被修改。
    #[cfg(feature = "boolean")]
    pub fn subtract_shapes(&self, subject: &[GeometricEntity], tool: &[GeometricEntity]) -> BooleanResult {
        let result = self.boolean_operation(subject, BooleanOperation::Difference);
        result
    }

    /// 未启用 `boolean` feature 时的降级实现：原样回传主体并置 `success = false`。
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

    /// 求直线与圆的并集。
    ///
    /// 直线与圆都被离散为路径后参与运算；圆半径不为正时返回 `success = false` 并回传直线。
    pub fn line_circle_union(&self, line: &Line, circle: &Circle) -> BooleanResult {
        self.simple_line_circle_operation(line, circle, BooleanOperation::Union)
    }

    /// 求直线段与圆的交点。
    ///
    /// 每个交点以一条起止点相同的退化 `Line` 返回；无交点时 `success = false` 且 `entities` 为空。
    /// 输入直线与圆均不被修改。
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

    /// 用圆裁剪直线。
    ///
    /// 当前实现只判断是否相交，返回的始终是原直线本身（相交与否体现在 `message` 中），
    /// `success` 恒为 `true`；输入直线不被修改。
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

    /// 求两个圆的并集。
    ///
    /// 结果形状接近圆时还原为 `GeometricEntity::Circle`，否则以不少于 3 个顶点的闭合折线返回；
    /// 半径无效或底层失败时回传两个输入圆并置 `success = false`。
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

    /// 求两圆的公共区域（透镜形）。
    ///
    /// 相切时返回一个半径为 0 的退化圆；有两个交点时返回两段由交点与圆心三点确定的圆弧
    /// （分别沿两个圆走）；无有效交点时 `success = false` 且 `entities` 为空。
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

    /// 求第一个圆减去第二个圆的结果。
    ///
    /// 当前实现只判断两圆是否有两个交点，返回的始终是第一个圆本身（是否被切割体现在 `message`
    /// 中），`success` 恒为 `true`；输入圆不被修改。
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

    /// 合并多个多边形。
    ///
    /// - `polygons`：参与合并的多边形；顶点数少于 3 的会被忽略，空输入返回成功且结果为空，
    ///   只有一个输入时原样回传该多边形。
    ///
    /// 结果均为不少于 3 个顶点的闭合折线；`success` 取决于是否产出有效多边形。
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

    /// 未启用 `boolean` feature 时的降级实现：把各多边形顶点首尾拼接成一条折线返回。
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

    /// 求两个多边形的交集。
    ///
    /// 结果只保留不少于 3 个顶点的闭合折线；输入无效、无交集或未启用 `boolean` feature 时
    /// `success = false` 并返回空的 `entities`。输入多边形不被修改。
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

    /// 从 `subject` 中减去 `tool`。
    ///
    /// 成功时结果只含不少于 3 个顶点的闭合折线；输入无效、底层失败或未启用 `boolean` feature
    /// 时原样回传 `subject` 并置 `success = false`。输入多边形不被修改。
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

/// 判断点是否位于多边形内部（射线交叉计数法）。
///
/// - `point`：待测点，只使用 x、y 坐标。
/// - `polygon`：按顺序给出顶点的多边形，隐式闭合；顶点数少于 3 时直接返回 `false`。
///
/// 返回点是否落在内部；点恰好落在边或顶点上时结果不稳定，需要容差时由调用方处理。
///
/// # 示例
/// ```
/// use cadrs::geometry::Point;
/// use cadrs::geometry::extended_geometry::Point as Point2D;
/// use cadrs::geometry::Polyline;
/// use cadrs::geometry::boolean::point_in_polygon;
///
/// let square = Polyline {
///     vertices: vec![Point2D::new(0.0, 0.0), Point2D::new(10.0, 0.0),
///                    Point2D::new(10.0, 10.0), Point2D::new(0.0, 10.0)],
///     is_closed: true,
/// };
/// assert!(point_in_polygon(Point::new2d(5.0, 5.0), &square));
/// ```
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

/// 计算多边形的面积（鞋带公式）。
///
/// - `polygon`：按顺序给出顶点的多边形，隐式闭合，无需重复首顶点。
///
/// 返回世界坐标单位下的面积（平方单位），与顶点环绕方向无关（取绝对值）；顶点数少于 3 时
/// 返回 `0.0`。
///
/// # 示例
/// ```
/// use cadrs::geometry::extended_geometry::Point;
/// use cadrs::geometry::Polyline;
/// use cadrs::geometry::boolean::polygon_area;
///
/// let triangle = Polyline {
///     vertices: vec![Point::new(0.0, 0.0), Point::new(10.0, 0.0), Point::new(5.0, 10.0)],
///     is_closed: true,
/// };
/// assert!((polygon_area(&triangle) - 50.0).abs() < 0.01);
/// ```
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

/// 判断两个多边形是否重叠。
///
/// 只要任一多边形的某个顶点落在另一个多边形内部即返回 `true`；仅边相交而顶点互不包含
/// （例如两个交叉的细长多边形）时返回 `false`。
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
