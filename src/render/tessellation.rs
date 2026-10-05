//! 实体细分：把各类实体几何展开为折线集合与填充多边形，并计算文档包围盒。
//! 标注与文字实体分解为图形笔画（含矢量文字），从而被渲染与所有导出路径复用。
//! 输出的点一律为世界坐标（`Point`，使用 x、y 分量），不做视口变换；曲线按固定或自适应段数离散为折线。

use crate::data_structure::{Document, Entity, EntityGeometry};
use crate::dimension::strokes;
use crate::geometry::{Arc, BSpline, Circle, Ellipse, NURBS, Point, Polyline};

// ---------------- 细分 ----------------

/// 把整圆均匀细分为折线点集（世界坐标）。
/// 从角度 0（+x 方向）起逆时针采样，返回 `segments + 1` 个点，首尾两点重合以表示闭合环。
/// - `segments`：分段数，越大越接近真实圆；为 0 时会因除以零得到非数坐标，调用方需传入正数。
pub fn circle_points(center: Point, radius: f64, segments: usize) -> Vec<Point> {
    (0..=segments)
        .map(|i| {
            let a = 2.0 * std::f64::consts::PI * i as f64 / segments as f64;
            Point::new2d(center.x + radius * a.cos(), center.y + radius * a.sin())
        })
        .collect()
}

/// 圆弧细分为折线点（按弧线方向采样）
/// 返回 `segments + 1` 个点，包含弧的起点与终点；采样方向由 `Arc::is_counter_clockwise` 决定，
/// 角度为弧度。`segments` 常用 `arc_segments` 的建议值，传 0 会得到非数坐标。
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
/// 按夹角每 0.08 弧度一段估算，并夹在 8~180 之间，兼顾圆滑度与顶点数量。
pub fn arc_segments(arc: &Arc) -> usize {
    ((arc.angle_span() / 0.08).ceil() as usize).clamp(8, 180)
}

/// 把椭圆均匀细分为折线点集（世界坐标）。
/// 参数在 `[0, 1]` 上等分，返回 `segments + 1` 个点，首尾两点重合以表示闭合；旋转角已由椭圆自身应用。
/// - `segments`：分段数；为 0 时只返回参数 0 处的单点。
pub fn ellipse_points(e: &Ellipse, segments: usize) -> Vec<Point> {
    (0..=segments)
        .map(|i| e.point_at_parameter(i as f64 / segments as f64))
        .collect()
}

fn curve_samples(points: &[Point]) -> usize {
    // 采样密度随控制点数增长
    (points.len() * 24).clamp(48, 480)
}

/// 在归一化参数 `[0, 1]` 上均匀采样 B 样条曲线。
/// 返回 `samples + 1` 个点，包含曲线两端；`samples` 为 0 时只返回起点。
/// - `samples`：采样段数，曲线控制点越多所需段数越大，可参考内部 `curve_samples` 的取值策略。
pub fn bspline_points(s: &BSpline, samples: usize) -> Vec<Point> {
    (0..=samples)
        .map(|i| s.point_at(i as f64 / samples as f64))
        .collect()
}

/// 在归一化参数 `[0, 1]` 上均匀采样 NURBS 曲线（含权因子影响）。
/// 返回 `samples + 1` 个点，包含曲线两端；`samples` 为 0 时只返回起点。
/// - `samples`：采样段数，控制点越多所需段数越大，可参考内部 `curve_samples` 的取值策略。
pub fn nurbs_points(n: &NURBS, samples: usize) -> Vec<Point> {
    (0..=samples)
        .map(|i| n.point_at(i as f64 / samples as f64))
        .collect()
}

/// 实体几何展开为折线集合（世界坐标）。返回 (点列, 是否闭合)，不支持的类型返回空。
/// 圆固定 72 段、椭圆固定 96 段，圆弧按 `arc_segments` 自适应，样条按控制点数量取 48~480 段；
/// 标注与文字实体转为图形笔画（文字为矢量笔画），因此也能参与捕捉与导出。
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
/// 等价于对 `entity.geometry()` 调用 `geometry_polylines`，不修改实体本身。
pub fn entity_polylines(entity: &Entity) -> Vec<(Vec<Point>, bool)> {
    geometry_polylines(entity.geometry())
}

/// 实体几何的填充多边形（世界坐标）与 RGB 颜色。支持 Solid 与实心 Hatch。
/// 多边形顶点按边界顺序给出，不带重复的闭合点；点列少于 3 个点或 Hatch 未设置实心填充时返回空向量。
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
/// 等价于对 `entity.geometry()` 调用 `geometry_fills`，不做任何缓存或状态修改。
pub fn entity_fills(entity: &Entity) -> Vec<(Vec<Point>, (u8, u8, u8))> {
    geometry_fills(entity.geometry())
}

// ---------------- 包围盒 ----------------

/// 全文档包围盒（支持所有可细分实体）
/// 遍历文档全部实体的折线点与填充多边形顶点，逐点累积最小/最大值。
/// 返回 `(最小角点, 最大角点)`，均为世界坐标；文档为空或所有实体都不可细分时返回 `None`。
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
