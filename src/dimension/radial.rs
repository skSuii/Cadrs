use crate::geometry::{Point, Vector2, Circle, Arc, Line};
use crate::data_structure::{Entity, EntityType, EntityGeometry};
use super::linear::{DimensionGeometry, DimensionStyle, DimensionType};
use serde::{Serialize, Deserialize};
use std::fmt;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RadialDimension {
    pub geometry: DimensionGeometry,
    pub center_mark: CenterMark,
    pub dimension_line: Line,
    pub extension_line: Line,
    pub arrow: Point,
    pub center_point: Point,
    pub arc_point: Point,
    pub is_diameter: bool,
    pub joggle_enabled: bool,
    pub joggle_location: Option<Point>,
    pub joggle_offset: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CenterMark {
    pub center: Point,
    pub size: f64,
    pub mark_type: CenterMarkType,
    pub extension_line_length: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum CenterMarkType {
    None,
    Mark,
    Cross,
}

impl RadialDimension {
    pub fn new_radial(
        circle: Circle,
        p: Point,
        style: DimensionStyle,
    ) -> Self {
        let mut geometry = DimensionGeometry::new(DimensionType::Radial, style.clone());
        geometry.definition_points = vec![circle.center, p];
        
        let radius = circle.center.distance_to(&p);
        geometry.measurement = radius;
        geometry.actual_measurement = radius;
        geometry.update_text();
        
        let center_mark = CenterMark {
            center: circle.center,
            size: style.arrow_size * 2.0,
            mark_type: CenterMarkType::Cross,
            extension_line_length: style.extension_line_extension,
        };
        
        let direction = (p.to_vector2() - circle.center.to_vector2()).normalize();
        let offset = style.arrow_size;
        let arrow = Point::new(
            p.x - direction.x * offset,
            p.y - direction.y * offset,
            0.0,
        );
        
        let dim_line_end = Point::new(
            p.x + direction.x * (style.text_height + style.dimension_line_gap),
            p.y + direction.y * (style.text_height + style.dimension_line_gap),
            0.0,
        );
        let dimension_line = Line::new(p, dim_line_end);
        
        let extension_line = Line::new(p, circle.center);
        
        Self {
            geometry,
            center_mark,
            dimension_line,
            extension_line,
            arrow,
            center_point: circle.center,
            arc_point: p,
            is_diameter: false,
            joggle_enabled: false,
            joggle_location: None,
            joggle_offset: style.text_height,
        }
    }
    
    pub fn new_diameter(
        circle: Circle,
        p1: Point,
        p2: Point,
        style: DimensionStyle,
    ) -> Self {
        let mut geometry = DimensionGeometry::new(DimensionType::Diameter, style.clone());
        geometry.definition_points = vec![circle.center, p1, p2];
        
        let diameter = circle.radius * 2.0;
        geometry.measurement = diameter;
        geometry.actual_measurement = diameter;
        geometry.update_text();
        
        let center_mark = CenterMark {
            center: circle.center,
            size: style.arrow_size * 2.0,
            mark_type: CenterMarkType::Cross,
            extension_line_length: style.extension_line_extension,
        };
        
        let direction = (p1.to_vector2() - circle.center.to_vector2()).normalize();
        let opposite_direction = Vector2::new(-direction.x, -direction.y);
        let offset = style.arrow_size;
        
        let arrow = Point::new(
            p1.x - direction.x * offset,
            p1.y - direction.y * offset,
            0.0,
        );
        
        let dim_line_end = Point::new(
            p1.x + direction.x * (style.text_height + style.dimension_line_gap),
            p1.y + direction.y * (style.text_height + style.dimension_line_gap),
            0.0,
        );
        let dimension_line = Line::new(p1, dim_line_end);
        
        let extension_line = Line::new(p1, p2);
        
        Self {
            geometry,
            center_mark,
            dimension_line,
            extension_line,
            arrow,
            center_point: circle.center,
            arc_point: p1,
            is_diameter: true,
            joggle_enabled: false,
            joggle_location: None,
            joggle_offset: style.text_height,
        }
    }
    
    pub fn with_joggle(mut self, offset: f64) -> Self {
        self.joggle_enabled = true;
        self.joggle_offset = offset;
        self
    }
    
    pub fn set_center_mark_type(&mut self, mark_type: CenterMarkType) {
        self.center_mark.mark_type = mark_type;
    }
    
    pub fn flip(&mut self) {
        std::mem::swap(&mut self.dimension_line.start, &mut self.dimension_line.end);
        self.geometry.calculate_measurement();
    }
    
    pub fn set_measurement(&mut self, measurement: f64) {
        self.geometry.measurement = measurement;
        self.geometry.actual_measurement = measurement;
        self.geometry.update_text();
    }
}

impl From<RadialDimension> for Entity {
    fn from(dim: RadialDimension) -> Self {
        let arrow = dim.arrow;
        let geometry = dim.geometry;
        Entity::new(
            EntityType::Dimension,
            super::linear::entity_geometry_from_dimension(&geometry, arrow, 0.0, true, true),
        )
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SmallRadialDimension {
    pub geometry: DimensionGeometry,
    pub center_mark: CenterMark,
    pub dimension_line: Arc,
    pub arrow1: Point,
    pub arrow2: Point,
    pub text_location: Point,
    pub center_point: Point,
}

impl SmallRadialDimension {
    pub fn new(
        circle: Circle,
        p: Point,
        style: DimensionStyle,
    ) -> Self {
        let mut geometry = DimensionGeometry::new(DimensionType::Radial, style.clone());
        geometry.definition_points = vec![circle.center, p];
        
        let radius = circle.center.distance_to(&p);
        geometry.measurement = radius;
        geometry.actual_measurement = radius;
        geometry.update_text();
        
        let center_mark = CenterMark {
            center: circle.center,
            size: style.arrow_size,
            mark_type: CenterMarkType::Mark,
            extension_line_length: 0.0,
        };
        
        let direction = (p.to_vector2() - circle.center.to_vector2()).normalize();
        let arrow_size = style.arrow_size;
        
        let arrow1 = Point::new(
            p.x - direction.x * arrow_size * 0.5,
            p.y - direction.y * arrow_size * 0.5,
            0.0,
        );
        let arrow2 = Point::new(
            p.x + direction.x * arrow_size * 0.5,
            p.y + direction.y * arrow_size * 0.5,
            0.0,
        );
        
        let text_location = Point::new(
            p.x + direction.x * (style.text_height + style.dimension_line_gap),
            p.y + direction.y * (style.text_height + style.dimension_line_gap),
            0.0,
        );
        
        let dim_line_radius = circle.center.distance_to(&arrow1);
        let angle = (p - circle.center).to_vector2().angle();
        let start_angle = angle - std::f64::consts::PI / 6.0;
        let end_angle = angle + std::f64::consts::PI / 6.0;
        let dimension_line = Arc::new(circle.center, dim_line_radius, start_angle, end_angle);
        
        Self {
            geometry,
            center_mark,
            dimension_line,
            arrow1,
            arrow2,
            text_location,
            center_point: circle.center,
        }
    }
}

impl From<SmallRadialDimension> for Entity {
    fn from(dim: SmallRadialDimension) -> Self {
        let text_location = dim.text_location;
        let geometry = dim.geometry;
        Entity::new(
            EntityType::Dimension,
            super::linear::entity_geometry_from_dimension(&geometry, text_location, 0.0, false, true),
        )
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DiameterDimension {
    pub geometry: DimensionGeometry,
    pub center_mark: CenterMark,
    pub dimension_line: Line,
    pub text_location: Point,
    pub center_point: Point,
    pub far_point: Point,
}

impl DiameterDimension {
    pub fn new_with_center_line(
        circle: Circle,
        style: DimensionStyle,
        text_location: Option<Point>,
    ) -> Self {
        let mut geometry = DimensionGeometry::new(DimensionType::Diameter, style.clone());
        geometry.definition_points = vec![circle.center];
        
        let diameter = circle.radius * 2.0;
        geometry.measurement = diameter;
        geometry.actual_measurement = diameter;
        geometry.update_text();
        
        let center_mark = CenterMark {
            center: circle.center,
            size: style.arrow_size * 2.0,
            mark_type: CenterMarkType::Cross,
            extension_line_length: style.extension_line_extension,
        };
        
        let far_point = Point::new(
            circle.center.x + circle.radius,
            circle.center.y,
            0.0,
        );
        
        let text_location = text_location.unwrap_or_else(|| {
            Point::new(
                circle.center.x + circle.radius * 1.5,
                circle.center.y,
                0.0,
            )
        });
        
        let dimension_line = Line::new(circle.center, far_point);
        
        Self {
            geometry,
            center_mark,
            dimension_line,
            text_location,
            center_point: circle.center,
            far_point,
        }
    }
}

impl From<DiameterDimension> for Entity {
    fn from(dim: DiameterDimension) -> Self {
        let text_location = dim.text_location;
        let geometry = dim.geometry;
        Entity::new(
            EntityType::Dimension,
            super::linear::entity_geometry_from_dimension(&geometry, text_location, 0.0, false, true),
        )
    }
}
