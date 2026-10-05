//! 曲线求交：直线段、圆、圆弧、折线、NURBS 与椭圆两两之间的交点计算。
//!
//! 统一返回 `IntersectionResult`：无交点、单点、多点或重叠（两条共线直线段）。每个交点附带
//! `IntersectionPoint`，其中 `parameter1`/`parameter2` 是两条曲线各自的参数——线段为 `[0, 1]`
//! 上的比例，圆/圆弧为弧度角，折线为按段累计的归一化比例。
//!
//! 计算只使用世界坐标的 x、y，z 分量不参与判断；除圆弧与椭圆会按自身角度范围过滤外，
//! 直线一律按“线段”而不是无限直线处理。折线与 NURBS 相关函数分别受 `polyline`、`nurbs`、
//! `ellipse` feature 控制。

use crate::geometry::{Point, Line, Circle, Arc};
#[cfg(feature = "ellipse")]
use crate::geometry::Ellipse;
#[cfg(feature = "polyline")]
use crate::geometry::Polyline;
#[cfg(any(feature = "nurbs", feature = "ellipse"))]
use crate::geometry::NURBS;

/// 一个交点及其在两条曲线上的参数。
#[derive(Debug, Clone)]
pub struct IntersectionPoint {
    /// 交点坐标（世界坐标；z 取输入曲线对应的值或 0）。
    pub point: Point,
    /// 第一条曲线上的参数：线段与折线为归一化比例，圆/圆弧为弧度角。
    pub parameter1: f64,
    /// 第二条曲线上的参数，含义同 `parameter1`；部分函数不填充该值（恒为 `0.0`）。
    pub parameter2: f64,
}

/// 求交结果。
#[derive(Debug, Clone)]
pub enum IntersectionResult {
    /// 无交点，或交点全部落在曲线的有效范围之外。
    None,
    /// 恰好一个交点，例如相切或只有一端命中。
    Point(IntersectionPoint),
    /// 两个及以上交点。
    Points(Vec<IntersectionPoint>),
    /// 两条直线段共线重叠，携带第一条线段。
    Overlapping(Line),
}

/// 求两条直线段的交点（按线段处理，不是无限直线）。
///
/// - `line1` / `line2`：两条线段，按值传入，函数不会修改调用方的数据。
///
/// 平行且共线时返回 `IntersectionResult::Overlapping(line1)`，平行但不共线返回 `None`；
/// 交点落在任一线段的延长线上（参数不在 `[0, 1]`）时同样返回 `None`。平行判定使用 `1e-10` 容差，
/// 交点的两个参数分别是两条线段上的比例。
///
/// # 示例
/// ```
/// use cadrs::geometry::{Point, Line};
/// use cadrs::geometry::intersection::{intersect_line_line, IntersectionResult};
///
/// let line1 = Line::new(Point::origin(), Point::new(1.0, 0.0, 0.0));
/// let line2 = Line::new(Point::new(0.5, -0.5, 0.0), Point::new(0.5, 0.5, 0.0));
///
/// match intersect_line_line(line1, line2) {
///     IntersectionResult::Point(ip) => assert!((ip.point.x - 0.5).abs() < 1e-10),
///     _ => panic!("期望得到单个交点"),
/// }
/// ```
#[inline]
pub fn intersect_line_line(line1: Line, line2: Line) -> IntersectionResult {
    let x1 = line1.start.x;
    let y1 = line1.start.y;
    let x2 = line1.end.x;
    let y2 = line1.end.y;
    let x3 = line2.start.x;
    let y3 = line2.start.y;
    let x4 = line2.end.x;
    let y4 = line2.end.y;

    let denom = (y4 - y3) * (x2 - x1) - (x4 - x3) * (y2 - y1);
    
    if denom.abs() < 1e-10 {
        if ((x3 - x1) * (y2 - y1) - (y3 - y1) * (x2 - x1)).abs() < 1e-10 {
            IntersectionResult::Overlapping(line1)
        } else {
            IntersectionResult::None
        }
    } else {
        let ua = ((x4 - x3) * (y1 - y3) - (y4 - y3) * (x1 - x3)) / denom;
        let ub = ((x2 - x1) * (y1 - y3) - (y2 - y1) * (x1 - x3)) / denom;

        if ua >= 0.0 && ua <= 1.0 && ub >= 0.0 && ub <= 1.0 {
            let point = Point::new(
                x1 + ua * (x2 - x1),
                y1 + ua * (y2 - y1),
                0.0,
            );
            IntersectionResult::Point(IntersectionPoint {
                point,
                parameter1: ua,
                parameter2: ub,
            })
        } else {
            IntersectionResult::None
        }
    }
}

