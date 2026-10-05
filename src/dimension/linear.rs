//! 线性与对齐标注的实现，同时提供整个标注（Dimension）子系统共用的样式与几何基础。
//!
//! 本模块包含三部分：样式 [`DimensionStyle`]（箭头、尺寸线与尺寸界线偏移、文字排版、
//! 单位格式与公差显示）、几何 [`DimensionGeometry`]（定义点、测量值、显示文字）与
//! 具体标注 [`LinearDimension`]、[`AlignedDimension`]、[`AngularDimension`]、
//! [`RadialDimension`]、[`OrdinateDimension`]。
//!
//! 与其他模块的关系：各标注均以 [`crate::geometry`] 的图元描述形状、以 [`crate::data_structure`]
//! 的 [`Entity`] 承载结果，经 `From` 实现转换为 [`EntityType::Dimension`] 实体后交给
//! Tessellation 与各导出模块；`angular`、`radial`、`ordinate` 等模块复用此处的样式与几何结构。
//!
//! 所有坐标与长度均为世界坐标下的图形单位，角度一律为弧度。

use crate::geometry::{Point, Vector2, Line, Arc, Circle};
use crate::data_structure::{Entity, EntityType, EntityGeometry, ObjectId, Transform, Visibility, TextStyle};
use serde::{Serialize, Deserialize};
use std::fmt;

/// 标注的测量类型，决定 [`DimensionGeometry::calculate_measurement`] 如何由定义点推导测量值。
///
/// 与 [`crate::data_structure::DimensionType`] 并非同一类型：转换为实体时由
/// [`entity_geometry_from_dimension`] 按同名规则映射（半径归入半径标注，基线与连续归入线性标注）。
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum DimensionType {
    /// 线性标注：测量值为前两个定义点之间的直线距离。
    Linear,
    /// 对齐标注：测量值与线性相同，但尺寸线与实际测量方向对齐。
    Aligned,
    /// 角度标注：三个定义点依次为中心、起点、终点，测量值为弧度。
    Angular,
    /// 半径标注：测量值为第一个定义点（圆心）到第二个定义点的距离。
    Radial,
    /// 直径标注：测量值为半径的两倍。
    Diameter,
    /// 弧长标注：`calculate_measurement` 不处理该类型，需调用方直接设置测量值。
    ArcLength,
    /// 坐标标注：测量值为特征点的 X 或 Y 坐标。
    Ordinate,
    /// 基线标注：多个标注共用同一条基准，转换为实体时按线性标注处理。
    Baseline,
    /// 连续标注：多个标注首尾相接，转换为实体时按线性标注处理。
    Continued,
}

