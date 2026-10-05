//! 尺寸标注（Dimension）：线性 / 半径 / 直径 / 角度标注的构造与图形分解。
//! 文字使用内置矢量笔画字体（数字与 CAD 符号），整体分解为折线，
//! 因此渲染、拾取、包围盒和所有导出格式（DXF/SVG/EPS/PDF/WMF/位图）自动支持。

use crate::data_structure::{DimensionType, Entity, EntityGeometry, EntityType};
use crate::geometry::Point;

const TAU: f64 = std::f64::consts::TAU;

// ---------------- 向量助手 ----------------

/// 单位向量；零向量返回 (1, 0)
fn unit(a: Point) -> Point {
    let d = a.distance_to(&Point::origin());
    if d < 1e-12 {
        Point::new2d(1.0, 0.0)
    } else {
        a * (1.0 / d)
    }
}

fn perp(a: Point) -> Point {
    Point::new2d(-a.y, a.x)
}

// ---------------- 数值文字格式化 ----------------

fn trim_zeros(mut s: String) -> String {
    if s.contains('.') {
        while s.ends_with('0') {
            s.pop();
        }
        if s.ends_with('.') {
            s.pop();
        }
    }
    s
}

fn fmt_num(v: f64) -> String {
    trim_zeros(format!("{v:.2}"))
}

fn fmt_angle(v: f64) -> String {
    trim_zeros(format!("{v:.1}"))
}

/// 文字旋转角保持从左到右可读（世界坐标 y 向上）
fn readable(rot: f64) -> f64 {
    let mut r = rot % TAU;
    if r < 0.0 {
        r += TAU;
    }
    if r > std::f64::consts::FRAC_PI_2 && r <= std::f64::consts::PI * 1.5 {
        r += std::f64::consts::PI;
    }
    r
}

// ---------------- 笔画字体 ----------------
// 字形坐标系：基线 y=0，字高 y=1，x 向右。字形宽度约 0.5。

struct GlyphStroke {
    closed: bool,
    pts: &'static [(f64, f64)],
}

struct Glyph {
    advance: f64,
    strokes: &'static [GlyphStroke],
}

macro_rules! glyph_strokes {
    ([$(($closed:expr, $pts:expr)),* $(,)?]) => {
        &[$(GlyphStroke { closed: $closed, pts: $pts }),*]
    };
}

