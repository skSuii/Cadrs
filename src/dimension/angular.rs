use crate::geometry::{Point, Vector2, Arc, Line};
use crate::data_structure::{Entity, EntityType, EntityGeometry};
use super::linear::{DimensionGeometry, DimensionStyle, DimensionType};
use serde::{Serialize, Deserialize};
use std::fmt;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AngularDimension {
    pub geometry: DimensionGeometry,
    pub arc: Arc,
    pub extension_line1: Line,
    pub extension_line2: Line,
    pub text_location: Point,
    pub arc_center: Point,
    pub arc_radius: f64,
    pub start_angle: f64,
    pub end_angle: f64,
    pub is_2_line: bool,
    pub is_3_point: bool,
}

impl AngularDimension {
    pub fn new_from_arc(
        center: Point,
        arc: Arc,
        style: DimensionStyle,
    ) -> Self {
        let mut geometry = DimensionGeometry::new(DimensionType::Angular, style.clone());
        geometry.definition_points = vec![
            center,
            Point::new(
                center.x + arc.radius * arc.start_angle.cos(),
                center.y + arc.radius * arc.start_angle.sin(),
                0.0,
            ),
            Point::new(
                center.x + arc.radius * arc.end_angle.cos(),
                center.y + arc.radius * arc.end_angle.sin(),
                0.0,
            ),
        ];
        
        let ext_length = style.extension_line_extension * 2.0;
        
        let v1 = Vector2::new(
            geometry.definition_points[1].x - center.x,
            geometry.definition_points[1].y - center.y,
        ).normalize();
        let v2 = Vector2::new(
            geometry.definition_points[2].x - center.x,
            geometry.definition_points[2].y - center.y,
        ).normalize();
        
        let ext1_end = Point::new(
            center.x + v1.x * ext_length,
            center.y + v1.y * ext_length,
            0.0,
        );
        let extension_line1 = Line::new(center, ext1_end);
        
        let ext2_end = Point::new(
            center.x + v2.x * ext_length,
            center.y + v2.y * ext_length,
            0.0,
        );
        let extension_line2 = Line::new(center, ext2_end);
        
        let arc_radius = style.arrow_size * 8.0;
        let mid_angle = (arc.start_angle + arc.end_angle) / 2.0;
        let text_location = Point::new(
            center.x + mid_angle.cos() * (arc_radius + style.text_height + style.dimension_line_gap),
            center.y + mid_angle.sin() * (arc_radius + style.text_height + style.dimension_line_gap),
            0.0,
        );
        
        geometry.calculate_measurement();
        
        Self {
            geometry,
            arc: Arc::new(center, arc_radius, arc.start_angle, arc.end_angle),
            extension_line1,
            extension_line2,
            text_location,
            arc_center: center,
            arc_radius,
            start_angle: arc.start_angle,
            end_angle: arc.end_angle,
            is_2_line: false,
            is_3_point: true,
        }
    }
    
    pub fn new_from_3_point(
        center: Point,
        p1: Point,
        p2: Point,
        style: DimensionStyle,
    ) -> Self {
        let mut geometry = DimensionGeometry::new(DimensionType::Angular, style.clone());
        geometry.definition_points = vec![center, p1, p2];
        
        let radius = center.distance_to(&p1).max(center.distance_to(&p2));
        let v1 = (p1 - center).to_vector2();
        let v2 = (p2 - center).to_vector2();
        let angle1 = v1.angle();
        let angle2 = v2.angle();
        
        let arc_radius = style.arrow_size * 8.0;
        let ext_length = style.extension_line_extension * 2.0;
        
        let ext1_end = Point::new(
            center.x + v1.normalize().x * ext_length,
            center.y + v1.normalize().y * ext_length,
            0.0,
        );
        let extension_line1 = Line::new(center, ext1_end);
        
        let ext2_end = Point::new(
            center.x + v2.normalize().x * ext_length,
            center.y + v2.normalize().y * ext_length,
            0.0,
        );
        let extension_line2 = Line::new(center, ext2_end);
        
        let start_angle = angle1.min(angle2);
        let end_angle = angle1.max(angle2);
        let mid_angle = (start_angle + end_angle) / 2.0;
        let text_location = Point::new(
            center.x + mid_angle.cos() * (arc_radius + style.text_height + style.dimension_line_gap),
            center.y + mid_angle.sin() * (arc_radius + style.text_height + style.dimension_line_gap),
            0.0,
        );
        
        geometry.calculate_measurement();
        
        Self {
            geometry,
            arc: Arc::new(center, arc_radius, start_angle, end_angle),
            extension_line1,
            extension_line2,
            text_location,
            arc_center: center,
            arc_radius,
            start_angle,
            end_angle,
            is_2_line: false,
            is_3_point: true,
        }
    }
    
    pub fn new_from_2_line(
        p1: Point,
        p2: Point,
        center: Point,
        style: DimensionStyle,
    ) -> Self {
        let mut geometry = DimensionGeometry::new(DimensionType::Angular, style.clone());
        geometry.definition_points = vec![center, p1, p2];
        
        let arc_radius = style.arrow_size * 8.0;
        let ext_length = style.extension_line_extension * 2.0;
        
        let v1 = (p1 - center).to_vector2().normalize();
        let v2 = (p2 - center).to_vector2().normalize();
        
        let ext1_end = Point::new(
            p1.x + v1.x * ext_length,
            p1.y + v1.y * ext_length,
            0.0,
        );
        let extension_line1 = Line::new(p1, ext1_end);
        
        let ext2_end = Point::new(
            p2.x + v2.x * ext_length,
            p2.y + v2.y * ext_length,
            0.0,
        );
        let extension_line2 = Line::new(p2, ext2_end);
        
        let angle1 = v1.angle();
        let angle2 = v2.angle();
        let start_angle = angle1.min(angle2);
        let end_angle = angle1.max(angle2);
        let mid_angle = (start_angle + end_angle) / 2.0;
        let text_location = Point::new(
            center.x + mid_angle.cos() * (arc_radius + style.text_height + style.dimension_line_gap),
            center.y + mid_angle.sin() * (arc_radius + style.text_height + style.dimension_line_gap),
            0.0,
        );
        
        geometry.calculate_measurement();
        
        Self {
            geometry,
            arc: Arc::new(center, arc_radius, start_angle, end_angle),
            extension_line1,
            extension_line2,
            text_location,
            arc_center: center,
            arc_radius,
            start_angle,
            end_angle,
            is_2_line: true,
            is_3_point: false,
        }
    }
    
