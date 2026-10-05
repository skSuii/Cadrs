//! 实体构造工具：按几何类型构造带正确 EntityType 的实体。
//!
//! 各 `make_*` 函数是创建实体的推荐入口：它们负责把 [`EntityGeometry`] 变体与
//! [`EntityType`] 配对，避免手工构造时二者不一致。返回的实体标识随机生成、图层为空标识
//! （[`crate::data_structure::ObjectId::nil`]），需由调用方通过 [`crate::data_structure::Document::add_entity`]
//! 登记入文档后才参与显示与查询。坐标单位为文档单位，角度参数一律为弧度。

use super::{Entity, EntityGeometry, EntityType};
use crate::geometry::extended_geometry;
use crate::geometry::{Arc, BSpline, Circle, Ellipse, Line, Point, Polyline};

/// 构造带正确类型的实体。
///
/// 按 `geometry` 的变体推断 [`EntityType`]，因此实体的类别始终与几何数据一致；
/// 无法识别的变体（如块参照）退化为 [`EntityType::Point`] 标记。
///
/// - `geometry`：实体几何数据，被移动进返回的实体。
///
/// # 示例
/// ```text
/// let entity = make_entity(EntityGeometry::Line(Line::new(Point::origin(), Point::new(1.0, 0.0, 0.0))));
/// assert_eq!(entity.entity_type(), &EntityType::Line);
/// ```
pub fn make_entity(geometry: EntityGeometry) -> Entity {
    let entity_type = match &geometry {
        EntityGeometry::Line(_) => EntityType::Line,
        EntityGeometry::Circle(_) => EntityType::Circle,
        EntityGeometry::Arc(_) => EntityType::Arc,
        EntityGeometry::Ellipse(_) => EntityType::Ellipse,
        EntityGeometry::Polyline(_) => EntityType::Polyline,
        EntityGeometry::BSpline(_) => EntityType::BSpline,
        EntityGeometry::NURBS(_) => EntityType::NURBS,
        EntityGeometry::Point(_) => EntityType::Point,
        EntityGeometry::Dimension { .. } => EntityType::Dimension,
        EntityGeometry::Text { .. } => EntityType::Text,
        EntityGeometry::Solid { .. } => EntityType::Solid,
        EntityGeometry::Hatch { .. } => EntityType::Hatch,
        _ => EntityType::Point,
    };
    Entity::new(entity_type, geometry)
}

/// 复制实体并生成新 id（复制编辑用；克隆体保留图层/可见性/属性）
///
/// 几何、图层、可见性、变换与自定义属性均被浅拷贝，唯独标识是新的，因此可作为独立实体
/// 加入同一文档而不会覆盖原对象。
///
/// - `e`：被复制的源实体，不修改。
pub fn clone_with_new_id(e: &Entity) -> Entity {
    let mut n = Entity::new(e.entity_type.clone(), e.geometry().clone());
    n.layer_id = e.layer_id.clone();
    n.properties = e.properties.clone();
    n.visibility = e.visibility;
    n.transform = e.transform.clone();
    n
}

/// 构造直线段实体，含两端点，坐标为文档单位。
///
/// - `a`：起点；
/// - `b`：终点。
pub fn make_line(a: Point, b: Point) -> Entity {
    make_entity(EntityGeometry::Line(Line::new(a, b)))
}

/// 构造圆实体。
///
/// - `center`：圆心坐标；
/// - `radius`：半径，单位为文档单位，负值不合法。
pub fn make_circle(center: Point, radius: f64) -> Entity {
    make_entity(EntityGeometry::Circle(Circle::new(center, radius)))
}

/// 构造点实体，包围盒退化为该点自身。
pub fn make_point(p: Point) -> Entity {
    make_entity(EntityGeometry::Point(p))
}

/// 圆弧：始终从 start 到 end 逆时针
///
/// 无论 `start_angle` 与 `end_angle` 的大小关系如何，生成的圆弧一律按逆时针方向从起点角
/// 扫到终点角，即跨越 0 弧度时弧段会经过正 X 轴。
///
/// - `center`：圆心坐标；
/// - `radius`：半径；
/// - `start_angle`：起始角，弧度；
/// - `end_angle`：终止角，弧度。
pub fn make_arc(center: Point, radius: f64, start_angle: f64, end_angle: f64) -> Entity {
    let mut arc = Arc::new(center, radius, start_angle, end_angle);
    arc.is_counter_clockwise = true;
    make_entity(EntityGeometry::Arc(arc))
}

