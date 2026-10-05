use crate::geometry::Point;
use serde::{Serialize, Deserialize};
use std::fmt;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GeometricTolerance {
    pub feature_control_frame: FeatureControlFrame,
    pub placement: TolerancePlacement,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FeatureControlFrame {
    pub frames: Vec<ToleranceFrame>,
    pub composite_frame: Option<CompositeFrame>,
    pub material_condition: MaterialCondition,
    pub datum_reference_frame: Vec<DatumReference>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ToleranceFrame {
    pub symbol: ToleranceSymbol,
    pub tolerance_value: f64,
    pub datum_references: Vec<DatumReference>,
    pub modifier: ToleranceModifier,
    pub secondary_tolerance: Option<f64>,
    pub projected_tolerance: Option<ProjectedTolerance>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CompositeFrame {
    pub primary_tolerance: ToleranceFrame,
    pub secondary_tolerance: ToleranceFrame,
    pub position_symbol: ToleranceSymbol,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ToleranceSymbol {
    Straightness,
    Flatness,
    Circularity,
    Cylindricity,
    ProfileOfLine,
    ProfileOfSurface,
    Angularity,
    Perpendicularity,
    Parallelism,
    Position,
    Symmetry,
    Concentricity,
    CircularRunout,
    TotalRunout,
}

impl fmt::Display for ToleranceSymbol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ToleranceSymbol::Straightness => write!(f, "⌖"),
            ToleranceSymbol::Flatness => write!(f, "⏥"),
            ToleranceSymbol::Circularity => write!(f, "○"),
            ToleranceSymbol::Cylindricity => write!(f, "⏦"),
            ToleranceSymbol::ProfileOfLine => write!(f, "⌒"),
            ToleranceSymbol::ProfileOfSurface => write!(f, "⌔"),
            ToleranceSymbol::Angularity => write!(f, "∠"),
            ToleranceSymbol::Perpendicularity => write!(f, "⊥"),
            ToleranceSymbol::Parallelism => write!(f, "∥"),
            ToleranceSymbol::Position => write!(f, "Ⓟ"),
            ToleranceSymbol::Symmetry => write!(f, "⌖"),
            ToleranceSymbol::Concentricity => write!(f, "◎"),
            ToleranceSymbol::CircularRunout => write!(f, "⇵"),
            ToleranceSymbol::TotalRunout => write!(f, "⬒"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DatumReference {
    pub datum: String,
    pub modifier: DatumModifier,
    pub material_condition: MaterialCondition,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum DatumModifier {
    None,
    MaximumMaterialCondition,
    LeastMaterialCondition,
    RegardlessOfFeatureSize,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum MaterialCondition {
    None,
    MaximumMaterialCondition,
    LeastMaterialCondition,
    RegardlessOfFeatureSize,
}

impl Default for MaterialCondition {
    fn default() -> Self {
        MaterialCondition::None
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum ToleranceModifier {
    None,
    MaximumMaterialCondition,
    LeastMaterialCondition,
    ProjectedTolerance,
    FreeState,
    Envelope,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProjectedTolerance {
    pub zone_height: f64,
    pub zone_diameter: Option<f64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TolerancePlacement {
    OnFeature,
    Projected,
    RegardlessOfFeatureSize,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ToleranceValue {
    pub diameter_symbol: bool,
    pub value: f64,
    pub tolerance_zone_shape: ToleranceZoneShape,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum ToleranceZoneShape {
    Cylinder,
    Sphere,
    Circle,
    BetweenTwoLines,
    BetweenTwoPlanes,
    Polygon,
}

impl GeometricTolerance {
    pub fn new(symbol: ToleranceSymbol, value: f64) -> Self {
        let frame = ToleranceFrame {
            symbol,
            tolerance_value: value,
            datum_references: Vec::new(),
            modifier: ToleranceModifier::None,
            secondary_tolerance: None,
            projected_tolerance: None,
        };
        
        let control_frame = FeatureControlFrame {
            frames: vec![frame],
            composite_frame: None,
            material_condition: MaterialCondition::None,
            datum_reference_frame: Vec::new(),
        };
        
        Self {
            feature_control_frame: control_frame,
            placement: TolerancePlacement::OnFeature,
        }
    }
    
    pub fn add_datum(&mut self, datum: String, modifier: DatumModifier) {
        let reference = DatumReference {
            datum,
            modifier,
            material_condition: self.feature_control_frame.material_condition,
        };
        
        if let Some(frame) = self.feature_control_frame.frames.first_mut() {
            frame.datum_references.push(reference.clone());
        }
        
        self.feature_control_frame.datum_reference_frame.push(reference.clone());
    }
    
    pub fn set_material_condition(&mut self, condition: MaterialCondition) {
        self.feature_control_frame.material_condition = condition;
    }
    
    pub fn set_composite(&mut self, primary_value: f64, secondary_value: f64, datum_references: Vec<DatumReference>) {
        let primary_frame = ToleranceFrame {
            symbol: ToleranceSymbol::Position,
            tolerance_value: primary_value,
            datum_references: datum_references.clone(),
            modifier: ToleranceModifier::None,
            secondary_tolerance: None,
            projected_tolerance: None,
        };
        
        let secondary_frame = ToleranceFrame {
            symbol: ToleranceSymbol::Position,
            tolerance_value: secondary_value,
            datum_references,
            modifier: ToleranceModifier::None,
            secondary_tolerance: None,
            projected_tolerance: None,
        };
        
        self.feature_control_frame.composite_frame = Some(CompositeFrame {
            primary_tolerance: primary_frame,
            secondary_tolerance: secondary_frame,
            position_symbol: ToleranceSymbol::Position,
        });
    }
    
    pub fn add_projected_tolerance(&mut self, height: f64) {
        if let Some(frame) = self.feature_control_frame.frames.first_mut() {
            frame.projected_tolerance = Some(ProjectedTolerance {
                zone_height: height,
                zone_diameter: None,
            });
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DatumTarget {
    pub target_point: Option<Point>,
    pub target_line: Option<(Point, Point)>,
    pub target_area: Option<(Point, f64)>,
    pub target_type: DatumTargetType,
    pub identifier: String,
    pub size: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum DatumTargetType {
    Point,
    Line,
    Area,
}

impl DatumTarget {
    pub fn new_point(identifier: String, point: Point) -> Self {
        Self {
            target_point: Some(point),
            target_line: None,
            target_area: None,
            target_type: DatumTargetType::Point,
            identifier,
            size: 3.0,
        }
    }
    
    pub fn new_line(identifier: String, p1: Point, p2: Point) -> Self {
        Self {
            target_point: None,
            target_line: Some((p1, p2)),
            target_area: None,
            target_type: DatumTargetType::Line,
            identifier,
            size: 3.0,
        }
    }
    
    pub fn new_area(identifier: String, center: Point, diameter: f64) -> Self {
        Self {
            target_point: None,
            target_line: None,
            target_area: Some((center, diameter)),
            target_type: DatumTargetType::Area,
            identifier,
            size: diameter,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DatumReferenceFrame {
    pub designation: Vec<Datum>,
    pub order: usize,
    pub material_condition: MaterialCondition,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Datum {
    pub identifier: String,
    pub type_id: String,
    pub targets: Vec<DatumTarget>,
    pub material_condition: MaterialCondition,
    pub shift: Option<(f64, f64, f64)>,
}

impl Datum {
    pub fn new(identifier: String) -> Self {
        Self {
            identifier,
            type_id: String::new(),
            targets: Vec::new(),
            material_condition: MaterialCondition::None,
            shift: None,
        }
    }
    
    pub fn add_target(&mut self, target: DatumTarget) {
        self.targets.push(target);
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DatumSystem {
    pub primary_datum: Datum,
    pub secondary_datum: Option<Datum>,
    pub tertiary_datum: Option<Datum>,
    pub references: Vec<DatumReference>,
}

impl DatumSystem {
    pub fn new(primary: Datum) -> Self {
        Self {
            primary_datum: primary,
            secondary_datum: None,
            tertiary_datum: None,
            references: Vec::new(),
        }
    }
    
    pub fn add_secondary(&mut self, datum: Datum) {
        self.secondary_datum = Some(datum);
    }
    
    pub fn add_tertiary(&mut self, datum: Datum) {
        self.tertiary_datum = Some(datum);
    }
    
    pub fn build_references(&mut self) {
        self.references.clear();
        
        {
            let datum = &self.primary_datum;
            self.references.push(DatumReference {
                datum: datum.identifier.clone(),
                modifier: DatumModifier::None,
                material_condition: datum.material_condition,
            });
        }
        
        if let Some(datum) = &self.secondary_datum {
            self.references.push(DatumReference {
                datum: datum.identifier.clone(),
                modifier: DatumModifier::None,
                material_condition: datum.material_condition,
            });
        }
        
        if let Some(datum) = &self.tertiary_datum {
            self.references.push(DatumReference {
                datum: datum.identifier.clone(),
                modifier: DatumModifier::None,
                material_condition: datum.material_condition,
            });
        }
    }
}

impl From<GeometricTolerance> for crate::data_structure::Entity {
    fn from(tol: GeometricTolerance) -> Self {
        let content = tol
            .feature_control_frame
            .frames
            .first()
            .map(|frame| format!("{} {:.3}", frame.symbol, frame.tolerance_value))
            .unwrap_or_default();
        crate::data_structure::Entity::new(
            crate::data_structure::EntityType::Text,
            crate::data_structure::EntityGeometry::Text {
                content,
                position: crate::geometry::Point::origin(),
                height: 2.5,
                rotation: 0.0,
                width_factor: 1.0,
                font_name: "Standard".to_string(),
                style: crate::data_structure::TextStyle::default(),
            },
        )
    }
}
