use crate::geometry::{Point, Vector2, Line};
use crate::data_structure::{Entity, EntityType, EntityGeometry};
use super::linear::{DimensionGeometry, DimensionStyle, DimensionType};
use serde::{Serialize, Deserialize};
use std::fmt;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OrdinateDimension {
    pub geometry: DimensionGeometry,
    pub feature_point: Point,
    pub leader_point: Point,
    pub dimension_line: Line,
    pub orientation: OrdinateOrientation,
    pub use_x_axis: bool,
    pub use_y_axis: bool,
    pub baseline_origin: Option<Point>,
    pub is_baseline: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum OrdinateOrientation {
    X,
    Y,
    Horizontal,
    Vertical,
}

impl OrdinateDimension {
    pub fn new_x(
        feature_point: Point,
        leader_point: Point,
        style: DimensionStyle,
        baseline_origin: Option<Point>,
    ) -> Self {
        let mut geometry = DimensionGeometry::new(DimensionType::Ordinate, style.clone());
        geometry.definition_points = vec![feature_point, leader_point];
        
        geometry.measurement = feature_point.x;
        geometry.actual_measurement = geometry.measurement;
        geometry.update_text();
        
        let direction = (leader_point.to_vector2() - feature_point.to_vector2()).normalize();
        
        let dimension_line = Line::new(feature_point, leader_point);
        
        Self {
            geometry,
            feature_point,
            leader_point,
            dimension_line,
            orientation: OrdinateOrientation::X,
            use_x_axis: true,
            use_y_axis: false,
            baseline_origin,
            is_baseline: false,
        }
    }
    
    pub fn new_y(
        feature_point: Point,
        leader_point: Point,
        style: DimensionStyle,
        baseline_origin: Option<Point>,
    ) -> Self {
        let mut geometry = DimensionGeometry::new(DimensionType::Ordinate, style.clone());
        geometry.definition_points = vec![feature_point, leader_point];
        
        geometry.measurement = feature_point.y;
        geometry.actual_measurement = geometry.measurement;
        geometry.update_text();
        
        let direction = (leader_point.to_vector2() - feature_point.to_vector2()).normalize();
        
        let dimension_line = Line::new(feature_point, leader_point);
        
        Self {
            geometry,
            feature_point,
            leader_point,
            dimension_line,
            orientation: OrdinateOrientation::Y,
            use_x_axis: false,
            use_y_axis: true,
            baseline_origin,
            is_baseline: false,
        }
    }
    
    pub fn new_horizontal(
        feature_point: Point,
        leader_point: Point,
        style: DimensionStyle,
    ) -> Self {
        let mut geometry = DimensionGeometry::new(DimensionType::Ordinate, style.clone());
        geometry.definition_points = vec![feature_point, leader_point];
        
        geometry.measurement = feature_point.x;
        geometry.actual_measurement = geometry.measurement;
        geometry.update_text();
        
        let dimension_line = Line::new(feature_point, leader_point);
        
        Self {
            geometry,
            feature_point,
            leader_point,
            dimension_line,
            orientation: OrdinateOrientation::Horizontal,
            use_x_axis: true,
            use_y_axis: false,
            baseline_origin: None,
            is_baseline: false,
        }
    }
    
    pub fn new_vertical(
        feature_point: Point,
        leader_point: Point,
        style: DimensionStyle,
    ) -> Self {
        let mut geometry = DimensionGeometry::new(DimensionType::Ordinate, style.clone());
        geometry.definition_points = vec![feature_point, leader_point];
        
        geometry.measurement = feature_point.y;
        geometry.actual_measurement = geometry.measurement;
        geometry.update_text();
        
        let dimension_line = Line::new(feature_point, leader_point);
        
        Self {
            geometry,
            feature_point,
            leader_point,
            dimension_line,
            orientation: OrdinateOrientation::Vertical,
            use_x_axis: false,
            use_y_axis: true,
            baseline_origin: None,
            is_baseline: false,
        }
    }
    
    pub fn from_baseline(
        baseline_origin: Point,
        feature_points: Vec<Point>,
        style: DimensionStyle,
        orientation: OrdinateOrientation,
    ) -> Vec<Self> {
        let mut dimensions = Vec::new();
        
        for (i, feature_point) in feature_points.iter().enumerate() {
            let is_baseline = i == 0;
            
            let mut geometry = DimensionGeometry::new(DimensionType::Ordinate, style.clone());
            geometry.definition_points = vec![*feature_point];
            
            match orientation {
                OrdinateOrientation::X | OrdinateOrientation::Horizontal => {
                    geometry.measurement = feature_point.x - baseline_origin.x;
                }
                OrdinateOrientation::Y | OrdinateOrientation::Vertical => {
                    geometry.measurement = feature_point.y - baseline_origin.y;
                }
            }
            
            geometry.actual_measurement = geometry.measurement;
            geometry.update_text();
            
            let offset = (i as f64 + 1.0) * style.extension_line_offset * 3.0;
            
            let direction = match orientation {
                OrdinateOrientation::X | OrdinateOrientation::Horizontal => Vector2::new(0.0, 1.0),
                _ => Vector2::new(1.0, 0.0),
            };
            
            let leader_point = Point::new(
                feature_point.x + direction.x * offset,
                feature_point.y + direction.y * offset,
                0.0,
            );
            
            let dimension_line = Line::new(*feature_point, leader_point);
            
            dimensions.push(Self {
                geometry,
                feature_point: *feature_point,
                leader_point,
                dimension_line,
                orientation,
                use_x_axis: matches!(orientation, OrdinateOrientation::X | OrdinateOrientation::Horizontal),
                use_y_axis: matches!(orientation, OrdinateOrientation::Y | OrdinateOrientation::Vertical),
                baseline_origin: Some(baseline_origin),
                is_baseline,
            });
        }
        
        dimensions
    }
    
    pub fn flip(&mut self) {
        std::mem::swap(&mut self.feature_point, &mut self.leader_point);
        self.geometry.calculate_measurement();
    }
    
    pub fn set_feature_point(&mut self, point: Point) {
        self.feature_point = point;
        self.geometry.definition_points[0] = point;
        match self.orientation {
            OrdinateOrientation::X | OrdinateOrientation::Horizontal => {
                self.geometry.measurement = point.x;
            }
            _ => {
                self.geometry.measurement = point.y;
            }
        }
        self.geometry.actual_measurement = self.geometry.measurement;
        self.geometry.update_text();
    }
    
    pub fn set_leader_point(&mut self, point: Point) {
        self.leader_point = point;
        self.geometry.definition_points[1] = point;
        self.dimension_line = Line::new(self.feature_point, point);
    }
}

impl From<OrdinateDimension> for Entity {
    fn from(dim: OrdinateDimension) -> Self {
        let leader_point = dim.leader_point;
        let geometry = dim.geometry;
        Entity::new(
            EntityType::Dimension,
            super::linear::entity_geometry_from_dimension(&geometry, leader_point, 0.0, false, false),
        )
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OrdinateDimensionSet {
    pub x_dimensions: Vec<OrdinateDimension>,
    pub y_dimensions: Vec<OrdinateDimension>,
    pub baseline_origin: Point,
    pub style: DimensionStyle,
}

impl OrdinateDimensionSet {
    pub fn new(
        origin: Point,
        feature_points: Vec<Point>,
        style: DimensionStyle,
    ) -> Self {
        let x_points: Vec<Point> = feature_points.iter().map(|p| Point::new(p.x, origin.y, p.z)).collect();
        let y_points: Vec<Point> = feature_points.iter().map(|p| Point::new(origin.x, p.y, p.z)).collect();
        
        let x_dimensions = OrdinateDimension::from_baseline(
            origin,
            x_points,
            style.clone(),
            OrdinateOrientation::Horizontal,
        );
        let y_dimensions = OrdinateDimension::from_baseline(
            origin,
            y_points,
            style.clone(),
            OrdinateOrientation::Vertical,
        );
        
        Self {
            x_dimensions,
            y_dimensions,
            baseline_origin: origin,
            style,
        }
    }
    
    pub fn add_feature_point(&mut self, point: Point) {
        let x_point = Point::new(point.x, self.baseline_origin.y, point.z);
        let y_point = Point::new(self.baseline_origin.x, point.y, point.z);
        
        self.x_dimensions.extend(OrdinateDimension::from_baseline(
            self.baseline_origin,
            vec![x_point],
            self.style.clone(),
            OrdinateOrientation::Horizontal,
        ));
        
        self.y_dimensions.extend(OrdinateDimension::from_baseline(
            self.baseline_origin,
            vec![y_point],
            self.style.clone(),
            OrdinateOrientation::Vertical,
        ));
    }
    
    pub fn to_entities(&self) -> Vec<Entity> {
        let mut entities = Vec::new();
        
        for dim in &self.x_dimensions {
            entities.push(dim.clone().into());
        }
        
        for dim in &self.y_dimensions {
            entities.push(dim.clone().into());
        }
        
        entities
    }
}