/// 求直线段与圆的交点。
///
/// - `line`：线段；只有落在段内（参数 `[0, 1]`）的根才算命中，延长线上的交点被丢弃。
/// - `circle`：圆，由圆心与半径确定。
///
/// 判别式为负返回 `None`，相切或只有一个根落在段内返回 `Point`，两个根都在段内返回 `Points`；
/// 交点的 `parameter1` 是线段上的比例，`parameter2` 未填充（恒为 `0.0`）。
///
/// # 示例
/// ```
/// use cadrs::geometry::{Point, Line, Circle};
/// use cadrs::geometry::intersection::{intersect_line_circle, IntersectionResult};
///
/// let line = Line::new(Point::new(-1.0, 0.0, 0.0), Point::new(1.0, 0.0, 0.0));
/// let circle = Circle::new(Point::origin(), 0.5);
///
/// match intersect_line_circle(line, circle) {
///     IntersectionResult::Points(points) => assert_eq!(points.len(), 2),
///     _ => panic!("期望得到两个交点"),
/// }
/// ```
#[inline]
pub fn intersect_line_circle(line: Line, circle: Circle) -> IntersectionResult {
    let dx = line.end.x - line.start.x;
    let dy = line.end.y - line.start.y;
    let fx = line.start.x - circle.center.x;
    let fy = line.start.y - circle.center.y;

    let a = dx * dx + dy * dy;
    let b = 2.0 * (fx * dx + fy * dy);
    let c = fx * fx + fy * fy - circle.radius * circle.radius;

    let discriminant: f64 = b * b - 4.0 * a * c;

    if discriminant < -1e-10 {
        IntersectionResult::None
    } else if discriminant.abs() < 1e-10 {
        let t = -b / (2.0 * a);
        if t >= 0.0 && t <= 1.0 {
            let point = Point::new(
                line.start.x + t * dx,
                line.start.y + t * dy,
                line.start.z,
            );
            IntersectionResult::Point(IntersectionPoint {
                point,
                parameter1: t,
                parameter2: 0.0,
            })
        } else {
            IntersectionResult::None
        }
    } else {
        let sqrt_disc = discriminant.sqrt();
        let t1 = (-b - sqrt_disc) / (2.0 * a);
        let t2 = (-b + sqrt_disc) / (2.0 * a);

        let mut points = Vec::new();
        
        if t1 >= 0.0 && t1 <= 1.0 {
            points.push(IntersectionPoint {
                point: Point::new(line.start.x + t1 * dx, line.start.y + t1 * dy, line.start.z),
                parameter1: t1,
                parameter2: 0.0,
            });
        }
        if t2 >= 0.0 && t2 <= 1.0 {
            points.push(IntersectionPoint {
                point: Point::new(line.start.x + t2 * dx, line.start.y + t2 * dy, line.start.z),
                parameter1: t2,
                parameter2: 0.0,
            });
        }

        if points.is_empty() {
            IntersectionResult::None
        } else if points.len() == 1 {
            IntersectionResult::Point(points.remove(0))
        } else {
            IntersectionResult::Points(points)
        }
    }
}

/// 求两个圆的交点。
///
/// 圆心距大于半径和（外离）、小于半径差（内含）或两圆重合（同心且等半径）时返回 `None`；
/// 相切返回一个交点，两个交点时返回 `Points`。
/// 交点的 `parameter1` 是点在第一个圆上的弧度角，`parameter2` 未填充（恒为 `0.0`）。
#[inline]
pub fn intersect_circle_circle(circle1: Circle, circle2: Circle) -> IntersectionResult {
    let dx = circle2.center.x - circle1.center.x;
    let dy = circle2.center.y - circle1.center.y;
    let d = (dx * dx + dy * dy).sqrt();

    if d > circle1.radius + circle2.radius + 1e-10 {
        return IntersectionResult::None;
    }
    if d < (circle1.radius - circle2.radius).abs() - 1e-10 {
        return IntersectionResult::None;
    }
    if d < 1e-10 && (circle1.radius - circle2.radius).abs() < 1e-10 {
        return IntersectionResult::None;
    }

    let a = (circle1.radius * circle1.radius - circle2.radius * circle2.radius + d * d) / (2.0 * d);
    let h = (circle1.radius * circle1.radius - a * a).sqrt();

    let xm = circle1.center.x + a * dx / d;
    let ym = circle1.center.y + a * dy / d;

    if h.abs() < 1e-10 {
        let point = Point::new(xm, ym, 0.0);
        IntersectionResult::Point(IntersectionPoint {
            point,
            parameter1: 0.0,
            parameter2: 0.0,
        })
    } else {
        let rx = -dy * (h / d);
        let ry = dx * (h / d);

        let point1 = Point::new(xm + rx, ym + ry, 0.0);
        let point2 = Point::new(xm - rx, ym - ry, 0.0);

        let param1 = circle1.angle_from_center(&point1);
        let param2 = circle1.angle_from_center(&point2);

        IntersectionResult::Points(vec![
            IntersectionPoint { point: point1, parameter1: param1, parameter2: 0.0 },
            IntersectionPoint { point: point2, parameter1: param2, parameter2: 0.0 },
        ])
    }
}

