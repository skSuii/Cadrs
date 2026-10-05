//! 焊接符号（Welding Symbol）：按 GB/T 324、AWS A2.4 的语义描述焊缝标注的组成。
//!
//! - `BasicWeldSymbol`：基本符号，决定焊缝类型（角焊缝、坡口焊、点焊、塞焊等）；
//! - `SupplementarySymbol`：辅助符号，如周焊、现场焊、交错焊、凸/凹面；
//! - `WeldDetail` 给出坡口细节，`FinishSymbol` / `RootSymbol` / `ContourSymbol`
//!   分别描述表面加工、根部与焊缝轮廓；
//! - `pitch` / `length` / `angle` 为可选的间距、长度与角度，`None` 表示未指定；
//! - `WeldingSymbolPlacement` 配合 `WeldReferenceLine` 把符号摆放到图纸上（箭头线 + 基准线）。
//!
//! 长度使用世界坐标单位；角度按原样保存，本模块不做角度/弧度换算，调用方需自行与图纸约定一致。
//! 各类型的 `Display` 实现输出用于预览的 Unicode/ASCII 串，不参与几何计算。

use crate::geometry::{Point, Vector2};
use serde::{Serialize, Deserialize};
use std::fmt;

/// 焊接符号：描述一条焊缝标注的完整内容。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WeldingSymbol {
    /// 基本符号，决定焊缝类型；默认 `None`。
    pub basic_symbol: BasicWeldSymbol,
    /// 辅助符号列表，按加入顺序叠加；可重复添加，本模块不去重。
    pub supplementary_symbol: Vec<SupplementarySymbol>,
    /// 焊缝/坡口细节；默认单面 V 形坡口。
    pub weld_detail: WeldDetail,
    /// 表面加工符号。
    pub finish: FinishSymbol,
    /// 根部符号。
    pub root: RootSymbol,
    /// 焊缝轮廓符号。
    pub contour: ContourSymbol,
    /// 断续焊缝间距，`None` 表示未指定。
    pub pitch: Option<f64>,
    /// 焊缝长度，`None` 表示未指定。
    pub length: Option<f64>,
    /// 焊缝角度，`None` 表示未指定；单位由调用方约定，本模块不换算。
    pub angle: Option<f64>,
    /// 尾部注释（工艺、标准号等），`None` 表示无尾注。
    pub tail: Option<String>,
}

/// 焊缝基本符号。
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum BasicWeldSymbol {
    /// 未指定。
    None,
    /// 坡口焊缝，具体坡口形式由 `WeldDetail` 决定。
    Groove,
    /// 角焊缝。
    Fillet,
    /// 塞焊缝。
    Plug,
    /// 点焊缝。
    Spot,
    /// 缝焊缝。
    Seam,
    /// 堆焊。
    Surfacing,
    /// 电弧焊。
    Arc,
    /// 闪光焊。
    Flash,
    /// 螺柱焊。
    Stud,
    /// 背面焊缝。
    Back,
    /// 气刨（碳弧气刨）焊缝。
    Gouging,
}

/// 辅助符号。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SupplementarySymbol {
    /// 熔透焊（熔化焊道）。
    MeltRun,
    /// 交错断续焊。
    Staggered,
    /// 周圈焊（沿工件一周）。
    WeldAllAround,
    /// 现场焊接。
    FieldWeld,
    /// 凸面焊缝。
    Convex,
    /// 凹面焊缝。
    Concave,
}

/// 焊缝/坡口细节形式。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum WeldDetail {
    /// 单面 V 形坡口。
    SingleVGroove,
    /// 双面 V 形坡口。
    DoubleVGroove,
    /// 单面 U 形坡口。
    SingleUgroove,
    /// 双面 U 形坡口。
    DoubleUGroove,
    /// 单面 J 形坡口。
    SingleJGroove,
    /// 双面 J 形坡口。
    DoubleJGroove,
    /// I 形（直边）坡口。
    SquareGroove,
    /// 卷边 V 形坡口。
    FlareV,
    /// 卷边斜角坡口。
    FlareBevel,
}

