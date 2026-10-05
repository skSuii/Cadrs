//! 几何公差（GeometricTolerance）：按 GB/T 1182、ASME Y14.5 的语义组织公差框格、基准与公差带。
//!
//! - `FeatureControlFrame` 是标注主体，含一至多个 `ToleranceFrame`，可再附一个复合框格；
//! - `DatumReference`、`Datum`、`DatumReferenceFrame`、`DatumSystem` 描述基准及其材料条件与基准目标；
//! - `ToleranceValue`、`ToleranceZoneShape`、`ProjectedTolerance` 描述公差带形状与投影公差区；
//! - `DatumTarget` 描述基准目标（点 / 线 / 面）。
//!
//! 所有坐标为世界坐标，公差数值使用图纸计量单位；本模块只组织数据，不做几何求解或合法性校验。
//! 末尾的 `From<GeometricTolerance> for Entity` 会把首个框格近似转成一个位于原点的文字实体。

use crate::geometry::Point;
use serde::{Serialize, Deserialize};
use std::fmt;

/// 几何公差标注：一个特征控制框格加放置信息。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GeometricTolerance {
    /// 特征控制框格，承载公差符号、数值与基准。
    pub feature_control_frame: FeatureControlFrame,
    /// 放置方式（在要素上、投影区、与尺寸无关）。
    pub placement: TolerancePlacement,
}

/// 特征控制框格：公差框格的完整内容。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FeatureControlFrame {
    /// 框格列表，通常只有一个；`add_datum`、`add_projected_tolerance` 只作用于首个框格。
    pub frames: Vec<ToleranceFrame>,
    /// 复合公差框格（上下两格），`None` 表示无复合要求。
    pub composite_frame: Option<CompositeFrame>,
    /// 整个框格适用的材料条件。
    pub material_condition: MaterialCondition,
    /// 基准参考框：按主、次、第三基准的顺序排列。
    pub datum_reference_frame: Vec<DatumReference>,
}

/// 单个公差框格（一个公差符号 + 数值 + 基准字母格）。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ToleranceFrame {
    /// 公差特征符号（直线度、平面度、位置度等）。
    pub symbol: ToleranceSymbol,
    /// 公差值，使用图纸计量单位。
    pub tolerance_value: f64,
    /// 该框格引用的基准，按优先顺序排列。
    pub datum_references: Vec<DatumReference>,
    /// 公差值后的修饰符号（最大实体、最小实体等）。
    pub modifier: ToleranceModifier,
    /// 第二公差值（复合/双向公差时的另一方向），`None` 表示未指定。
    pub secondary_tolerance: Option<f64>,
    /// 投影公差区，`None` 表示非投影公差。
    pub projected_tolerance: Option<ProjectedTolerance>,
}

/// 复合公差框格：上下两个公差框格共用一个位置度符号。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CompositeFrame {
    /// 上框格（较大公差值）。
    pub primary_tolerance: ToleranceFrame,
    /// 下框格（较小公差值）。
    pub secondary_tolerance: ToleranceFrame,
    /// 框格左侧的符号，固定为位置度。
    pub position_symbol: ToleranceSymbol,
}

/// 几何公差特征符号。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ToleranceSymbol {
    /// 直线度。
    Straightness,
    /// 平面度。
    Flatness,
    /// 圆度。
    Circularity,
    /// 圆柱度。
    Cylindricity,
    /// 线轮廓度。
    ProfileOfLine,
    /// 面轮廓度。
    ProfileOfSurface,
    /// 倾斜度。
    Angularity,
    /// 垂直度。
    Perpendicularity,
    /// 平行度。
    Parallelism,
    /// 位置度。
    Position,
    /// 对称度。
    Symmetry,
    /// 同轴度。
    Concentricity,
    /// 圆跳动。
    CircularRunout,
    /// 全跳动。
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

/// 对某个基准的引用：基准字母 + 修饰 + 材料条件。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DatumReference {
    /// 基准标识字母，如 `"A"`；本模块不校验长度与合法性。
    pub datum: String,
    /// 基准修饰符号。
    pub modifier: DatumModifier,
    /// 该基准适用的材料条件。
    pub material_condition: MaterialCondition,
}

/// 基准修饰符号。
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum DatumModifier {
    /// 无修饰。
    None,
    /// 最大实体要求。
    MaximumMaterialCondition,
    /// 最小实体要求。
    LeastMaterialCondition,
    /// 与尺寸无关（RFS）。
    RegardlessOfFeatureSize,
}

