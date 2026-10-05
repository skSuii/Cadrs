//! 几何公差与工艺符号的独立实现：公差框格、基准体系、表面纹理与焊接符号。
//!
//! 本模块自带一套 `GeometricTolerance`、`ToleranceFrame`、`Datum`、`SurfaceTexture`、
//! `WeldSymbol` 等类型，与 `crate::geometric_tolerance` 中的同名类型互不影响。
//!
//! - 公差部分：`ToleranceType` 给出特征符号，`ToleranceFrame::generate_boxes` 把框格
//!   拆成带宽度的文本块，供排版绘制；
//! - 基准部分：`Datum` / `DatumFeature` / `DatumIdentifier` 描述基准要素、基准目标与标注；
//! - 工艺符号：`SurfaceTexture` 与 `WeldSymbol` 负责表面纹理和焊接标注的取值与标签拼接。
//!
//! 坐标为世界坐标，公差与长度使用图纸计量单位；旋转角原样保存，SDK 统一按弧度解释。
//! 各 `generate_*` 方法只做数据到文本/尺寸的换算，不写盘、不修改 `self`。

use super::geometry::Point;

/// 几何公差标注（本模块版本）：一个公差框格加放置信息。
#[derive(Debug, Clone)]
pub struct GeometricTolerance {
    /// 公差框格。
    pub frame: ToleranceFrame,
    /// 标注放置位置，世界坐标。
    pub position: Point,
    /// 标注旋转角，弧度。
    pub rotation: f64,
    /// 公差带形状。
    pub tolerance_zone_shape: ToleranceZoneShape,
    /// 材料条件，默认最大实体要求。
    pub material_condition: MaterialCondition,
    /// 基准标识标注，`None` 表示不绘制。
    pub datum_identifier: Option<DatumIdentifier>,
}

impl GeometricTolerance {
    /// 创建公差标注。
    ///
    /// - `tolerance_type`：公差特征符号。
    /// - `value`：公差值，图纸计量单位。
    /// - `primary_datum`：主基准，`None` 表示无基准。
    ///
    /// 位置为原点、旋转 0.0、公差带为圆柱形、材料条件为最大实体要求、无基准标识标注。
    pub fn new(
        tolerance_type: ToleranceType,
        value: f64,
        primary_datum: Option<Datum>,
    ) -> Self {
        let frame = ToleranceFrame::new(tolerance_type, value, primary_datum);

        Self {
            frame,
            position: Point::default(),
            rotation: 0.0,
            tolerance_zone_shape: ToleranceZoneShape::Cylindrical,
            material_condition: MaterialCondition::MaximumMaterialCondition,
            datum_identifier: None,
        }
    }

    /// 设置放置位置（世界坐标），链式返回修改后的自身。
    pub fn with_position(mut self, position: Point) -> Self {
        self.position = position;
        self
    }

    /// 设置标注旋转角，链式返回修改后的自身。
    ///
    /// - `rotation`：弧度，本模块只保存不换算。
    pub fn with_rotation(mut self, rotation: f64) -> Self {
        self.rotation = rotation;
        self
    }

    /// 设置次基准，链式返回修改后的自身。
    pub fn with_secondary_datum(mut self, datum: Datum) -> Self {
        self.frame.secondary_datum = Some(datum);
        self
    }

    /// 设置第三基准，链式返回修改后的自身。
    pub fn with_tertiary_datum(mut self, datum: Datum) -> Self {
        self.frame.tertiary_datum = Some(datum);
        self
    }

    /// 生成框格的绘制块序列（符号格 + 各基准字母格），直接转调 `ToleranceFrame::generate_boxes`。
    ///
    /// 返回的每个块含文本与建议宽度，按从左到右的绘制顺序排列；不修改 `self`。
    pub fn generate_frame_box(&self) -> Vec<FrameBox> {
        self.frame.generate_boxes()
    }
}

/// 公差框格：特征符号、公差值与基准序列。
#[derive(Debug, Clone)]
pub struct ToleranceFrame {
    /// 公差特征符号。
    pub tolerance_type: ToleranceType,
    /// 公差值，图纸计量单位；为 0.0 时生成的文本省略数值。
    pub value: f64,
    /// 主基准，`None` 表示不绘制该格。
    pub primary_datum: Option<Datum>,
    /// 次基准，`None` 表示不绘制该格。
    pub secondary_datum: Option<Datum>,
    /// 第三基准，`None` 表示不绘制该格。
    pub tertiary_datum: Option<Datum>,
    /// 投影公差区高度，图纸计量单位；`None` 表示无投影公差。
    pub projected_tolerance: Option<f64>,
    /// 框格修饰符号；`None` 表示无修饰。
    pub modifier: Option<FrameModifier>,
}