/// 标注样式：集中描述箭头、尺寸线与尺寸界线偏移、文字排版、单位格式与公差显示。
///
/// 各标注构造函数按值克隆该样式并保存到 [`DimensionGeometry::style`]，
/// 因此创建标注之后再修改样式不会影响已生成的标注；`Default` 给出名为 "Standard" 的常用初值。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DimensionStyle {
    /// 样式的唯一标识，用于在文档的样式集合中引用本样式。
    pub id: ObjectId,
    /// 样式名称，`Default` 为 "Standard"。
    pub name: String,
    /// 标注文字的字高（图形单位），同时作为推导尺寸线、角度圆弧与文字偏移量的基准。
    pub text_height: f64,
    /// 箭头长度（图形单位），并作为中心标记尺寸与角度标注圆弧半径的推导基准。
    pub arrow_size: f64,
    /// 箭头形状；仅记录所选形式，箭头图形由下游分解与渲染完成。
    pub arrow_style: ArrowStyle,
    /// 尺寸界线超出尺寸线的延伸长度（图形单位）。
    pub extension_line_extension: f64,
    /// 尺寸界线起点与被测定义点之间的间隙（图形单位）。
    pub extension_line_offset: f64,
    /// 尺寸线与标注文字之间的间隙（图形单位）。
    pub dimension_line_gap: f64,
    /// 文字沿尺寸线方向的位置。
    pub text_horizontal_placement: HorizontalTextPlacement,
    /// 文字相对尺寸线的垂直位置。
    pub text_vertical_placement: VerticalTextPlacement,
    /// 文字的书写方向。
    pub text_direction: TextDirection,
    /// 测量值使用的单位与数字格式。
    pub unit_format: UnitFormat,
    /// 十进制格式保留的小数位数。
    pub decimal_places: u32,
    /// 测量值的取整步长（与测量值同单位）；取整按 值 ÷ round_off 四舍五入后再乘回，
    /// 因此默认值 0.0 会使格式化得到非数值文本，实际使用时应设为正数。
    pub round_off: f64,
    /// 测量值前缀，例如半径标注填 "R"、直径标注填 "Ø"。
    pub prefix: String,
    /// 测量值后缀，例如单位符号 "mm"。
    pub suffix: String,
    /// 是否显示换算后的备用单位；当前文字生成逻辑未读取该字段。
    pub alternate_units: bool,
    /// 备用单位的换算系数，默认 25.4（英寸换算为毫米）；当前文字生成逻辑未读取该字段。
    pub alternate_units_factor: f64,
    /// 公差显示方式；为 `None` 时 [`DimensionGeometry::update_text`] 不附加公差文本。
    pub tolerance_display: ToleranceDisplay,
    /// 公差值保留的小数位数。
    pub tolerance_precision: u32,
    /// 上偏差（带符号），仅在公差显示方式不为 `None` 时写入文字。
    pub tolerance_upper_value: f64,
    /// 下偏差（带符号）。
    pub tolerance_lower_value: f64,
    /// 标注文字颜色的 RGB 分量。
    pub text_color: (u8, u8, u8),
    /// 尺寸界线颜色的 RGB 分量。
    pub extension_line_color: (u8, u8, u8),
    /// 尺寸线与箭头颜色的 RGB 分量。
    pub dimension_line_color: (u8, u8, u8),
    /// 使用该样式的标注是否可见。
    pub visible: bool,
}

/// 箭头形状，由 [`DimensionStyle::arrow_style`] 保存，供绘制端决定尺寸线两端的标记形式。
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum ArrowStyle {
    /// 封闭空心箭头。
    Closed,
    /// 封闭实心箭头，样式默认值。
    ClosedFilled,
    /// 圆点标记。
    Dot,
    /// 小号封闭实心箭头，用于尺寸线空间狭窄处。
    SmallClosed,
    /// 开口箭头（V 形）。
    Open,
    /// 原点指示符（小圆与十字组合）。
    OriginIndicator,
    /// 原点指示符的第二种变体。
    Origin02,
    /// 斜线标记。
    Oblique,
    /// 建筑标记：以 45° 斜短线代替箭头。
    ArchitecturalTick,
}

/// 标注文字沿尺寸线方向的放置方式，默认居中。
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum HorizontalTextPlacement {
    /// 文字在尺寸线中段居中放置。
    Centered,
    /// 文字放置在尺寸线上方。
    Above,
    /// 文字放置在尺寸线下方。
    Below,
}

/// 标注文字相对尺寸线的垂直放置方式，样式默认在尺寸线上方。
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum VerticalTextPlacement {
    /// 文字在尺寸线处垂直居中（尺寸线穿过文字）。
    Centered,
    /// 按 JIS 规则放置，尺寸线为文字让位断开。
    JIS,
    /// 文字放置在尺寸线上方。
    Above,
    /// 文字自尺寸线向上偏移一个文字高度的位置放置。
    AboveFromDimensionLine,
}

/// 标注文字的书写方向，默认从左到右。
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum TextDirection {
    /// 从左到右书写。
    LeftToRight,
    /// 从右到左书写。
    RightToLeft,
}

/// 测量值使用的单位与数字格式，仅影响 [`DimensionGeometry::update_text`] 生成的文字。
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum UnitFormat {
    /// 十进制小数，保留 `decimal_places` 位小数。
    Decimal,
    /// 科学计数法（指数形式）。
    Scientific,
    /// 工程格式：按英尺-英寸输出，英寸部分保留 `decimal_places` 位小数。
    Engineering,
    /// 建筑格式；当前实现回退为十进制小数输出。
    Architectural,
    /// 分数格式：以 64 为分母取整为分数，输出形如 整英寸-分子/64。
    Fractional,
}

