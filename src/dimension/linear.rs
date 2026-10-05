use crate::geometry::{Point, Vector2, Line, Arc, Circle};
use crate::data_structure::{Entity, EntityType, EntityGeometry, ObjectId, Transform, Visibility, TextStyle};
use serde::{Serialize, Deserialize};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum DimensionType {
    Linear,
    Aligned,
    Angular,
    Radial,
    Diameter,
    ArcLength,
    Ordinate,
    Baseline,
    Continued,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DimensionStyle {
    pub id: ObjectId,
    pub name: String,
    pub text_height: f64,
    pub arrow_size: f64,
    pub arrow_style: ArrowStyle,
    pub extension_line_extension: f64,
    pub extension_line_offset: f64,
    pub dimension_line_gap: f64,
    pub text_horizontal_placement: HorizontalTextPlacement,
    pub text_vertical_placement: VerticalTextPlacement,
    pub text_direction: TextDirection,
    pub unit_format: UnitFormat,
    pub decimal_places: u32,
    pub round_off: f64,
    pub prefix: String,
    pub suffix: String,
    pub alternate_units: bool,
    pub alternate_units_factor: f64,
    pub tolerance_display: ToleranceDisplay,
    pub tolerance_precision: u32,
    pub tolerance_upper_value: f64,
    pub tolerance_lower_value: f64,
    pub text_color: (u8, u8, u8),
    pub extension_line_color: (u8, u8, u8),
    pub dimension_line_color: (u8, u8, u8),
    pub visible: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum ArrowStyle {
    Closed,
    ClosedFilled,
    Dot,
    SmallClosed,
    Open,
    OriginIndicator,
    Origin02,
    Oblique,
    ArchitecturalTick,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum HorizontalTextPlacement {
    Centered,
    Above,
    Below,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum VerticalTextPlacement {
    Centered,
    JIS,
    Above,
    AboveFromDimensionLine,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum TextDirection {
    LeftToRight,
    RightToLeft,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum UnitFormat {
    Decimal,
    Scientific,
    Engineering,
    Architectural,
    Fractional,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum ToleranceDisplay {
    None,
    Symmetrical,
    Deviation,
    Limits,
    Basic,
}

impl Default for DimensionStyle {
    fn default() -> Self {
        Self {
            id: ObjectId::new(),
            name: "Standard".to_string(),
            text_height: 2.5,
            arrow_size: 2.5,
            arrow_style: ArrowStyle::ClosedFilled,
            extension_line_extension: 1.75,
            extension_line_offset: 0.625,
            dimension_line_gap: 0.625,
            text_horizontal_placement: HorizontalTextPlacement::Centered,
            text_vertical_placement: VerticalTextPlacement::Above,
            text_direction: TextDirection::LeftToRight,
            unit_format: UnitFormat::Decimal,
            decimal_places: 2,
            round_off: 0.0,
            prefix: "".to_string(),
            suffix: "".to_string(),
            alternate_units: false,
            alternate_units_factor: 25.4,
            tolerance_display: ToleranceDisplay::None,
            tolerance_precision: 2,
            tolerance_upper_value: 0.0,
            tolerance_lower_value: 0.0,
            text_color: (0, 0, 0),
            extension_line_color: (0, 0, 0),
            dimension_line_color: (0, 0, 0),
            visible: true,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DimensionGeometry {
    pub dimension_type: DimensionType,
    pub definition_points: Vec<Point>,
    pub measurement: f64,
    pub text: String,
    pub style: DimensionStyle,
    pub attachment_point: AttachmentPoint,
    pub user_text_location: Option<Point>,
    pub text_rotation: f64,
    pub actual_measurement: f64,
}

impl DimensionGeometry {
    pub fn new(dimension_type: DimensionType, style: DimensionStyle) -> Self {
        Self {
            dimension_type,
            definition_points: Vec::new(),
            measurement: 0.0,
            text: String::new(),
            style,
            attachment_point: AttachmentPoint::MiddleCenter,
            user_text_location: None,
            text_rotation: 0.0,
            actual_measurement: 0.0,
        }
    }
    
    pub fn calculate_measurement(&mut self) {
        match self.dimension_type {
            DimensionType::Linear | DimensionType::Aligned => {
                if self.definition_points.len() >= 2 {
                    let p1 = self.definition_points[0];
                    let p2 = self.definition_points[1];
                    self.measurement = (p2.to_vector2() - p1.to_vector2()).magnitude();
                    self.actual_measurement = self.measurement;
                }
            }
            DimensionType::Angular => {
                if self.definition_points.len() >= 3 {
                    let center = self.definition_points[0];
                    let p1 = self.definition_points[1];
                    let p2 = self.definition_points[2];
                    let v1 = (p1 - center).to_vector2();
                    let v2 = (p2 - center).to_vector2();
                    let angle1 = v1.angle();
                    let angle2 = v2.angle();
                    let mut diff = (angle2 - angle1).abs();
                    if diff > std::f64::consts::PI {
                        diff = 2.0 * std::f64::consts::PI - diff;
                    }
                    self.measurement = diff;
                    self.actual_measurement = self.measurement;
                }
            }
            DimensionType::Radial | DimensionType::Diameter => {
                if self.definition_points.len() >= 2 {
                    let center = self.definition_points[0];
                    let p = self.definition_points[1];
                    let radius = center.distance_to(&p);
                    if self.dimension_type == DimensionType::Diameter {
                        self.measurement = radius * 2.0;
                    } else {
                        self.measurement = radius;
                    }
                    self.actual_measurement = self.measurement;
                }
            }
            _ => {}
        }
        
        self.update_text();
    }
    
    pub fn update_text(&mut self) {
        let formatted_value = self.format_measurement(self.measurement);
        
        let tolerance_text = if self.style.tolerance_display != ToleranceDisplay::None {
            format!(
                "{:+.*} {} {:+.*}",
                self.style.tolerance_precision as usize,
                self.style.tolerance_upper_value,
                formatted_value,
                self.style.tolerance_precision as usize,
                self.style.tolerance_lower_value
            )
        } else {
            formatted_value
        };
        
        self.text = format!(
            "{}{}{}",
            self.style.prefix,
            tolerance_text,
            self.style.suffix
        );
    }
    
    fn format_measurement(&self, value: f64) -> String {
        let rounded = (value / self.style.round_off).round() * self.style.round_off;
        
        match self.style.unit_format {
            UnitFormat::Decimal => {
                format!("{:.*}", self.style.decimal_places as usize, rounded)
            }
            UnitFormat::Scientific => {
                format!("{:e}", rounded)
            }
            UnitFormat::Engineering => {
                let feet = (rounded / 12.0).floor();
                let inches = rounded - feet * 12.0;
                format!("{}-{:.*}", feet as u64, self.style.decimal_places as usize, inches)
            }
            UnitFormat::Fractional => {
                let inches = rounded;
                let whole = inches.floor();
                let fraction = inches - whole;
                let denominator = 64;
                let numerator = (fraction * denominator as f64).round() as u64;
                if numerator == 0 {
                    format!("{}", whole as u64)
                } else {
                    format!("{}-{}/{}", whole as u64, numerator, denominator)
                }
            }
            _ => {
                format!("{:.*}", self.style.decimal_places as usize, rounded)
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum AttachmentPoint {
    TopLeft,
    TopCenter,
    TopRight,
    MiddleLeft,
    MiddleCenter,
    MiddleRight,
    BottomLeft,
    BottomCenter,
    BottomRight,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LinearDimension {
    pub geometry: DimensionGeometry,
    pub extension_line1: Line,
    pub extension_line2: Line,
    pub dimension_line: Line,
    pub arrow1: Point,
    pub arrow2: Point,
    pub center_mark: Option<CenterMark>,
}

impl LinearDimension {
    pub fn new(
        definition_line: Line,
        style: DimensionStyle,
        location: Option<Point>,
    ) -> Self {
        let mut geometry = DimensionGeometry::new(DimensionType::Linear, style.clone());
        geometry.definition_points = vec![definition_line.start, definition_line.end];
        
        let direction = (definition_line.end.to_vector2() - definition_line.start.to_vector2()).normalize();
        let perpendicular = Vector2::new(-direction.y, direction.x);
        
        let offset = style.extension_line_offset;
        let extension = style.extension_line_extension;
        
        let ext1_start = Point::new(
            definition_line.start.x - direction.x * offset,
            definition_line.start.y - direction.y * offset,
            0.0,
        );
        let ext1_end = Point::new(
            definition_line.start.x + direction.x * extension,
            definition_line.start.y + direction.y * extension,
            0.0,
        );
        let extension_line1 = Line::new(ext1_start, ext1_end);
        
        let ext2_start = Point::new(
            definition_line.end.x - direction.x * offset,
            definition_line.end.y - direction.y * offset,
            0.0,
        );
        let ext2_end = Point::new(
            definition_line.end.x + direction.x * extension,
            definition_line.end.y + direction.y * extension,
            0.0,
        );
        let extension_line2 = Line::new(ext2_start, ext2_end);
        
        let mid_point = definition_line.midpoint();
        let dim_line_start = Point::new(
            mid_point.x + perpendicular.x * offset * 2.0,
            mid_point.y + perpendicular.y * offset * 2.0,
            0.0,
        );
        let dim_line_end = Point::new(
            mid_point.x + perpendicular.x * (offset * 2.0 + style.text_height),
            mid_point.y + perpendicular.y * (offset * 2.0 + style.text_height),
            0.0,
        );
        let dimension_line = Line::new(dim_line_start, dim_line_end);
        
        geometry.calculate_measurement();
        
        let arrow_offset = style.arrow_size;
        let arrow1 = Point::new(
            dim_line_start.x + perpendicular.x * arrow_offset,
            dim_line_start.y + perpendicular.y * arrow_offset,
            0.0,
        );
        let arrow2 = Point::new(
            dim_line_end.x - perpendicular.x * arrow_offset,
            dim_line_end.y - perpendicular.y * arrow_offset,
            0.0,
        );
        
        Self {
            geometry,
            extension_line1,
            extension_line2,
            dimension_line,
            arrow1,
            arrow2,
            center_mark: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AlignedDimension {
    pub geometry: DimensionGeometry,
    pub extension_line1: Line,
    pub extension_line2: Line,
    pub dimension_line: Line,
    pub arrow1: Point,
    pub arrow2: Point,
}

impl AlignedDimension {
    pub fn new(
        p1: Point,
        p2: Point,
        style: DimensionStyle,
        location: Option<Point>,
    ) -> Self {
        let mut geometry = DimensionGeometry::new(DimensionType::Aligned, style.clone());
        geometry.definition_points = vec![p1, p2];
        
        let direction = (p2.to_vector2() - p1.to_vector2()).normalize();
        
        let offset = style.extension_line_offset;
        let extension = style.extension_line_extension;
        
        let ext1_start = Point::new(
            p1.x - direction.y * offset,
            p1.y + direction.x * offset,
            0.0,
        );
        let ext1_end = Point::new(
            p1.x + direction.y * (extension + offset),
            p1.y - direction.x * (extension + offset),
            0.0,
        );
        let extension_line1 = Line::new(ext1_start, ext1_end);
        
        let ext2_start = Point::new(
            p2.x - direction.y * offset,
            p2.y + direction.x * offset,
            0.0,
        );
        let ext2_end = Point::new(
            p2.x + direction.y * (extension + offset),
            p2.y - direction.x * (extension + offset),
            0.0,
        );
        let extension_line2 = Line::new(ext2_start, ext2_end);
        
        let mid_point = p1.midpoint(&p2);
        let perpendicular = Vector2::new(-direction.y, direction.x);
        let dim_line_start = Point::new(
            mid_point.x + perpendicular.x * offset * 2.0,
            mid_point.y + perpendicular.y * offset * 2.0,
            0.0,
        );
        let dim_line_end = Point::new(
            mid_point.x + perpendicular.x * (offset * 2.0 + style.text_height),
            mid_point.y + perpendicular.y * (offset * 2.0 + style.text_height),
            0.0,
        );
        let dimension_line = Line::new(dim_line_start, dim_line_end);
        
        geometry.calculate_measurement();
        
        let arrow_offset = style.arrow_size;
        let arrow1 = Point::new(
            dim_line_start.x + perpendicular.x * arrow_offset,
            dim_line_start.y + perpendicular.y * arrow_offset,
            0.0,
        );
        let arrow2 = Point::new(
            dim_line_end.x - perpendicular.x * arrow_offset,
            dim_line_end.y - perpendicular.y * arrow_offset,
            0.0,
        );
        
        Self {
            geometry,
            extension_line1,
            extension_line2,
            dimension_line,
            arrow1,
            arrow2,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AngularDimension {
    pub geometry: DimensionGeometry,
    pub arc: Arc,
    pub extension_line1: Line,
    pub extension_line2: Line,
    pub text_location: Point,
}

impl AngularDimension {
    pub fn new(
        center: Point,
        p1: Point,
        p2: Point,
        style: DimensionStyle,
        location: Option<Point>,
    ) -> Self {
        let mut geometry = DimensionGeometry::new(DimensionType::Angular, style.clone());
        geometry.definition_points = vec![center, p1, p2];
        
        let radius = center.distance_to(&p1).max(center.distance_to(&p2));
        let v1 = (p1 - center).to_vector2();
        let v2 = (p2 - center).to_vector2();
        let angle1 = v1.angle();
        let angle2 = v2.angle();
        
        let start_angle = angle1.min(angle2);
        let end_angle = angle1.max(angle2);
        let sweep = (end_angle - start_angle).abs();
        
        let arc = Arc::new(center, radius, start_angle, end_angle);
        
        let ext_length = style.extension_line_extension;
        let ext1_start = center;
        let ext1_end = Point::new(
            center.x + v1.normalize().x * ext_length * 2.0,
            center.y + v1.normalize().y * ext_length * 2.0,
            0.0,
        );
        let extension_line1 = Line::new(ext1_start, ext1_end);
        
        let ext2_start = center;
        let ext2_end = Point::new(
            center.x + v2.normalize().x * ext_length * 2.0,
            center.y + v2.normalize().y * ext_length * 2.0,
            0.0,
        );
        let extension_line2 = Line::new(ext2_start, ext2_end);
        
        geometry.calculate_measurement();
        
        let mid_angle = (start_angle + end_angle) / 2.0;
        let text_location = Point::new(
            center.x + mid_angle.cos() * (radius + style.text_height + style.dimension_line_gap),
            center.y + mid_angle.sin() * (radius + style.text_height + style.dimension_line_gap),
            0.0,
        );
        
        Self {
            geometry,
            arc,
            extension_line1,
            extension_line2,
            text_location,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RadialDimension {
    pub geometry: DimensionGeometry,
    pub center_mark: CenterMark,
    pub dimension_line: Line,
    pub extension_line: Line,
    pub arrow: Point,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CenterMark {
    pub center: Point,
    pub size: f64,
    pub mark_type: CenterMarkType,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum CenterMarkType {
    None,
    Mark,
    Cross,
}

impl RadialDimension {
    pub fn new(
        circle: Circle,
        p: Point,
        style: DimensionStyle,
        is_diameter: bool,
    ) -> Self {
        let dim_type = if is_diameter { DimensionType::Diameter } else { DimensionType::Radial };
        let mut geometry = DimensionGeometry::new(dim_type, style.clone());
        geometry.definition_points = vec![circle.center, p];
        
        let radius = circle.center.distance_to(&p);
        if is_diameter {
            geometry.measurement = radius * 2.0;
        } else {
            geometry.measurement = radius;
        }
        geometry.actual_measurement = geometry.measurement;
        geometry.update_text();
        
        let center_mark_size = style.arrow_size * 2.0;
        let center_mark = CenterMark {
            center: circle.center,
            size: center_mark_size,
            mark_type: CenterMarkType::Cross,
        };
        
        let direction = (p.to_vector2() - circle.center.to_vector2()).normalize();
        let offset = style.arrow_size;
        let arrow = Point::new(
            p.x - direction.x * offset,
            p.y - direction.y * offset,
            0.0,
        );
        
        let dim_line_start = Point::new(
            circle.center.x + direction.x * radius,
            circle.center.y + direction.y * radius,
            0.0,
        );
        let dim_line_end = Point::new(
            circle.center.x + direction.x * (radius + style.text_height + style.dimension_line_gap),
            circle.center.y + direction.y * (radius + style.text_height + style.dimension_line_gap),
            0.0,
        );
        let dimension_line = Line::new(dim_line_start, dim_line_end);
        
        let extension_line = Line::new(p, dim_line_start);
        
        Self {
            geometry,
            center_mark,
            dimension_line,
            extension_line,
            arrow,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OrdinateDimension {
    pub geometry: DimensionGeometry,
    pub feature_point: Point,
    pub leader_point: Point,
    pub dimension_line: Line,
    pub orientation: OrdinateOrientation,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum OrdinateOrientation {
    X,
    Y,
}

impl OrdinateDimension {
    pub fn new(
        feature_point: Point,
        leader_point: Point,
        style: DimensionStyle,
        orientation: OrdinateOrientation,
    ) -> Self {
        let dim_type = match orientation {
            OrdinateOrientation::X => DimensionType::Ordinate,
            _ => DimensionType::Ordinate,
        };
        let mut geometry = DimensionGeometry::new(dim_type, style.clone());
        geometry.definition_points = vec![feature_point, leader_point];
        
        match orientation {
            OrdinateOrientation::X => {
                geometry.measurement = feature_point.x;
            }
            OrdinateOrientation::Y => {
                geometry.measurement = feature_point.y;
            }
        }
        geometry.actual_measurement = geometry.measurement;
        geometry.update_text();
        
        let dimension_line = Line::new(feature_point, leader_point);
        
        Self {
            geometry,
            feature_point,
            leader_point,
            dimension_line,
            orientation,
        }
    }
}

pub fn text_position_for(geometry: &DimensionGeometry) -> Point {
    geometry
        .user_text_location
        .clone()
        .unwrap_or_else(|| {
            geometry
                .definition_points
                .first()
                .cloned()
                .unwrap_or_else(Point::origin)
        })
}

pub fn entity_geometry_from_dimension(
    geometry: &DimensionGeometry,
    text_position: Point,
    angle: f64,
    extension_lines: bool,
    center_marks: bool,
) -> EntityGeometry {
    let point_at = |i: usize| {
        geometry
            .definition_points
            .get(i)
            .cloned()
            .unwrap_or_else(Point::origin)
    };
    let dim_type = match geometry.dimension_type {
        DimensionType::Linear | DimensionType::Baseline | DimensionType::Continued => {
            crate::data_structure::DimensionType::Linear
        }
        DimensionType::Aligned => crate::data_structure::DimensionType::Aligned,
        DimensionType::Angular => crate::data_structure::DimensionType::Angular,
        DimensionType::Radial => crate::data_structure::DimensionType::Radius,
        DimensionType::Diameter => crate::data_structure::DimensionType::Diameter,
        DimensionType::ArcLength => crate::data_structure::DimensionType::ArcLength,
        DimensionType::Ordinate => crate::data_structure::DimensionType::Ordinate,
    };
    EntityGeometry::Dimension {
        dim_type,
        measurement: geometry.measurement,
        text: geometry.text.clone(),
        text_position,
        text_height: geometry.style.text_height,
        text_rotation: geometry.text_rotation,
        definition_point: point_at(0),
        def_point_1: point_at(1),
        def_point_2: point_at(2),
        def_point_3: Point::origin(),
        def_point_4: Point::origin(),
        angle,
        extension_lines,
        center_marks,
    }
}

impl From<LinearDimension> for Entity {
    fn from(dim: LinearDimension) -> Self {
        let center_marks = dim.center_mark.is_some();
        let geometry = dim.geometry;
        Entity::new(
            EntityType::Dimension,
            entity_geometry_from_dimension(&geometry, text_position_for(&geometry), 0.0, true, center_marks),
        )
    }
}

impl From<AlignedDimension> for Entity {
    fn from(dim: AlignedDimension) -> Self {
        let geometry = dim.geometry;
        Entity::new(
            EntityType::Dimension,
            entity_geometry_from_dimension(&geometry, text_position_for(&geometry), 0.0, true, false),
        )
    }
}

impl From<AngularDimension> for Entity {
    fn from(dim: AngularDimension) -> Self {
        let text_location = dim.text_location;
        let angle = {
            let pts = &dim.geometry.definition_points;
            if pts.len() >= 3 {
                let center = pts[0];
                (pts[1] - center).to_vector2().angle() - (pts[2] - center).to_vector2().angle()
            } else {
                0.0
            }
        };
        let geometry = dim.geometry;
        Entity::new(
            EntityType::Dimension,
            entity_geometry_from_dimension(&geometry, text_location, angle, true, false),
        )
    }
}

impl From<RadialDimension> for Entity {
    fn from(dim: RadialDimension) -> Self {
        let arrow = dim.arrow;
        let geometry = dim.geometry;
        Entity::new(
            EntityType::Dimension,
            entity_geometry_from_dimension(&geometry, arrow, 0.0, true, true),
        )
    }
}

impl From<OrdinateDimension> for Entity {
    fn from(dim: OrdinateDimension) -> Self {
        let leader_point = dim.leader_point;
        let geometry = dim.geometry;
        Entity::new(
            EntityType::Dimension,
            entity_geometry_from_dimension(&geometry, leader_point, 0.0, false, false),
        )
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum DimensionGeometryData {
    Linear(LinearDimension),
    Aligned(AlignedDimension),
    Angular(AngularDimension),
    Radial(RadialDimension),
    Ordinate(OrdinateDimension),
}