impl ToleranceFrame {
    /// 创建公差框格。
    ///
    /// - `tolerance_type`：公差特征符号。
    /// - `value`：公差值，图纸计量单位。
    /// - `primary_datum`：主基准，`None` 表示无基准。
    ///
    /// 次基准、第三基准、投影公差与修饰符号均为空。
    pub fn new(
        tolerance_type: ToleranceType,
        value: f64,
        primary_datum: Option<Datum>,
    ) -> Self {
        Self {
            tolerance_type,
            value,
            primary_datum,
            secondary_datum: None,
            tertiary_datum: None,
            projected_tolerance: None,
            modifier: None,
        }
    }

    /// 设置投影公差区高度，链式返回修改后的自身。
    ///
    /// - `value`：图纸计量单位。
    pub fn with_projected_tolerance(mut self, value: f64) -> Self {
        self.projected_tolerance = Some(value);
        self
    }

    /// 设置框格修饰符号，链式返回修改后的自身。
    pub fn with_modifier(mut self, modifier: FrameModifier) -> Self {
        self.modifier = Some(modifier);
        self
    }

    /// 把框格拆分为按顺序排列的绘制块。
    ///
    /// 首块为「符号字符 + 数值」，`value` 为 0.0 时略去数值，文本保留 3 位小数；
    /// 其后按主、次、第三基准的顺序各追加一个形如 `[A]` 的块。宽度按字符数估算
    /// （每字符 3.0，符号格另加 4.0，基准格另加 2.0），仅用于粗略排版。
    ///
    /// 不修改 `self`；无任何基准时返回单元素列表。
    pub fn generate_boxes(&self) -> Vec<FrameBox> {
        let mut boxes = Vec::new();

        let symbol_char = self.tolerance_type.symbol_char();
        let value_str = if self.value == 0.0 {
            String::new()
        } else {
            format!("{:.3}", self.value)
        };

        let tolerance_text = format!("{}{}", symbol_char, value_str);

        boxes.push(FrameBox {
            text: tolerance_text,
            width: tolerance_text.len() as f64 * 3.0 + 4.0,
        });

        if let Some(ref datum) = self.primary_datum {
            let datum_text = format!("[{}]", datum.identifier);
            boxes.push(FrameBox {
                text: datum_text,
                width: datum_text.len() as f64 * 3.0 + 2.0,
            });
        }

        if let Some(ref datum) = self.secondary_datum {
            let datum_text = format!("[{}]", datum.identifier);
            boxes.push(FrameBox {
                text: datum_text,
                width: datum_text.len() as f64 * 3.0 + 2.0,
            });
        }

        if let Some(ref datum) = self.tertiary_datum {
            let datum_text = format!("[{}]", datum.identifier);
            boxes.push(FrameBox {
                text: datum_text,
                width: datum_text.len() as f64 * 3.0 + 2.0,
            });
        }

        boxes
    }
}

/// 公差框格中的一个绘制块。
#[derive(Debug, Clone)]
pub struct FrameBox {
    /// 块内文本（符号、数值或 `[基准字母]`）。
    pub text: String,
    /// 建议宽度，图纸计量单位。
    pub width: f64,
}

/// 几何公差特征类型。
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ToleranceType {
    /// 平面度。
    Flatness,
    /// 直线度。
    Straightness,
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
    /// 圆跳动。
    CircularRunout,
    /// 全跳动。
    TotalRunout,
}

impl ToleranceType {
    /// 返回该公差特征的显示字符（静态字符串，可直接拼进框格文本）。
    ///
    /// 部分字符仅为近似占位，渲染端可按需替换为正式图标。
    pub fn symbol_char(&self) -> &'static str {
        match self {
            ToleranceType::Flatness => "⏤",
            ToleranceType::Straightness => "—",
            ToleranceType::Circularity => "○",
            ToleranceType::Cylindricity => "⌥",
            ToleranceType::ProfileOfLine => "⌒",
            ToleranceType::ProfileOfSurface => "⏭",
            ToleranceType::Angularity => "∠",
            ToleranceType::Perpendicularity => "⊥",
            ToleranceType::Parallelism => "∥",
            ToleranceType::Position => "⏨",
            ToleranceType::Symmetry => "⌖",
            ToleranceType::CircularRunout => "↻",
            ToleranceType::TotalRunout => "⟳",
        }
    }
}