/// 公差在标注文字中的显示方式，默认不显示。
///
/// 除 `None` 之外，当前 [`DimensionGeometry::update_text`] 统一按
/// 「上偏差 测量值 下偏差」拼接，各形式的具体差别由绘制端体现。
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum ToleranceDisplay {
    /// 不显示公差，文字只含测量值。
    None,
    /// 对称公差：上下偏差相同。
    Symmetrical,
    /// 偏差形式：分别给出上、下偏差。
    Deviation,
    /// 极限尺寸形式：显示上下极限值。
    Limits,
    /// 基本尺寸形式，通常加框表示理论精确尺寸。
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

/// 标注的公共几何数据：定义点、测量值、显示文字与样式。
///
/// 定义点的含义随 [`DimensionType`] 变化（线性/对齐为两个端点，角度为中心与两个端点，
/// 半径/直径为圆心与圆周点），由各标注构造函数填充后再由 `calculate_measurement` 推导测量值。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DimensionGeometry {
    /// 测量类型，决定测量值的推导方式以及转换为实体后的标注类别。
    pub dimension_type: DimensionType,
    /// 定义点序列，均为世界坐标；缺失的点按原点处理。
    pub definition_points: Vec<Point>,
    /// 由定义点推导出的测量值：线性/对齐为长度，角度为弧度，半径/直径为半径或直径。
    pub measurement: f64,
    /// 最终显示的文字，由前缀、格式化后的测量值与后缀拼接而成。
    pub text: String,
    /// 该标注使用的样式副本。
    pub style: DimensionStyle,
    /// 文字相对其定位点的附着方式。
    pub attachment_point: AttachmentPoint,
    /// 用户指定的文字位置；为 `None` 时文字位置由调用方按标注类型自行推导。
    pub user_text_location: Option<Point>,
    /// 文字旋转角，单位为弧度。
    pub text_rotation: f64,
    /// 未经过取整与格式化的测量真值，供需要原始尺寸的场合使用。
    pub actual_measurement: f64,
}

impl DimensionGeometry {
    /// 创建空的标注几何：无定义点、测量值为 0、文字为空，附着点为 `MiddleCenter`。
    ///
    /// - `dimension_type`：测量类型，创建后仍可直接修改该字段。
    /// - `style`：标注样式，按值保存为副本。
    /// 返回的几何需先填充 `definition_points`，再调用 [`DimensionGeometry::calculate_measurement`]
    /// 才能得到测量值与文字。
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
    
    /// 依据 `dimension_type` 与 `definition_points` 重算测量值，并同步刷新显示文字。
    ///
    /// 线性与对齐取前两点的直线距离；半径取圆心到第二点的距离，直径取其两倍；
    /// 角度取中心到两点的夹角（弧度，折算到 0 至 π）。定义点不足，或类型为弧长、坐标、
    /// 基线、连续时不改变测量值。会就地写入 `measurement`、`actual_measurement` 与 `text`。
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
    
    /// 依据当前样式把 `measurement` 格式化为 `text`，不改变测量值。
    ///
    /// 文字为「前缀 + 数值 + 后缀」；`tolerance_display` 不为 `None` 时数值按
    /// 「上偏差 测量值 下偏差」拼接。数值格式由 `unit_format`、`decimal_places`
    /// 与 `round_off` 共同决定。会就地写入 `self.text`。
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

/// 标注文字相对其定位点的九宫格附着位置，用于决定文字对齐方式。
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum AttachmentPoint {
    /// 左上角。
    TopLeft,
    /// 上边中点。
    TopCenter,
    /// 右上角。
    TopRight,
    /// 左边中点。
    MiddleLeft,
    /// 正中心，[`DimensionGeometry::new`] 的默认值。
    MiddleCenter,
    /// 右边中点。
    MiddleRight,
    /// 左下角。
    BottomLeft,
    /// 下边中点。
    BottomCenter,
    /// 右下角。
    BottomRight,
}

/// 线性标注：由一条被测线段生成，测量值为两端点之间的直线距离。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LinearDimension {
    /// 定义点、测量值与显示文字，测量类型为 [`DimensionType::Linear`]。
    pub geometry: DimensionGeometry,
    /// 起点一侧的尺寸界线：自起点沿被测方向反向偏移后向外延伸。
    pub extension_line1: Line,
    /// 终点一侧的尺寸界线。
    pub extension_line2: Line,
    /// 尺寸线：自被测线段中点沿垂线方向偏移一段距离后给出。
    pub dimension_line: Line,
    /// 第一个箭头的尖端位置（自尺寸线端点沿其方向内缩 `arrow_size`）。
    pub arrow1: Point,
    /// 第二个箭头的尖端位置。
    pub arrow2: Point,
    /// 可选的中心标记；为 `Some` 时转换为实体后开启中心标记绘制。
    pub center_mark: Option<CenterMark>,
}