/// 材料条件（MMC / LMC / RFS）。
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum MaterialCondition {
    /// 未指定。
    None,
    /// 最大实体要求。
    MaximumMaterialCondition,
    /// 最小实体要求。
    LeastMaterialCondition,
    /// 与尺寸无关（RFS）。
    RegardlessOfFeatureSize,
}

impl Default for MaterialCondition {
    fn default() -> Self {
        MaterialCondition::None
    }
}

/// 公差值修饰符号。
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum ToleranceModifier {
    /// 无修饰。
    None,
    /// 最大实体要求。
    MaximumMaterialCondition,
    /// 最小实体要求。
    LeastMaterialCondition,
    /// 投影公差区。
    ProjectedTolerance,
    /// 自由状态。
    FreeState,
    /// 包容要求。
    Envelope,
}

/// 投影公差区：公差带沿垂直于被测要素方向延伸的高度与直径。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProjectedTolerance {
    /// 投影区高度，图纸计量单位。
    pub zone_height: f64,
    /// 投影区直径；`None` 表示未指定（非圆柱形公差带）。
    pub zone_diameter: Option<f64>,
}

/// 公差的放置方式。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TolerancePlacement {
    /// 标注在要素上。
    OnFeature,
    /// 标注在投影公差区上。
    Projected,
    /// 与尺寸无关。
    RegardlessOfFeatureSize,
}

/// 公差值的图形化描述（直径符号 + 数值 + 公差带形状）。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ToleranceValue {
    /// 数值前是否加直径符号 `⌀`。
    pub diameter_symbol: bool,
    /// 公差数值，图纸计量单位。
    pub value: f64,
    /// 公差带形状。
    pub tolerance_zone_shape: ToleranceZoneShape,
}

/// 公差带形状。
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum ToleranceZoneShape {
    /// 圆柱形。
    Cylinder,
    /// 球形。
    Sphere,
    /// 圆形。
    Circle,
    /// 两条平行直线之间的区域。
    BetweenTwoLines,
    /// 两个平行平面之间的区域。
    BetweenTwoPlanes,
    /// 多边形。
    Polygon,
}

impl GeometricTolerance {
    /// 用特征符号与公差值创建一个公差标注。
    ///
    /// - `symbol`：公差特征符号。
    /// - `value`：公差值，图纸计量单位。
    ///
    /// 生成只含该框格的特征控制框格（无基准、无修饰、无材料条件、无投影区），
    /// 放置方式为 `OnFeature`。
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
    
    /// 添加一个基准引用。
    ///
    /// - `datum`：基准标识字母，如 `"A"`。
    /// - `modifier`：基准修饰符号。
    ///
    /// 同一份引用会写入首个公差框格的 `datum_references` 与整体 `datum_reference_frame`；
    /// 框格列表为空时只写后者。引用的材料条件取自框格当前的 `material_condition`。
    /// 原地修改 `self`，重复调用只会追加（不去重）。
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
    
    /// 设置整个特征控制框格的材料条件。
    ///
    /// - `condition`：最大实体 / 最小实体 / 与尺寸无关。
    ///
    /// 只改框格字段；已写入的 `DatumReference` 保存的是当时的副本，不会被同步更新。
    pub fn set_material_condition(&mut self, condition: MaterialCondition) {
        self.feature_control_frame.material_condition = condition;
    }
    
    /// 设置复合公差框格（上下两格位置度）。
    ///
    /// - `primary_value`：上框格公差值。
    /// - `secondary_value`：下框格公差值。
    /// - `datum_references`：两格共用的基准引用，会被克隆一份分别写入。
    ///
    /// 两格符号固定为 `ToleranceSymbol::Position`；已存在的 `composite_frame` 会被覆盖。
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
    
    /// 为首个公差框格设置投影公差区。
    ///
    /// - `height`：投影区高度，图纸计量单位。
    ///
    /// `zone_diameter` 保持 `None`；框格列表为空时不做任何修改。
    pub fn add_projected_tolerance(&mut self, height: f64) {
        if let Some(frame) = self.feature_control_frame.frames.first_mut() {
            frame.projected_tolerance = Some(ProjectedTolerance {
                zone_height: height,
                zone_diameter: None,
            });
        }
    }
}