/// 公差带形状。
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ToleranceZoneShape {
    /// 圆柱形。
    Cylindrical,
    /// 球形。
    Spherical,
    /// 两条平行直线之间的区域。
    TwoParallelLines,
    /// 两个平行平面之间的区域。
    TwoParallelPlanes,
    /// 圆形。
    Circle,
    /// 球面。
    Sphere,
}

/// 材料条件（MMC / LMC / RFS）。
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MaterialCondition {
    /// 最大实体要求。
    MaximumMaterialCondition,
    /// 最小实体要求。
    LeastMaterialCondition,
    /// 与尺寸无关（RFS）。
    RegardlessOfFeatureSize,
}

/// 公差框格修饰符号。
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FrameModifier {
    /// 最大实体要求。
    MaximumMaterialCondition,
    /// 最小实体要求。
    LeastMaterialCondition,
    /// 与尺寸无关（RFS）。
    RegardlessOfFeatureSize,
    /// 投影公差区。
    ProjectedToleranceZone,
    /// 包容要求。
    EnvelopeRequirement,
    /// 相切平面。
    TangentPlane,
}

/// 基准：标识字母与其上的基准要素。
#[derive(Debug, Clone)]
pub struct Datum {
    /// 基准标识字母，如 `"A"`。
    pub identifier: String,
    /// 基准要素；`None` 表示只声明了字母，尚未指定实际要素。
    pub datum_feature: Option<DatumFeature>,
}

impl Datum {
    /// 用基准标识字母创建基准，`datum_feature` 为空。
    ///
    /// 可用 `with_secondary_datum` 之外的途径自行填充 `datum_feature`。
    pub fn new(identifier: String) -> Self {
        Self {
            identifier,
            datum_feature: None,
        }
    }
}

/// 基准要素：几何类型及其上的点/线/面基准目标。
#[derive(Debug, Clone)]
pub struct DatumFeature {
    /// 要素几何类型。
    pub feature_type: DatumFeatureType,
    /// 点目标集合，世界坐标。
    pub target_points: Vec<Point>,
    /// 线目标集合。
    pub target_lines: Vec<TargetLine>,
    /// 面目标集合。
    pub target_areas: Vec<TargetArea>,
}

impl DatumFeature {
    /// 按几何类型创建空基准要素，三类基准目标集合均为空。
    pub fn new(feature_type: DatumFeatureType) -> Self {
        Self {
            feature_type,
            target_points: Vec::new(),
            target_lines: Vec::new(),
            target_areas: Vec::new(),
        }
    }

    /// 追加一个点基准目标，链式返回修改后的自身。
    ///
    /// - `point`：世界坐标。
    pub fn add_target_point(mut self, point: Point) -> Self {
        self.target_points.push(point);
        self
    }

    /// 追加一个线基准目标，链式返回修改后的自身。
    pub fn add_target_line(mut self, line: TargetLine) -> Self {
        self.target_lines.push(line);
        self
    }

    /// 追加一个面基准目标，链式返回修改后的自身。
    pub fn add_target_area(mut self, area: TargetArea) -> Self {
        self.target_areas.push(area);
        self
    }
}

/// 基准要素的几何类型。
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DatumFeatureType {
    /// 平面。
    Plane,
    /// 直线（轴线）。
    Line,
    /// 点。
    Point,
    /// 圆柱面。
    Cylinder,
    /// 球面。
    Sphere,
    /// 圆锥面。
    Cone,
    /// 圆环面。
    Torus,
}

/// 点基准目标。
#[derive(Debug, Clone)]
pub struct TargetPoint {
    /// 目标位置，世界坐标。
    pub position: Point,
    /// 目标符号直径，图纸计量单位。
    pub diameter: f64,
}

/// 线基准目标。
#[derive(Debug, Clone)]
pub struct TargetLine {
    /// 起点，世界坐标。
    pub start: Point,
    /// 终点，世界坐标。
    pub end: Point,
    /// 线段长度，图纸计量单位；由调用方给出，本结构不校验是否等于两端点距离。
    pub length: f64,
}

