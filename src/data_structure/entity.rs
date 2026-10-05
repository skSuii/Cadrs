use crate::geometry::{Point, Line, Circle, Arc, Ellipse, Polyline, BSpline, NURBS};
use crate::data_structure::ObjectId;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::any::Any;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum EntityType {
    Point,
    Line,
    Circle,
    Arc,
    Ellipse,
    Polyline,
    BSpline,
    NURBS,
    Dimension,
    Text,
    BlockRef,
    Image,
    Hatch,
    Solid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entity {
    pub id: ObjectId,
    pub entity_type: EntityType,
    pub layer_id: ObjectId,
    pub properties: HashMap<String, String>,
    pub visibility: Visibility,
    pub transform: Transform,
    pub geometry: EntityGeometry,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Visibility {
    Visible,
    Hidden,
    Frozen,
    Locked,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Transform {
    pub translation: Point,
    pub rotation: f64,
    pub scale: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EntityGeometry {
    Point(Point),
    Line(Line),
    Circle(Circle),
    Arc(Arc),
    Ellipse(Ellipse),
    Polyline(Polyline),
    BSpline(BSpline),
    NURBS(NURBS),
    Text {
        content: String,
        position: Point,
        height: f64,
        rotation: f64,
        width_factor: f64,
        font_name: String,
        style: TextStyle,
    },
    Dimension {
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
        def_point_4: Point,
        angle: f64,
        extension_lines: bool,
        center_marks: bool,
    },
    BlockRef {
        block_name: String,
        position: Point,
        scale_x: f64,
        scale_y: f64,
        scale_z: f64,
        rotation: f64,
        column_count: u32,
        row_count: u32,
        column_spacing: f64,
        row_spacing: f64,
    },
    Hatch {
        pattern_name: String,
        pattern_scale: f64,
        pattern_angle: f64,
        solid_fill: bool,
        fill_color: (u8, u8, u8),
        boundary_paths: Vec<HatchBoundary>,
        associativity: bool,
    },
    Solid {
        points: [Point; 4],
        color: (u8, u8, u8),
    },
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum DimensionType {
    Linear,
    Aligned,
    Angular,
    Diameter,
    Radius,
    Ordinate,
    ArcLength,
    Coordinate,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TextStyle {
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
    pub alignment: TextAlignment,
}

impl Default for TextStyle {
    fn default() -> Self {
        Self {
            bold: false,
            italic: false,
            underline: false,
            alignment: TextAlignment::Left,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum TextAlignment {
    Left,
    Center,
    Right,
    Middle,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HatchBoundary {
    pub boundary_type: BoundaryType,
    pub edges: Vec<HatchEdge>,
    pub is_outer: bool,
    pub is_polyline: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BoundaryType {
    External,
    Outer,
    Hole,
    Derived,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HatchEdge {
    pub edge_type: EdgeType,
    pub start_point: Point,
    pub end_point: Point,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub center_point: Option<Point>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub radius: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_angle: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_angle: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bulge: Option<f64>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum EdgeType {
    Line,
    Arc,
    Ellipse,
}

impl Default for Visibility {
    fn default() -> Self {
        Visibility::Visible
    }
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            translation: Point::origin(),
            rotation: 0.0,
            scale: 1.0,
        }
    }
}

impl Entity {
    #[inline]
    pub fn new(entity_type: EntityType, geometry: EntityGeometry) -> Self {
        Self {
            id: ObjectId::new(),
            entity_type,
            layer_id: ObjectId::nil(),
            properties: HashMap::new(),
            visibility: Visibility::default(),
            transform: Transform::default(),
            geometry,
        }
    }

    #[inline]
    pub fn id(&self) -> &ObjectId {
        &self.id
    }

    #[inline]
    pub fn entity_type(&self) -> &EntityType {
        &self.entity_type
    }

    #[inline]
    pub fn layer_id(&self) -> &ObjectId {
        &self.layer_id
    }

    #[inline]
    pub fn set_layer_id(&mut self, layer_id: ObjectId) {
        self.layer_id = layer_id;
    }

    #[inline]
    pub fn properties(&self) -> &HashMap<String, String> {
        &self.properties
    }

    #[inline]
    pub fn set_property(&mut self, key: String, value: String) {
        self.properties.insert(key, value);
    }

    #[inline]
    pub fn get_property(&self, key: &str) -> Option<&String> {
        self.properties.get(key)
    }

    #[inline]
    pub fn visibility(&self) -> Visibility {
        self.visibility
    }

    #[inline]
    pub fn set_visibility(&mut self, visibility: Visibility) {
        self.visibility = visibility;
    }

    #[inline]
    pub fn transform(&self) -> Transform {
        self.transform
    }

    #[inline]
    pub fn set_transform(&mut self, transform: Transform) {
        self.transform = transform;
    }

    #[inline]
    pub fn transform_mut(&mut self) -> &mut Transform {
        &mut self.transform
    }

    #[inline]
    pub fn get_position(&self) -> Option<Point> {
        match &self.geometry {
            EntityGeometry::Point(p) => Some(*p),
            EntityGeometry::Line(l) => Some(l.start),
            EntityGeometry::Circle(c) => Some(c.center),
            EntityGeometry::Arc(a) => Some(a.center),
            EntityGeometry::Ellipse(e) => Some(e.center),
            EntityGeometry::Text { position, .. } => Some(*position),
            EntityGeometry::Dimension { definition_point, .. } => Some(*definition_point),
            EntityGeometry::BlockRef { position, .. } => Some(*position),
            _ => None,
        }
    }

    #[inline]
    pub fn geometry(&self) -> &EntityGeometry {
        &self.geometry
    }

    #[inline]
    pub fn geometry_mut(&mut self) -> &mut EntityGeometry {
        &mut self.geometry
    }

    #[inline]
    pub fn bounding_box(&self) -> Option<(Point, Point)> {
        match &self.geometry {
            EntityGeometry::Point(p) => Some((*p, *p)),
            EntityGeometry::Line(l) => {
                let min = Point::new(l.start.x.min(l.end.x), l.start.y.min(l.end.y), 0.0);
                let max = Point::new(l.start.x.max(l.end.x), l.start.y.max(l.end.y), 0.0);
                Some((min, max))
            }
            EntityGeometry::Circle(c) => {
                let min = Point::new(c.center.x - c.radius, c.center.y - c.radius, 0.0);
                let max = Point::new(c.center.x + c.radius, c.center.y + c.radius, 0.0);
                Some((min, max))
            }
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_entity_creation() {
        let line = Line::new(Point::origin(), Point::new(1.0, 0.0, 0.0));
        let entity = Entity::new(EntityType::Line, EntityGeometry::Line(line));
        
        assert_eq!(entity.entity_type(), &EntityType::Line);
        assert_eq!(entity.visibility(), Visibility::Visible);
    }

    #[test]
    fn test_entity_properties() {
        let point = Point::origin();
        let mut entity = Entity::new(EntityType::Point, EntityGeometry::Point(point));

        entity.set_property("color".to_string(), "red".to_string());
        assert_eq!(entity.get_property("color"), Some(&"red".to_string()));
    }
}