fn glyph(c: char) -> Option<Glyph> {
    let g = match c {
        '0' => Glyph { advance: 0.55, strokes: glyph_strokes!([(true, &[(0.07, 0.0), (0.0, 0.3), (0.0, 0.7), (0.07, 1.0), (0.38, 1.0), (0.45, 0.7), (0.45, 0.3), (0.38, 0.0)])]) },
        '1' => Glyph { advance: 0.55, strokes: glyph_strokes!([
            (false, &[(0.1, 0.75), (0.25, 1.0)]),
            (false, &[(0.25, 0.0), (0.25, 1.0)]),
            (false, &[(0.08, 0.0), (0.45, 0.0)]),
        ]) },
        '2' => Glyph { advance: 0.55, strokes: glyph_strokes!([(false, &[(0.0, 0.75), (0.08, 0.92), (0.25, 1.0), (0.4, 0.92), (0.45, 0.75), (0.35, 0.5), (0.15, 0.28), (0.0, 0.12), (0.0, 0.0), (0.5, 0.0)])]) },
        '3' => Glyph { advance: 0.55, strokes: glyph_strokes!([(false, &[(0.0, 0.85), (0.12, 0.98), (0.3, 1.0), (0.43, 0.9), (0.45, 0.72), (0.33, 0.58), (0.2, 0.55), (0.36, 0.5), (0.48, 0.35), (0.45, 0.1), (0.3, 0.0), (0.08, 0.05)])]) },
        '4' => Glyph { advance: 0.55, strokes: glyph_strokes!([
            (false, &[(0.32, 0.0), (0.32, 1.0)]),
            (false, &[(0.0, 0.42), (0.32, 1.0)]),
            (false, &[(0.02, 0.42), (0.5, 0.42)]),
        ]) },
        '5' => Glyph { advance: 0.55, strokes: glyph_strokes!([(false, &[(0.45, 1.0), (0.05, 1.0), (0.02, 0.55), (0.3, 0.6), (0.45, 0.45), (0.42, 0.15), (0.25, 0.0), (0.02, 0.08)])]) },
        '6' => Glyph { advance: 0.55, strokes: glyph_strokes!([(false, &[(0.45, 0.95), (0.2, 1.0), (0.03, 0.75), (0.0, 0.35), (0.12, 0.05), (0.33, 0.0), (0.47, 0.15), (0.42, 0.4), (0.2, 0.48), (0.03, 0.3)])]) },
        '7' => Glyph { advance: 0.55, strokes: glyph_strokes!([(false, &[(0.0, 1.0), (0.5, 1.0), (0.18, 0.0)])]) },
        '8' => Glyph { advance: 0.55, strokes: glyph_strokes!([
            (true, &[(0.08, 0.52), (0.02, 0.72), (0.1, 0.92), (0.3, 1.0), (0.45, 0.88), (0.44, 0.66), (0.3, 0.52)]),
            (true, &[(0.08, 0.52), (0.0, 0.3), (0.1, 0.05), (0.32, 0.0), (0.47, 0.18), (0.44, 0.4), (0.3, 0.52)]),
        ]) },
        '9' => Glyph { advance: 0.55, strokes: glyph_strokes!([(false, &[(0.05, 0.05), (0.28, 0.0), (0.45, 0.22), (0.5, 0.6), (0.4, 0.92), (0.18, 1.0), (0.03, 0.9), (0.02, 0.65), (0.25, 0.55), (0.46, 0.72)])]) },
        '.' => Glyph { advance: 0.3, strokes: glyph_strokes!([(false, &[(0.08, 0.0), (0.14, 0.0)])]) },
        '-' => Glyph { advance: 0.45, strokes: glyph_strokes!([(false, &[(0.0, 0.5), (0.4, 0.5)])]) },
        'R' => Glyph { advance: 0.62, strokes: glyph_strokes!([
            (false, &[(0.05, 0.0), (0.05, 1.0)]),
            (false, &[(0.05, 1.0), (0.35, 1.0), (0.5, 0.85), (0.42, 0.6), (0.05, 0.58)]),
            (false, &[(0.28, 0.58), (0.52, 0.0)]),
        ]) },
        'Ø' => Glyph { advance: 0.58, strokes: glyph_strokes!([
            (true, &[(0.07, 0.0), (0.0, 0.3), (0.0, 0.7), (0.07, 1.0), (0.38, 1.0), (0.45, 0.7), (0.45, 0.3), (0.38, 0.0)]),
            (false, &[(0.0, 0.0), (0.5, 1.0)]),
        ]) },
        '°' => Glyph { advance: 0.45, strokes: glyph_strokes!([(true, &[(0.16, 0.68), (0.1, 0.8), (0.16, 0.95), (0.32, 0.95), (0.38, 0.8), (0.32, 0.68)])]) },
        ' ' => Glyph { advance: 0.4, strokes: &[] },
        _ => return None,
    };
    Some(g)
}

/// 文字总宽度（世界单位）
pub fn text_width(content: &str, height: f64) -> f64 {
    let units: f64 = content
        .chars()
        .map(|c| glyph(c).map_or(0.5, |g| g.advance))
        .sum();
    units * height
}

/// 文字 → 折线笔画。origin 为首字符基线左端点，rotation 为弧度（逆时针）。
pub fn text_strokes(content: &str, origin: Point, height: f64, rotation: f64) -> Vec<(Vec<Point>, bool)> {
    let (s, c) = rotation.sin_cos();
    let place = |gx: f64, gy: f64| -> Point {
        let x = gx * height;
        let y = gy * height;
        Point::new2d(origin.x + x * c - y * s, origin.y + x * s + y * c)
    };
    let mut cursor = 0.0f64;
    let mut out = Vec::new();
    for ch in content.chars() {
        if let Some(g) = glyph(ch) {
            for st in g.strokes {
                let pts: Vec<Point> = st.pts.iter().map(|&(x, y)| place(x + cursor, y)).collect();
                if pts.len() >= 2 {
                    out.push((pts, st.closed));
                }
            }
            cursor += g.advance;
        } else {
            cursor += 0.5;
        }
    }
    out
}

// ---------------- 标注图形元素 ----------------

/// 箭头（闭合三角形，尖端在 tip，指向 dir）
fn arrow(tip: Point, dir: Point, len: f64, width: f64) -> (Vec<Point>, bool) {
    let n = perp(dir);
    let base = tip - dir * len;
    (
        vec![
            tip,
            base + n * (width / 2.0),
            base - n * (width / 2.0),
        ],
        true,
    )
}