/// 面（圆）基准目标。
#[derive(Debug, Clone)]
pub struct TargetArea {
    /// 圆心，世界坐标。
    pub center: Point,
    /// 目标圆直径，图纸计量单位。
    pub diameter: f64,
}

/// 基准标识标注：基准字母在图纸上的显示方式与位置。
#[derive(Debug, Clone)]
pub struct DatumIdentifier {
    /// 被标注的基准。
    pub datum: Datum,
    /// 标注位置，世界坐标。
    pub position: Point,
    /// 标注旋转角，弧度。
    pub rotation: f64,
    /// 显示样式（是否带框、是否带方括号）。
    pub style: DatumIdentifierStyle,
    /// 文字高度，图纸计量单位，默认 3.5。
    pub height: f64,
}

impl DatumIdentifier {
    /// 创建基准标识标注。
    ///
    /// - `datum`：被标注的基准。
    /// - `position`：标注位置，世界坐标。
    ///
    /// 旋转角 0.0、样式为 `Basic`、文字高度 3.5。
    pub fn new(datum: Datum, position: Point) -> Self {
        Self {
            datum,
            position,
            rotation: 0.0,
            style: DatumIdentifierStyle::Basic,
            height: 3.5,
        }
    }

    /// 设置旋转角，链式返回修改后的自身。
    ///
    /// - `rotation`：弧度。
    pub fn with_rotation(mut self, rotation: f64) -> Self {
        self.rotation = rotation;
        self
    }

    /// 设置显示样式，链式返回修改后的自身。
    pub fn with_style(mut self, style: DatumIdentifierStyle) -> Self {
        self.style = style;
        self
    }

    /// 生成用于绘制的几何描述。
    ///
    /// 标签固定为 `[基准字母]` 形式（样式字段当前不影响标签内容），
    /// 位置、旋转角与文字高度原样透传；不修改 `self`。
    pub fn generate_display_geometry(&self) -> DisplayGeometry {
        let label = format!("[{}]", self.datum.identifier);

        DisplayGeometry {
            label,
            position: self.position,
            rotation: self.rotation,
            height: self.height,
        }
    }
}

/// 基准标识的显示样式。
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DatumIdentifierStyle {
    /// 基本样式（当前 `generate_display_geometry` 与其它样式输出相同）。
    Basic,
    /// 带方框。
    WithFrame,
    /// 不带方括号。
    WithoutBracket,
}

/// 可直接交给渲染端的显示描述。
#[derive(Debug, Clone)]
pub struct DisplayGeometry {
    /// 待绘制的标签文本。
    pub label: String,
    /// 文本位置，世界坐标。
    pub position: Point,
    /// 文本旋转角，弧度。
    pub rotation: f64,
    /// 文本高度，图纸计量单位。
    pub height: f64,
}

/// 表面纹理（表面粗糙度）标注（本模块版本）。
#[derive(Debug, Clone)]
pub struct SurfaceTexture {
    /// 纹理符号类型。
    pub symbol: SurfaceTextureSymbol,
    /// 评定参数值（如 Ra 数值），图纸计量单位。
    pub value: f64,
    /// 评定长度；`None` 表示未指定。
    pub evaluation_length: Option<f64>,
    /// 材料条件（加工面/非加工面），默认 `Machined`。
    pub material_condition: SurfaceTextureCondition,
    /// 加工方法文本；`None` 表示未指定。
    pub production_method: Option<String>,
    /// 纹理方向（刀纹方向）；`None` 表示未指定。
    pub direction_ofLay: Option<LayDirection>,
    /// 取样长度；`None` 表示未指定。
    pub sampling_length: Option<f64>,
    /// 标注位置，世界坐标。
    pub position: Point,
    /// 标注旋转角，弧度。
    pub rotation: f64,
    /// 文字高度，图纸计量单位，默认 3.5。
    pub height: f64,
}