/// 焊缝表面加工符号。
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum FinishSymbol {
    /// 未指定。
    None,
    /// 机械加工。
    Machined,
    /// 磨削。
    Ground,
    /// 抛光。
    Polished,
    /// 锤击。
    Hammered,
    /// 轧制。
    Rolled,
}

/// 焊缝根部符号。
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum RootSymbol {
    /// 未指定。
    None,
    /// 根部齐平。
    Flush,
    /// 根部凸出。
    Convex,
    /// 根部开口（留间隙）。
    Open,
    /// 带垫板。
    Backing,
}

/// 焊缝轮廓符号。
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum ContourSymbol {
    /// 未指定。
    None,
    /// 平轮廓。
    Flat,
    /// 凸轮廓。
    Convex,
    /// 凹轮廓。
    Concave,
    /// 平轮廓（填充）。
    FlatFilled,
    /// 凸轮廓（填充）。
    ConvexFilled,
    /// 凹轮廓（填充）。
    ConcaveFilled,
}

/// 坡口焊缝的几何参数。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GrooveWeldSymbol {
    /// 坡口形式。
    pub groove_type: GrooveType,
    /// 焊缝尺寸（焊脚或熔深），世界坐标单位；`None` 表示未指定。
    pub size: Option<f64>,
    /// 坡口角度；单位由调用方约定。
    pub angle: Option<f64>,
    /// 根部间隙，世界坐标单位。
    pub root_opening: Option<f64>,
    /// 钝边高度，世界坐标单位。
    pub root_face: Option<f64>,
}

/// 坡口形式。
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum GrooveType {
    /// 单面 V 形。
    SingleV,
    /// 双面 V 形。
    DoubleV,
    /// 单面 U 形。
    SingleU,
    /// 双面 U 形。
    DoubleU,
    /// 单面 J 形。
    SingleJ,
    /// 双面 J 形。
    DoubleJ,
    /// I 形（直边）。
    Square,
    /// 单面斜角。
    SingleBevel,
    /// 双面斜角。
    DoubleBevel,
}

/// 角焊缝符号的参数（尺寸、长度、间距与断续方式）。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FilletWeldSymbol {
    /// 焊脚尺寸，世界坐标单位；`None` 表示未指定。
    pub size: Option<f64>,
    /// 焊缝长度，世界坐标单位。
    pub length: Option<f64>,
    /// 断续焊缝间距，世界坐标单位。
    pub pitch: Option<f64>,
    /// 是否为断续焊。
    pub intermittent: bool,
    /// 断续焊是否为链式（串联）布置。
    pub chain_intermittent: bool,
    /// 断续焊是否为交错布置。
    pub staggered_intermittent: bool,
}

/// 点焊缝符号的参数。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SpotWeldSymbol {
    /// 焊点直径或当量尺寸，世界坐标单位。
    pub size: Option<f64>,
    /// 焊缝长度，世界坐标单位。
    pub length: Option<f64>,
    /// 焊点间距，世界坐标单位。
    pub pitch: Option<f64>,
    /// 是否为凸焊（投影焊）。
    pub projection: bool,
}

/// 缝焊缝符号的参数。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SeamWeldSymbol {
    /// 焊缝尺寸，世界坐标单位。
    pub size: Option<f64>,
    /// 焊缝长度，世界坐标单位。
    pub length: Option<f64>,
    /// 焊缝间距，世界坐标单位。
    pub pitch: Option<f64>,
    /// 焊缝宽度，世界坐标单位。
    pub width: Option<f64>,
}

/// 塞焊缝符号的参数。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PlugWeldSymbol {
    /// 塞孔直径或当量尺寸，世界坐标单位。
    pub size: Option<f64>,
    /// 孔壁角度；单位由调用方约定。
    pub angle: Option<f64>,
    /// 塞焊点数量。
    pub number: u32,
    /// 塞焊点间距，世界坐标单位。
    pub spacing: Option<f64>,
}