/// 构造椭圆（弧）实体。
///
/// - `center`：椭圆中心；
/// - `semi_major`：长半轴长度；
/// - `semi_minor`：短半轴长度，通常不大于长半轴；
/// - `rotation`：长轴相对 X 轴的旋转角，弧度。
pub fn make_ellipse(center: Point, semi_major: f64, semi_minor: f64, rotation: f64) -> Entity {
    make_entity(EntityGeometry::Ellipse(Ellipse::new(
        center,
        semi_major,
        semi_minor,
        rotation,
    )))
}

/// B 样条：控制点拟合，阶数 = min(3, 点数-1)
///
/// - `points`：控制点序列。
///
/// 点数少于 3 时无法拟合，返回 [`None`]，调用方需处理该失败分支。
pub fn make_spline(points: &[Point]) -> Option<Entity> {
    if points.len() < 3 {
        return None;
    }
    let degree = 3.min(points.len() - 1);
    Some(make_entity(EntityGeometry::BSpline(BSpline::from_points(
        points.to_vec(),
        degree,
    ))))
}

/// 构造折线实体，按给定顺序连接各顶点。
///
/// - `points`：顶点序列，两两点之间为直线段；
/// - `closed`：为真时自动补一段由末点回到首点的闭合边。
pub fn make_polyline(points: &[Point], closed: bool) -> Entity {
    let vertices: Vec<extended_geometry::Point> = points
        .iter()
        .map(|p| extended_geometry::Point { x: p.x, y: p.y })
        .collect();
    make_entity(EntityGeometry::Polyline(Polyline::new(vertices, closed)))
}

/// 实心填充多边形（Solid 实体，三角形时第 4 点重复第 3 点）
///
/// - `points`：多边形顶点，仅接受 3 或 4 个；不足 4 个时自动复制末点补齐；
/// - `color`：填充颜色的 RGB 分量。
///
/// 顶点数少于 3 或多于 4 时返回 [`None`]。
pub fn make_solid(points: &[Point], color: (u8, u8, u8)) -> Option<Entity> {
    if points.len() < 3 || points.len() > 4 {
        return None;
    }
    let mut pts = points.to_vec();
    while pts.len() < 4 {
        pts.push(*pts.last()?);
    }
    Some(make_entity(EntityGeometry::Solid {
        points: [pts[0], pts[1], pts[2], pts[3]],
        color,
    }))
}

/// 闭合边界的实心填充（Hatch 实体，SOLID 图案）
///
/// 按顶点顺序生成首尾相接的直线边并自动闭合，生成单条外部边界（`is_outer = true`）、
/// `solid_fill = true`、`associativity = false` 的填充：边界后续变动不会自动更新填充。
///
/// - `boundary`：边界顶点，按顺序连线，首尾自动闭合，不需要重复首点；
/// - `color`：填充颜色的 RGB 分量。
///
/// 顶点少于 3 个时返回 [`None`]。
pub fn make_hatch(boundary: &[Point], color: (u8, u8, u8)) -> Option<Entity> {
    if boundary.len() < 3 {
        return None;
    }
    use super::{BoundaryType, EdgeType, HatchBoundary, HatchEdge};
    let edges: Vec<HatchEdge> = boundary
        .windows(2)
        .map(|w| HatchEdge {
            edge_type: EdgeType::Line,
            start_point: w[0],
            end_point: w[1],
            center_point: None,
            radius: None,
            start_angle: None,
            end_angle: None,
            bulge: None,
        })
        .chain(std::iter::once(HatchEdge {
            edge_type: EdgeType::Line,
            start_point: *boundary.last()?,
            end_point: boundary[0],
            center_point: None,
            radius: None,
            start_angle: None,
            end_angle: None,
            bulge: None,
        }))
        .collect();
    Some(make_entity(EntityGeometry::Hatch {
        pattern_name: "SOLID".to_string(),
        pattern_scale: 1.0,
        pattern_angle: 0.0,
        solid_fill: true,
        fill_color: color,
        boundary_paths: vec![HatchBoundary {
            boundary_type: BoundaryType::External,
            edges,
            is_outer: true,
            is_polyline: true,
        }],
        associativity: false,
    }))
}