fn arrow_size(text_height: f64) -> (f64, f64) {
    (text_height * 1.4, text_height * 0.6)
}

// ---------------- 构造 ----------------

#[allow(clippy::too_many_arguments)]
fn dim_entity(
    dim_type: DimensionType,
    measurement: f64,
    text: String,
    text_position: Point,
    text_height: f64,
    text_rotation: f64,
    definition_point: Point,
    def_point_1: Point,
    def_point_2: Point,
    def_point_3: Point,
    angle: f64,
    extension_lines: bool,
    center_marks: bool,
) -> Entity {
    let geometry = EntityGeometry::Dimension {
        dim_type,
        measurement,
        text,
        text_position,
        text_height,
        text_rotation,
        definition_point,
        def_point_1,
        def_point_2,
        def_point_3,
        def_point_4: Point::origin(),
        angle,
        extension_lines,
        center_marks,
    };
    Entity::new(EntityType::Dimension, geometry)
}

/// 线性（对齐）标注：p1、p2 为测量点，placement 决定标注线偏移侧。
pub fn make_linear(p1: Point, p2: Point, placement: Point, text_height: f64) -> Option<Entity> {
    let len = p1.distance_to(&p2);
    if len < 1e-9 || text_height < 1e-9 {
        return None;
    }
    let u = unit(p2 - p1);
    let n = perp(u);
    let off = (placement - p1).dot(&n);
    let a = p1 + n * off;
    let b = p2 + n * off;
    let side = if off >= 0.0 { 1.0 } else { -1.0 };
    let text = fmt_num(len);
    let tw = text_width(&text, text_height);
    let mid = (a + b) * 0.5;
    // 文字基线：标注线中点沿法向抬高，沿方向居中
    let text_pos = mid + n * (side * text_height * 0.55) + u * (-tw / 2.0);
    let rot = readable((u.y).atan2(u.x));
    Some(dim_entity(
        DimensionType::Aligned,
        len,
        text,
        text_pos,
        text_height,
        rot,
        a,
        p1,
        p2,
        Point::origin(),
        rot,
        true,
        false,
    ))
}

/// 半径 / 直径标注：aim 决定标注方向（文字放置侧）。
pub fn make_radial(
    center: Point,
    radius: f64,
    aim: Point,
    diameter: bool,
    text_height: f64,
) -> Option<Entity> {
    if radius < 1e-9 || text_height < 1e-9 {
        return None;
    }
    let t = unit(aim - center);
    let q = center + t * radius;
    let q2 = center - t * radius;
    let (prefix, measurement, dim_type) = if diameter {
        ("Ø", radius * 2.0, DimensionType::Diameter)
    } else {
        ("R", radius, DimensionType::Radius)
    };
    let text = format!("{prefix}{}", fmt_num(measurement));
    // 文字基线在圆周外侧沿 t 方向；旋转取可读方向后文字朝外延伸
    let text_pos = q + t * (text_height * 0.4);
    let rot = readable((t.y).atan2(t.x));
    Some(dim_entity(
        dim_type,
        measurement,
        text,
        text_pos,
        text_height,
        rot,
        q,
        center,
        q,
        q2,
        rot,
        false,
        true,
    ))
}

/// 角度标注：vertex 为顶点，ray1、ray2 为两条边上的点，逆时针从边 1 到边 2。
pub fn make_angular(
    vertex: Point,
    ray1: Point,
    ray2: Point,
    text_height: f64,
) -> Option<Entity> {
    let r1 = vertex.distance_to(&ray1);
    let r2 = vertex.distance_to(&ray2);
    if r1 < 1e-9 || r2 < 1e-9 || text_height < 1e-9 {
        return None;
    }
    let a0 = (ray1.y - vertex.y).atan2(ray1.x - vertex.x);
    let a1 = (ray2.y - vertex.y).atan2(ray2.x - vertex.x);
    let span = ((a1 - a0) % TAU + TAU) % TAU;
    if span < 1e-9 {
        return None;
    }
    // 标注弧半径取第一条边
    let r = r1;
    let s = vertex + Point::new2d(a0.cos() * r, a0.sin() * r);
    let e = vertex + Point::new2d(a1.cos() * r, a1.sin() * r);
    let mid_a = a0 + span / 2.0;
    let text = format!("{}°", fmt_angle(span.to_degrees()));
    let tw = text_width(&text, text_height);
    // 文字沿弧切向居中，法向抬离弧线
    let tangent = Point::new2d(-(mid_a.sin()), mid_a.cos());
    let text_pos = vertex
        + Point::new2d(
            mid_a.cos() * (r + text_height * 0.7),
            mid_a.sin() * (r + text_height * 0.7),
        )
        + tangent * (-tw / 2.0);
    let rot = readable(mid_a + std::f64::consts::FRAC_PI_2);
    Some(dim_entity(
        DimensionType::Angular,
        span.to_degrees(),
        text,
        text_pos,
        text_height,
        rot,
        s,
        vertex,
        s,
        e,
        a0,
        true,
        false,
    ))
}