impl SurfaceTexture {
    /// 创建表面纹理标注。
    ///
    /// - `symbol`：纹理符号类型。
    /// - `value`：评定参数值，图纸计量单位。
    /// - `position`：标注位置，世界坐标。
    ///
    /// 材料条件默认为 `Machined`，旋转角 0.0，文字高度 3.5，其余可选字段为 `None`。
    pub fn new(
        symbol: SurfaceTextureSymbol,
        value: f64,
        position: Point,
    ) -> Self {
        Self {
            symbol,
            value,
            evaluation_length: None,
            material_condition: SurfaceTextureCondition::Machined,
            production_method: None,
            direction_ofLay: None,
            sampling_length: None,
            position,
            rotation: 0.0,
            height: 3.5,
        }
    }

    /// 设置评定长度，链式返回修改后的自身。
    ///
    /// - `length`：图纸计量单位。
    pub fn with_evaluation_length(mut self, length: f64) -> Self {
        self.evaluation_length = Some(length);
        self
    }

    /// 设置材料条件（加工面 / 非加工面），链式返回修改后的自身。
    pub fn with_material_condition(mut self, condition: SurfaceTextureCondition) -> Self {
        self.material_condition = condition;
        self
    }

    /// 设置加工方法文本，链式返回修改后的自身。
    ///
    /// - `method`：如 `"车"`、`"磨"`，会出现在 `generate_label` 的数值之前。
    pub fn with_production_method(mut self, method: String) -> Self {
        self.production_method = Some(method);
        self
    }

    /// 设置纹理方向，链式返回修改后的自身。
    pub fn with_lay_direction(mut self, lay: LayDirection) -> Self {
        self.direction_ofLay = Some(lay);
        self
    }

    /// 设置旋转角，链式返回修改后的自身。
    ///
    /// - `rotation`：弧度。
    pub fn with_rotation(mut self, rotation: f64) -> Self {
        self.rotation = rotation;
        self
    }

