use serde::{Serialize, Deserialize};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum GripType {
    Endpoint,
    Midpoint,
    Center,
    Node,
    Quadrant,
    Perpendicular,
    Tangent,
    Intersection,
    Extension,
    Parallel,
    None,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum GripMode {
    None,
    Move,
    Stretch,
    Rotate,
    Scale,
    Mirror,
    Array,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GripPoint {
    pub entity_id: super::super::data_structure::ObjectId,
    pub position: crate::geometry::Point,
    pub grip_type: GripType,
    pub is_hovered: bool,
    pub is_selected: bool,
    pub size: f64,
    pub color: (u8, u8, u8),
    pub hovered_color: (u8, u8, u8),
    pub selected_color: (u8, u8, u8),
}

impl Default for GripPoint {
    fn default() -> Self {
        Self {
            entity_id: super::super::data_structure::ObjectId::new(),
            position: crate::geometry::Point::new(0.0, 0.0, 0.0),
            grip_type: GripType::None,
            is_hovered: false,
            is_selected: false,
            size: 5.0,
            color: (0, 0, 255),
            hovered_color: (255, 255, 0),
            selected_color: (0, 255, 0),
        }
    }
}

impl GripPoint {
    pub fn new(entity_id: super::super::data_structure::ObjectId, position: crate::geometry::Point, grip_type: GripType) -> Self {
        Self {
            entity_id,
            position,
            grip_type,
            ..Default::default()
        }
    }

    pub fn set_selected(&mut self, selected: bool) {
        self.is_selected = selected;
    }

    pub fn set_hovered(&mut self, hovered: bool) {
        self.is_hovered = hovered;
    }

    pub fn get_display_color(&self) -> (u8, u8, u8) {
        if self.is_selected {
            self.selected_color
        } else if self.is_hovered {
            self.hovered_color
        } else {
            self.color
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GripHotSpot {
    pub grip_points: Vec<GripPoint>,
    pub active_grip: Option<usize>,
    pub base_point: Option<crate::geometry::Point>,
    pub drag_point: Option<crate::geometry::Point>,
    pub preview_entities: Vec<super::super::data_structure::Entity>,
}

impl Default for GripHotSpot {
    fn default() -> Self {
        Self {
            grip_points: Vec::new(),
            active_grip: None,
            base_point: None,
            drag_point: None,
            preview_entities: Vec::new(),
        }
    }
}

impl GripHotSpot {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_grip(&mut self, grip: GripPoint) {
        self.grip_points.push(grip);
    }

    pub fn clear(&mut self) {
        self.grip_points.clear();
        self.active_grip = None;
        self.base_point = None;
        self.drag_point = None;
        self.preview_entities.clear();
    }

    pub fn find_grip_at(&self, point: crate::geometry::Point, threshold: f64) -> Option<usize> {
        for (index, grip) in self.grip_points.iter().enumerate() {
            let distance = grip.position.distance_to(&point);
            if distance <= threshold {
                return Some(index);
            }
        }
        None
    }

    pub fn select_grip(&mut self, index: usize) {
        self.active_grip = Some(index);
        if index < self.grip_points.len() {
            self.base_point = Some(self.grip_points[index].position);
        }
    }

    pub fn update_drag_point(&mut self, point: crate::geometry::Point) {
        self.drag_point = Some(point);
    }

    pub fn get_displacement(&self) -> Option<(f64, f64)> {
        match (self.base_point, self.drag_point) {
            (Some(base), Some(drag)) => Some((drag.x - base.x, drag.y - base.y)),
            _ => None,
        }
    }
}

pub trait GripEditHandler {
    fn get_grips(&self, entity_id: super::super::data_structure::ObjectId, entity: &super::super::data_structure::Entity) -> Vec<GripPoint>;
    fn handle_grip_drag(&self, entity: &mut super::super::data_structure::Entity, grip_index: usize, new_position: crate::geometry::Point) -> bool;
    fn get_preview(&self, entity: &super::super::data_structure::Entity, grip_index: usize, new_position: crate::geometry::Point) -> Option<super::super::data_structure::Entity>;
}

pub struct GripManager {
    grip_hotspots: std::collections::HashMap<super::super::data_structure::ObjectId, GripHotSpot>,
    active_entity: Option<super::super::data_structure::ObjectId>,
    current_mode: GripMode,
    grip_size: f64,
    grip_size_screen: f64,
    enable_grips: bool,
    grip_colors: GripColors,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct GripColors {
    pub unselected: (u8, u8, u8),
    pub hovered: (u8, u8, u8),
    pub selected: (u8, u8, u8),
    pub aperture: (u8, u8, u8),
}

impl Default for GripColors {
    fn default() -> Self {
        Self {
            unselected: (0, 0, 255),
            hovered: (255, 255, 0),
            selected: (0, 255, 0),
            aperture: (128, 128, 128),
        }
    }
}

impl Default for GripManager {
    fn default() -> Self {
        Self {
            grip_hotspots: std::collections::HashMap::new(),
            active_entity: None,
            current_mode: GripMode::None,
            grip_size: 5.0,
            grip_size_screen: 10.0,
            enable_grips: true,
            grip_colors: GripColors::default(),
        }
    }
}

impl GripManager {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn generate_grips_for_entity(
        &mut self,
        entity_id: super::super::data_structure::ObjectId,
        entity: &super::super::data_structure::Entity,
    ) -> Vec<GripPoint> {
        let mut grips = Vec::new();

        match &entity.entity_type {
            super::super::data_structure::EntityType::Line => {
                if let Some(line) = self.extract_line(entity) {
                    grips.push(GripPoint::new(entity_id.clone(), line.start, GripType::Endpoint));
                    grips.push(GripPoint::new(entity_id.clone(), line.end, GripType::Endpoint));
                    let mid = line.start.midpoint(&line.end);
                    grips.push(GripPoint::new(entity_id.clone(), mid, GripType::Midpoint));
                }
            }
            super::super::data_structure::EntityType::Circle => {
                if let Some(circle) = self.extract_circle(entity) {
                    let c = circle.center;
                    grips.push(GripPoint::new(entity_id.clone(), c, GripType::Center));
                    grips.push(GripPoint::new(entity_id.clone(), crate::geometry::Point::new(c.x + circle.radius, c.y, c.z), GripType::Quadrant));
                    grips.push(GripPoint::new(entity_id.clone(), crate::geometry::Point::new(c.x, c.y + circle.radius, c.z), GripType::Quadrant));
                }
            }
            super::super::data_structure::EntityType::Arc => {
                if let Some(arc) = self.extract_arc(entity) {
                    let c = arc.center;
                    grips.push(GripPoint::new(entity_id.clone(), c, GripType::Center));
                    let start = crate::geometry::Point::new(c.x + arc.radius * arc.start_angle.cos(), c.y + arc.radius * arc.start_angle.sin(), c.z);
                    let end = crate::geometry::Point::new(c.x + arc.radius * arc.end_angle.cos(), c.y + arc.radius * arc.end_angle.sin(), c.z);
                    grips.push(GripPoint::new(entity_id.clone(), start, GripType::Endpoint));
                    grips.push(GripPoint::new(entity_id.clone(), end, GripType::Endpoint));
                }
            }
            _ => {}
        }

        if let Some(hotspot) = self.grip_hotspots.get_mut(&entity_id) {
            hotspot.grip_points = grips.clone();
        }

        grips
    }

    pub fn get_grips(&self, entity_id: super::super::data_structure::ObjectId) -> Option<&Vec<GripPoint>> {
        self.grip_hotspots.get(&entity_id).map(|h| &h.grip_points)
    }

    pub fn get_grips_mut(&mut self, entity_id: super::super::data_structure::ObjectId) -> Option<&mut Vec<GripPoint>> {
        self.grip_hotspots.get_mut(&entity_id).map(|h| &mut h.grip_points)
    }

    pub fn find_grip_at(&self, entity_id: super::super::data_structure::ObjectId, point: crate::geometry::Point, threshold: f64) -> Option<usize> {
        if let Some(hotspot) = self.grip_hotspots.get(&entity_id) {
            hotspot.find_grip_at(point, threshold)
        } else {
            None
        }
    }

    pub fn select_grip(&mut self, entity_id: super::super::data_structure::ObjectId, grip_index: usize) -> bool {
        if let Some(hotspot) = self.grip_hotspots.get_mut(&entity_id) {
            hotspot.select_grip(grip_index);
            self.active_entity = Some(entity_id);
            true
        } else {
            false
        }
    }

    pub fn clear_grips(&mut self) {
        self.grip_hotspots.clear();
        self.active_entity = None;
    }

    pub fn clear_grips_for_entity(&mut self, entity_id: super::super::data_structure::ObjectId) {
        self.grip_hotspots.remove(&entity_id);
        if self.active_entity == Some(entity_id) {
            self.active_entity = None;
        }
    }

    pub fn set_grip_size(&mut self, size: f64) {
        self.grip_size = size;
    }

    pub fn get_grip_size(&self) -> f64 {
        self.grip_size
    }

    pub fn enable(&mut self, enable: bool) {
        self.enable_grips = enable;
    }

    pub fn is_enabled(&self) -> bool {
        self.enable_grips
    }

    pub fn get_active_entity(&self) -> Option<&super::super::data_structure::ObjectId> {
        self.active_entity.as_ref()
    }

    pub fn set_mode(&mut self, mode: GripMode) {
        self.current_mode = mode;
    }

    pub fn get_mode(&self) -> GripMode {
        self.current_mode
    }

    fn extract_line(&self, entity: &super::super::data_structure::Entity) -> Option<super::super::geometry::Line> {
        if let super::super::data_structure::EntityGeometry::Line(line) = &entity.geometry {
            Some(line.clone())
        } else {
            None
        }
    }

    fn extract_circle(&self, entity: &super::super::data_structure::Entity) -> Option<super::super::geometry::Circle> {
        if let super::super::data_structure::EntityGeometry::Circle(circle) = &entity.geometry {
            Some(circle.clone())
        } else {
            None
        }
    }

    fn extract_arc(&self, entity: &super::super::data_structure::Entity) -> Option<super::super::geometry::Arc> {
        if let super::super::data_structure::EntityGeometry::Arc(arc) = &entity.geometry {
            Some(arc.clone())
        } else {
            None
        }
    }
}

impl fmt::Display for GripPoint {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Grip(type={:?}, position={})",
            self.grip_type, self.position
        )
    }
}

impl fmt::Display for GripManager {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "GripManager(entities={}, mode={:?})",
            self.grip_hotspots.len(),
            self.current_mode
        )
    }
}
