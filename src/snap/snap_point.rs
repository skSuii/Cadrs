use serde::{Serialize, Deserialize};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum SnapType {
    EndPoint,
    MidPoint,
    Center,
    Node,
    Quadrant,
    Intersection,
    Perpendicular,
    Tangent,
    Nearest,
    Extension,
    Parallel,
    Horizontal,
    Vertical,
    None,
}

impl Default for SnapType {
    fn default() -> Self {
        SnapType::None
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct SnapPriority {
    pub priority: u32,
    pub weight: f64,
}

impl Default for SnapPriority {
    fn default() -> Self {
        Self {
            priority: 0,
            weight: 1.0,
        }
    }
}

impl SnapPriority {
    pub fn new(priority: u32) -> Self {
        Self {
            priority,
            weight: 1.0,
        }
    }

    pub fn with_weight(mut self, weight: f64) -> Self {
        self.weight = weight;
        self
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SnapPoint {
    pub point: crate::geometry::Point,
    pub snap_type: SnapType,
    pub entity_id: Option<super::super::data_structure::ObjectId>,
    pub distance: f64,
    pub priority: SnapPriority,
}

impl Default for SnapPoint {
    fn default() -> Self {
        Self {
            point: crate::geometry::Point::new(0.0, 0.0, 0.0),
            snap_type: SnapType::None,
            entity_id: None,
            distance: f64::MAX,
            priority: SnapPriority::default(),
        }
    }
}

impl SnapPoint {
    pub fn new(point: crate::geometry::Point, snap_type: SnapType) -> Self {
        Self {
            point,
            snap_type,
            entity_id: None,
            distance: 0.0,
            priority: SnapPriority::default(),
        }
    }

    pub fn with_entity(point: crate::geometry::Point, snap_type: SnapType, entity_id: super::super::data_structure::ObjectId) -> Self {
        Self {
            point,
            snap_type,
            entity_id: Some(entity_id),
            distance: 0.0,
            priority: SnapPriority::default(),
        }
    }

    pub fn set_distance(&mut self, distance: f64) {
        self.distance = distance;
    }

    pub fn set_priority(&mut self, priority: u32) {
        self.priority = SnapPriority::new(priority);
    }
}

pub trait SnapCalculator {
    fn calculate_endpoint(&self, entity: &super::super::data_structure::Entity) -> Option<Snapshot>;
    fn calculate_midpoint(&self, entity: &super::super::data_structure::Entity) -> Option<Snapshot>;
    fn calculate_center(&self, entity: &super::super::data_structure::Entity) -> Option<Snapshot>;
    fn calculate_node(&self, entity: &super::super::data_structure::Entity) -> Option<Snapshot>;
    fn calculate_quadrant(&self, entity: &super::super::data_structure::Entity) -> Vec<Snapshot>;
    fn calculate_intersection(&self, entity1: &super::super::data_structure::Entity, entity2: &super::super::data_structure::Entity) -> Option<Snapshot>;
    fn calculate_perpendicular(&self, entity: &super::super::data_structure::Entity, from_point: crate::geometry::Point) -> Option<Snapshot>;
    fn calculate_tangent(&self, entity: &super::super::data_structure::Entity, from_point: crate::geometry::Point) -> Option<Snapshot>;
    fn calculate_nearest(&self, entity: &super::super::data_structure::Entity, to_point: crate::geometry::Point) -> Option<Snapshot>;
    fn calculate_extension(&self, entity: &super::super::data_structure::Entity, from_point: crate::geometry::Point, extension_distance: f64) -> Option<Snapshot>;
}

#[derive(Debug, Clone)]
pub struct Snapshot {
    pub point: crate::geometry::Point,
    pub snap_type: SnapType,
    pub entity_id: super::super::data_structure::ObjectId,
    pub sub_entity_index: Option<usize>,
}

impl Snapshot {
    pub fn new(point: crate::geometry::Point, snap_type: SnapType, entity_id: super::super::data_structure::ObjectId) -> Self {
        Self {
            point,
            snap_type,
            entity_id,
            sub_entity_index: None,
        }
    }

    pub fn with_sub_index(mut self, index: usize) -> Self {
        self.sub_entity_index = Some(index);
        self
    }
}

pub struct SnapCalculatorImpl;

impl SnapCalculator for SnapCalculatorImpl {
    fn calculate_endpoint(&self, entity: &super::super::data_structure::Entity) -> Option<Snapshot> {
        match &entity.geometry {
            super::super::data_structure::EntityGeometry::Line(line) => {
                Some(Snapshot::new(line.start.clone(), SnapType::EndPoint, entity.id.clone()))
            }
            super::super::data_structure::EntityGeometry::Polyline(polyline) => {
                polyline.vertices.first().map(|first| {
                    Snapshot::new(crate::geometry::Point::new(first.x, first.y, 0.0), SnapType::EndPoint, entity.id.clone())
                })
            }
            _ => None,
        }
    }

    fn calculate_midpoint(&self, entity: &super::super::data_structure::Entity) -> Option<Snapshot> {
        match &entity.geometry {
            super::super::data_structure::EntityGeometry::Line(line) => {
                let mid = line.start.midpoint(&line.end);
                Some(Snapshot::new(mid, SnapType::MidPoint, entity.id.clone()))
            }
            _ => None,
        }
    }

    fn calculate_center(&self, entity: &super::super::data_structure::Entity) -> Option<Snapshot> {
        match &entity.geometry {
            super::super::data_structure::EntityGeometry::Circle(circle) => {
                Some(Snapshot::new(circle.center.clone(), SnapType::Center, entity.id.clone()))
            }
            super::super::data_structure::EntityGeometry::Arc(arc) => {
                Some(Snapshot::new(arc.center.clone(), SnapType::Center, entity.id.clone()))
            }
            _ => None,
        }
    }

    fn calculate_node(&self, entity: &super::super::data_structure::Entity) -> Option<Snapshot> {
        match &entity.geometry {
            super::super::data_structure::EntityGeometry::Point(_) => {
                Some(Snapshot::new(entity.get_position()?, SnapType::Node, entity.id.clone()))
            }
            _ => None,
        }
    }

    fn calculate_quadrant(&self, entity: &super::super::data_structure::Entity) -> Vec<Snapshot> {
        match &entity.geometry {
            super::super::data_structure::EntityGeometry::Circle(circle) => {
                let c = &circle.center;
                vec![
                    Snapshot::new(crate::geometry::Point::new(c.x + circle.radius, c.y, c.z), SnapType::Quadrant, entity.id.clone()),
                    Snapshot::new(crate::geometry::Point::new(c.x - circle.radius, c.y, c.z), SnapType::Quadrant, entity.id.clone()),
                    Snapshot::new(crate::geometry::Point::new(c.x, c.y + circle.radius, c.z), SnapType::Quadrant, entity.id.clone()),
                    Snapshot::new(crate::geometry::Point::new(c.x, c.y - circle.radius, c.z), SnapType::Quadrant, entity.id.clone()),
                ]
            }
            _ => Vec::new(),
        }
    }

    fn calculate_intersection(&self, entity1: &super::super::data_structure::Entity, entity2: &super::super::data_structure::Entity) -> Option<Snapshot> {
        use super::super::data_structure::EntityGeometry;
        use super::super::geometry::intersection::{
            intersect_line_line, intersect_line_circle, intersect_circle_circle, IntersectionResult,
        };

        let result = match (&entity1.geometry, &entity2.geometry) {
            (EntityGeometry::Line(l1), EntityGeometry::Line(l2)) => intersect_line_line(l1.clone(), l2.clone()),
            (EntityGeometry::Line(l), EntityGeometry::Circle(c)) => intersect_line_circle(l.clone(), c.clone()),
            (EntityGeometry::Circle(c), EntityGeometry::Line(l)) => intersect_line_circle(l.clone(), c.clone()),
            (EntityGeometry::Circle(c1), EntityGeometry::Circle(c2)) => intersect_circle_circle(c1.clone(), c2.clone()),
            _ => IntersectionResult::None,
        };

        match result {
            IntersectionResult::Point(ip) => {
                Some(Snapshot::new(ip.point, SnapType::Intersection, entity1.id.clone()))
            }
            IntersectionResult::Points(points) => points.first().map(|ip| {
                Snapshot::new(ip.point.clone(), SnapType::Intersection, entity1.id.clone())
            }),
            _ => None,
        }
    }

    fn calculate_perpendicular(&self, entity: &super::super::data_structure::Entity, from_point: crate::geometry::Point) -> Option<Snapshot> {
        match &entity.geometry {
            super::super::data_structure::EntityGeometry::Line(line) => {
                let v = line.end.to_vector2() - line.start.to_vector2();
                let w = from_point.to_vector2() - line.start.to_vector2();
                let projection_length = (w.x * v.x + w.y * v.y) / (v.x * v.x + v.y * v.y);
                let projection = line.start.to_vector2() + v * projection_length.clamp(0.0, 1.0);
                Some(Snapshot::new(crate::geometry::Point::new(projection.x, projection.y, 0.0), SnapType::Perpendicular, entity.id.clone()))
            }
            _ => None,
        }
    }

    fn calculate_tangent(&self, entity: &super::super::data_structure::Entity, from_point: crate::geometry::Point) -> Option<Snapshot> {
        match &entity.geometry {
            super::super::data_structure::EntityGeometry::Circle(circle) => {
                let dx = from_point.x - circle.center.x;
                let dy = from_point.y - circle.center.y;
                let distance = (dx * dx + dy * dy).sqrt();
                if distance > circle.radius {
                    let angle = (dy / distance).atan2(dx / distance);
                    let alpha = (circle.radius / distance).asin();
                    let tangent_angle = angle + alpha;

                    let tangent_point = crate::geometry::Point::new(
                        circle.center.x + circle.radius * tangent_angle.cos(),
                        circle.center.y + circle.radius * tangent_angle.sin(),
                        0.0,
                    );

                    Some(Snapshot::new(tangent_point, SnapType::Tangent, entity.id.clone()))
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    fn calculate_nearest(&self, entity: &super::super::data_structure::Entity, to_point: crate::geometry::Point) -> Option<Snapshot> {
        let position = entity.get_position()?;
        let distance = position.distance_to(&to_point);
        Some(Snapshot::new(position, SnapType::Nearest, entity.id.clone()))
    }

    fn calculate_extension(&self, entity: &super::super::data_structure::Entity, from_point: crate::geometry::Point, extension_distance: f64) -> Option<Snapshot> {
        match &entity.geometry {
            super::super::data_structure::EntityGeometry::Line(line) => {
                let direction = (line.end.to_vector2() - line.start.to_vector2()).normalize();
                let extension_point = line.end.to_vector2() + direction * extension_distance;
                Some(Snapshot::new(crate::geometry::Point::new(extension_point.x, extension_point.y, 0.0), SnapType::Extension, entity.id.clone()))
            }
            _ => None,
        }
    }
}

pub struct SnapManager {
    snap_types: Vec<SnapType>,
    snap_radius: f64,
    aperture_size: f64,
    is_enabled: bool,
    snap_to_grid: bool,
    ortho_mode: bool,
    polar_tracking: bool,
    polar_angle: f64,
    snap_increment: f64,
    last_snap_point: Option<Snapshot>,
}

impl Default for SnapManager {
    fn default() -> Self {
        Self {
            snap_types: vec![
                SnapType::EndPoint,
                SnapType::MidPoint,
                SnapType::Center,
                SnapType::Intersection,
            ],
            snap_radius: 10.0,
            aperture_size: 5.0,
            is_enabled: true,
            snap_to_grid: true,
            ortho_mode: false,
            polar_tracking: false,
            polar_angle: 90.0,
            snap_increment: 1.0,
            last_snap_point: None,
        }
    }
}

impl SnapManager {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn enable(&mut self, enabled: bool) {
        self.is_enabled = enabled;
    }

    pub fn is_enabled(&self) -> bool {
        self.is_enabled
    }

    pub fn set_snap_radius(&mut self, radius: f64) {
        self.snap_radius = radius;
    }

    pub fn get_snap_radius(&self) -> f64 {
        self.snap_radius
    }

    pub fn set_aperture_size(&mut self, size: f64) {
        self.aperture_size = size;
    }

    pub fn get_aperture_size(&self) -> f64 {
        self.aperture_size
    }

    pub fn enable_snap_type(&mut self, snap_type: SnapType, enabled: bool) {
        if enabled {
            if !self.snap_types.contains(&snap_type) {
                self.snap_types.push(snap_type);
            }
        } else {
            self.snap_types.retain(|&t| t != snap_type);
        }
    }

    pub fn get_enabled_snap_types(&self) -> &[SnapType] {
        &self.snap_types
    }

    pub fn enable_grid_snapping(&mut self, enabled: bool) {
        self.snap_to_grid = enabled;
    }

    pub fn is_grid_snapping_enabled(&self) -> bool {
        self.snap_to_grid
    }

    pub fn enable_ortho(&mut self, enabled: bool) {
        self.ortho_mode = enabled;
    }

    pub fn is_ortho_enabled(&self) -> bool {
        self.ortho_mode
    }

    pub fn enable_polar_tracking(&mut self, enabled: bool) {
        self.polar_tracking = enabled;
    }

    pub fn is_polar_tracking_enabled(&self) -> bool {
        self.polar_tracking
    }

    pub fn set_polar_angle(&mut self, angle: f64) {
        self.polar_angle = angle;
    }

    pub fn get_polar_angle(&self) -> f64 {
        self.polar_angle
    }

    pub fn set_snap_increment(&mut self, increment: f64) {
        self.snap_increment = increment.max(0.0);
    }

    pub fn get_snap_increment(&self) -> f64 {
        self.snap_increment
    }

    pub fn snap_to_grid_point(&self, point: crate::geometry::Point) -> crate::geometry::Point {
        if self.snap_increment > 0.0 {
            crate::geometry::Point::new(
                (point.x / self.snap_increment).round() * self.snap_increment,
                (point.y / self.snap_increment).round() * self.snap_increment,
                (point.z / self.snap_increment).round() * self.snap_increment,
            )
        } else {
            point
        }
    }

    pub fn snap_point(
        &mut self,
        cursor_point: crate::geometry::Point,
        entities: &[super::super::data_structure::Entity],
        reference_point: Option<crate::geometry::Point>,
    ) -> Option<Snapshot> {
        if !self.is_enabled {
            return None;
        }

        let mut best_snap: Option<Snapshot> = None;
        let mut best_distance = self.snap_radius;

        if self.snap_to_grid {
            let grid_point = self.snap_to_grid_point(cursor_point);
            let grid_distance = grid_point.distance_to(&cursor_point);
            if grid_distance < best_distance {
                best_snap = Some(Snapshot::new(grid_point, SnapType::Nearest, super::super::data_structure::ObjectId::null()));
                best_distance = grid_distance;
            }
        }

        let calculator = SnapCalculatorImpl;

        for entity in entities {
            for snap_type in &self.snap_types {
                let snap_result = match snap_type {
                    SnapType::EndPoint => calculator.calculate_endpoint(entity),
                    SnapType::MidPoint => calculator.calculate_midpoint(entity),
                    SnapType::Center => calculator.calculate_center(entity),
                    SnapType::Node => calculator.calculate_node(entity),
                    SnapType::Quadrant => calculator.calculate_quadrant(entity).first().cloned(),
                    SnapType::Perpendicular => {
                        if let Some(ref_point) = reference_point {
                            calculator.calculate_perpendicular(entity, ref_point)
                        } else {
                            None
                        }
                    }
                    SnapType::Tangent => {
                        if let Some(ref_point) = reference_point {
                            calculator.calculate_tangent(entity, ref_point)
                        } else {
                            None
                        }
                    }
                    SnapType::Nearest => calculator.calculate_nearest(entity, cursor_point),
                    _ => None,
                };

                if let Some(snap) = snap_result {
                    let distance = snap.point.distance_to(&cursor_point);
                    if distance < best_distance {
                        best_snap = Some(snap);
                        best_distance = distance;
                    }
                }
            }
        }

        if let Some(snap) = &best_snap {
            self.last_snap_point = Some(snap.clone());
        }

        best_snap
    }

    pub fn apply_ortho(&self, from_point: crate::geometry::Point, to_point: crate::geometry::Point) -> crate::geometry::Point {
        if !self.ortho_mode {
            return to_point;
        }

        let dx = to_point.x - from_point.x;
        let dy = to_point.y - from_point.y;

        if dx.abs() >= dy.abs() {
            crate::geometry::Point::new(to_point.x, from_point.y, to_point.z)
        } else {
            crate::geometry::Point::new(from_point.x, to_point.y, to_point.z)
        }
    }

    pub fn apply_polar_tracking(&self, from_point: crate::geometry::Point, to_point: crate::geometry::Point) -> crate::geometry::Point {
        if !self.polar_tracking {
            return to_point;
        }

        let direction = (to_point.to_vector2() - from_point.to_vector2()).normalize();
        let angle = direction.angle();

        let angle_step = (self.polar_angle * std::f64::consts::PI / 180.0).to_radians();
        let snapped_angle = (angle / angle_step).round() * angle_step;

        let distance = to_point.distance_to(&from_point);

        crate::geometry::Point::new(
            from_point.x + distance * snapped_angle.cos(),
            from_point.y + distance * snapped_angle.sin(),
            to_point.z,
        )
    }

    pub fn get_last_snap_point(&self) -> Option<&Snapshot> {
        self.last_snap_point.as_ref()
    }

    pub fn clear_last_snap_point(&mut self) {
        self.last_snap_point = None;
    }
}

impl fmt::Display for SnapPoint {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "SnapPoint(point={}, type={:?}, distance={})",
            self.point, self.snap_type, self.distance
        )
    }
}

impl fmt::Display for SnapManager {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "SnapManager(enabled={}, snap_types={}, radius={})",
            self.is_enabled,
            self.snap_types.len(),
            self.snap_radius
        )
    }
}
