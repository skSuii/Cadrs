use crate::data_structure::{Entity, ObjectId, EntityType, EntityGeometry};
use crate::geometry::{Point, Vector2, Line, Circle, Arc, Ellipse, Polyline, BSpline};
use serde::{Serialize, Deserialize};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Grip {
    pub id: GripId,
    pub position: Point,
    pub grip_type: GripType,
    pub is_dragging: bool,
    pub is_highlighted: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct GripId(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum GripType {
    Endpoint,
    Midpoint,
    Center,
    Quadrant,
    Intersection,
    Vertex,
    ControlPoint,
    Move,
    Stretch,
    Rotate,
    Scale,
    Mirror,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EntityGrips {
    pub entity_id: ObjectId,
    pub grips: Vec<Grip>,
    pub is_selected: bool,
}

impl EntityGrips {
    pub fn new(entity_id: ObjectId, entity: &Entity) -> Self {
        let grips = Self::extract_grips(entity);
        Self {
            entity_id,
            grips,
            is_selected: false,
        }
    }
    
    fn extract_grips(entity: &Entity) -> Vec<Grip> {
        let mut grips = Vec::new();
        let mut grip_id = 0;
        
        match &entity.geometry {
            EntityGeometry::Line(line) => {
                grips.push(Grip::new(
                    GripId(grip_id),
                    line.start,
                    GripType::Endpoint,
                ));
                grip_id += 1;
                
                grips.push(Grip::new(
                    GripId(grip_id),
                    line.end,
                    GripType::Endpoint,
                ));
                grip_id += 1;
                
                let midpoint = Point::new(
                    (line.start.x + line.end.x) / 2.0,
                    (line.start.y + line.end.y) / 2.0,
                    0.0,
                );
                grips.push(Grip::new(
                    GripId(grip_id),
                    midpoint,
                    GripType::Midpoint,
                ));
            }
            
            EntityGeometry::Circle(circle) => {
                grips.push(Grip::new(
                    GripId(grip_id),
                    circle.center,
                    GripType::Center,
                ));
                grip_id += 1;
                
                for i in 0..4 {
                    let angle = i as f64 * std::f64::consts::PI / 2.0;
                    let quadrant_point = Point::new(
                        circle.center.x + circle.radius * angle.cos(),
                        circle.center.y + circle.radius * angle.sin(),
                        0.0,
                    );
                    grips.push(Grip::new(
                        GripId(grip_id),
                        quadrant_point,
                        GripType::Quadrant,
                    ));
                    grip_id += 1;
                }
            }
            
            EntityGeometry::Arc(arc) => {
                grips.push(Grip::new(
                    GripId(grip_id),
                    arc.center,
                    GripType::Center,
                ));
                grip_id += 1;
                
                let start_point = Point::new(
                    arc.center.x + arc.radius * arc.start_angle.cos(),
                    arc.center.y + arc.radius * arc.start_angle.sin(),
                    0.0,
                );
                grips.push(Grip::new(
                    GripId(grip_id),
                    start_point,
                    GripType::Endpoint,
                ));
                grip_id += 1;
                
                let end_point = Point::new(
                    arc.center.x + arc.radius * arc.end_angle.cos(),
                    arc.center.y + arc.radius * arc.end_angle.sin(),
                    0.0,
                );
                grips.push(Grip::new(
                    GripId(grip_id),
                    end_point,
                    GripType::Endpoint,
                ));
                grip_id += 1;
                
                let midpoint_angle = (arc.start_angle + arc.end_angle) / 2.0;
                let midpoint = Point::new(
                    arc.center.x + arc.radius * midpoint_angle.cos(),
                    arc.center.y + arc.radius * midpoint_angle.sin(),
                    0.0,
                );
                grips.push(Grip::new(
                    GripId(grip_id),
                    midpoint,
                    GripType::Midpoint,
                ));
            }
            
            EntityGeometry::Ellipse(ellipse) => {
                grips.push(Grip::new(
                    GripId(grip_id),
                    ellipse.center,
                    GripType::Center,
                ));
                grip_id += 1;
                
                let major_end = Point::new(
                    ellipse.center.x + ellipse.semi_major * ellipse.rotation.cos(),
                    ellipse.center.y + ellipse.semi_major * ellipse.rotation.sin(),
                    0.0,
                );
                grips.push(Grip::new(
                    GripId(grip_id),
                    major_end,
                    GripType::Vertex,
                ));
                grip_id += 1;
                
                let minor_end = Point::new(
                    ellipse.center.x + ellipse.semi_minor * (ellipse.rotation + std::f64::consts::PI / 2.0).cos(),
                    ellipse.center.y + ellipse.semi_minor * (ellipse.rotation + std::f64::consts::PI / 2.0).sin(),
                    0.0,
                );
                grips.push(Grip::new(
                    GripId(grip_id),
                    minor_end,
                    GripType::Vertex,
                ));
            }
            
            EntityGeometry::Polyline(polyline) => {
                for (i, vertex) in polyline.vertices.iter().enumerate() {
                    grips.push(Grip::new(
                        GripId(grip_id),
                        Point::new(vertex.x, vertex.y, 0.0),
                        GripType::Vertex,
                    ));
                    grip_id += 1;

                    if i < polyline.vertices.len().saturating_sub(1) {
                        let next = &polyline.vertices[i + 1];
                        let midpoint = Point::new(
                            (vertex.x + next.x) / 2.0,
                            (vertex.y + next.y) / 2.0,
                            0.0,
                        );
                        grips.push(Grip::new(
                            GripId(grip_id),
                            midpoint,
                            GripType::Midpoint,
                        ));
                        grip_id += 1;
                    }
                }
            }

            EntityGeometry::BSpline(spline) => {
                for (i, point) in spline.control_points().iter().enumerate() {
                    grips.push(Grip::new(
                        GripId(grip_id),
                        *point,
                        GripType::ControlPoint,
                    ));
                    grip_id += 1;
                }
            }

            EntityGeometry::Dimension { definition_point, def_point_1, def_point_2, text_position, .. } => {
                grips.push(Grip::new(
                    GripId(grip_id),
                    *definition_point,
                    GripType::Endpoint,
                ));
                grip_id += 1;

                grips.push(Grip::new(
                    GripId(grip_id),
                    *def_point_1,
                    GripType::Endpoint,
                ));
                grip_id += 1;

                grips.push(Grip::new(
                    GripId(grip_id),
                    *def_point_2,
                    GripType::Endpoint,
                ));
                grip_id += 1;

                grips.push(Grip::new(
                    GripId(grip_id),
                    *text_position,
                    GripType::Midpoint,
                ));
            }
            
            _ => {}
        }
        
        grips
    }
    
    pub fn get_grip_by_id(&self, id: GripId) -> Option<&Grip> {
        self.grips.iter().find(|g| g.id == id)
    }
    
    pub fn get_grip_by_id_mut(&mut self, id: GripId) -> Option<&mut Grip> {
        self.grips.iter_mut().find(|g| g.id == id)
    }
    
    pub fn highlight_grip(&mut self, id: GripId) {
        if let Some(grip) = self.get_grip_by_id_mut(id) {
            grip.is_highlighted = true;
        }
    }
    
    pub fn unhighlight_all(&mut self) {
        for grip in &mut self.grips {
            grip.is_highlighted = false;
        }
    }
    
    pub fn contains_point(&self, point: Point, tolerance: f64) -> Option<GripId> {
        for grip in &self.grips {
            if grip.position.distance_to(&point) < tolerance {
                return Some(grip.id);
            }
        }
        None
    }
}

impl Grip {
    pub fn new(id: GripId, position: Point, grip_type: GripType) -> Self {
        Self {
            id,
            position,
            grip_type,
            is_dragging: false,
            is_highlighted: false,
        }
    }
    
    pub fn start_drag(&mut self) {
        self.is_dragging = true;
    }
    
    pub fn end_drag(&mut self) {
        self.is_dragging = false;
    }
    
    pub fn move_to(&mut self, new_position: Point) {
        self.position = new_position;
    }
    
    pub fn move_by(&mut self, delta: Vector2) {
        self.position = Point::new(
            self.position.x + delta.x,
            self.position.y + delta.y,
            0.0,
        );
    }
}

pub struct GripEditor {
    entity_grips: HashMap<ObjectId, EntityGrips>,
    dragged_grip: Option<(ObjectId, GripId)>,
    original_positions: HashMap<(ObjectId, GripId), Point>,
}

impl GripEditor {
    pub fn new() -> Self {
        Self {
            entity_grips: HashMap::new(),
            dragged_grip: None,
            original_positions: HashMap::new(),
        }
    }
    
    pub fn set_entity_grips(&mut self, entity_id: ObjectId, entity: &Entity) {
        let entity_grips = EntityGrips::new(entity_id.clone(), entity);
        self.entity_grips.insert(entity_id, entity_grips);
    }
    
    pub fn clear_all_grips(&mut self) {
        self.entity_grips.clear();
        self.dragged_grip = None;
        self.original_positions.clear();
    }
    
    pub fn remove_entity_grips(&mut self, entity_id: &ObjectId) {
        self.entity_grips.remove(entity_id);
    }
    
    pub fn get_all_grips(&self) -> Vec<&Grip> {
        self.entity_grips.values()
            .flat_map(|eg| eg.grips.iter())
            .collect()
    }
    
    pub fn find_grip_at_point(&self, point: Point, tolerance: f64) -> Option<(ObjectId, GripId)> {
        for (entity_id, entity_grips) in &self.entity_grips {
            if let Some(grip_id) = entity_grips.contains_point(point, tolerance) {
                return Some((entity_id.clone(), grip_id));
            }
        }
        None
    }
    
    pub fn start_drag(&mut self, entity_id: ObjectId, grip_id: GripId) -> Option<Point> {
        self.dragged_grip = Some((entity_id.clone(), grip_id));
        
        if let Some(entity_grips) = self.entity_grips.get_mut(&entity_id) {
            if let Some(grip) = entity_grips.get_grip_by_id_mut(grip_id) {
                grip.start_drag();
                self.original_positions.insert((entity_id, grip_id), grip.position);
                return Some(grip.position);
            }
        }
        None
    }
    
    pub fn drag_to(&mut self, new_position: Point) -> Vec<(ObjectId, GripId, Point)> {
        let mut moved_grips = Vec::new();
        
        if let Some((entity_id, grip_id)) = self.dragged_grip.clone() {
            if let Some(entity_grips) = self.entity_grips.get_mut(&entity_id) {
                if let Some(grip) = entity_grips.get_grip_by_id_mut(grip_id) {
                    let old_position = grip.position;
                    grip.move_to(new_position);
                    moved_grips.push((entity_id, grip_id, old_position));
                }
            }
        }
        
        moved_grips
    }
    
    pub fn end_drag(&mut self) -> Vec<((ObjectId, GripId), Point)> {
        let mut changes = Vec::new();
        
        if let Some((entity_id, grip_id)) = self.dragged_grip.clone() {
            if let Some(entity_grips) = self.entity_grips.get_mut(&entity_id) {
                if let Some(grip) = entity_grips.get_grip_by_id_mut(grip_id) {
                    grip.end_drag();
                    if let Some(original) = self.original_positions.remove(&(entity_id.clone(), grip_id.clone())) {
                        changes.push(((entity_id, grip_id), original));
                    }
                }
            }
        }
        
        self.dragged_grip = None;
        changes
    }
    
    pub fn cancel_drag(&mut self) {
        if let Some((entity_id, grip_id)) = self.dragged_grip.clone() {
            if let Some(entity_grips) = self.entity_grips.get_mut(&entity_id) {
                if let Some(grip) = entity_grips.get_grip_by_id_mut(grip_id) {
                    if let Some(original) = self.original_positions.get(&(entity_id, grip_id)) {
                        grip.move_to(*original);
                    }
                    grip.end_drag();
                }
            }
        }
        
        self.dragged_grip = None;
        self.original_positions.clear();
    }
    
    pub fn update_entity_from_grips(&self, entity_id: ObjectId, entity: &mut Entity) {
        if let Some(entity_grips) = self.entity_grips.get(&entity_id) {
            self.apply_grip_modifications(entity_grips, entity);
        }
    }
    
    fn apply_grip_modifications(&self, entity_grips: &EntityGrips, entity: &mut Entity) {
        match &mut entity.geometry {
            EntityGeometry::Line(line) => {
                for grip in &entity_grips.grips {
                    match grip.grip_type {
                        GripType::Endpoint => {
                            if grip.position.distance_to(&line.start) < 1e-6 {
                                line.start = grip.position;
                            } else if grip.position.distance_to(&line.end) < 1e-6 {
                                line.end = grip.position;
                            }
                        }
                        GripType::Midpoint => {
                            let dir = line.end - line.start;
                            line.end = Point::new(
                                grip.position.x + dir.x / 2.0,
                                grip.position.y + dir.y / 2.0,
                                0.0,
                            );
                            line.start = Point::new(
                                grip.position.x - dir.x / 2.0,
                                grip.position.y - dir.y / 2.0,
                                0.0,
                            );
                        }
                        _ => {}
                    }
                }
            }
            
            EntityGeometry::Circle(circle) => {
                for grip in &entity_grips.grips {
                    match grip.grip_type {
                        GripType::Center => {
                            let delta = grip.position - circle.center;
                            circle.center = grip.position;
                        }
                        GripType::Quadrant => {
                            let new_radius = circle.center.distance_to(&grip.position);
                            circle.radius = new_radius;
                        }
                        _ => {}
                    }
                }
            }
            
            EntityGeometry::Arc(arc) => {
                for grip in &entity_grips.grips {
                    match grip.grip_type {
                        GripType::Center => {
                            arc.center = grip.position;
                        }
                        GripType::Endpoint => {
                            let new_radius = arc.center.distance_to(&grip.position);
                            arc.radius = new_radius;
                            
                            let v = (grip.position - arc.center).to_vector2();
                            let new_angle = v.angle();
                            
                            if grip.position.distance_to(&Point::new(
                                arc.center.x + arc.radius * arc.start_angle.cos(),
                                arc.center.y + arc.radius * arc.start_angle.sin(),
                                0.0,
                            )) < 1e-6 {
                                arc.start_angle = new_angle;
                            } else {
                                arc.end_angle = new_angle;
                            }
                        }
                        GripType::Midpoint => {
                            let new_radius = arc.center.distance_to(&grip.position);
                            arc.radius = new_radius;
                        }
                        _ => {}
                    }
                }
            }
            
            EntityGeometry::Polyline(polyline) => {
                for grip in &entity_grips.grips {
                    if let GripType::Vertex = grip.grip_type {
                        for vertex in &mut polyline.vertices {
                            if (vertex.x - grip.position.x).abs() < 1e-6 &&
                               (vertex.y - grip.position.y).abs() < 1e-6 {
                                vertex.x = grip.position.x;
                                vertex.y = grip.position.y;
                                break;
                            }
                        }
                    }
                }
            }

            EntityGeometry::BSpline(spline) => {
                for grip in &entity_grips.grips {
                    if let GripType::ControlPoint = grip.grip_type {
                        for point in spline.control_points_mut() {
                            if point.distance_to(&grip.position) < 1e-6 {
                                *point = grip.position;
                                break;
                            }
                        }
                    }
                }
            }
            
            _ => {}
        }
    }
    
    pub fn highlight_grip(&mut self, entity_id: ObjectId, grip_id: GripId) {
        if let Some(entity_grips) = self.entity_grips.get_mut(&entity_id) {
            entity_grips.unhighlight_all();
            entity_grips.highlight_grip(grip_id);
        }
    }
    
    pub fn unhighlight_all(&mut self) {
        for entity_grips in self.entity_grips.values_mut() {
            entity_grips.unhighlight_all();
        }
    }
    
    pub fn get_entity_count(&self) -> usize {
        self.entity_grips.len()
    }
    
    pub fn get_grip_count(&self) -> usize {
        self.entity_grips.values().map(|eg| eg.grips.len()).sum()
    }
}

impl Default for GripEditor {
    fn default() -> Self {
        Self::new()
    }
}

pub struct GripMode {
    mode: GripEditMode,
    options: GripOptions,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GripEditMode {
    Stretch,
    Move,
    Rotate,
    Scale,
    Mirror,
}

#[derive(Debug, Clone)]
pub struct GripOptions {
    pub base_point: Point,
    pub reference_angle: Option<f64>,
    pub scale_factor: Option<f64>,
    pub mirror_line: Option<(Point, Point)>,
}

impl GripMode {
    pub fn new() -> Self {
        Self {
            mode: GripEditMode::Stretch,
            options: GripOptions {
                base_point: Point::origin(),
                reference_angle: None,
                scale_factor: None,
                mirror_line: None,
            },
        }
    }
    
    pub fn set_mode(&mut self, mode: GripEditMode) {
        self.mode = mode;
    }
    
    pub fn get_mode(&self) -> GripEditMode {
        self.mode
    }
    
    pub fn set_base_point(&mut self, point: Point) {
        self.options.base_point = point;
    }
    
    pub fn set_reference_angle(&mut self, angle: f64) {
        self.options.reference_angle = Some(angle);
    }
    
    pub fn set_scale_factor(&mut self, factor: f64) {
        self.options.scale_factor = Some(factor);
    }
    
    pub fn set_mirror_line(&mut self, p1: Point, p2: Point) {
        self.options.mirror_line = Some((p1, p2));
    }
    
    pub fn clear_options(&mut self) {
        self.options.reference_angle = None;
        self.options.scale_factor = None;
        self.options.mirror_line = None;
    }
}

impl Default for GripMode {
    fn default() -> Self {
        Self::new()
    }
}
