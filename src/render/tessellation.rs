//! 实体细分：把各类实体几何展开为折线集合与填充多边形，并计算文档包围盒。
//! 标注与文字实体分解为图形笔画（含矢量文字），从而被渲染与所有导出路径复用。

use crate::data_structure::{Document, Entity, EntityGeometry};
use crate::dimension::strokes;
use crate::geometry::{Arc, BSpline, Circle, Ellipse, NURBS, Point, Polyline};

// ---------------- 细分 ----------------

pub fn circle_points(center: Point, radius: f64, segments: usize) -> Vec<Point> {
    (0..=segments)
        .map(|i| {
            let a = 2.0 * std::f64::consts::PI * i as f64 / segments as f64;
            Point::new2d(center.x + radius * a.cos(), center.y + radius * a.sin())
        })
        .collect()
}

/// 圆弧细分为折线点（按弧线方向采样）
pub fn arc_points(arc: &Arc, segments: usize) -> Vec<Point> {
    let span = arc.angle_span();
    let signed = if arc.is_counter_clockwise { span } else { -span };
    (0..=segments)
        .map(|i| {
            arc.point_at_angle(arc.start_angle + signed * i as f64 / segments as f64)
        })
        .collect()
}

/// 圆弧的建议细分段数
pub fn arc_segments(arc: &Arc) -> usize {
    ((arc.angle_span() / 0.08).ceil() as usize).clamp(8, 180)
}

pub fn ellipse_points(e: &Ellipse, segments: usize) -> Vec<Point> {
    (0..=segments)
        .map(|i| e.point_at_parameter(i as f64 / segments as f64))
        .collect()
}

fn curve_samples(points: &[Point]) -> usize {
    // 采样密度随控制点数增长
    (points.len() * 24).clamp(48, 480)
}

pub fn bspline_points(s: &BSpline, samples: usize) -> Vec<Point> {
    (0..=samples)
        .map(|i| s.point_at(i as f64 / samples as f64))
        .collect()
}

pub fn nurbs_points(n: &NURBS, samples: usize) -> Vec<Point> {
    (0..=samples)
        .map(|i| n.point_at(i as f64 / samples as f64))
        .collect()
}

/// 实体几何展开为折线集合（世界坐标）。返回 (点列, 是否闭合)，不支持的类型返回空。
pub fn geometry_polylines(geometry: &EntityGeometry) -> Vec<(Vec<Point>, bool)> {
    match geometry {
        EntityGeometry::Line(l) => vec![(vec![l.start, l.end], false)],
        EntityGeometry::Circle(c) => vec![(circle_points(c.center, c.radius, 72), true)],
        EntityGeometry::Arc(a) => vec![(arc_points(a, arc_segments(a)), false)],
        EntityGeometry::Ellipse(e) => vec![(ellipse_points(e, 96), true)],
        EntityGeometry::Polyline(p) => {
            let pts: Vec<Point> = p
                .vertices
                .iter()
                .map(|v| Point::new2d(v.x, v.y))
                .collect();
            let closed = p.is_closed && pts.len() > 2;
            vec![(pts, closed)]
        }
        EntityGeometry::BSpline(s) => {
            let pts = bspline_points(s, curve_samples(s.control_points()));
            vec![(pts, false)]
        }
        EntityGeometry::NURBS(n) => {
            let pts = nurbs_points(n, curve_samples(n.control_points()));
            vec![(pts, false)]
        }
        EntityGeometry::Dimension { .. } => strokes::decompose(geometry),
        EntityGeometry::Text {
            content,
            position,
            height,
            rotation,
            ..
        } => strokes::text_strokes(content, *position, *height, *rotation),
        _ => Vec::new(),
    }
}

/// 实体展开为折线集合（世界坐标）。返回 (点列, 是否闭合)，不支持的类型返回空。
/// 标注与文字实体分解为图形笔画（含矢量文字），从而被渲染与所有导出路径复用。
pub fn entity_polylines(entity: &Entity) -> Vec<(Vec<Point>, bool)> {
    geometry_polylines(entity.geometry())
}

/// 实体几何的填充多边形（世界坐标）与 RGB 颜色。支持 Solid 与实心 Hatch。
pub fn geometry_fills(geometry: &EntityGeometry) -> Vec<(Vec<Point>, (u8, u8, u8))> {
    match geometry {
        EntityGeometry::Solid { points, color } => {
            let pts: Vec<Point> = points.to_vec();
            if pts.len() >= 3 {
                vec![(pts, *color)]
            } else {
                Vec::new()
            }
        }
        EntityGeometry::Hatch {
            solid_fill,
            fill_color,
            boundary_paths,
            ..
        } => {
            if !*solid_fill {
                return Vec::new();
            }
            boundary_paths
                .iter()
                .filter(|bp| bp.is_polyline)
                .filter_map(|bp| {
                    let pts: Vec<Point> = bp.edges.iter().map(|e| e.start_point).collect();
                    (pts.len() >= 3).then(|| (pts, *fill_color))
                })
                .collect()
        }
        _ => Vec::new(),
    }
}

/// 实体填充多边形（世界坐标）与 RGB 颜色。支持 Solid 与实心 Hatch。
pub fn entity_fills(entity: &Entity) -> Vec<(Vec<Point>, (u8, u8, u8))> {
    geometry_fills(entity.geometry())
}

// ---------------- 包围盒 ----------------

/// 全文档包围盒（支持所有可细分实体）
pub fn document_bbox(doc: &Document) -> Option<(Point, Point)> {
    let mut result: Option<(Point, Point)> = None;
    let mut acc = |pts: Vec<Point>| {
        for p in pts {
            result = Some(match result {
                None => (p, p),
                Some((min, max)) => (
                    Point::new2d(min.x.min(p.x), min.y.min(p.y)),
                    Point::new2d(max.x.max(p.x), max.y.max(p.y)),
                ),
            });
        }
    };
    for entity in doc.entities().values() {
        for (pts, _) in entity_polylines(entity) {
            acc(pts);
        }
        for (pts, _) in entity_fills(entity) {
            acc(pts);
        }
    }
    result
}
