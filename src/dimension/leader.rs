use crate::geometry::{Point, Vector2, Line, Arc, Polyline};
use crate::data_structure::{Entity, EntityType, EntityGeometry, TextStyle};
use crate::geometric_tolerance::GeometricTolerance;
use serde::{Serialize, Deserialize};
use std::fmt;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Leader {
    pub start_point: Point,
    pub landing: LeaderLanding,
    pub content: LeaderContent,
    pub style: LeaderStyle,
    pub annotation: Option<Annotation>,
    pub is_mleader: bool,
    pub dogleg: DoglegSettings,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LeaderLanding {
    pub landing_point: Point,
    pub landing_length: f64,
    pub has_landing: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum LeaderContent {
    None,
    Text(TextContent),
    Block(BlockContent),
    Tolerance(GeometricTolerance),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TextContent {
    pub text: String,
    pub style: TextStyle,
    pub width_factor: f64,
    pub is_mtext: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BlockContent {
    pub block_name: String,
    pub scale: f64,
    pub rotation: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LeaderStyle {
    pub id: String,
    pub name: String,
    pub arrow_size: f64,
    pub arrow_style: LeaderArrowStyle,
    pub leader_line_type: LeaderLineType,
    pub leader_line_weight: f64,
    pub landing_gap: f64,
    pub landing_distance: f64,
    pub text_height: f64,
    pub text_style: TextStyle,
    pub first_segment_angle: f64,
    pub second_segment_angle: f64,
    pub dogleg_length: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum LeaderArrowStyle {
    Closed,
    ClosedFilled,
    Dot,
    DotSmall,
    Open,
    Origin,
    Origin02,
    Oblique,
    Box,
    BoxFilled,
    Circle,
    CircleFilled,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum LeaderLineType {
    Straight,
    Splined,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DoglegSettings {
    pub enabled: bool,
    pub direction: DoglegDirection,
    pub length: f64,
    pub first_segment_angle: f64,
    pub second_segment_angle: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum DoglegDirection {
    Left,
    Right,
    Automatic,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Annotation {
    pub content: AnnotationContent,
    pub position: Point,
    pub attachment_point: AnnotationAttachment,
    pub text_direction: TextDirection,
    pub line_rotation: f64,
    pub dogleg_position: Point,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum AnnotationAttachment {
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
pub enum AnnotationContent {
    Text(String),
    MText(String),
    Block(String, f64),
    Tolerance,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum TextDirection {
    LeftToRight,
    RightToLeft,
}

impl Default for LeaderStyle {
    fn default() -> Self {
        Self {
            id: "Standard".to_string(),
            name: "Standard".to_string(),
            arrow_size: 2.5,
            arrow_style: LeaderArrowStyle::ClosedFilled,
            leader_line_type: LeaderLineType::Straight,
            leader_line_weight: 0.0,
            landing_gap: 0.625,
            landing_distance: 6.35,
            text_height: 2.5,
            text_style: TextStyle::default(),
            first_segment_angle: 0.0,
            second_segment_angle: 90.0,
            dogleg_length: 6.35,
        }
    }
}

impl Default for LeaderLanding {
    fn default() -> Self {
        Self {
            landing_point: Point::origin(),
            landing_length: 6.35,
            has_landing: true,
        }
    }
}

impl Default for TextContent {
    fn default() -> Self {
        Self {
            text: String::new(),
            style: TextStyle::default(),
            width_factor: 1.0,
            is_mtext: false,
        }
    }
}

impl Leader {
    pub fn new(start_point: Point, content: LeaderContent, style: LeaderStyle) -> Self {
        let dogleg_length = style.dogleg_length;
        let first_segment_angle = style.first_segment_angle;
        let second_segment_angle = style.second_segment_angle;
        
        Self {
            start_point,
            landing: LeaderLanding::default(),
            content,
            style,
            annotation: None,
            is_mleader: false,
            dogleg: DoglegSettings {
                enabled: true,
                direction: DoglegDirection::Automatic,
                length: dogleg_length,
                first_segment_angle,
                second_segment_angle,
            },
        }
    }
    
    pub fn add_segment(&mut self, point: Point) {
        match &mut self.content {
            LeaderContent::Text(text_content) => {
                text_content.text.push('\n');
                text_content.text.push_str(&format!("-> ({:.1}, {:.1})", point.x, point.y));
            }
            _ => {}
        }
    }
    
    pub fn set_landing(&mut self, landing_point: Point) {
        self.landing.landing_point = landing_point;
        self.landing.has_landing = true;
    }
    
    pub fn remove_landing(&mut self) {
        self.landing.has_landing = false;
    }
    
    pub fn flip_dogleg(&mut self) {
        self.dogleg.direction = match self.dogleg.direction {
            DoglegDirection::Left => DoglegDirection::Right,
            DoglegDirection::Right => DoglegDirection::Left,
            DoglegDirection::Automatic => DoglegDirection::Left,
        };
    }
    
    pub fn set_annotation(&mut self, annotation: Annotation) {
        self.annotation = Some(annotation);
    }
    
    pub fn to_entity(&self) -> Entity {
        let text = match &self.content {
            LeaderContent::Text(text_content) => text_content.text.clone(),
            LeaderContent::Block(block_content) => block_content.block_name.clone(),
            LeaderContent::Tolerance(_) => "Tolerance".to_string(),
            LeaderContent::None => String::new(),
        };
        Entity::new(
            EntityType::Dimension,
            EntityGeometry::Dimension {
                dim_type: crate::data_structure::DimensionType::Linear,
                measurement: 0.0,
                text,
                text_position: self.landing.landing_point,
                text_height: self.style.text_height,
                text_rotation: 0.0,
                definition_point: self.start_point,
                def_point_1: self.landing.landing_point,
                def_point_2: Point::origin(),
                def_point_3: Point::origin(),
                def_point_4: Point::origin(),
                angle: 0.0,
                extension_lines: false,
                center_marks: false,
            },
        )
    }
}

impl From<Leader> for Entity {
    fn from(leader: Leader) -> Self {
        leader.to_entity()
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MultiLeader {
    pub leaders: Vec<Leader>,
    pub common_content: LeaderContent,
    pub style: LeaderStyle,
    pub overall_scale: f64,
    pub landing_alignment: LandingAlignment,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum LandingAlignment {
    AlignAll,
    AlignFirst,
    Distribute,
}

impl MultiLeader {
    pub fn new(style: LeaderStyle, content: LeaderContent) -> Self {
        Self {
            leaders: Vec::new(),
            common_content: content,
            style,
            overall_scale: 1.0,
            landing_alignment: LandingAlignment::AlignAll,
        }
    }
    
    pub fn add_leader(&mut self, start_point: Point) -> &mut Leader {
        let leader = Leader::new(start_point, self.common_content.clone(), self.style.clone());
        self.leaders.push(leader);
        self.leaders.last_mut().unwrap()
    }
    
    pub fn remove_leader(&mut self, index: usize) -> bool {
        if index < self.leaders.len() {
            self.leaders.remove(index);
            true
        } else {
            false
        }
    }
    
    pub fn set_overall_scale(&mut self, scale: f64) {
        self.overall_scale = scale;
        for leader in &mut self.leaders {
            leader.style.arrow_size *= scale;
            leader.style.landing_gap *= scale;
            leader.style.landing_distance *= scale;
            leader.style.text_height *= scale;
            leader.style.dogleg_length *= scale;
        }
    }
    
    pub fn align_landings(&mut self) {
        if self.leaders.is_empty() {
            return;
        }
        
        match self.landing_alignment {
            LandingAlignment::AlignFirst => {
                let first_landing = self.leaders[0].landing.landing_point;
                for leader in &mut self.leaders[1..] {
                    leader.landing.landing_point = first_landing;
                }
            }
            LandingAlignment::Distribute => {
                let min_y = self.leaders.iter()
                    .filter_map(|l| Some(l.landing.landing_point.y))
                    .fold(f64::MAX, f64::min);
                
                for leader in &mut self.leaders {
                    leader.landing.landing_point.y = min_y;
                }
            }
            _ => {}
        }
    }
    
    pub fn to_entities(&self) -> Vec<Entity> {
        self.leaders.iter()
            .map(|l| l.to_entity())
            .collect()
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LeaderAssociation {
    pub leader_id: usize,
    pub arrow_point: Point,
    pub target_point: Point,
    pub dogleg_point: Option<Point>,
}

impl LeaderAssociation {
    pub fn new(leader_id: usize, arrow_point: Point, target_point: Point) -> Self {
        Self {
            leader_id,
            arrow_point,
            target_point,
            dogleg_point: None,
        }
    }
    
    pub fn with_dogleg(mut self, dogleg_point: Point) -> Self {
        self.dogleg_point = Some(dogleg_point);
        self
    }
}