#[inline]
pub fn intersect_line_arc(line: Line, arc: Arc) -> IntersectionResult {
    let circle = Circle::new(arc.center, arc.radius);
    let result = intersect_line_circle(line, circle);

    match result {
        IntersectionResult::None => IntersectionResult::None,
        IntersectionResult::Point(ip) => {
            let normalized_angle = arc.normalize_angle(arc.angle_from_center(&ip.point));
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

            if on_arc {
                IntersectionResult::Point(ip)
            } else {
                IntersectionResult::None
            }
        }
        IntersectionResult::Points(points) => {
            let filtered: Vec<IntersectionPoint> = points
                .into_iter()
                .filter(|ip| {
                    let normalized_angle = arc.normalize_angle(arc.angle_from_center(&ip.point));
                    let start = arc.normalize_angle(arc.start_angle);
                    let end = arc.normalize_angle(arc.end_angle);
                    
                    if arc.is_counter_clockwise {
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
                    }
                })
                .collect();

            if filtered.is_empty() {
                IntersectionResult::None
            } else if filtered.len() == 1 {
                IntersectionResult::Point(filtered[0].clone())
            } else {
                IntersectionResult::Points(filtered)
            }
        }
        IntersectionResult::Overlapping(_) => IntersectionResult::None,
    }
}

#[cfg(feature = "ellipse")]
#[inline]
pub fn intersect_arc_ellipse(arc: Arc, ellipse: Ellipse) -> IntersectionResult {
    let dx = ellipse.rotation.cos();
    let dy = ellipse.rotation.sin();
    let cos_rot = dx;
    let sin_rot = dy;

    let rx = arc.radius;
    let ry = arc.radius;

    let transformed_center_x = arc.center.x - ellipse.center.x;
    let transformed_center_y = arc.center.y - ellipse.center.y;

    let local_center_x = transformed_center_x * cos_rot + transformed_center_y * sin_rot;
    let local_center_y = -transformed_center_x * sin_rot + transformed_center_y * cos_rot;

    let a = ellipse.semi_major;
    let b = ellipse.semi_minor;

    let coefficients: Vec<f64> = vec![
        (a * a - local_center_x.powi(2)) * (b * b - local_center_y.powi(2)) - (local_center_x * local_center_y).powi(2),
        2.0 * local_center_x * local_center_y.powi(2),
        (local_center_x.powi(2) + local_center_y.powi(2)) * b.powi(2) - 2.0 * local_center_x.powi(2) * b.powi(2) - 2.0 * local_center_y.powi(2) * a.powi(2) + (a.powi(2) * b.powi(2) - rx.powi(2) * b.powi(2) - ry.powi(2) * a.powi(2)),
        2.0 * local_center_x * local_center_y * (rx.powi(2) + ry.powi(2) - a.powi(2) - b.powi(2)),
        local_center_x.powi(2) * (rx.powi(2) + ry.powi(2)) + local_center_y.powi(2) * (rx.powi(2) + ry.powi(2)) - rx.powi(2) * b.powi(2) - ry.powi(2) * a.powi(2),
    ];

    let roots = solve_quartic(coefficients);
    let mut intersection_points: Vec<IntersectionPoint> = Vec::new();

    for angle in roots {
        if angle.is_finite() {
            let t = (angle - arc.start_angle).abs() / (2.0 * std::f64::consts::PI);
            let point = Point::new(
                arc.center.x + arc.radius * angle.cos(),
                arc.center.y + arc.radius * angle.sin(),
                arc.center.z,
            );

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

            if on_arc && ellipse.contains_point(&point) {
                let param_ellipse = ((point.x - ellipse.center.x) * cos_rot + (point.y - ellipse.center.y) * sin_rot) / ellipse.semi_major;
                let param_ellipse2 = ((-point.x + ellipse.center.x) * sin_rot + (point.y - ellipse.center.y) * cos_rot) / ellipse.semi_minor;
                let ellipse_param = (param_ellipse + param_ellipse2) / 2.0;

                intersection_points.push(IntersectionPoint {
                    point,
                    parameter1: t.clamp(0.0, 1.0),
                    parameter2: ellipse_param.clamp(0.0, 1.0),
                });
            }
        }
    }

    if intersection_points.is_empty() {
        IntersectionResult::None
    } else if intersection_points.len() == 1 {
        IntersectionResult::Point(intersection_points[0].clone())
    } else {
        intersection_points.sort_by(|a, b| a.parameter1.partial_cmp(&b.parameter1).unwrap());
        IntersectionResult::Points(intersection_points)
    }
}