impl LinearDimension {
    /// 依据被测线段生成线性标注，并立即计算测量值与显示文字。
    ///
    /// - `definition_line`：被测线段，其两个端点作为定义点，生成的几何统一取 z = 0。
    /// - `style`：标注样式，按值保存为副本。
    /// - `location`：期望的尺寸线位置；当前实现未使用该参数，尺寸线位置由样式偏移推导。
    /// 返回的标注已含尺寸界线、尺寸线与箭头，`center_mark` 为 `None`。
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

/// 对齐标注：尺寸线与两个定义点的连线方向一致，适合测量倾斜对象。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AlignedDimension {
    /// 定义点、测量值与显示文字，测量类型为 [`DimensionType::Aligned`]。
    pub geometry: DimensionGeometry,
    /// 第一个定义点一侧的尺寸界线。
    pub extension_line1: Line,
    /// 第二个定义点一侧的尺寸界线。
    pub extension_line2: Line,
    /// 尺寸线：自两点中点沿垂直于测量方向偏移后给出。
    pub dimension_line: Line,
    /// 第一个箭头的尖端位置。
    pub arrow1: Point,
    /// 第二个箭头的尖端位置。
    pub arrow2: Point,
}

impl AlignedDimension {
    /// 依据两个定义点生成对齐标注，并立即计算测量值与显示文字。
    ///
    /// - `p1`、`p2`：被测的两点（世界坐标），测量值取两点的直线距离。
    /// - `style`：标注样式，按值保存为副本。
    /// - `location`：期望的尺寸线位置；当前实现未使用该参数。
    /// 两个点重合时方向归一化得到零向量，尺寸界线与尺寸线随之退化到定义点附近。
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

/// 角度标注：由中心与两个端点定义，测量值为两条射线之间的夹角（弧度）。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AngularDimension {
    /// 定义点（中心、起点、终点）、角度测量值与显示文字，测量类型为 [`DimensionType::Angular`]。
    pub geometry: DimensionGeometry,
    /// 标示角度的圆弧，半径取中心到两个定义点距离的较大者。
    pub arc: Arc,
    /// 自中心沿第一个定义点方向绘制的尺寸界线。
    pub extension_line1: Line,
    /// 自中心沿第二个定义点方向绘制的尺寸界线。
    pub extension_line2: Line,
    /// 文字位置：位于角平分线方向上，偏移量为圆弧半径加文字高与尺寸线间隙。
    pub text_location: Point,
}

impl AngularDimension {
    /// 依据角顶点与两条边上的点生成角度标注，并立即计算角度测量值与显示文字。
    ///
    /// - `center`：角的顶点。
    /// - `p1`、`p2`：两条边上的点，与顶点的距离不要求相等，圆弧半径取二者中的较大者。
    /// - `style`：标注样式，按值保存为副本。
    /// - `location`：期望的文字位置；当前实现未使用该参数，文字位置由角平分线方向推导。
    /// 测量值取两条射线的夹角并折算到 0 至 π 弧度，因此大于平角的角按补角记录。
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

/// 半径或直径标注：由圆与圆周上的点生成。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RadialDimension {
    /// 定义点（圆心、圆周点）、测量值与显示文字，测量类型为 [`DimensionType::Radial`] 或 `Diameter`。
    pub geometry: DimensionGeometry,
    /// 圆心处的中心标记，尺寸为箭头尺寸的两倍。
    pub center_mark: CenterMark,
    /// 尺寸线：自半径与圆的交点沿半径方向向外延伸一个文字高加间隙的距离。
    pub dimension_line: Line,
    /// 引线：连接传入的圆周点与尺寸线起点。
    pub extension_line: Line,
    /// 箭头尖端位置，位于圆周点内侧 `arrow_size` 处。
    pub arrow: Point,
}

/// 中心标记：记录圆心位置、标记尺寸与绘制形式。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CenterMark {
    /// 圆心位置（世界坐标）。
    pub center: Point,
    /// 标记的尺寸（图形单位），标注构造函数通常取箭头尺寸的两倍。
    pub size: f64,
    /// 标记的绘制形式，[`RadialDimension::new`] 固定使用 `Cross`。
    pub mark_type: CenterMarkType,
}