// ---------------- 分解（标注 → 折线笔画） ----------------

/// 把 Dimension 实体几何分解为折线（含箭头与文字笔画）。
pub fn decompose(geometry: &EntityGeometry) -> Vec<(Vec<Point>, bool)> {
    let EntityGeometry::Dimension {
        dim_type,
        text,
        text_position,
        text_height,
        text_rotation,
        definition_point,
        def_point_1,
        def_point_2,
        def_point_3,
        extension_lines,
        center_marks,
        ..
    } = geometry
    else {
        return Vec::new();
    };
    let h = *text_height;
    if h < 1e-9 {
        return Vec::new();
    }
    let (alen, awid) = arrow_size(h);
    let mut out: Vec<(Vec<Point>, bool)> = Vec::new();

    match dim_type {
        DimensionType::Linear | DimensionType::Aligned => {
            let p1 = *def_point_1;
            let p2 = *def_point_2;
            let len = p1.distance_to(&p2);
            if len < 1e-9 {
                return out;
            }
            let u = unit(p2 - p1);
            let n = perp(u);
            let off = (*definition_point - p1).dot(&n);
            let a = p1 + n * off;
            let b = p2 + n * off;
            out.push((vec![a, b], false));
            if *extension_lines && off.abs() > h * 0.35 {
                let sgn = if off >= 0.0 { 1.0 } else { -1.0 };
                let gap = h * 0.35;
                let over = h * 0.5;
                out.push((vec![p1 + n * (sgn * gap), p1 + n * (off + sgn * over)], false));
                out.push((vec![p2 + n * (sgn * gap), p2 + n * (off + sgn * over)], false));
            }
            // 空间不足时箭头放到外侧
            let inside = len >= alen * 2.0 + text_width(text, h);
            out.push(arrow(a, if inside { -u } else { u }, alen, awid));
            out.push(arrow(b, if inside { u } else { -u }, alen, awid));
        }
        DimensionType::Radius | DimensionType::Diameter => {
            let c = *def_point_1;
            let q = *def_point_2;
            let r = c.distance_to(&q);
            if r < 1e-9 {
                return out;
            }
            let t = unit(q - c);
            if *dim_type == DimensionType::Radius {
                out.push((vec![c, q], false));
            } else {
                let q2 = if def_point_3.distance_to(&Point::origin()) > 1e-9 {
                    *def_point_3
                } else {
                    c - t * r
                };
                out.push((vec![q2, q], false));
                out.push(arrow(q2, -t, alen, awid));
            }
            out.push(arrow(q, t, alen, awid));
            if *center_marks {
                let k = h * 0.25;
                out.push((vec![Point::new2d(c.x - k, c.y), Point::new2d(c.x + k, c.y)], false));
                out.push((vec![Point::new2d(c.x, c.y - k), Point::new2d(c.x, c.y + k)], false));
            }
        }
        _ => {
            // 角度标注（Angular / ArcLength）
            let c = *def_point_1;
            let s = *def_point_2;
            let e = *def_point_3;
            let r = c.distance_to(&s);
            if r < 1e-9 {
                return out;
            }
            let a0 = (s.y - c.y).atan2(s.x - c.x);
            let a1 = (e.y - c.y).atan2(e.x - c.x);
            let span = ((a1 - a0) % TAU + TAU) % TAU;
            if span < 1e-9 {
                return out;
            }
            let segs = ((span / 0.08).ceil() as usize).clamp(8, 180);
            let arc: Vec<Point> = (0..=segs)
                .map(|i| {
                    let a = a0 + span * i as f64 / segs as f64;
                    Point::new2d(c.x + r * a.cos(), c.y + r * a.sin())
                })
                .collect();
            out.push((arc, false));
            if *extension_lines {
                out.push((vec![c, s], false));
                out.push((vec![c, e], false));
            }
            // 箭头沿弧切线（起点逆时针方向，终点反向）
            let t0 = Point::new2d(-a0.sin(), a0.cos());
            let t1 = Point::new2d(-a1.sin(), a1.cos());
            out.push(arrow(s, t0, alen, awid));
            out.push(arrow(e, -t1, alen, awid));
        }
    }

    out.extend(text_strokes(text, *text_position, h, *text_rotation));
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data_structure::DimensionType;

    /// 提取 Dimension 实体中的字段，便于断言
    fn fields(e: &Entity) -> (
        DimensionType,
        f64,
        String,
        Point,
        f64,
        Point,
        Point,
        Point,
        Point,
        bool,
        bool,
    ) {
        match e.geometry() {
            EntityGeometry::Dimension {
                dim_type,
                measurement,
                text,
                text_position,
                text_height,
                definition_point,
                def_point_1,
                def_point_2,
                def_point_3,
                extension_lines,
                center_marks,
                ..
            } => (
                dim_type.clone(),
                *measurement,
                text.clone(),
                *text_position,
                *text_height,
                *definition_point,
                *def_point_1,
                *def_point_2,
                *def_point_3,
                *extension_lines,
                *center_marks,
            ),
            other => panic!("不是标注实体: {other:?}"),
        }
    }

    fn bbox(strokes: &[(Vec<Point>, bool)]) -> (f64, f64, f64, f64) {
        let mut b = (f64::MAX, f64::MAX, f64::MIN, f64::MIN);
        for (pts, _) in strokes {
            for p in pts {
                b.0 = b.0.min(p.x);
                b.1 = b.1.min(p.y);
                b.2 = b.2.max(p.x);
                b.3 = b.3.max(p.y);
            }
        }
        b
    }

    fn has_stroke_between(strokes: &[(Vec<Point>, bool)], a: Point, b: Point) -> bool {
        strokes.iter().any(|(pts, _)| {
            pts.first().map(|f| f.distance_to(&a)).unwrap_or(f64::MAX) < 1e-9
                && pts.last().map(|l| l.distance_to(&b)).unwrap_or(f64::MAX) < 1e-9
        })
    }

    // ---------------- 文字笔画字体 ----------------

    #[test]
    fn text_strokes_digits_and_symbols() {
        for s in ["0", "1", "2", "3", "4", "5", "6", "7", "8", "9", "R", "Ø", "°", ".", "-"] {
            let strokes = text_strokes(s, Point::origin(), 2.0, 0.0);
            assert!(!strokes.is_empty(), "字符 {s} 缺少笔画");
            for (pts, _) in &strokes {
                assert!(pts.len() >= 2, "字符 {s} 存在退化笔画");
                for p in pts {
                    assert!(p.x.is_finite() && p.y.is_finite(), "字符 {s} 出现非法坐标");
                }
            }
        }
    }

    #[test]
    fn text_width_scales_with_height() {
        let w1 = text_width("R12.5", 2.0);
        let w2 = text_width("R12.5", 4.0);
        assert!(w1 > 0.0);
        assert!((w2 - w1 * 2.0).abs() < 1e-9, "文字宽度应随字高线性缩放");
        // 未知字符按 0.5 单位宽度回退，不应 panic
        assert!(text_width("中", 2.0) > 0.0);
    }

    #[test]
    fn text_strokes_respect_rotation() {
        let straight = text_strokes("1", Point::origin(), 2.0, 0.0);
        let rotated = text_strokes("1", Point::origin(), 2.0, std::f64::consts::FRAC_PI_2);
        let (sx0, sy0, sx1, sy1) = bbox(&straight);
        let (rx0, ry0, rx1, ry1) = bbox(&rotated);
        // 旋转 90° 后宽度与高度互换
        assert!(((sx1 - sx0) - (ry1 - ry0)).abs() < 1e-9);
        assert!(((sy1 - sy0) - (rx1 - rx0)).abs() < 1e-9);
    }

    // ---------------- 线性标注 ----------------

    #[test]
    fn linear_dimension_measures_and_labels() {
        let e = make_linear(
            Point::new2d(0.0, 0.0),
            Point::new2d(10.0, 0.0),
            Point::new2d(5.0, 4.0),
            2.5,
        )
        .expect("线性标注应构造成功");
        let (ty, m, text, _tp, th, _dp, p1, p2, _p3, ext, cm) = fields(&e);
        assert_eq!(ty, DimensionType::Aligned);
        assert!((m - 10.0).abs() < 1e-9);
        assert_eq!(text, "10");
        assert!((th - 2.5).abs() < 1e-9);
        assert!(p1.distance_to(&Point::new2d(0.0, 0.0)) < 1e-9);
        assert!(p2.distance_to(&Point::new2d(10.0, 0.0)) < 1e-9);
        assert!(ext, "线性标注应带尺寸界线");
        assert!(!cm, "线性标注不应带圆心标记");

        let strokes = decompose(e.geometry());
        assert!(strokes.len() > 2, "线性标注应包含尺寸线、箭头与文字");
        assert!(has_stroke_between(
            &strokes,
            Point::new2d(0.0, 4.0),
            Point::new2d(10.0, 4.0)
        ));
        let (x0, y0, x1, y1) = bbox(&strokes);
        assert!(x0.is_finite() && y0.is_finite() && x1 > x0 && y1 > y0);
    }

    #[test]
    fn linear_dimension_rejects_degenerate_input() {
        assert!(make_linear(Point::origin(), Point::origin(), Point::new2d(1.0, 1.0), 2.5).is_none());
        assert!(make_linear(
            Point::origin(),
            Point::new2d(10.0, 0.0),
            Point::new2d(5.0, 1.0),
            0.0
        )
        .is_none());
    }

    // ---------------- 半径标注 ----------------

    #[test]
    fn radial_dimension_geometry() {
        let center = Point::new2d(50.0, 50.0);
        let radius = 10.0;
        let aim = Point::new2d(60.0, 50.0); // 右侧象限点
        let e = make_radial(center, radius, aim, false, 2.5).expect("半径标注应构造成功");

        let (ty, m, text, text_pos, _th, dp, p1, p2, _p3, ext, cm) = fields(&e);
        assert_eq!(ty, DimensionType::Radius);
        assert!((m - radius).abs() < 1e-9, "半径测量值应为 10");
        assert_eq!(text, "R10");
        assert!(dp.distance_to(&aim) < 1e-9, "定义点应位于圆周上");
        assert!(p1.distance_to(&center) < 1e-9, "def_point_1 应为圆心");
        assert!(p2.distance_to(&aim) < 1e-9, "def_point_2 应为圆周点");
        assert!(!ext, "半径标注无需尺寸界线");
        assert!(cm, "半径标注应带圆心标记");
        assert!(text_pos.x > aim.x, "文字应位于圆周外侧");

        let strokes = decompose(e.geometry());
        // 尺寸线（圆心 → 圆周）+ 箭头 + 圆心标记 2 条 + 文字笔画
        assert!(strokes.len() >= 5, "实际笔画数 {}", strokes.len());
        assert!(has_stroke_between(&strokes, center, aim), "缺少圆心到圆周的尺寸线");
        // 圆心标记为十字
        assert!(has_stroke_between(
            &strokes,
            Point::new2d(center.x - 2.5 * 0.25, center.y),
            Point::new2d(center.x + 2.5 * 0.25, center.y)
        ));
        let (x0, y0, x1, y1) = bbox(&strokes);
        assert!(x0 >= center.x - radius - 2.5 - 1e-6, "标注越界: {x0}");
        assert!(y1 <= center.y + radius + 1e-6, "标注越界: {y1}");
    }

    #[test]
    fn radial_dimension_direction_follows_aim() {
        let center = Point::origin();
        let radius = 5.0;
        // 明确朝向 +x、+y、-x、-y 四个方向
        for aim in [
            Point::new2d(5.0, 0.0),
            Point::new2d(0.0, 5.0),
            Point::new2d(-5.0, 0.0),
            Point::new2d(0.0, -5.0),
        ] {
            let e = make_radial(center, radius, aim, false, 2.5).unwrap();
            let (_ty, _m, _t, text_pos, _th, dp, _p1, _p2, _p3, _ext, _cm) = fields(&e);
            assert!(dp.distance_to(&aim) < 1e-9, "定义点未落在指定方向");
            assert!(
                text_pos.distance_to(&center) > radius,
                "文字应始终位于圆周外侧"
            );
            // 文字方向与 aim 方向一致（点积为正）
            let v = text_pos - center;
            let u = aim - center;
            assert!(v.dot(&u) > 0.0, "文字应沿标注方向放置");
        }
    }

    /// 关键回归：圆心与 aim 重合（例如对象捕捉把光标吸附到圆心）时，
    /// 半径/直径方向不可用，但不得 panic、不得产生 NaN，且仍需给出可读标注。
    #[test]
    fn radial_dimension_handles_aim_at_center() {
        let center = Point::new2d(3.0, 7.0);
        let e = make_radial(center, 4.0, center, false, 2.5).expect("应退化为默认方向而不是失败");
        let (_ty, m, text, text_pos, _th, dp, _p1, _p2, _p3, _ext, _cm) = fields(&e);
        assert!((m - 4.0).abs() < 1e-9);
        assert_eq!(text, "R4");
        assert!(dp.distance_to(&center) - 4.0 < 1e-9, "定义点应落在圆周上");
        assert!(text_pos.x.is_finite() && text_pos.y.is_finite());
        for (pts, _) in decompose(e.geometry()) {
            for p in pts {
                assert!(p.x.is_finite() && p.y.is_finite(), "退化输入产生了非法坐标");
            }
        }
    }

    #[test]
    fn radial_dimension_rejects_invalid_input() {
        let c = Point::origin();
        assert!(make_radial(c, 0.0, Point::new2d(1.0, 0.0), false, 2.5).is_none());
        assert!(make_radial(c, -1.0, Point::new2d(1.0, 0.0), false, 2.5).is_none());
        assert!(make_radial(c, 5.0, Point::new2d(5.0, 0.0), false, 0.0).is_none());
    }

    // ---------------- 直径标注 ----------------

    #[test]
    fn diameter_dimension_geometry() {
        let center = Point::new2d(0.0, 0.0);
        let radius = 6.0;
        let aim = Point::new2d(6.0, 0.0);
        let e = make_radial(center, radius, aim, true, 2.5).expect("直径标注应构造成功");

        let (ty, m, text, text_pos, _th, dp, p1, p2, p3, _ext, cm) = fields(&e);
        assert_eq!(ty, DimensionType::Diameter);
        assert!((m - radius * 2.0).abs() < 1e-9, "直径测量值应为 12");
        assert_eq!(text, "Ø12");
        assert!(text.starts_with('Ø'), "直径标注应带 Ø 前缀");
        assert!(dp.distance_to(&aim) < 1e-9);
        assert!(p1.distance_to(&center) < 1e-9);
        assert!(p2.distance_to(&aim) < 1e-9);
        assert!(
            p3.distance_to(&Point::new2d(-radius, 0.0)) < 1e-9,
            "直径标注应记录对侧点"
        );
        assert!(cm, "直径标注应带圆心标记");
        assert!(text_pos.x > aim.x);

        let strokes = decompose(e.geometry());
        // 对侧点 → 圆周点的整条直径线
        assert!(has_stroke_between(
            &strokes,
            Point::new2d(-radius, 0.0),
            Point::new2d(radius, 0.0)
        ));
        // 两端箭头（尖端分别位于两个圆周点上）
        let tips: Vec<Point> = strokes
            .iter()
            .filter(|(pts, closed)| *closed && pts.len() == 3)
            .map(|(pts, _)| pts[0])
            .collect();
        assert!(
            tips.iter().any(|t| t.distance_to(&Point::new2d(radius, 0.0)) < 1e-9),
            "缺少 +x 侧箭头"
        );
        assert!(
            tips.iter().any(|t| t.distance_to(&Point::new2d(-radius, 0.0)) < 1e-9),
            "缺少 -x 侧箭头"
        );
    }

    #[test]
    fn diameter_dimension_text_matches_measurement() {
        for radius in [0.5, 1.0, 2.5, 12.25, 100.0] {
            let e = make_radial(
                Point::origin(),
                radius,
                Point::new2d(radius, 0.0),
                true,
                2.5,
            )
            .unwrap();
            let (_ty, m, text, _tp, _th, _dp, _p1, _p2, _p3, _ext, _cm) = fields(&e);
            assert!((m - radius * 2.0).abs() < 1e-9);
            let value = text.trim_start_matches('Ø');
            let parsed: f64 = value.parse().expect("直径文字应为数值");
            assert!(
                (parsed - radius * 2.0).abs() < 0.01,
                "文字 {text} 与实际直径 {} 不符",
                radius * 2.0
            );
        }
    }

    // ---------------- 角度标注 ----------------

    #[test]
    fn angular_dimension_geometry() {
        let vertex = Point::origin();
        let e = make_angular(
            vertex,
            Point::new2d(10.0, 0.0),
            Point::new2d(0.0, 10.0),
            2.5,
        )
        .expect("角度标注应构造成功");
        let (ty, m, text, _tp, _th, _dp, p1, p2, p3, ext, cm) = fields(&e);
        assert_eq!(ty, DimensionType::Angular);
        assert!((m - 90.0).abs() < 1e-9, "夹角应为 90°，实际 {m}");
        assert_eq!(text, "90°");
        assert!(p1.distance_to(&vertex) < 1e-9, "def_point_1 应为顶点");
        assert!(
            (p2.distance_to(&vertex) - 10.0).abs() < 1e-9,
            "def_point_2 应为起始边上的弧起点"
        );
        assert!(
            (p3.distance_to(&vertex) - 10.0).abs() < 1e-9,
            "def_point_3 应为终止边上的弧终点"
        );
        assert!(ext, "角度标注应带尺寸界线");
        assert!(!cm);

        let strokes = decompose(e.geometry());
        assert!(strokes.len() >= 4, "角度标注应含圆弧、界线、箭头与文字");
        // 圆弧采样点的半径应等于第一条边长
        let (arc, _) = strokes.first().unwrap();
        for p in arc {
            assert!((p.distance_to(&vertex) - 10.0).abs() < 1e-9);
        }
    }

    #[test]
    fn angular_dimension_rejects_degenerate_input() {
        let v = Point::origin();
        assert!(make_angular(v, v, Point::new2d(1.0, 0.0), 2.5).is_none());
        assert!(make_angular(v, Point::new2d(1.0, 0.0), v, 2.5).is_none());
        assert!(make_angular(v, Point::new2d(1.0, 0.0), Point::new2d(1.0, 0.0), 2.5).is_none());
        assert!(make_angular(v, Point::new2d(1.0, 0.0), Point::new2d(0.0, 1.0), 0.0).is_none());
    }

    // ---------------- 分解与集成 ----------------

    #[test]
    fn decompose_ignores_non_dimension_geometry() {
        let other = EntityGeometry::Line(crate::geometry::Line::new(
            Point::origin(),
            Point::new2d(1.0, 1.0),
        ));
        assert!(decompose(&other).is_empty());
    }

    #[test]
    fn decompose_rejects_non_positive_text_height() {
        let e = make_radial(Point::origin(), 5.0, Point::new2d(5.0, 0.0), false, 2.5).unwrap();
        let mut g = e.geometry().clone();
        if let EntityGeometry::Dimension { text_height, .. } = &mut g {
            *text_height = 0.0;
        }
        assert!(decompose(&g).is_empty());
    }

    /// 标注几何应能被通用细分管线渲染 / 导出（tessellation 走 decompose）
    #[test]
    fn dimension_geometry_is_tessellated() {
        use crate::render::tessellation::{document_bbox, geometry_polylines};
        for e in [
            make_linear(
                Point::new2d(0.0, 0.0),
                Point::new2d(10.0, 0.0),
                Point::new2d(5.0, 4.0),
                2.5,
            )
            .unwrap(),
            make_radial(Point::origin(), 5.0, Point::new2d(5.0, 0.0), false, 2.5).unwrap(),
            make_radial(Point::origin(), 5.0, Point::new2d(0.0, 5.0), true, 2.5).unwrap(),
            make_angular(
                Point::origin(),
                Point::new2d(5.0, 0.0),
                Point::new2d(0.0, 5.0),
                2.5,
            )
            .unwrap(),
        ] {
            let polylines = geometry_polylines(e.geometry());
            assert!(!polylines.is_empty(), "标注实体未产生任何折线");

            let mut doc = crate::data_structure::Document::new("t".to_string());
            doc.add_entity(e);
            let (min, max) = document_bbox(&doc).expect("文档包围盒应包含标注");
            assert!(min.x.is_finite() && max.x.is_finite());
            assert!(max.x > min.x || max.y > min.y);
        }
    }

    #[test]
    fn readable_keeps_text_upright() {
        let pi = std::f64::consts::PI;
        // 第一、四象限方向保持原角
        assert!((readable(0.0) - 0.0).abs() < 1e-12);
        assert!((readable(pi / 4.0) - pi / 4.0).abs() < 1e-12);
        // 指向左侧时翻转 180°，保证从左到右可读
        let r = readable(pi);
        assert!((r - 0.0).abs() < 1e-12 || (r - 2.0 * pi).abs() < 1e-12);
    }

    #[test]
    fn number_formatting_trims_trailing_zeros() {
        assert_eq!(fmt_num(10.0), "10");
        assert_eq!(fmt_num(10.5), "10.5");
        assert_eq!(fmt_num(10.25), "10.25");
        assert_eq!(fmt_angle(90.0), "90");
        assert_eq!(fmt_angle(45.5), "45.5");
    }
}
