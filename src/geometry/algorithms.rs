//! 几何算法模块：以自由函数形式提供常用平面几何运算。
//!
//! 这里集中放置距离计算、最近点、凸包、偏移、裁剪与延长等对图元的操作，避免把这些
//! 过程性算法塞进图元类型本身；函数均不修改入参（图元按值或引用传入），长度单位为模型
//! 空间单位，角度为弧度。多数函数只使用 `x`/`y`，属于二维算法，Z 坐标处理方式见各函数说明。
//! 上层 Snap 捕捉、Dimension 标注与编辑命令可直接复用这些函数。

use crate::geometry::{Point, Line, Circle, Arc};
use crate::math::Vector2;
use std::cmp::Ordering;

/// 返回点到线段的最近距离，非负。
///
/// - `point`：查询点，仅使用其 `x`/`y`。
/// - `line`：目标线段，最近点被限制在线段范围内，因此点位于端点外侧时取到最近端点的距离。
///
/// 线段长度小于 `1e-10` 时退化为点到 `start` 的距离。
#[inline]
pub fn distance_point_to_line(point: Point, line: Line) -> f64 {
    let start = line.start;
    let end = line.end;
    let dx = end.x - start.x;
    let dy = end.y - start.y;
    
    let length_sq = dx * dx + dy * dy;
    if length_sq < 1e-10 {
        return point.distance_to(&start);
    }
    
    let t = ((point.x - start.x) * dx + (point.y - start.y) * dy) / length_sq;
    let t = t.clamp(0.0, 1.0);
    
    let closest = Point::new(
        start.x + t * dx,
        start.y + t * dy,
        start.z,
    );
    
    point.distance_to(&closest)
}

/// 返回点到圆周（而非圆面）的距离，非负。
///
/// 圆心处的点得到最大距离 `radius`；点在圆内、圆外都会返回其到圆周的最短距离，
/// 判定不区分内外。计算使用三维距离，故查询点与圆心的 Z 差异会影响结果。
///
/// - `point`：查询点。
/// - `circle`：目标圆。
#[inline]
pub fn distance_point_to_circle(point: Point, circle: Circle) -> f64 {
    let dist = point.distance_to(&circle.center);
    (dist - circle.radius).abs()
}

/// 返回点到圆弧的距离，非负。
///
/// - `point`：查询点，仅使用其 `x`/`y`。
/// - `arc`：目标圆弧，其 `is_counter_clockwise` 决定角度区间落在哪一侧。
///
/// 若查询点相对圆心的角度落在弧的扫掠范围内，返回点到圆周的径向距离；否则返回径向距离与
/// 两个端点到该点距离中的最小值，因此结果始终不超过到端点的距离。
#[inline]
pub fn distance_point_to_arc(point: Point, arc: Arc) -> f64 {
    let angle = arc.angle_from_center(&point);
    let normalized_angle = arc.normalize_angle(angle);
    
    let start = arc.normalize_angle(arc.start_angle);
    let end = arc.normalize_angle(arc.end_angle);
    
    let on_arc = if arc.is_counter_clockwise {
        if end >= start {
            normalized_angle >= start && normalized_angle <= end
        } else {
            normalized_angle >= start || normalized_angle <= end
        }
    } else {
        if end <= start {
            normalized_angle <= start && normalized_angle >= end
        } else {
            normalized_angle <= start || normalized_angle >= end
        }
    };
    
    let dist_to_center = point.distance_to(&arc.center);
    let radial_dist = (dist_to_center - arc.radius).abs();
    
    if on_arc {
        radial_dist
    } else {
        let to_start = point.distance_to(&arc.start_point());
        let to_end = point.distance_to(&arc.end_point());
        radial_dist.min(to_start).min(to_end)
    }
}

/// 返回点到线段所在直线的垂足，结果被限制在线段范围内。
///
/// - `point`：查询点，仅使用其 `x`/`y`。
/// - `line`：目标线段。
///
/// 与 [`distance_point_to_line`] 使用同一投影规则：垂足落在线段外时返回较近的端点，
/// 返回点的 Z 坐标取自线段上的插值位置。
#[inline]
pub fn perpendicular_point_to_line(point: Point, line: Line) -> Point {
    line.closest_point(&point)
}