#[inline]
#[allow(dead_code)]
fn solve_quartic(coeffs: Vec<f64>) -> Vec<f64> {
    if coeffs.len() != 5 { return Vec::new(); }

    let a0 = coeffs[0];
    let a1 = coeffs[1];
    let a2 = coeffs[2];
    let a3 = coeffs[3];
    let a4 = coeffs[4];

    if a0.abs() < 1e-15 {
        solve_cubic(vec![a1, a2, a3, a4])
    } else {
        let normalized = vec![
            a4,
            a3,
            a2,
            a1,
            a0,
        ];

        if normalized[0].abs() < 1e-15 {
            solve_cubic(vec![normalized[1], normalized[2], normalized[3], normalized[4]])
        } else {
            let a = normalized[0];
            let b = normalized[1];
            let c = normalized[2];
            let d = normalized[3];
            let e = normalized[4];

            let ba = b / a;
            let ca = c / a;
            let da = d / a;
            let ea = e / a;

            let b2 = ba * ba;
            let _c2 = ca * ca;
            let _d2 = da * da;

            let p = (8.0 * ca - 3.0 * b2) / 8.0;
            let q = (b2 * ba - 4.0 * ca * ba + 8.0 * da - b2 * ba) / 8.0;
            let r = (b2 * b2 * ba - 4.0 * b2 * ca + 2.0 * ca * b2 + 4.0 * ba * da - 4.0 * ea) / 16.0;

            if q.abs() < 1e-15 && p.abs() < 1e-15 {
                let mut roots: Vec<f64> = Vec::new();
                let z = (-2.0 * b2 + 4.0 * ca - 8.0 * da + 3.0 * b2).sqrt() / 2.0;
                roots.push(-ba / 2.0 + z);
                roots.push(-ba / 2.0 - z);
                roots.push(-ba / 2.0 + z);
                roots.push(-ba / 2.0 - z);
                roots
            } else if p.abs() < 1e-15 {
                let discriminant = q * q - r;
                if discriminant < 0.0 { return Vec::new(); }
                let sqrt_disc = discriminant.sqrt();
                let z = (-sqrt_disc - q).sqrt() / 2.0;
                let z2 = (sqrt_disc - q).sqrt() / 2.0;
                let mut roots: Vec<f64> = Vec::new();
                roots.push(-ba / 2.0 + z);
                roots.push(-ba / 2.0 - z);
                roots.push(-ba / 2.0 + z2);
                roots.push(-ba / 2.0 - z2);
                roots
            } else {
                let discriminant = q * q - r * (4.0 * p - b2);
                if discriminant < 0.0 { return Vec::new(); }
                let sqrt_disc = discriminant.sqrt();
                let z1 = (-q + sqrt_disc) / 2.0;
                let z2 = (-q - sqrt_disc) / 2.0;

                let y1 = -p / 2.0 + (b2 - 4.0 * p).sqrt() / 2.0;
                let y2 = -p / 2.0 - (b2 - 4.0 * p).sqrt() / 2.0;

                let w1 = (-z1 + y1.sqrt()).sqrt();
                let w2 = (-z1 - y1.sqrt()).sqrt();
                let w3 = (-z2 + y2.sqrt()).sqrt();
                let w4 = (-z2 - y2.sqrt()).sqrt();

                let mut roots: Vec<f64> = Vec::new();
                roots.push(-ba / 2.0 + w1 + w2);
                roots.push(-ba / 2.0 + w3 + w4);
                roots.push(-ba / 2.0 - w1 - w2);
                roots.push(-ba / 2.0 - w3 - w4);
                roots
            }
        }
    }
}

#[inline]
#[allow(dead_code)]
fn solve_cubic(coeffs: Vec<f64>) -> Vec<f64> {
    if coeffs.len() != 4 { return Vec::new(); }

    let a = coeffs[0];
    let b = coeffs[1];
    let c = coeffs[2];
    let d = coeffs[3];

    if a.abs() < 1e-15 {
        solve_quadratic(vec![b, c, d])
    } else {
        let a1 = b / a;
        let a2 = c / a;
        let a3 = d / a;

        let q = (3.0 * a2 - a1.powi(2)) / 9.0;
        let r = (9.0 * a1 * a2 - 27.0 * a3 - 2.0 * a1.powi(3)) / 54.0;

        let discriminant = q * q * q + r * r;

        if discriminant > 1e-15 {
            let sqrt_disc = discriminant.sqrt();
            let s = (-r + sqrt_disc).cbrt();
            let t = (-r - sqrt_disc).cbrt();
            let mut roots: Vec<f64> = Vec::new();
            roots.push(s + t - a1 / 3.0);
            roots
        } else if discriminant.abs() < 1e-15 {
            if r.abs() < 1e-15 {
                let mut roots: Vec<f64> = Vec::new();
                roots.push(-a1 / 3.0);
                roots.push(-a1 / 3.0);
                roots.push(-a1 / 3.0);
                roots
            } else {
                let s = (-r).cbrt();
                let mut roots: Vec<f64> = Vec::new();
                roots.push(2.0 * s - a1 / 3.0);
                roots.push(-s - a1 / 3.0);
                roots
            }
        } else {
            let theta = (r / (-q * q * q).sqrt()).acos();
            let q_sqrt = (-q).sqrt();
            let mut roots: Vec<f64> = Vec::new();
            roots.push(2.0 * q_sqrt * (theta / 3.0).cos() - a1 / 3.0);
            roots.push(2.0 * q_sqrt * ((theta + 2.0 * std::f64::consts::PI) / 3.0).cos() - a1 / 3.0);
            roots.push(2.0 * q_sqrt * ((theta + 4.0 * std::f64::consts::PI) / 3.0).cos() - a1 / 3.0);
            roots
        }
    }
}

#[inline]
fn solve_quadratic(coeffs: Vec<f64>) -> Vec<f64> {
    if coeffs.len() != 3 { return Vec::new(); }

    let a = coeffs[0];
    let b = coeffs[1];
    let c = coeffs[2];

    if a.abs() < 1e-15 {
        if b.abs() < 1e-15 { return Vec::new(); }
        vec![-c / b]
    } else {
        let discriminant = b * b - 4.0 * a * c;
        if discriminant < -1e-15 { return Vec::new(); }
        if discriminant.abs() < 1e-15 { vec![-b / (2.0 * a)] } else {
            let sqrt_disc = discriminant.sqrt();
            vec![(-b - sqrt_disc) / (2.0 * a), (-b + sqrt_disc) / (2.0 * a)]
        }
    }
}

