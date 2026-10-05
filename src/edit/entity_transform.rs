//! 实体几何变换：使用 math::Transform2D（平移/旋转/缩放）与镜像反射。
//! 直接修改实体几何（世界坐标），与 edit::transformation 中基于元数据的工具互补。

use crate::data_structure::{Entity, EntityGeometry};
use crate::geometry::{BSpline, NURBS, Point};
use crate::math::{Transform2D, Vector2};

/// 绕点旋转的 Transform2D：平移 = c - R(c)
pub fn rotate_about(center: Point, angle: f64) -> Transform2D {
    let (s, c) = angle.sin_cos();
    Transform2D {
        translation: Vector2::new(center.x - (center.x * c - center.y * s), center.y - (center.x * s + center.y * c)),
        rotation: angle,
        scale: Vector2::new(1.0, 1.0),
    }
}

/// 以点为基准缩放的 Transform2D：平移 = c(1-k)
pub fn scale_about(center: Point, k: f64) -> Transform2D {
    Transform2D {
        translation: Vector2::new(center.x * (1.0 - k), center.y * (1.0 - k)),
        rotation: 0.0,
        scale: Vector2::new(k, k),
    }
}

fn tp(p: Point, t: &Transform2D) -> Point {
    let v = t.apply(&Vector2::new(p.x, p.y));
    Point::new2d(v.x, v.y)
}

fn reflect(p: Point, p1: Point, p2: Point) -> Point {
    let dx = p2.x - p1.x;
    let dy = p2.y - p1.y;
    let len_sq = dx * dx + dy * dy;
    if len_sq < 1e-12 {
        return p;
    }
    let t = ((p.x - p1.x) * dx + (p.y - p1.y) * dy) / len_sq;
    let proj = Point::new2d(p1.x + t * dx, p1.y + t * dy);
    Point::new2d(2.0 * proj.x - p.x, 2.0 * proj.y - p.y)
}

/// 镜像角：关于由 axis 角定义的直线反射
fn reflect_angle(theta: f64, axis: f64) -> f64 {
    2.0 * axis - theta
}

/// 用 Transform2D 变换实体（平移 / 旋转 / 缩放，旋转+缩放为均匀缩放）。
pub fn transform_entity(entity: &mut Entity, t: &Transform2D) {
    let rot = t.rotation;
    let sc = t.scale.x;
    apply_affine(entity, &mut |p| tp(p, t), rot, sc, false);
}

/// 关于直线 p1→p2 镜像实体
pub fn mirror_entity(entity: &mut Entity, p1: Point, p2: Point) {
    let axis = (p2.y - p1.y).atan2(p2.x - p1.x);
    apply_affine(entity, &mut |p| reflect(p, p1, p2), 0.0, 1.0, true);
    // 镜像角修正
    if let EntityGeometry::Ellipse(e) = entity.geometry_mut() {
        e.rotation = reflect_angle(e.rotation, axis);
    }
    if let EntityGeometry::Text { rotation, .. } = entity.geometry_mut() {
        *rotation = reflect_angle(*rotation, axis) + std::f64::consts::PI;
    }
    if let EntityGeometry::Dimension {
        text_rotation,
        angle,
        ..
    } = entity.geometry_mut()
    {
        *text_rotation = reflect_angle(*text_rotation, axis) + std::f64::consts::PI;
        *angle = reflect_angle(*angle, axis);
    }
    if let EntityGeometry::Arc(a) = entity.geometry_mut() {
        // 反射翻转方向：交换起止角并保持逆时针
        let (s, e) = (reflect_angle(a.start_angle, axis), reflect_angle(a.end_angle, axis));
        a.start_angle = e;
        a.end_angle = s;
        a.is_counter_clockwise = true;
    }
}

/// 通用仿射应用：point_fn 变换每个点；rot/sc 用于半径、角度与文字高度；mirror 时文字角由调用方修正
fn apply_affine(
    entity: &mut Entity,
    point_fn: &mut impl FnMut(Point) -> Point,
    rot: f64,
    sc: f64,
    _mirror: bool,
) {
    match entity.geometry_mut() {
        EntityGeometry::Line(l) => {
            l.start = point_fn(l.start);
            l.end = point_fn(l.end);
        }
        EntityGeometry::Circle(c) => {
            c.center = point_fn(c.center);
            c.radius *= sc;
        }
        EntityGeometry::Arc(a) => {
            a.center = point_fn(a.center);
            a.radius *= sc;
            a.start_angle += rot;
            a.end_angle += rot;
        }
        EntityGeometry::Ellipse(e) => {
            e.center = point_fn(e.center);
            e.semi_major *= sc;
            e.semi_minor *= sc;
            e.rotation += rot;
        }
        EntityGeometry::Polyline(p) => {
            for v in &mut p.vertices {
                let q = point_fn(Point::new2d(v.x, v.y));
                v.x = q.x;
                v.y = q.y;
            }
        }
        EntityGeometry::BSpline(b) => {
            let pts: Vec<Point> = b.control_points().iter().map(|p| point_fn(*p)).collect();
            let knots: Vec<f64> = b.knots().to_vec();
            let degree = b.degree();
            *b = BSpline::new(pts, knots, degree);
        }
        EntityGeometry::NURBS(n) => {
            let pts: Vec<Point> = n.control_points().iter().map(|p| point_fn(*p)).collect();
            let weights: Vec<f64> = n.weights().to_vec();
            let knots: Vec<f64> = n.knots().to_vec();
            let degree = n.degree();
            *n = NURBS::new(pts, weights, knots, degree);
        }
        EntityGeometry::Point(p) => *p = point_fn(*p),
        EntityGeometry::Text {
            position,
            height,
            rotation,
            ..
        } => {
            *position = point_fn(*position);
            *height *= sc;
            *rotation += rot;
        }
        EntityGeometry::Dimension {
            text_position,
            text_height,
            text_rotation,
            definition_point,
            def_point_1,
            def_point_2,
            def_point_3,
            def_point_4,
            angle,
            ..
        } => {
            *text_position = point_fn(*text_position);
            *definition_point = point_fn(*definition_point);
            *def_point_1 = point_fn(*def_point_1);
            *def_point_2 = point_fn(*def_point_2);
            *def_point_3 = point_fn(*def_point_3);
            *def_point_4 = point_fn(*def_point_4);
            *text_height *= sc;
            *text_rotation += rot;
            *angle += rot;
        }
        EntityGeometry::Solid { points, .. } => {
            for p in points.iter_mut() {
                *p = point_fn(*p);
            }
        }
        EntityGeometry::Hatch {
            boundary_paths, ..
        } => {
            for bp in boundary_paths.iter_mut() {
                for e in bp.edges.iter_mut() {
                    e.start_point = point_fn(e.start_point);
                    e.end_point = point_fn(e.end_point);
                    if let Some(c) = e.center_point {
                        e.center_point = Some(point_fn(c));
                    }
                }
            }
        }
        _ => {}
    }
}