/// 返回两条线段上彼此最近的一对点，形式为 `(line1 上的点, line2 上的点)`。
///
/// - `line1`：第一条线段。
/// - `line2`：第二条线段。
///
/// 为二维算法：只考虑 `x`/`y`，返回的两个点 Z 坐标均为 `0.0`。两个参数都被截断到
/// `[0, 1]`，因此结果一定落在线段内；当两线段平行或某条线段退化（长度接近 0）时返回
/// 一侧端点作为结果，不做特殊补偿。
#[inline]
pub fn closest_points_on_lines(line1: Line, line2: Line) -> (Point, Point) {
    let p1 = line1.start.to_vector2();
    let p2 = line1.end.to_vector2();
    let p3 = line2.start.to_vector2();
    let p4 = line2.end.to_vector2();
    
    let d1 = p2 - p1;
    let d2 = p4 - p3;
    
    let r = p1 - p3;
    let a = d1.dot(&d1);
    let e = d2.dot(&d2);
    let f = d2.dot(&r);
    
    let (s, t) = if a <= 1e-10 && e <= 1e-10 {
        (0.0, 0.0)
    } else if a <= 1e-10 {
        (0.0, f / e)
    } else {
        let c = d1.dot(&r);
        let denom = a * e - d1.dot(&d2).powi(2);
        
        if denom.abs() < 1e-10 {
            (0.0, 0.0)
        } else {
            let b = d1.dot(&d2);
            let s = (b * f - c * e) / denom;
            let t = s * b - f;
            (s.clamp(0.0, 1.0), t.clamp(0.0, 1.0))
        }
    };
    
    (
        Point::new(p1.x + d1.x * s, p1.y + d1.y * s, 0.0),
        Point::new(p3.x + d2.x * t, p3.y + d2.y * t, 0.0),
    )
}

/// 返回点集的二维凸包顶点序列（逆时针，不含重复的起点）。
///
/// - `points`：输入点集，仅使用 `x`/`y`；点数为 0 时返回空 `Vec`，为 1 或 2 时原样返回。
///
/// 使用 Andrew 单调链算法，位于凸包边上的共线点会被舍弃，因此三角形成 3 个顶点、含一个
/// 内部点的正方形得到 4 个顶点；返回顺序从最小 `x` 的点开始。若输入含 `NaN` 坐标，
/// 排序比较会 panic。
#[inline]
pub fn convex_hull(points: &[Point]) -> Vec<Point> {
    if points.len() <= 2 {
        return points.to_vec();
    }

    let mut sorted_points: Vec<_> = points.iter().collect();
    sorted_points.sort_by(|a, b| {
        let cmp = a.x.partial_cmp(&b.x).unwrap();
        if cmp != Ordering::Equal {
            cmp
        } else {
            a.y.partial_cmp(&b.y).unwrap()
        }
    });

    let cross = |o: &Point, a: &Point, b: &Point| -> f64 {
        (a.x - o.x) * (b.y - o.y) - (a.y - o.y) * (b.x - o.x)
    };

    let mut lower = Vec::new();
    for p in &sorted_points {
        while lower.len() >= 2 && cross(&lower[lower.len()-2], &lower[lower.len()-1], p) <= 0.0 {
            lower.pop();
        }
        lower.push(**p);
    }

    let mut upper = Vec::new();
    for p in sorted_points.iter().rev() {
        while upper.len() >= 2 && cross(&upper[upper.len()-2], &upper[upper.len()-1], p) <= 0.0 {
            upper.pop();
        }
        upper.push(**p);
    }

    lower.pop();
    upper.pop();
    lower.extend(upper);
    lower
}

/// 返回两条直线的方向夹角，单位为弧度。
///
/// - `line1`：第一条线段，取其 `start` → `end` 方向。
/// - `line2`：第二条线段，取其 `start` → `end` 方向。
///
/// 结果为有向夹角，落在 `(-π, π]`：交换两条线的顺序会得到相反的符号；零长度线段的方向为
/// 零向量，结果不可靠。若只需要大小，请对返回值取绝对值。
#[inline]
pub fn angle_between_lines(line1: Line, line2: Line) -> f64 {
    let dir1 = line1.direction();
    let dir2 = line2.direction();
    dir1.angle_to(&dir2)
}