#[cfg(feature = "polyline")]
#[inline]
pub fn intersect_polyline_polyline(poly1: Polyline, poly2: Polyline) -> IntersectionResult {
    let mut intersection_points: Vec<IntersectionPoint> = Vec::new();

    for i in 0..poly1.vertices.len().saturating_sub(if poly1.is_closed { 0 } else { 1 }) {
        let next_i = if poly1.is_closed && i + 1 >= poly1.vertices.len() { 0 } else { i + 1 };
        if next_i > poly1.vertices.len() - 1 { continue; }

        let line1 = Line::new(
            Point::new(poly1.vertices[i].x, poly1.vertices[i].y, 0.0),
            Point::new(poly1.vertices[next_i].x, poly1.vertices[next_i].y, 0.0)
        );

        for j in 0..poly2.vertices.len().saturating_sub(if poly2.is_closed { 0 } else { 1 }) {
            let next_j = if poly2.is_closed && j + 1 >= poly2.vertices.len() { 0 } else { j + 1 };
            if next_j > poly2.vertices.len() - 1 { continue; }

            let line2 = Line::new(
                Point::new(poly2.vertices[j].x, poly2.vertices[j].y, 0.0),
                Point::new(poly2.vertices[next_j].x, poly2.vertices[next_j].y, 0.0)
            );

            match intersect_line_line(line1, line2) {
                IntersectionResult::Point(ip) => {
                    let param1 = (i as f64 + ip.parameter1) / (poly1.vertices.len() as f64 - 1.0);
                    let param2 = (j as f64 + ip.parameter2) / (poly2.vertices.len() as f64 - 1.0);
                    intersection_points.push(IntersectionPoint {
                        point: ip.point,
                        parameter1: param1.clamp(0.0, 1.0),
                        parameter2: param2.clamp(0.0, 1.0),
                    });
                }
                IntersectionResult::Points(ips) => {
                    for ip in ips {
                        let param1 = (i as f64 + ip.parameter1) / (poly1.vertices.len() as f64 - 1.0);
                        let param2 = (j as f64 + ip.parameter2) / (poly2.vertices.len() as f64 - 1.0);
                        intersection_points.push(IntersectionPoint {
                            point: ip.point,
                            parameter1: param1.clamp(0.0, 1.0),
                            parameter2: param2.clamp(0.0, 1.0),
                        });
                    }
                }
                IntersectionResult::Overlapping(_) => {
                    let mid_point = Point::new(
                        (line1.start.x + line1.end.x) / 2.0,
                        (line1.start.y + line1.end.y) / 2.0,
                        0.0,
                    );
                    let param1 = (i as f64 + 0.5) / (poly1.vertices.len() as f64 - 1.0);
                    let param2 = (j as f64 + 0.5) / (poly2.vertices.len() as f64 - 1.0);
                    intersection_points.push(IntersectionPoint {
                        point: mid_point,
                        parameter1: param1.clamp(0.0, 1.0),
                        parameter2: param2.clamp(0.0, 1.0),
                    });
                }
                IntersectionResult::None => {}
            }
        }
    }

    let mut unique_points: Vec<IntersectionPoint> = Vec::new();
    for ip in intersection_points {
        let mut is_duplicate = false;
        for existing in &unique_points {
            if ip.point.distance_to(&existing.point) < 1e-8 {
                is_duplicate = true;
                break;
            }
        }
        if !is_duplicate {
            unique_points.push(ip);
        }
    }

    if unique_points.is_empty() {
        IntersectionResult::None
    } else if unique_points.len() == 1 {
        IntersectionResult::Point(unique_points[0].clone())
    } else {
        IntersectionResult::Points(unique_points)
    }
}

#[cfg(feature = "polyline")]
#[inline]
pub fn intersect_polyline_line(polyline: Polyline, line: Line) -> IntersectionResult {
    let mut points: Vec<IntersectionPoint> = Vec::new();

    for i in 0..polyline.vertices.len().saturating_sub(if polyline.is_closed { 0 } else { 1 }) {
        let next_i = if polyline.is_closed && i + 1 >= polyline.vertices.len() { 0 } else { i + 1 };
        if next_i > polyline.vertices.len() - 1 { continue; }

        let segment = Line::new(
            Point::new(polyline.vertices[i].x, polyline.vertices[i].y, 0.0),
            Point::new(polyline.vertices[next_i].x, polyline.vertices[next_i].y, 0.0)
        );

        match intersect_line_line(segment, line) {
            IntersectionResult::Point(ip) => {
                let param = (i as f64 + ip.parameter1) / (polyline.vertices.len() as f64 - 1.0);
                points.push(IntersectionPoint {
                    point: ip.point,
                    parameter1: param.clamp(0.0, 1.0),
                    parameter2: ip.parameter2,
                });
            }
            IntersectionResult::Points(ips) => {
                for ip in ips {
                    let param = (i as f64 + ip.parameter1) / (polyline.vertices.len() as f64 - 1.0);
                    points.push(IntersectionPoint {
                        point: ip.point,
                        parameter1: param.clamp(0.0, 1.0),
                        parameter2: ip.parameter2,
                    });
                }
            }
            IntersectionResult::Overlapping(_) => {
                let mid = Point::new(
                    (segment.start.x + segment.end.x) / 2.0,
                    (segment.start.y + segment.end.y) / 2.0,
                    0.0,
                );
                let param = (i as f64 + 0.5) / (polyline.vertices.len() as f64 - 1.0);
                points.push(IntersectionPoint {
                    point: mid,
                    parameter1: param.clamp(0.0, 1.0),
                    parameter2: 0.5,
                });
            }
            IntersectionResult::None => {}
        }
    }

    if points.is_empty() {
        IntersectionResult::None
    } else if points.len() == 1 {
        IntersectionResult::Point(points[0].clone())
    } else {
        IntersectionResult::Points(points)
    }
}