/// 中心标记的绘制形式。
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum CenterMarkType {
    /// 不绘制中心标记。
    None,
    /// 绘制中心点处的短十字标记。
    Mark,
    /// 绘制贯穿的中心十字线。
    Cross,
}

impl RadialDimension {
    /// 依据圆与圆周上的点生成半径或直径标注，并立即生成测量值与显示文字。
    ///
    /// - `circle`：被标注的圆，仅使用其圆心。
    /// - `p`：圆周上的点，决定尺寸线方向；测量值取圆心到该点的距离。
    /// - `style`：标注样式，按值保存为副本。
    /// - `is_diameter`：为 `true` 时生成直径标注（测量值为半径的两倍），否则生成半径标注。
    /// 生成的标注总带有 `Cross` 形式的中心标记，尺寸为 `arrow_size` 的两倍。
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

/// 坐标标注：以特征点的 X 或 Y 坐标作为测量值。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OrdinateDimension {
    /// 定义点（特征点、引线点）、坐标测量值与显示文字，测量类型为 [`DimensionType::Ordinate`]。
    pub geometry: DimensionGeometry,
    /// 被测特征点，其坐标即测量值。
    pub feature_point: Point,
    /// 引线端点，尺寸线由特征点连向该点。
    pub leader_point: Point,
    /// 从特征点到引线点的直线。
    pub dimension_line: Line,
    /// 测量 X 坐标还是 Y 坐标。
    pub orientation: OrdinateOrientation,
}

/// 坐标标注的测量方向。
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum OrdinateOrientation {
    /// 测量特征点的 X 坐标。
    X,
    /// 测量特征点的 Y 坐标。
    Y,
}

impl OrdinateDimension {
    /// 依据特征点、引线点与测量方向生成坐标标注，并立即生成测量值与显示文字。
    ///
    /// - `feature_point`：被测特征点，其 X 或 Y 坐标即测量值。
    /// - `leader_point`：引线端点，仅用于构造尺寸线，不参与测量。
    /// - `style`：标注样式，按值保存为副本。
    /// - `orientation`：选择测量 X 坐标还是 Y 坐标。
    /// 测量值不经 [`DimensionGeometry::calculate_measurement`]，直接由特征点坐标写入。
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

/// 求标注文字的默认位置：优先取用户指定的 `user_text_location`，否则取第一个定义点。
///
/// - `geometry`：待求位置的标注几何，只读，不被修改。
/// 返回 `user_text_location` 为 `None` 且没有定义点时的原点。
/// # 示例
/// ```text
/// let mut dim = DimensionGeometry::new(DimensionType::Linear, DimensionStyle::default());
/// dim.user_text_location = Some(Point::new(10.0, 5.0, 0.0));
/// let at = text_position_for(&dim); // Point::new(10.0, 5.0, 0.0)
/// ```
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

/// 把标注几何转换为可放进 [`Entity`] 的 [`EntityGeometry::Dimension`]。
///
/// - `geometry`：标注几何，提供测量值、文字、字高与文字旋转角；只读，不被修改。
/// - `text_position`：文字定位点（世界坐标），一般由 [`text_position_for`] 求得。
/// - `angle`：标注角度，单位为弧度，需由调用方按标注类型计算（线性与坐标标注传 0.0）。
/// - `extension_lines`：转换出的实体是否绘制尺寸界线。
/// - `center_marks`：转换出的实体是否绘制中心标记。
///
/// 定义点最多取前三个，缺失的以原点补齐，`def_point_3` 与 `def_point_4` 恒为原点；
/// 测量类型映射到 [`crate::data_structure::DimensionType`] 时，半径归入半径标注，
/// 基线与连续归入线性标注。
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

/// 各类具体标注的枚举封装，便于统一存储、序列化与分发。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum DimensionGeometryData {
    /// 线性标注。
    Linear(LinearDimension),
    /// 对齐标注。
    Aligned(AlignedDimension),
    /// 角度标注。
    Angular(AngularDimension),
    /// 半径或直径标注。
    Radial(RadialDimension),
    /// 坐标标注。
    Ordinate(OrdinateDimension),
}