/// 返回沿法线偏移后的新线段，原线段不变。
///
/// - `line`：源线段。
/// - `distance`：偏移距离，取绝对值，负值与正值效果相同（方向由 `side` 决定）。
/// - `side`：偏移方向，`1` 表示沿前进方向左侧的法线，`-1` 表示右侧，其他值按数值参与乘法。
///
/// 只使用 `x`/`y` 计算法线，两个端点的 Z 坐标原样保留。
#[inline]
pub fn offset_line(line: Line, distance: f64, side: i8) -> Line {
    let dir = line.direction();
    let normal = Vector2::new(-dir.y, dir.x);
    
    let offset_x = normal.x * (distance * side as f64);
    let offset_y = normal.y * (distance * side as f64);
    
    Line::new(
        Point::new(line.start.x + offset_x, line.start.y + offset_y, line.start.z),
        Point::new(line.end.x + offset_x, line.end.y + offset_y, line.end.z),
    )
}

/// 返回以指定点裁剪后的新线段，原线段不变。
///
/// - `line`：源线段。
/// - `trim_point`：裁剪点，函数不检查它是否落在源线段上或其延长线上。
/// - `keep_start`：`true` 保留 `start` 到 `trim_point` 的一段，`false` 保留 `trim_point` 到 `end` 的一段。
///
/// 结果始终是源线段两个端点与 `trim_point` 的组合，因此传入区间外的点会得到反向或被拉长的线段。
#[inline]
pub fn trim_line_at_point(line: Line, trim_point: Point, keep_start: bool) -> Line {
    if keep_start {
        Line::new(line.start, trim_point)
    } else {
        Line::new(trim_point, line.end)
    }
}

/// 返回沿自身方向延长或缩短后的新线段，原线段不变。
///
/// - `line`：源线段。
/// - `extension`：延长量，单位与线段一致；取负值表示从该端向内缩短。
/// - `extend_start`：是否调整起点（沿方向反向移动）。
/// - `extend_end`：是否调整终点（沿方向正向移动）。
///
/// 两端可分别控制；两个标志都为 `false` 时返回与源线段等价的副本。
#[inline]
pub fn extend_line(line: Line, extension: f64, extend_start: bool, extend_end: bool) -> Line {
    let direction = line.direction();
    
    let new_start = if extend_start {
        Point::new(line.start.x - direction.x * extension, line.start.y - direction.y * extension, line.start.z)
    } else {
        line.start
    };
    
    let new_end = if extend_end {
        Point::new(line.end.x + direction.x * extension, line.end.y + direction.y * extension, line.end.z)
    } else {
        line.end
    };
    
    Line::new(new_start, new_end)
}

/// 返回两点连线的中点，三个坐标分量分别取平均。
///
/// - `p1`：第一个点。
/// - `p2`：第二个点。
#[inline]
pub fn midpoint(p1: Point, p2: Point) -> Point {
    Point::new(
        (p1.x + p2.x) / 2.0,
        (p1.y + p2.y) / 2.0,
        (p1.z + p2.z) / 2.0,
    )
}