#[cfg(feature = "polyline")]
#[inline]
pub fn intersect_polyline_circle(polyline: Polyline, circle: Circle) -> IntersectionResult {
    let mut points: Vec<IntersectionPoint> = Vec::new();

    for i in 0..polyline.vertices.len().saturating_sub(if polyline.is_closed { 0 } else { 1 }) {
        let next_i = if polyline.is_closed && i + 1 >= polyline.vertices.len() { 0 } else { i + 1 };
        if next_i > polyline.vertices.len() - 1 { continue; }

        let segment = Line::new(
            Point::new(polyline.vertices[i].x, polyline.vertices[i].y, 0.0),
            Point::new(polyline.vertices[next_i].x, polyline.vertices[next_i].y, 0.0)
        );

        match intersect_line_circle(segment, circle) {
            IntersectionResult::Point(ip) => {
                let param = (i as f64 + ip.parameter1) / (polyline.vertices.len() as f64 - 1.0);
                points.push(IntersectionPoint {
                    point: ip.point,
                    parameter1: param.clamp(0.0, 1.0),
                    parameter2: ip.parameter2,
                });
            }
            IntersectionResult::Points(ips) => {
                for ip in ips {
                    let param = (i as f64 + ip.parameter1) / (polyline.vertices.len() as f64 - 1.0);
                    points.push(IntersectionPoint {
                        point: ip.point,
                        parameter1: param.clamp(0.0, 1.0),
                        parameter2: ip.parameter2,
                    });
                }
            }
            IntersectionResult::None => {}
            IntersectionResult::Overlapping(_) => {}
        }
    }

    if points.is_empty() {
        IntersectionResult::None
    } else if points.len() == 1 {
        IntersectionResult::Point(points[0].clone())
    } else {
        IntersectionResult::Points(points)
    }
}

#[cfg(feature = "nurbs")]
#[inline]
pub fn intersect_nurbs_nurbs(nurbs1: NURBS, nurbs2: NURBS) -> IntersectionResult {
    let num_samples = 100;
    let mut potential_intersections: Vec<(f64, f64)> = Vec::new();

    for i in 0..=num_samples {
        let t1 = i as f64 / num_samples as f64;
        let p1 = nurbs1.point_at(t1);

        for j in 0..=num_samples {
            let t2 = j as f64 / num_samples as f64;
            let p2 = nurbs2.point_at(t2);

            if p1.distance_to(&p2) < 1e-3 {
                potential_intersections.push((t1, t2));
            }
        }
    }

    let mut refined_points: Vec<IntersectionPoint> = Vec::new();

    for (t1_init, t2_init) in potential_intersections {
        let result = refine_nurbs_intersection(&nurbs1, &nurbs2, t1_init, t2_init);
        if let Some((t1, t2, point)) = result {
            if !refined_points.iter().any(|ip| ip.point.distance_to(&point) < 1e-6) {
                refined_points.push(IntersectionPoint {
                    point,
                    parameter1: t1,
                    parameter2: t2,
                });
            }
        }
    }

    if refined_points.is_empty() {
        IntersectionResult::None
    } else if refined_points.len() == 1 {
        IntersectionResult::Point(refined_points[0].clone())
    } else {
        refined_points.sort_by(|a, b| a.parameter1.partial_cmp(&b.parameter1).unwrap());
        IntersectionResult::Points(refined_points)
    }
}

#[cfg(feature = "nurbs")]
#[inline]
fn refine_nurbs_intersection(nurbs1: &NURBS, nurbs2: &NURBS, t1: f64, t2: f64) -> Option<(f64, f64, Point)> {
    let mut t1 = t1.clamp(0.0, 1.0);
    let mut t2 = t2.clamp(0.0, 1.0);

    for _ in 0..20 {
        let p1 = nurbs1.point_at(t1);
        let p2 = nurbs2.point_at(t2);
        let d = p1.distance_to(&p2);

        if d < 1e-8 {
            return Some((t1, t2, p1));
        }

        let tangent1 = nurbs1.point_at((t1 + 0.01).clamp(0.0, 1.0)) - p1;
        let tangent2 = nurbs2.point_at((t2 + 0.01).clamp(0.0, 1.0)) - p2;

        let cross_z = tangent1.x * tangent2.y - tangent1.y * tangent2.x;

        if cross_z.abs() < 1e-10 {
            break;
        }

        let delta_p = p2 - p1;
        let dt1 = (delta_p.x * tangent2.y - delta_p.y * tangent2.x) / cross_z;
        let dt2 = (delta_p.x * tangent1.y - delta_p.y * tangent1.x) / cross_z;

        t1 = (t1 + dt1 * 0.5).clamp(0.0, 1.0);
        t2 = (t2 + dt2 * 0.5).clamp(0.0, 1.0);
    }

    let p1 = nurbs1.point_at(t1);
    let p2 = nurbs2.point_at(t2);

    if p1.distance_to(&p2) < 1e-4 {
        Some((t1, t2, Point::new((p1.x + p2.x) / 2.0, (p1.y + p2.y) / 2.0, 0.0)))
    } else {
        None
    }
}

