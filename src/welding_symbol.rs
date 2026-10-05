use crate::geometry::{Point, Vector2};
use serde::{Serialize, Deserialize};
use std::fmt;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WeldingSymbol {
    pub basic_symbol: BasicWeldSymbol,
    pub supplementary_symbol: Vec<SupplementarySymbol>,
    pub weld_detail: WeldDetail,
    pub finish: FinishSymbol,
    pub root: RootSymbol,
    pub contour: ContourSymbol,
    pub pitch: Option<f64>,
    pub length: Option<f64>,
    pub angle: Option<f64>,
    pub tail: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum BasicWeldSymbol {
    None,
    Groove,
    Fillet,
    Plug,
    Spot,
    Seam,
    Surfacing,
    Arc,
    Flash,
    Stud,
    Back,
    Gouging,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SupplementarySymbol {
    MeltRun,
    Staggered,
    WeldAllAround,
    FieldWeld,
    Convex,
    Concave,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum WeldDetail {
    SingleVGroove,
    DoubleVGroove,
    SingleUgroove,
    DoubleUGroove,
    SingleJGroove,
    DoubleJGroove,
    SquareGroove,
    FlareV,
    FlareBevel,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum FinishSymbol {
    None,
    Machined,
    Ground,
    Polished,
    Hammered,
    Rolled,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum RootSymbol {
    None,
    Flush,
    Convex,
    Open,
    Backing,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum ContourSymbol {
    None,
    Flat,
    Convex,
    Concave,
    FlatFilled,
    ConvexFilled,
    ConcaveFilled,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GrooveWeldSymbol {
    pub groove_type: GrooveType,
    pub size: Option<f64>,
    pub angle: Option<f64>,
    pub root_opening: Option<f64>,
    pub root_face: Option<f64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum GrooveType {
    SingleV,
    DoubleV,
    SingleU,
    DoubleU,
    SingleJ,
    DoubleJ,
    Square,
    SingleBevel,
    DoubleBevel,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FilletWeldSymbol {
    pub size: Option<f64>,
    pub length: Option<f64>,
    pub pitch: Option<f64>,
    pub intermittent: bool,
    pub chain_intermittent: bool,
    pub staggered_intermittent: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SpotWeldSymbol {
    pub size: Option<f64>,
    pub length: Option<f64>,
    pub pitch: Option<f64>,
    pub projection: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SeamWeldSymbol {
    pub size: Option<f64>,
    pub length: Option<f64>,
    pub pitch: Option<f64>,
    pub width: Option<f64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PlugWeldSymbol {
    pub size: Option<f64>,
    pub angle: Option<f64>,
    pub number: u32,
    pub spacing: Option<f64>,
}

impl WeldingSymbol {
    pub fn new() -> Self {
        Self {
            basic_symbol: BasicWeldSymbol::None,
            supplementary_symbol: Vec::new(),
            weld_detail: WeldDetail::SingleVGroove,
            finish: FinishSymbol::None,
            root: RootSymbol::None,
            contour: ContourSymbol::None,
            pitch: None,
            length: None,
            angle: None,
            tail: None,
        }
    }
    
    pub fn fillet() -> Self {
        Self {
            basic_symbol: BasicWeldSymbol::Fillet,
            ..Self::new()
        }
    }
    
    pub fn groove(groove_type: GrooveType) -> Self {
        Self {
            basic_symbol: BasicWeldSymbol::Groove,
            weld_detail: match groove_type {
                GrooveType::SingleV => WeldDetail::SingleVGroove,
                GrooveType::DoubleV => WeldDetail::DoubleVGroove,
                GrooveType::SingleU => WeldDetail::SingleUgroove,
                GrooveType::DoubleU => WeldDetail::DoubleUGroove,
                GrooveType::SingleJ => WeldDetail::SingleJGroove,
                GrooveType::DoubleJ => WeldDetail::DoubleJGroove,
                GrooveType::Square => WeldDetail::SquareGroove,
                GrooveType::SingleBevel => WeldDetail::SingleVGroove,
                GrooveType::DoubleBevel => WeldDetail::DoubleVGroove,
            },
            ..Self::new()
        }
    }
    
    pub fn spot() -> Self {
        Self {
            basic_symbol: BasicWeldSymbol::Spot,
            ..Self::new()
        }
    }
    
    pub fn seam() -> Self {
        Self {
            basic_symbol: BasicWeldSymbol::Seam,
            ..Self::new()
        }
    }
    
    pub fn stud() -> Self {
        Self {
            basic_symbol: BasicWeldSymbol::Stud,
            ..Self::new()
        }
    }
    
    pub fn with_size(mut self, size: f64) -> Self {
        self.weld_detail = match self.basic_symbol {
            BasicWeldSymbol::Fillet => WeldDetail::SingleVGroove,
            _ => self.weld_detail,
        };
        self
    }
    
    pub fn with_length(mut self, length: f64) -> Self {
        self.length = Some(length);
        self
    }
    
    pub fn with_pitch(mut self, pitch: f64) -> Self {
        self.pitch = Some(pitch);
        self
    }
    
    pub fn with_angle(mut self, angle: f64) -> Self {
        self.angle = Some(angle);
        self
    }
    
    pub fn with_supplementary(mut self, symbol: SupplementarySymbol) -> Self {
        self.supplementary_symbol.push(symbol);
        self
    }
    
    pub fn add_supplementary(&mut self, symbol: SupplementarySymbol) {
        self.supplementary_symbol.push(symbol);
    }
    
    pub fn with_finish(mut self, finish: FinishSymbol) -> Self {
        self.finish = finish;
        self
    }
    
    pub fn with_contour(mut self, contour: ContourSymbol) -> Self {
        self.contour = contour;
        self
    }
    
    pub fn with_root(mut self, root: RootSymbol) -> Self {
        self.root = root;
        self
    }
    
    pub fn all_around(mut self) -> Self {
        self.add_supplementary(SupplementarySymbol::WeldAllAround);
        self
    }
    
    pub fn field_weld(mut self) -> Self {
        self.add_supplementary(SupplementarySymbol::FieldWeld);
        self
    }
    
    pub fn staggered(mut self) -> Self {
        self.add_supplementary(SupplementarySymbol::Staggered);
        self
    }
    
    pub fn with_tail(mut self, text: String) -> Self {
        self.tail = Some(text);
        self
    }
    
    pub fn intermittent(mut self, is_chain: bool) -> Self {
        self
    }
    
    pub fn format_size(&self) -> String {
        if let Some(size) = self.get_size() {
            format!("{}", size)
        } else {
            String::new()
        }
    }
    
    pub fn get_size(&self) -> Option<f64> {
        match &self.weld_detail {
            WeldDetail::SingleVGroove => Some(6.0),
            WeldDetail::DoubleVGroove => Some(6.0),
            WeldDetail::SingleUgroove => Some(6.0),
            WeldDetail::DoubleUGroove => Some(6.0),
            WeldDetail::SingleJGroove => Some(6.0),
            WeldDetail::DoubleJGroove => Some(6.0),
            WeldDetail::SquareGroove => Some(3.0),
            WeldDetail::FlareV => Some(6.0),
            WeldDetail::FlareBevel => Some(6.0),
            _ => None,
        }
    }
}

impl Default for WeldingSymbol {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for WeldingSymbol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut result = String::new();
        
        match self.basic_symbol {
            BasicWeldSymbol::Fillet => result.push('△'),
            BasicWeldSymbol::Groove => result.push_str(&format!("{}", self.weld_detail)),
            BasicWeldSymbol::Spot => result.push_str("○"),
            BasicWeldSymbol::Seam => result.push_str("═"),
            BasicWeldSymbol::Plug => result.push_str("□"),
            BasicWeldSymbol::Stud => result.push_str("▣"),
            BasicWeldSymbol::Back => result.push_str("─"),
            _ => {}
        }
        
        for symbol in &self.supplementary_symbol {
            match symbol {
                SupplementarySymbol::WeldAllAround => result.push('○'),
                SupplementarySymbol::FieldWeld => result.push('▲'),
                SupplementarySymbol::Staggered => result.push('↔'),
                SupplementarySymbol::MeltRun => result.push('≡'),
                SupplementarySymbol::Convex => result.push('∩'),
                SupplementarySymbol::Concave => result.push('∪'),
            }
        }
        
        if let Some(size) = self.get_size() {
            result.push_str(&format!("({})", size));
        }
        
        write!(f, "{}", result)
    }
}

impl fmt::Display for WeldDetail {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            WeldDetail::SingleVGroove => write!(f, "V"),
            WeldDetail::DoubleVGroove => write!(f, "XV"),
            WeldDetail::SingleUgroove => write!(f, "U"),
            WeldDetail::DoubleUGroove => write!(f, "XU"),
            WeldDetail::SingleJGroove => write!(f, "J"),
            WeldDetail::DoubleJGroove => write!(f, "XJ"),
            WeldDetail::SquareGroove => write!(f, "I"),
            WeldDetail::FlareV => write!(f, "V"),
            WeldDetail::FlareBevel => write!(f, "L"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WeldingSymbolPlacement {
    pub symbol: WeldingSymbol,
    pub location: Point,
    pub rotation: f64,
    pub scale: f64,
    pub side: WeldSide,
    pub reference_line: WeldReferenceLine,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum WeldSide {
    Above,
    Below,
    Both,
    Entire,
    Around,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WeldReferenceLine {
    pub start_point: Point,
    pub end_point: Point,
    pub arrow_line: Line,
    pub reference_line: Line,
    pub has_tail: bool,
}

impl WeldReferenceLine {
    pub fn new(start: Point, end: Point, arrow_side: WeldSide) -> Self {
        let direction = (end.to_vector2() - start.to_vector2()).normalize();
        let perpendicular = Vector2::new(-direction.y, direction.x);
        
        let arrow_line = Line::new(
            start,
            Point::new(start.x + direction.x * 20.0, start.y + direction.y * 20.0, 0.0),
        );
        
        let offset = 10.0;
        let reference_start = Point::new(
            start.x + perpendicular.x * offset,
            start.y + perpendicular.y * offset,
            0.0,
        );
        let reference_end = Point::new(
            end.x + perpendicular.x * offset,
            end.y + perpendicular.y * offset,
            0.0,
        );
        
        let reference_line = Line::new(reference_start, reference_end);
        
        Self {
            start_point: start,
            end_point: end,
            arrow_line,
            reference_line,
            has_tail: false,
        }
    }
    
    pub fn with_tail(mut self, tail_text: String) -> Self {
        self.has_tail = true;
        self
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct Line {
    start: Point,
    end: Point,
}

impl Line {
    fn new(start: Point, end: Point) -> Self {
        Self { start, end }
    }
}