/// 由三点构造中线线段，返回连接 `p1p2` 与 `p2p3` 两条边中点的线段。
///
/// - `p1`：第一条边的起点。
/// - `p2`：两条边的公共顶点。
/// - `p3`：第二条边的终点。
///
/// 结果不是角平分线，而是三角形中位线；三点共线时得到一条退化线段。
#[inline]
pub fn bisector(p1: Point, p2: Point, p3: Point) -> Line {
    let mid1 = midpoint(p1, p2);
    let mid2 = midpoint(p2, p3);
    Line::new(mid1, mid2)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_distance_point_to_line() {
        let line = Line::new(Point::origin(), Point::new(1.0, 0.0, 0.0));
        let point = Point::new(0.5, 1.0, 0.0);
        
        let dist = distance_point_to_line(point, line);
        assert!((dist - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_convex_hull() {
        let points = vec![
            Point::new(0.0, 0.0, 0.0),
            Point::new(1.0, 0.0, 0.0),
            Point::new(1.0, 1.0, 0.0),
            Point::new(0.0, 1.0, 0.0),
            Point::new(0.5, 0.5, 0.0),
        ];
        
        let hull = convex_hull(&points);
        assert_eq!(hull.len(), 4);
    }

    #[test]
    fn test_offset_line() {
        let line = Line::new(Point::origin(), Point::new(1.0, 0.0, 0.0));
        let offset = offset_line(line, 1.0, 1);
        
        assert!((offset.start.y - 1.0).abs() < 1e-10);
    }
    
    #[test]
    fn test_closest_points_on_lines_parallel() {
        let line1 = Line::new(Point::origin(), Point::new(1.0, 0.0, 0.0));
        let line2 = Line::new(Point::new(0.0, 1.0, 0.0), Point::new(1.0, 1.0, 0.0));
        
        let (p1, p2) = closest_points_on_lines(line1, line2);
        assert!((p1.y - 0.0).abs() < 1e-10);
        assert!((p2.y - 1.0).abs() < 1e-10);
    }
    
    #[test]
    fn test_angle_between_lines() {
        let line1 = Line::new(Point::origin(), Point::new(1.0, 0.0, 0.0));
        let line2 = Line::new(Point::origin(), Point::new(0.0, 1.0, 0.0));
        
        let angle = angle_between_lines(line1, line2);
        assert!((angle - std::f64::consts::FRAC_PI_2).abs() < 1e-10);
    }
    
    #[test]
    fn test_midpoint() {
        let p1 = Point::new(0.0, 0.0, 0.0);
        let p2 = Point::new(2.0, 4.0, 0.0);
        
        let mid = midpoint(p1, p2);
        assert!((mid.x - 1.0).abs() < 1e-10);
        assert!((mid.y - 2.0).abs() < 1e-10);
    }
    
    #[test]
    fn test_trim_line() {
        let line = Line::new(Point::origin(), Point::new(2.0, 0.0, 0.0));
        let trim_point = Point::new(1.0, 0.0, 0.0);
        
        let trimmed = trim_line_at_point(line, trim_point, true);
        assert!((trimmed.end.x - 1.0).abs() < 1e-10);
        
        let trimmed2 = trim_line_at_point(line, trim_point, false);
        assert!((trimmed2.start.x - 1.0).abs() < 1e-10);
    }
    
    #[test]
    fn test_extend_line() {
        let line = Line::new(Point::origin(), Point::new(1.0, 0.0, 0.0));
        
        let extended = extend_line(line, 0.5, true, true);
        assert!((extended.start.x - (-0.5)).abs() < 1e-10);
        assert!((extended.end.x - 1.5).abs() < 1e-10);
    }
    
    #[test]
    fn test_bisector() {
        let p1 = Point::new(0.0, 0.0, 0.0);
        let p2 = Point::new(2.0, 0.0, 0.0);
        let p3 = Point::new(1.0, 1.0, 0.0);
        
        let bis = bisector(p1, p2, p3);
        assert!((bis.start.x - 1.0).abs() < 1e-10);
        assert!((bis.start.y - 0.0).abs() < 1e-10);
    }
    
    #[test]
    fn test_convex_hull_triangle() {
        let points = vec![
            Point::new(0.0, 0.0, 0.0),
            Point::new(1.0, 0.0, 0.0),
            Point::new(0.5, 0.5, 0.0),
        ];
        
        let hull = convex_hull(&points);
        assert_eq!(hull.len(), 3);
    }
    
    #[test]
    fn test_distance_point_to_line_collinear() {
        let line = Line::new(Point::origin(), Point::new(1.0, 0.0, 0.0));
        let point = Point::new(0.5, 0.0, 0.0);
        
        let dist = distance_point_to_line(point, line);
        assert!(dist < 1e-10);
    }
    
    #[test]
    fn test_distance_point_to_line_outside_segment() {
        let line = Line::new(Point::origin(), Point::new(1.0, 0.0, 0.0));
        let point = Point::new(2.0, 1.0, 0.0);
        
        let dist = distance_point_to_line(point, line);
        let expected_dist = ((2.0_f64 - 1.0).powi(2) + (1.0_f64 - 0.0).powi(2)).sqrt();
        assert!((dist - expected_dist).abs() < 1e-10);
    }
    
    #[test]
    fn test_perpendicular_point() {
        let line = Line::new(Point::origin(), Point::new(1.0, 0.0, 0.0));
        let point = Point::new(0.5, 1.0, 0.0);
        
        let perp = perpendicular_point_to_line(point, line);
        assert!((perp.x - 0.5).abs() < 1e-10);
        assert!((perp.y - 0.0).abs() < 1e-10);
    }
}