#[cfg(feature = "nurbs")]
#[inline]
pub fn intersect_nurbs_line(nurbs: NURBS, line: Line) -> IntersectionResult {
    let num_samples = 100;
    let mut potential_params: Vec<f64> = Vec::new();

    for i in 0..=num_samples {
        let t = i as f64 / num_samples as f64;
        let point = nurbs.point_at(t);

        if line.distance_to_point(&point) < 1e-3 {
            potential_params.push(t);
        }
    }

    let mut refined_points: Vec<IntersectionPoint> = Vec::new();

    for t_init in potential_params {
        let result = refine_nurbs_line_intersection(&nurbs, &line, t_init);
        if let Some((t, point)) = result {
            if !refined_points.iter().any(|ip| ip.point.distance_to(&point) < 1e-6) {
                refined_points.push(IntersectionPoint {
                    point,
                    parameter1: t,
                    parameter2: 0.0,
                });
            }
        }
    }

    if refined_points.is_empty() {
        IntersectionResult::None
    } else if refined_points.len() == 1 {
        IntersectionResult::Point(refined_points[0].clone())
    } else {
        refined_points.sort_by(|a, b| a.parameter1.partial_cmp(&b.parameter1).unwrap());
        IntersectionResult::Points(refined_points)
    }
}

#[cfg(feature = "nurbs")]
#[inline]
fn refine_nurbs_line_intersection(nurbs: &NURBS, line: &Line, t: f64) -> Option<(f64, Point)> {
    let mut t = t.clamp(0.0, 1.0);

    for _ in 0..20 {
        let point = nurbs.point_at(t);
        let closest = line.closest_point(&point);

        let d = point.distance_to(&closest);
        if d < 1e-8 {
            return Some((t, closest));
        }

        let tangent = nurbs.point_at((t + 0.01).clamp(0.0, 1.0)) - point;

        let to_line = closest - point;
        let projection = tangent.x * to_line.x + tangent.y * to_line.y;

        if tangent.magnitude() < 1e-10 {
            break;
        }

        let dt = projection / tangent.magnitude().powi(2);
        t = (t + dt * 0.5).clamp(0.0, 1.0);
    }

    let point = nurbs.point_at(t);
    let closest = line.closest_point(&point);

    if point.distance_to(&closest) < 1e-4 {
        Some((t, closest))
    } else {
        None
    }
}

#[cfg(feature = "nurbs")]
#[inline]
pub fn intersect_nurbs_circle(nurbs: NURBS, circle: Circle) -> IntersectionResult {
    let num_samples = 100;
    let mut potential_params: Vec<f64> = Vec::new();

    for i in 0..=num_samples {
        let t = i as f64 / num_samples as f64;
        let point = nurbs.point_at(t);

        if point.distance_to(&circle.center) < circle.radius + 1e-3 {
            potential_params.push(t);
        }
    }

    let mut refined_points: Vec<IntersectionPoint> = Vec::new();

    for t_init in potential_params {
        let result = refine_nurbs_circle_intersection(&nurbs, &circle, t_init);
        if let Some((t, point)) = result {
            if !refined_points.iter().any(|ip| ip.point.distance_to(&point) < 1e-6) {
                let angle = circle.angle_from_center(&point);
                refined_points.push(IntersectionPoint {
                    point,
                    parameter1: t,
                    parameter2: angle,
                });
            }
        }
    }

    if refined_points.is_empty() {
        IntersectionResult::None
    } else if refined_points.len() == 1 {
        IntersectionResult::Point(refined_points[0].clone())
    } else {
        refined_points.sort_by(|a, b| a.parameter1.partial_cmp(&b.parameter1).unwrap());
        IntersectionResult::Points(refined_points)
    }
}

#[cfg(feature = "ellipse")]
#[inline]
fn refine_nurbs_circle_intersection(nurbs: &NURBS, circle: &Circle, t: f64) -> Option<(f64, Point)> {
    let mut t = t.clamp(0.0, 1.0);

    for _ in 0..20 {
        let point = nurbs.point_at(t);
        let to_center = circle.center - point;
        let d = to_center.magnitude();

        if (d - circle.radius).abs() < 1e-8 {
            return Some((t, point));
        }

        let tangent = nurbs.point_at((t + 0.01).clamp(0.0, 1.0)) - point;

        let normal = if d > 1e-10 { to_center / d } else { Point::new(1.0, 0.0, 0.0) };
        let projection = tangent.x * normal.x + tangent.y * normal.y;

        if tangent.magnitude() < 1e-10 {
            break;
        }

        let dt = ((circle.radius - d) / d * to_center.dot(&normal) + projection) / tangent.magnitude().powi(2);
        t = (t + dt * 0.5).clamp(0.0, 1.0);
    }

    let point = nurbs.point_at(t);
    if (point.distance_to(&circle.center) - circle.radius).abs() < 1e-4 {
        Some((t, point))
    } else {
        None
    }
}