    /// 拼装表面纹理标注文本。
    ///
    /// 顺序为：符号字符 +（加工方法 + 空格）+ 保留 3 位小数的数值 + 纹理方向字符 +
    /// `×取样长度` + `(评定长度)`；各可选字段为空时对应片段省略。
    /// 不修改 `self`，仅返回字符串。
    pub fn generate_label(&self) -> String {
        let mut parts = Vec::new();

        parts.push(self.symbol.symbol_char());

        let value_str = format!("{:.3}", self.value);
        if let Some(ref method) = self.production_method {
            parts.push(format!("{} {}", method, value_str));
        } else {
            parts.push(value_str);
        }

        if let Some(ref lay) = self.direction_ofLay {
            parts.push(lay.symbol_char());
        }

        if let Some(length) = self.sampling_length {
            parts.push(format!("×{:.3}", length));
        }

        if let Some(length) = self.evaluation_length {
            parts.push(format!("({:.3})", length));
        }

        parts.join("")
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SurfaceTextureSymbol {
    Roughness,
    RoughnessWithMachining,
    RemovalByMachining,
    NoRemovalByMachining,
    RoughnessParam1,
    RoughnessParam2,
    RoughnessParam3,
}

impl SurfaceTextureSymbol {
    pub fn symbol_char(&self) -> &'static str {
        match self {
            SurfaceTextureSymbol::Roughness => "⌔",
            SurfaceTextureSymbol::RoughnessWithMachining => "⌒",
            SurfaceTextureSymbol::RemovalByMachining => "▭",
            SurfaceTextureSymbol::NoRemovalByMachining => "□",
            SurfaceTextureSymbol::RoughnessParam1 => "Ra",
            SurfaceTextureSymbol::RoughnessParam2 => "Ry",
            SurfaceTextureSymbol::RoughnessParam3 => "Rz",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SurfaceTextureCondition {
    Machined,
    NonMachined,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LayDirection {
    Parallel,
    Perpendicular,
    Angular,
    Multidirectional,
    Radial,
    Circular,
}

impl LayDirection {
    pub fn symbol_char(&self) -> &'static str {
        match self {
            LayDirection::Parallel => "∥",
            LayDirection::Perpendicular => "⊥",
            LayDirection::Angular => "∠",
            LayDirection::Multidirectional => "≋",
            LayDirection::Radial => "⨀",
            LayDirection::Circular => "○",
        }
    }
}

#[derive(Debug, Clone)]
pub struct WeldSymbol {
    pub weld_type: WeldType,
    pub size: Option<f64>,
    pub length: Option<f64>,
    pub pitch: Option<f64>,
    pub tail: Option<WeldTail>,
    pub field_weld: bool,
    pub all_around: bool,
    pub reference_line: WeldReferenceLine,
    pub symbols: Vec<SupplementarySymbol>,
    pub position: Point,
    pub rotation: f64,
}

impl WeldSymbol {
    pub fn new(weld_type: WeldType) -> Self {
        Self {
            weld_type,
            size: None,
            length: None,
            pitch: None,
            tail: None,
            field_weld: false,
            all_around: false,
            reference_line: WeldReferenceLine::Top,
            symbols: Vec::new(),
            position: Point::default(),
            rotation: 0.0,
        }
    }

    pub fn with_size(mut self, size: f64) -> Self {
        self.size = Some(size);
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

    pub fn as_field_weld(mut self) -> Self {
        self.field_weld = true;
        self
    }

    pub fn as_all_around(mut self) -> Self {
        self.all_around = true;
        self
    }

    pub fn with_supplementary_symbol(mut self, symbol: SupplementarySymbol) -> Self {
        self.symbols.push(symbol);
        self
    }

    pub fn with_position(mut self, position: Point) -> Self {
        self.position = position;
        self
    }

    pub fn with_rotation(mut self, rotation: f64) -> Self {
        self.rotation = rotation;
        self
    }

    pub fn generate_symbol_string(&self) -> String {
        let mut parts = Vec::new();

        if let Some(size) = self.size {
            parts.push(format!("{}", size));
        }

        if let Some(length) = self.length {
            parts.push(format!("-{}", length));
        }

        if let Some(pitch) = self.pitch {
            parts.push(format!("@{}", pitch));
        }

        for symbol in &self.symbols {
            parts.push(symbol.symbol_char());
        }

        if self.field_weld {
            parts.push(" flag".to_string());
        }

        if self.all_around {
            parts.push("◠".to_string());
        }

        parts.join("")
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum WeldType {
    Fillet,
    Groove,
    Spot,
    Seam,
    Plug,
    Slot,
    Surface,
    Back,
    MeltThrough,
    Stud,
    Flange,
    Edge,
    SquareGroove,
    VGroove,
    BevelGroove,
    UGroove,
    JGroove,
}

impl WeldType {
    pub fn symbol_char(&self) -> &'static str {
        match self {
            WeldType::Fillet => "▭",
            WeldType::Groove => "⌒",
            WeldType::Spot => "○",
            WeldType::Seam => "≡",
            WeldType::Plug => "▭",
            WeldType::Slot => "□",
            WeldType::Surface => "▬",
            WeldType::Back => "▱",
            WeldType::MeltThrough => "▱",
            WeldType::Stud => "▭",
            WeldType::Flange => "⌒",
            WeldType::Edge => "▬",
            WeldType::SquareGroove => "▬",
            WeldType::V Groove => "V",
            WeldType::BevelGroove => "L",
            WeldType::UGroove => "U",
            WeldType::JGroove => "J",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum WeldReferenceLine {
    Top,
    Bottom,
    Both,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SupplementarySymbol {
    WeldRoot,
    Contour,
    Grind,
    Flush,
    Convex,
    Concave,
    Flat,
    ConvexContour,
    ConcaveContour,
    Melted,
    Unmelted,
    Backing,
}

impl SupplementarySymbol {
    pub fn symbol_char(&self) -> &'static str {
        match self {
            SupplementarySymbol::WeldRoot => "▸",
            SupplementarySymbol::Contour => "⌒",
            SupplementarySymbol::Grind => "G",
            SupplementarySymbol::Flush => "F",
            SupplementarySymbol::Convex => "C",
            SupplementarySymbol::Concave => "CC",
            SupplementarySymbol::Flat => "FL",
            SupplementarySymbol::ConvexContour => "C⌒",
            SupplementarySymbol::ConcaveContour => "CC⌒",
            SupplementarySymbol::Melted => "M",
            SupplementarySymbol::Unmelted => "UM",
            SupplementarySymbol::Backing => "B",
        }
    }
}

#[derive(Debug, Clone)]
pub struct WeldTail {
    pub specification: String,
    pub reference: String,
}

#[derive(Debug, Clone)]
pub struct WeldAllAround {
    pub symbol: char,
    pub position: Point,
    pub rotation: f64,
}

impl WeldAllAround {
    pub fn new(position: Point) -> Self {
        Self {
            symbol: '◠',
            position,
            rotation: 0.0,
        }
    }
}