/// 基准目标：用点、线或面指定基准要素上的具体接触位置。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DatumTarget {
    /// 点目标的位置；非点目标为 `None`。
    pub target_point: Option<Point>,
    /// 线目标的两端点；非线目标为 `None`。
    pub target_line: Option<(Point, Point)>,
    /// 面目标的圆心与直径；非面目标为 `None`。
    pub target_area: Option<(Point, f64)>,
    /// 目标类型，与上面三个字段保持一致。
    pub target_type: DatumTargetType,
    /// 目标标识，如 `"A1"`。
    pub identifier: String,
    /// 目标符号尺寸，世界坐标单位。
    pub size: f64,
}

/// 基准目标类型。
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum DatumTargetType {
    /// 点目标。
    Point,
    /// 线目标。
    Line,
    /// 面（圆）目标。
    Area,
}

impl DatumTarget {
    /// 创建点式基准目标。
    ///
    /// - `identifier`：目标标识。
    /// - `point`：目标位置，世界坐标。
    ///
    /// 类型为 `Point`，只填 `target_point`；`size` 固定为 3.0，线与面字段为 `None`。
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
    
    /// 创建线式基准目标。
    ///
    /// - `identifier`：目标标识。
    /// - `p1`、`p2`：线段两端点，世界坐标。
    ///
    /// 类型为 `Line`，只填 `target_line`；`size` 固定为 3.0。
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
    
    /// 创建面式（圆形）基准目标。
    ///
    /// - `identifier`：目标标识。
    /// - `center`：圆心，世界坐标。
    /// - `diameter`：目标圆直径，世界坐标单位；同时写入 `target_area` 与 `size`。
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

/// 基准参考框：标注中引用的基准及其顺序。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DatumReferenceFrame {
    /// 基准序列，通常按主、次、第三顺序。
    pub designation: Vec<Datum>,
    /// 基准顺序号（1 表示主基准）。
    pub order: usize,
    /// 基准参考框整体适用的材料条件。
    pub material_condition: MaterialCondition,
}

/// 单个基准：标识、类型、其上的基准目标与偏移。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Datum {
    /// 基准标识字母，如 `"A"`。
    pub identifier: String,
    /// 基准类型标识（如平面、轴线）；空串表示未指定。
    pub type_id: String,
    /// 该基准上的基准目标列表。
    pub targets: Vec<DatumTarget>,
    /// 该基准适用的材料条件。
    pub material_condition: MaterialCondition,
    /// 基准偏移量 (x, y, z)，世界坐标单位；`None` 表示无偏移。
    pub shift: Option<(f64, f64, f64)>,
}

impl Datum {
    /// 用基准标识创建一个空基准。
    ///
    /// `type_id` 为空串、无基准目标、材料条件为 `None`、无偏移。
    pub fn new(identifier: String) -> Self {
        Self {
            identifier,
            type_id: String::new(),
            targets: Vec::new(),
            material_condition: MaterialCondition::None,
            shift: None,
        }
    }
    
    /// 追加一个基准目标到末尾，原地修改 `self`；不去重。
    pub fn add_target(&mut self, target: DatumTarget) {
        self.targets.push(target);
    }
}

/// 基准体系：主基准（必需）加可选的次基准与第三基准。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DatumSystem {
    /// 主基准，必有。
    pub primary_datum: Datum,
    /// 次基准，`None` 表示未指定。
    pub secondary_datum: Option<Datum>,
    /// 第三基准，`None` 表示未指定。
    pub tertiary_datum: Option<Datum>,
    /// 由 `build_references` 生成的基准引用序列。
    pub references: Vec<DatumReference>,
}

impl DatumSystem {
    /// 用主基准建立基准体系，次基准与第三基准留空，引用序列为空。
    pub fn new(primary: Datum) -> Self {
        Self {
            primary_datum: primary,
            secondary_datum: None,
            tertiary_datum: None,
            references: Vec::new(),
        }
    }
    
    /// 设置次基准，已有值会被覆盖。
    pub fn add_secondary(&mut self, datum: Datum) {
        self.secondary_datum = Some(datum);
    }
    
    /// 设置第三基准，已有值会被覆盖。
    pub fn add_tertiary(&mut self, datum: Datum) {
        self.tertiary_datum = Some(datum);
    }
    
    /// 依据当前基准重建 `references`：先清空，再按主基准、次基准、第三基准的顺序追加。
    ///
    /// 每一项的 `modifier` 固定为 `None`，`material_condition` 取自对应基准自身的设置；
    /// 未设置的次/第三基准会被跳过。原地修改 `self`，重复调用结果相同。
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