#[cfg(feature = "ellipse")]
#[inline]
pub fn intersect_ellipse_ellipse(ellipse1: Ellipse, ellipse2: Ellipse) -> IntersectionResult {
    let dx = ellipse2.center.x - ellipse1.center.x;
    let dy = ellipse2.center.y - ellipse1.center.y;
    let d = (dx * dx + dy * dy).sqrt();

    if d > ellipse1.semi_major + ellipse2.semi_major + 1e-10 {
        return IntersectionResult::None;
    }

    if d < (ellipse1.semi_major - ellipse2.semi_major).abs() - 1e-10 && d < (ellipse1.semi_minor - ellipse2.semi_minor).abs() - 1e-10 {
        return IntersectionResult::None;
    }

    let num_samples = 72;
    let angle_step = 2.0 * std::f64::consts::PI / num_samples as f64;
    let mut points: Vec<Point> = Vec::new();

    for i in 0..num_samples {
        let angle = i as f64 * angle_step;
        let point = ellipse1.point_at_parameter(angle / (2.0 * std::f64::consts::PI));

        if ellipse2.contains_point(&point) {
            if !points.iter().any(|p| p.distance_to(&point) < 1e-6) {
                points.push(point);
            }
        }
    }

    for i in 0..num_samples {
        let angle = i as f64 * angle_step;
        let point = ellipse2.point_at_parameter(angle / (2.0 * std::f64::consts::PI));

        if ellipse1.contains_point(&point) {
            if !points.iter().any(|p| p.distance_to(&point) < 1e-6) {
                points.push(point);
            }
        }
    }

    if points.is_empty() {
        let mid = Point::new(
            (ellipse1.center.x + ellipse2.center.x) / 2.0,
            (ellipse1.center.y + ellipse2.center.y) / 2.0,
            0.0,
        );
        if ellipse1.contains_point(&mid) && ellipse2.contains_point(&mid) {
            return IntersectionResult::Point(IntersectionPoint {
                point: mid,
                parameter1: 0.5,
                parameter2: 0.5,
            });
        }
        return IntersectionResult::None;
    }

    let intersection_points: Vec<IntersectionPoint> = points
        .into_iter()
        .enumerate()
        .map(|(i, point)| {
            let angle1 = ((point.x - ellipse1.center.x) * (ellipse1.rotation).cos() + (point.y - ellipse1.center.y) * (ellipse1.rotation).sin()) / ellipse1.semi_major;
            let param1 = angle1.atan2(((point.x - ellipse1.center.x) * -(ellipse1.rotation).sin() + (point.y - ellipse1.center.y) * (ellipse1.rotation).cos()) / ellipse1.semi_minor) / (2.0 * std::f64::consts::PI);
            let param2 = ((point.x - ellipse2.center.x) * (ellipse2.rotation).cos() + (point.y - ellipse2.center.y) * (ellipse2.rotation).sin()) / ellipse2.semi_major;

            IntersectionPoint {
                point,
                parameter1: param1.clamp(0.0, 1.0),
                parameter2: param2.clamp(0.0, 1.0),
            }
        })
        .collect();

    if intersection_points.len() == 1 {
        IntersectionResult::Point(intersection_points[0].clone())
    } else {
        IntersectionResult::Points(intersection_points)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_intersect_line_line() {
        let line1 = Line::new(Point::origin(), Point::new(1.0, 0.0, 0.0));
        let line2 = Line::new(Point::new(0.5, -0.5, 0.0), Point::new(0.5, 0.5, 0.0));
        
        match intersect_line_line(line1, line2) {
            IntersectionResult::Point(ip) => {
                assert!((ip.point.x - 0.5).abs() < 1e-10, "X coordinate mismatch");
                assert!((ip.point.y - 0.0).abs() < 1e-10, "Y coordinate mismatch");
            }
            _ => {
                assert!(false, "Expected intersection point, but got no intersection");
            }
        }
    }

    #[test]
    fn test_intersect_line_circle() {
        let line = Line::new(Point::new(-1.0, 0.0, 0.0), Point::new(1.0, 0.0, 0.0));
        let circle = Circle::new(Point::origin(), 0.5);
        
        match intersect_line_circle(line, circle) {
            IntersectionResult::Points(points) => {
                assert_eq!(points.len(), 2, "Expected 2 intersection points");
            }
            _ => {
                assert!(false, "Expected intersection points, but got none");
            }
        }
    }

    #[test]
    fn test_intersect_circle_circle() {
        let circle1 = Circle::new(Point::new(-0.5, 0.0, 0.0), 1.0);
        let circle2 = Circle::new(Point::new(0.5, 0.0, 0.0), 1.0);
        
        match intersect_circle_circle(circle1, circle2) {
            IntersectionResult::Points(points) => {
                assert_eq!(points.len(), 2, "Expected 2 intersection points");
            }
            _ => {
                assert!(false, "Expected intersection points, but got none");
            }
        }
    }
}