    pub fn flip(&mut self) {
        std::mem::swap(&mut self.start_angle, &mut self.end_angle);
        std::mem::swap(&mut self.extension_line1, &mut self.extension_line2);
        self.geometry.calculate_measurement();
    }
    
    pub fn set_text_location(&mut self, location: Point) {
        self.text_location = location;
        self.geometry.user_text_location = Some(location);
    }
}

impl From<AngularDimension> for Entity {
    fn from(dim: AngularDimension) -> Self {
        let text_location = dim.text_location;
        let angle = dim.end_angle - dim.start_angle;
        let geometry = dim.geometry;
        Entity::new(
            EntityType::Dimension,
            super::linear::entity_geometry_from_dimension(&geometry, text_location, angle, true, false),
        )
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ArcLengthDimension {
    pub geometry: DimensionGeometry,
    pub arc: Arc,
    pub extension_line1: Line,
    pub extension_line2: Line,
    pub text_location: Point,
}

impl ArcLengthDimension {
    pub fn new(arc: Arc, style: DimensionStyle) -> Self {
        let mut geometry = DimensionGeometry::new(DimensionType::ArcLength, style.clone());
        geometry.definition_points = vec![
            arc.center,
            Point::new(
                arc.center.x + arc.radius * arc.start_angle.cos(),
                arc.center.y + arc.radius * arc.start_angle.sin(),
                0.0,
            ),
            Point::new(
                arc.center.x + arc.radius * arc.end_angle.cos(),
                arc.center.y + arc.radius * arc.end_angle.sin(),
                0.0,
            ),
        ];
        
        let arc_length = arc.radius * (arc.end_angle - arc.start_angle).abs();
        geometry.measurement = arc_length;
        geometry.actual_measurement = arc_length;
        geometry.update_text();
        
        let radius = style.arrow_size * 8.0;
        let mid_angle = (arc.start_angle + arc.end_angle) / 2.0;
        let text_location = Point::new(
            arc.center.x + mid_angle.cos() * (radius + style.text_height + style.dimension_line_gap),
            arc.center.y + mid_angle.sin() * (radius + style.text_height + style.dimension_line_gap),
            0.0,
        );
        
        let ext_length = style.extension_line_extension * 2.0;
        let v1 = Vector2::new(
            geometry.definition_points[1].x - arc.center.x,
            geometry.definition_points[1].y - arc.center.y,
        ).normalize();
        let v2 = Vector2::new(
            geometry.definition_points[2].x - arc.center.x,
            geometry.definition_points[2].y - arc.center.y,
        ).normalize();
        
        let ext1_end = Point::new(
            geometry.definition_points[1].x + v1.x * ext_length,
            geometry.definition_points[1].y + v1.y * ext_length,
            0.0,
        );
        let extension_line1 = Line::new(geometry.definition_points[1], ext1_end);
        
        let ext2_end = Point::new(
            geometry.definition_points[2].x + v2.x * ext_length,
            geometry.definition_points[2].y + v2.y * ext_length,
            0.0,
        );
        let extension_line2 = Line::new(geometry.definition_points[2], ext2_end);
        
        Self {
            geometry,
            arc: Arc::new(arc.center, radius, arc.start_angle, arc.end_angle),
            extension_line1,
            extension_line2,
            text_location,
        }
    }
}

impl From<ArcLengthDimension> for Entity {
    fn from(dim: ArcLengthDimension) -> Self {
        let text_location = dim.text_location;
        let angle = dim.arc.end_angle - dim.arc.start_angle;
        let geometry = dim.geometry;
        Entity::new(
            EntityType::Dimension,
            super::linear::entity_geometry_from_dimension(&geometry, text_location, angle, true, false),
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum AngularUnit {
    Degrees,
    DegreesMinutesSeconds,
    Gradians,
    Radians,
}

impl Default for AngularUnit {
    fn default() -> Self {
        AngularUnit::Degrees
    }
}

impl AngularUnit {
    pub fn format_angle(&self, angle_radians: f64) -> String {
        match self {
            AngularUnit::Degrees => {
                let degrees = angle_radians.to_degrees();
                format!("{:.2}°", degrees)
            }
            AngularUnit::DegreesMinutesSeconds => {
                let degrees = angle_radians.to_degrees();
                let d = degrees.floor() as i32;
                let minutes = ((degrees - d as f64) * 60.0).floor() as i32;
                let seconds = ((degrees - d as f64 - minutes as f64 / 60.0) * 3600.0).round();
                format!("{}°{}'{:.0}\"", d, minutes, seconds)
            }
            AngularUnit::Gradians => {
                let gradians = angle_radians.to_degrees() * 10.0 / 9.0;
                format!("{:.2}g", gradians)
            }
            AngularUnit::Radians => {
                format!("{:.4} rad", angle_radians)
            }
        }
    }
}
