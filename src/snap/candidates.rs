//! 对象捕捉候选点：端点 / 中点 / 圆心 / 交点。
//! 世界坐标候选生成；屏幕空间半径过滤由上层完成。

use crate::data_structure::{Entity, EntityGeometry};
use crate::geometry::intersection::{intersect_line_line, IntersectionResult};
use crate::geometry::{Line, Point};
use crate::render::tessellation::entity_polylines;

use super::snap_point::SnapType;

/// 实体的捕捉候选点（世界坐标）：直线端点/中点、圆/椭圆/圆弧圆心、圆弧端点、多段线首尾
pub fn snap_candidates(entity: &Entity) -> Vec<(Point, SnapType)> {
    let mut out = Vec::new();
    let mut push = |p: Point, kind: SnapType| out.push((p, kind));
    match entity.geometry() {
        EntityGeometry::Line(l) => {
            push(l.start, SnapType::EndPoint);
            push(l.end, SnapType::EndPoint);
            push(
                Point::new2d((l.start.x + l.end.x) / 2.0, (l.start.y + l.end.y) / 2.0),
                SnapType::MidPoint,
            );
        }
        EntityGeometry::Circle(c) => push(c.center, SnapType::Center),
        EntityGeometry::Arc(a) => {
            push(a.center, SnapType::Center);
            push(a.start_point(), SnapType::EndPoint);
            push(a.end_point(), SnapType::EndPoint);
        }
        EntityGeometry::Ellipse(e) => push(e.center, SnapType::Center),
        EntityGeometry::Polyline(p) => {
            if let (Some(f), Some(l)) = (p.vertices.first(), p.vertices.last()) {
                push(Point::new2d(f.x, f.y), SnapType::EndPoint);
                push(Point::new2d(l.x, l.y), SnapType::EndPoint);
            }
        }
        _ => {}
    }
    out
}

/// 实体细分线段（限制数量，用于交点计算）
pub fn entity_segments_capped(entity: &Entity, cap: usize) -> Vec<(Point, Point)> {
    let mut segs: Vec<(Point, Point)> = Vec::new();
    for (pts, _) in entity_polylines(entity) {
        let n = pts.len();
        if n < 2 {
            continue;
        }
        let step = ((n - 1) / cap).max(1);
        let mut i = 0;
        while i < n - 1 {
            let j = (i + step).min(n - 1);
            segs.push((pts[i], pts[j]));
            i = j;
        }
        if segs.len() > cap * 2 {
            break;
        }
    }
    segs
}

/// 两个实体的交点（细分线段两两求交，细分上限 32）
pub fn intersection_candidates(a: &Entity, b: &Entity) -> Vec<Point> {
    intersection_candidates_capped(a, b, 32)
}

/// 两个实体的交点（细分线段两两求交，细分上限 cap）
pub fn intersection_candidates_capped(a: &Entity, b: &Entity, cap: usize) -> Vec<Point> {
    let sa = entity_segments_capped(a, cap);
    let sb = entity_segments_capped(b, cap);
    let mut out = Vec::new();
    for (a1, a2) in &sa {
        for (b1, b2) in &sb {
            match intersect_line_line(Line::new(*a1, *a2), Line::new(*b1, *b2)) {
                IntersectionResult::Point(ip) => out.push(ip.point),
                IntersectionResult::Points(v) => {
                    for ip in v {
                        out.push(ip.point);
                    }
                }
                _ => {}
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data_structure::make_line;

    #[test]
    fn test_snap_candidates_line() {
        let line = make_line(Point::new2d(0.0, 0.0), Point::new2d(10.0, 0.0));
        let cands = snap_candidates(&line);
        assert_eq!(cands.len(), 3);
        assert!(cands.iter().any(|(p, k)| p.x == 5.0 && *k == SnapType::MidPoint));
    }

    #[test]
    fn test_intersection_candidates() {
        let a = make_line(Point::new2d(0.0, 0.0), Point::new2d(10.0, 10.0));
        let b = make_line(Point::new2d(0.0, 10.0), Point::new2d(10.0, 0.0));
        let pts = intersection_candidates(&a, &b);
        assert!(pts.iter().any(|p| (p.x - 5.0).abs() < 1e-9 && (p.y - 5.0).abs() < 1e-9));
    }
}
