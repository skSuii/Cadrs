//! 实体构造工具：按几何类型构造带正确 EntityType 的实体。

use super::{Entity, EntityGeometry, EntityType};
use crate::geometry::extended_geometry;
use crate::geometry::{Arc, BSpline, Circle, Ellipse, Line, Point, Polyline};

/// 构造带正确类型的实体
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
pub fn clone_with_new_id(e: &Entity) -> Entity {
    let mut n = Entity::new(e.entity_type.clone(), e.geometry().clone());
    n.layer_id = e.layer_id.clone();
    n.properties = e.properties.clone();
    n.visibility = e.visibility;
    n.transform = e.transform.clone();
    n
}

pub fn make_line(a: Point, b: Point) -> Entity {
    make_entity(EntityGeometry::Line(Line::new(a, b)))
}

pub fn make_circle(center: Point, radius: f64) -> Entity {
    make_entity(EntityGeometry::Circle(Circle::new(center, radius)))
}

pub fn make_point(p: Point) -> Entity {
    make_entity(EntityGeometry::Point(p))
}

/// 圆弧：始终从 start 到 end 逆时针
pub fn make_arc(center: Point, radius: f64, start_angle: f64, end_angle: f64) -> Entity {
    let mut arc = Arc::new(center, radius, start_angle, end_angle);
    arc.is_counter_clockwise = true;
    make_entity(EntityGeometry::Arc(arc))
}

pub fn make_ellipse(center: Point, semi_major: f64, semi_minor: f64, rotation: f64) -> Entity {
    make_entity(EntityGeometry::Ellipse(Ellipse::new(
        center,
        semi_major,
        semi_minor,
        rotation,
    )))
}

/// B 样条：控制点拟合，阶数 = min(3, 点数-1)
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

pub fn make_polyline(points: &[Point], closed: bool) -> Entity {
    let vertices: Vec<extended_geometry::Point> = points
        .iter()
        .map(|p| extended_geometry::Point { x: p.x, y: p.y })
        .collect();
    make_entity(EntityGeometry::Polyline(Polyline::new(vertices, closed)))
}

/// 实心填充多边形（Solid 实体，三角形时第 4 点重复第 3 点）
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
