//! 捕捉候选点：直接枚举实体上的端点、中点、圆心与实体之间的交点。
//!
//! 本模块只做「世界坐标下的候选生成」，不涉及捕捉半径、优先级与光标距离判断——
//! 靶框范围内的筛选由上层（如 [`SnapManager`](super::snap_point::SnapManager)）完成。
//! 曲线实体的交点由 Tessellation 细分折线两两求交近似得到，精度取决于细分上限 `cap`。

use crate::data_structure::{Entity, EntityGeometry};
use crate::geometry::intersection::{intersect_line_line, IntersectionResult};
use crate::geometry::{Line, Point};
use crate::render::tessellation::entity_polylines;

use super::snap_point::SnapType;

/// 枚举单个实体的捕捉候选点（世界坐标）及其捕捉类型。
///
/// 直线给出两个端点与中点，圆、椭圆给出圆心，圆弧给出圆心与两个端点，多段线给出首尾顶点；
/// 其它几何类型返回空向量。结果不去重，也不按屏幕距离过滤。
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

/// 把实体离散成用于求交的线段，并限制每轮的抽样段数。
///
/// - `cap`：细分上限，每段按下标步长 `(n - 1) / cap` 抽取，累计段数超过 `cap * 2` 时提前结束；
/// 必须大于 0，否则内部整数除法会 panic；
/// 返回：世界坐标下的线段端点对；实体无法离散出至少两个点时返回空向量。
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

/// 求两个实体的交点（世界坐标），每侧细分上限固定为 32。
///
/// 内部委托给 [`intersection_candidates_capped`]，结果为近似值，可能重复、无序。
pub fn intersection_candidates(a: &Entity, b: &Entity) -> Vec<Point> {
    intersection_candidates_capped(a, b, 32)
}

/// 求两个实体的交点（世界坐标），并指定每侧的细分上限。
///
/// - `a`、`b`：参与求交的两个实体；
/// - `cap`：每侧细分上限，越大越精确，代价约按 `cap²` 增长，必须大于 0；
/// 返回：所有细分线段对的交点，可能为空、可能重复，不做去重与排序。
///
/// # 示例
/// `intersection_candidates(&a, &b)` 对两条交叉直线返回其交点，如 `(0,0)-(10,10)` 与
/// `(0,10)-(10,0)` 的候选交点为 `(5,5)`。
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