impl WeldingSymbol {
    /// 创建空焊接符号：基本符号为 `None`，细节为单面 V 形坡口，其余可选字段均为 `None`。
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
    
    /// 创建角焊缝符号（基本符号置为 `Fillet`），其余字段取默认值。
    pub fn fillet() -> Self {
        Self {
            basic_symbol: BasicWeldSymbol::Fillet,
            ..Self::new()
        }
    }
    
    /// 创建坡口焊缝符号（基本符号置为 `Groove`），并按坡口形式设置细节。
    ///
    /// - `groove_type`：坡口形式；`SingleBevel` 映射为 `SingleVGroove`，
    ///   `DoubleBevel` 映射为 `DoubleVGroove`（本枚举未区分斜角细节）。
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
    
    /// 创建点焊缝符号（基本符号置为 `Spot`）。
    pub fn spot() -> Self {
        Self {
            basic_symbol: BasicWeldSymbol::Spot,
            ..Self::new()
        }
    }
    
    /// 创建缝焊缝符号（基本符号置为 `Seam`）。
    pub fn seam() -> Self {
        Self {
            basic_symbol: BasicWeldSymbol::Seam,
            ..Self::new()
        }
    }
    
    /// 创建螺柱焊符号（基本符号置为 `Stud`）。
    pub fn stud() -> Self {
        Self {
            basic_symbol: BasicWeldSymbol::Stud,
            ..Self::new()
        }
    }
    
    /// 预留的尺寸设置入口（当前实现不写入尺寸）。
    ///
    /// - `size`：被忽略，不会保存到任何字段。
    ///
    /// 实际行为：仅当基本符号为 `Fillet` 时把 `weld_detail` 重置为 `SingleVGroove`，
    /// 其余情况原样返回。要读取尺寸请用 `get_size`。
    pub fn with_size(mut self, size: f64) -> Self {
        self.weld_detail = match self.basic_symbol {
            BasicWeldSymbol::Fillet => WeldDetail::SingleVGroove,
            _ => self.weld_detail,
        };
        self
    }
    
    /// 设置焊缝长度，链式返回修改后的自身。
    ///
    /// - `length`：世界坐标单位，不做合法性与正负校验。
    pub fn with_length(mut self, length: f64) -> Self {
        self.length = Some(length);
        self
    }
    
    /// 设置断续焊缝间距，链式返回修改后的自身。
    ///
    /// - `pitch`：世界坐标单位；与 `length` 配合表示断续焊。
    pub fn with_pitch(mut self, pitch: f64) -> Self {
        self.pitch = Some(pitch);
        self
    }
    
    /// 设置焊缝角度，链式返回修改后的自身。
    ///
    /// - `angle`：原样保存；角度制或弧度制由调用方与图纸约定一致。
    pub fn with_angle(mut self, angle: f64) -> Self {
        self.angle = Some(angle);
        self
    }
    
    /// 追加一个辅助符号并链式返回自身；同一符号可重复追加。
    pub fn with_supplementary(mut self, symbol: SupplementarySymbol) -> Self {
        self.supplementary_symbol.push(symbol);
        self
    }
    
    /// 原地追加一个辅助符号，无返回值。
    pub fn add_supplementary(&mut self, symbol: SupplementarySymbol) {
        self.supplementary_symbol.push(symbol);
    }
    
    /// 设置表面加工符号，链式返回修改后的自身。
    pub fn with_finish(mut self, finish: FinishSymbol) -> Self {
        self.finish = finish;
        self
    }
    
    /// 设置焊缝轮廓符号，链式返回修改后的自身。
    pub fn with_contour(mut self, contour: ContourSymbol) -> Self {
        self.contour = contour;
        self
    }
    
    /// 设置根部符号，链式返回修改后的自身。
    pub fn with_root(mut self, root: RootSymbol) -> Self {
        self.root = root;
        self
    }
    
    /// 追加周圈焊辅助符号（`WeldAllAround`），链式返回自身。
    pub fn all_around(mut self) -> Self {
        self.add_supplementary(SupplementarySymbol::WeldAllAround);
        self
    }
    
    /// 追加现场焊接辅助符号（`FieldWeld`），链式返回自身。
    pub fn field_weld(mut self) -> Self {
        self.add_supplementary(SupplementarySymbol::FieldWeld);
        self
    }
    
    /// 追加交错断续焊辅助符号（`Staggered`），链式返回自身。
    pub fn staggered(mut self) -> Self {
        self.add_supplementary(SupplementarySymbol::Staggered);
        self
    }
    
    /// 设置尾部注释文本，链式返回修改后的自身。
    ///
    /// - `text`：工艺说明、标准号等自由文本，原样保存。
    pub fn with_tail(mut self, text: String) -> Self {
        self.tail = Some(text);
        self
    }
    
    /// 预留的断续焊设置入口，当前为空实现：原样返回 `self`，不写入任何字段。
    ///
    /// - `is_chain`：被忽略。
    pub fn intermittent(mut self, is_chain: bool) -> Self {
        self
    }
    
    /// 返回尺寸的文本形式，供拼装标注字符串使用。
    ///
    /// 尺寸来自 `get_size`；无尺寸时返回空串（不是 `"0"`）。
    pub fn format_size(&self) -> String {
        if let Some(size) = self.get_size() {
            format!("{}", size)
        } else {
            String::new()
        }
    }
    
    /// 按焊缝细节推导的名义尺寸。
    ///
    /// 返回：V/U/J 形坡口与卷边坡口为 6.0，I 形坡口为 3.0，其余细节为 `None`。
    /// 该值由 `weld_detail` 决定，与 `with_size` 的入参无关（后者不保存尺寸）。
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

/// 焊接符号在图纸上的放置：符号内容、位置、朝向与所依附的基准线。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WeldingSymbolPlacement {
    /// 待放置的焊接符号。
    pub symbol: WeldingSymbol,
    /// 放置位置，世界坐标。
    pub location: Point,
    /// 旋转角；原样保存，本模块不换算角度单位。
    pub rotation: f64,
    /// 统一缩放系数，1.0 表示原始大小。
    pub scale: f64,
    /// 符号位于基准线的哪一侧。
    pub side: WeldSide,
    /// 箭头线与基准线的几何定义。
    pub reference_line: WeldReferenceLine,
}

/// 焊接符号相对基准线的位置。
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum WeldSide {
    /// 基准线上方。
    Above,
    /// 基准线下方。
    Below,
    /// 上下两侧都有符号（双面焊）。
    Both,
    /// 沿整个接头（周圈）。
    Entire,
    /// 周圈焊接。
    Around,
}

/// 焊接符号的基准线几何：箭头线 + 基准线。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WeldReferenceLine {
    /// 箭头起点，世界坐标。
    pub start_point: Point,
    /// 引线终点，世界坐标。
    pub end_point: Point,
    /// 箭头线段。
    pub arrow_line: Line,
    /// 基准线段（符号文字所在的那条线）。
    pub reference_line: Line,
    /// 是否带尾部注释。
    pub has_tail: bool,
}

impl WeldReferenceLine {
    /// 由箭头起点与引线终点推导箭头线和基准线。
    ///
    /// - `start`：箭头起点（指向被焊件一侧）。
    /// - `end`：引线终点，与 `start` 共同确定方向。
    /// - `arrow_side`：当前未参与计算，仅作接口预留。
    ///
    /// 箭头线自 `start` 沿 start→end 方向延伸 20.0 个世界坐标单位；
    /// 基准线为 start→end 连线沿左法线偏移 10.0 后的平行线段；两条线的 z 均为 0.0。
    /// 返回的 `has_tail` 为 `false`。
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
    
    /// 标记该基准线带尾部注释，链式返回修改后的自身。
    ///
    /// - `tail_text`：仅用于表示意图，不会被保存；尾部文本请写在 `WeldingSymbol::tail`。
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

