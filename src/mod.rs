//! 2D CAD SDK - Rust实现的专业CAD几何核心库
//!
//! 本库提供完整的2D CAD几何处理功能，包括几何曲线、标注、公差、图案填充、
//! 文字处理、图层管理、空间索引和约束求解等模块。
//!
//! # 主要特性
//!
//! - **几何模块**：支持多种2D几何曲线，包括渐开线、齿轮齿廓、椭圆弧等
//! - **标注模块**：完整的尺寸标注系统，包括线性、角度、半径、坐标标注
//! - **图案填充**：支持ANSI/ISO标准图案和渐变填充
//! - **文字处理**：多行文字、字段、样式系统
//! - **图层管理**：完整的图层控制、过滤、状态管理
//! - **空间索引**：R-Tree、四叉树、网格索引
//! - **约束求解**：几何约束和尺寸约束求解
//! - **API层**：命令系统、事件处理、Python绑定
//!
//! # 本文件的职责
//!
//! 作为聚合入口，本文件声明并再导出 `geometry`、`dimension`、`hatch` 等子模块，
//! 并提供进程级的 SDK 全局状态（[`initialize`]、[`get_config`]、[`shutdown`]）
//! 以及 `geom_tools`、`unit_conversion` 两个纯函数工具子模块。
//!
//! 全局状态只保存一份 [`SDKConfig`]，同一进程只允许初始化一次；工具函数不读写
//! 文档（Document），也不产生副作用。
//!
//! 单位与角度约定：长度使用图形单位，`unit_conversion` 以毫米为基准；`geom_tools`
//! 中角度同时存在「度」与「弧度」两套接口，使用前请确认函数名（如 `deg_to_rad`、
//! `normalize_angle`（度）、`normalize_angle_rad`（弧度））。

/// 几何曲线子模块。
pub mod geometry;
/// 尺寸标注子模块。
pub mod dimension;
/// 几何公差子模块。
pub mod geometric_tolerance;
/// 图案填充子模块。
pub mod hatch;
/// 文字处理子模块。
pub mod text;
/// 图层管理子模块。
pub mod layer;
/// 空间索引子模块。
pub mod spatial;
/// 约束求解子模块。
pub mod constraint;
/// API 聚合子模块（命令系统、事件处理等）。
pub mod api;

/// 再导出 `geometry` 的全部公开项。
pub use geometry::*;
/// 再导出 `dimension` 的全部公开项。
pub use dimension::*;
/// 再导出 `geometric_tolerance` 的全部公开项。
pub use geometric_tolerance::*;
/// 再导出 `hatch` 的全部公开项。
pub use hatch::*;
/// 再导出 `text` 的全部公开项。
pub use text::*;
/// 再导出 `layer` 的全部公开项。
pub use layer::*;
/// 再导出 `spatial` 的全部公开项。
pub use spatial::*;
/// 再导出 `constraint` 的全部公开项。
pub use constraint::*;
/// 再导出 `api` 的全部公开项。
pub use api::*;

/// SDK 版本号，按 `(主版本, 次版本, 修订号)` 排列。
pub const CAD_SDK_VERSION: (u32, u32, u32) = (0, 1, 0);

/// 返回形如 `"0.1.0"` 的版本字符串，内容取自 [`CAD_SDK_VERSION`]。
#[inline]
pub fn version() -> String {
    format!("{}.{}.{}", CAD_SDK_VERSION.0, CAD_SDK_VERSION.1, CAD_SDK_VERSION.2)
}

/// SDK 初始化配置：新建文档与新建标注时采用的默认单位与尺寸。
///
/// [`Default`] 给出常用缺省值（毫米、文字高 2.5、箭头 2.5、标注文字 3.5）。
#[derive(Debug, Clone)]
pub struct SDKConfig {
    /// 默认图形单位，决定长度值的解释方式与显示单位。
    pub default_units: DrawingUnits,
    /// 默认文字高度，单位为图形单位。
    pub default_text_height: f64,
    /// 默认标注箭头大小，单位为图形单位。
    pub default_arrow_size: f64,
    /// 默认标注文字高度，单位为图形单位。
    pub default_dim_text_height: f64,
    /// 新建实体默认归属的图层名。
    pub default_layer: String,
    /// 长度显示保留的小数位数。
    pub decimal_places: u32,
    /// 角度显示保留的小数位数（单位为度）。
    pub angular_precision: u32,
}

impl Default for SDKConfig {
    fn default() -> Self {
        Self {
            default_units: DrawingUnits::Millimeters,
            default_text_height: 2.5,
            default_arrow_size: 2.5,
            default_dim_text_height: 3.5,
            default_layer: String::from("0"),
            decimal_places: 4,
            angular_precision: 0,
        }
    }
}

/// SDK全局状态
struct GlobalState {
    config: SDKConfig,
    is_initialized: bool,
}

impl GlobalState {
    fn new() -> Self {
        Self {
            config: SDKConfig::default(),
            is_initialized: false,
        }
    }
}

static mut GLOBAL_STATE: Option<GlobalState> = None;

/// 初始化 SDK 全局状态。
///
/// 副作用：写入进程级全局状态；已初始化时直接返回 `Err("SDK已经初始化")`，
/// 不做任何修改。该函数不创建文档，也不写盘。
///
/// # 示例
///
/// ```
/// use cad_sdk::initialize;
///
/// let config = cad_sdk::SDKConfig::default();
/// initialize(config).expect("SDK初始化失败");
/// ```
pub fn initialize(config: SDKConfig) -> Result<(), String> {
    if unsafe { GLOBAL_STATE.is_some() } {
        return Err("SDK已经初始化".to_string());
    }

    unsafe {
        GLOBAL_STATE = Some(GlobalState::new());
    }
    Ok(())
}

/// 判断 SDK 全局状态是否已建立。
///
/// 返回 `true` 时表示可以读取 [`get_config`] 中的配置。
#[inline]
pub fn is_initialized() -> bool {
    unsafe { GLOBAL_STATE.is_some() }
}

/// 读取当前全局配置的副本。
///
/// 未初始化时返回 [`SDKConfig::default`]，不会失败；读取本身不修改全局状态。
pub fn get_config() -> SDKConfig {
    unsafe {
        GLOBAL_STATE.as_ref()
            .map(|s| s.config.clone())
            .unwrap_or_default()
    }
}

/// 用 `config` 整体替换全局配置。
///
/// 副作用：修改全局状态；未初始化时静默忽略，不返回错误也不自动初始化。
pub fn update_config(config: SDKConfig) {
    unsafe {
        if let Some(state) = GLOBAL_STATE.as_mut() {
            state.config = config;
        }
    }
}

/// 关闭 SDK 并清空全局配置。
///
/// 副作用：重置全局状态，之后 [`is_initialized`] 返回 `false`、[`get_config`] 返回
/// 默认值；已创建的文档（Document）与实体不受影响，可再次 [`initialize`]。
pub fn shutdown() {
    unsafe {
        GLOBAL_STATE = None;
    }
}

/// 便捷函数：创建名为 `name` 的空图形文档。
///
/// 不依赖全局状态，也不写入磁盘。
///
/// # 示例
///
/// ```
/// use cad_sdk::*;
///
/// let mut doc = create_document("MyDrawing".to_string());
/// ```
pub fn create_document(name: String) -> CADDocument {
    CADDocument::new(name)
}

/// 便捷函数：创建一个不含任何图层的图层管理器。
///
/// 常用图层需由调用者自行添加；该函数不读取全局配置。
///
/// # 示例
///
/// ```
/// use cad_sdk::*;
///
/// let mut layer_manager = create_layer_manager();
/// ```
pub fn create_layer_manager() -> LayerManager {
    LayerManager::new()
}

/// 便捷函数：创建图案库
///
/// # 示例
///
/// ```
/// use cad_sdk::*;
///
/// let pattern_library = PatternLibrary;
/// let ansi31 = pattern_library.get_pattern("ANSI31");
/// ```
pub fn get_pattern_library() -> impl HatchPatternProvider {
    PatternLibrary
}

/// 图案库提供者特征：按名称取出填充图案，并枚举全部可用名称。
pub trait HatchPatternProvider {
    /// 按名称查询填充图案。
    ///
    /// - `name`：图案名，需与实现登记的名称完全一致（标准库要求全大写，如 `"ANSI31"`）。
    /// 返回：命中时给出图案副本；名称未知时为 `None`。
    fn get_pattern(&self, name: &str) -> Option<HatchPattern>;
    /// 列出本库登记的全部图案名称。
    ///
    /// 返回：名称列表，顺序与库内定义顺序一致，可直接用于界面下拉框。
    fn available_patterns(&self) -> Vec<&'static str>;
}

/// 标准图案库：支持 ANSI31–ANSI38、ISO01–ISO05、BRICK、GRID、CROSS 共 16 种图案。
pub struct StandardPatternLibrary;

impl HatchPatternProvider for StandardPatternLibrary {
    fn get_pattern(&self, name: &str) -> Option<HatchPattern> {
        match name {
            "ANSI31" => Some(self.create_ansi31()),
            "ANSI32" => Some(self.create_ansi32()),
            "ANSI33" => Some(self.create_ansi33()),
            "ANSI34" => Some(self.create_ansi34()),
            "ANSI35" => Some(self.create_ansi35()),
            "ANSI36" => Some(self.create_ansi36()),
            "ANSI37" => Some(self.create_ansi37()),
            "ANSI38" => Some(self.create_ansi38()),
            "ISO01" => Some(self.create_iso01()),
            "ISO02" => Some(self.create_iso02()),
            "ISO03" => Some(self.create_iso03()),
            "ISO04" => Some(self.create_iso04()),
            "ISO05" => Some(self.create_iso05()),
            "BRICK" => Some(self.create_brick()),
            "GRID" => Some(self.create_grid()),
            "CROSS" => Some(self.create_cross()),
            _ => None,
        }
    }

    fn available_patterns(&self) -> Vec<&'static str> {
        vec![
            "ANSI31", "ANSI32", "ANSI33", "ANSI34", "ANSI35",
            "ANSI36", "ANSI37", "ANSI38", "ISO01", "ISO02",
            "ISO03", "ISO04", "ISO05", "BRICK", "GRID", "CROSS",
        ]
    }
}

impl StandardPatternLibrary {
    fn create_ansi31(&self) -> HatchPattern {
        let mut pattern = HatchPattern::new(
            "ANSI31".to_string(),
            "ANSI Iron, Brick, and Masonry".to_string(),
        );
        pattern = pattern.with_angle(45.0);
        pattern
    }

    fn create_ansi32(&self) -> HatchPattern {
        let mut pattern = HatchPattern::new(
            "ANSI32".to_string(),
            "ANSI Steel".to_string(),
        );
        pattern = pattern.with_angle(45.0);
        let mut line1 = HatchLine::new(45.0, 8.0);
        let mut line2 = HatchLine::new(135.0, 8.0);
        pattern = pattern.add_line(line1);
        pattern = pattern.add_line(line2);
        pattern
    }

    fn create_ansi33(&self) -> HatchPattern {
        let mut pattern = HatchPattern::new(
            "ANSI33".to_string(),
            "ANSI Bronze, Brass, Copper".to_string(),
        );
        let mut line1 = HatchLine::new(45.0, 4.0);
        let mut line2 = HatchLine::new(135.0, 4.0);
        pattern = pattern.add_line(line1);
        pattern = pattern.add_line(line2);
        pattern = pattern.as_double();
        pattern
    }

    fn create_ansi34(&self) -> HatchPattern {
        let mut pattern = HatchPattern::new(
            "ANSI34".to_string(),
            "ANSI Plastics".to_string(),
        );
        let mut line1 = HatchLine::new(45.0, 6.0);
        let mut line2 = HatchLine::new(135.0, 6.0);
        pattern = pattern.add_line(line1);
        pattern = pattern.add_line(line2);
        pattern = pattern.as_double();
        pattern
    }

    fn create_ansi35(&self) -> HatchPattern {
        let mut pattern = HatchPattern::new(
            "ANSI35".to_string(),
            "ANSI Hard Rock".to_string(),
        );
        let mut line1 = HatchLine::new(45.0, 5.0);
        let mut line2 = HatchLine::new(135.0, 5.0);
        pattern = pattern.add_line(line1);
        pattern = pattern.add_line(line2);
        pattern = pattern.as_double();
        pattern
    }

    fn create_ansi36(&self) -> HatchPattern {
        let mut pattern = HatchPattern::new(
            "ANSI36".to_string(),
            "ANSI Earth".to_string(),
        );
        let mut line1 = HatchLine::new(45.0, 12.0);
        let mut line2 = HatchLine::new(135.0, 12.0);
        pattern = pattern.add_line(line1);
        pattern = pattern.add_line(line2);
        pattern = pattern.as_double();
        pattern
    }

    fn create_ansi37(&self) -> HatchPattern {
        let mut pattern = HatchPattern::new(
            "ANSI37".to_string(),
            "ANSI Concrete".to_string(),
        );
        let mut line1 = HatchLine::new(45.0, 10.0);
        let mut line2 = HatchLine::new(135.0, 10.0);
        pattern = pattern.add_line(line1);
        pattern = pattern.add_line(line2);
        pattern = pattern.as_double();
        pattern
    }

    fn create_ansi38(&self) -> HatchPattern {
        let mut pattern = HatchPattern::new(
            "ANSI38".to_string(),
            "ANSI Lead, Zinc, Magnesium, Aluminum".to_string(),
        );
        let mut line1 = HatchLine::new(45.0, 2.0);
        let mut line2 = HatchLine::new(135.0, 2.0);
        pattern = pattern.add_line(line1);
        pattern = pattern.add_line(line2);
        pattern = pattern.as_double();
        pattern
    }

    fn create_iso01(&self) -> HatchPattern {
        HatchPattern::new("ISO01".to_string(), "ISO Light".to_string())
    }

    fn create_iso02(&self) -> HatchPattern {
        HatchPattern::new("ISO02".to_string(), "ISO Medium".to_string())
    }

    fn create_iso03(&self) -> HatchPattern {
        HatchPattern::new("ISO03".to_string(), "ISO Dense".to_string())
    }

    fn create_iso04(&self) -> HatchPattern {
        let mut pattern = HatchPattern::new(
            "ISO04".to_string(),
            "ISO Light double".to_string(),
        );
        pattern = pattern.as_double();
        pattern
    }

    fn create_iso05(&self) -> HatchPattern {
        let mut pattern = HatchPattern::new(
            "ISO05".to_string(),
            "ISO Medium double".to_string(),
        );
        pattern = pattern.as_double();
        pattern
    }

    fn create_brick(&self) -> HatchPattern {
        HatchPattern::new("BRICK".to_string(), "Brick pattern".to_string())
    }

    fn create_grid(&self) -> HatchPattern {
        HatchPattern::new("GRID".to_string(), "Grid pattern".to_string())
    }

    fn create_cross(&self) -> HatchPattern {
        HatchPattern::new("CROSS".to_string(), "Crosshatch pattern".to_string())
    }
}

/// 便捷函数：取得标准图案库（无需实例化状态）。
///
/// 返回：实现了 [`HatchPatternProvider`] 的 [`StandardPatternLibrary`]。
#[inline]
pub fn standard_pattern_library() -> impl HatchPatternProvider {
    StandardPatternLibrary
}

/// 常用几何计算工具：点、直线、圆与多边形之间的纯函数运算。
///
/// 坐标一律为图形单位；角度接口中 `deg_to_rad`/`rad_to_deg`/`normalize_angle`
/// 处理「度」，`normalize_angle_rad`/`angle_between_lines` 及角度容差参数处理「弧度」。
/// 除「返回」中注明外，函数不修改入参。
pub mod geom_tools {
    use super::*;

    /// 计算两条直线的交点。
    ///
    /// - `l1`、`l2`：参与求交的两条直线（按其起点/终点定义的线段）。
    /// - `extend`：为 `false` 时交点必须同时落在两条线段范围内；为 `true` 时按无限长直线求交。
    /// 返回：唯一交点；两线平行或重合（判定分母近似为 0，阈值 1e-10）时为 `None`。
    pub fn line_intersection(l1: &Line, l2: &Line, extend: bool) -> Option<Point> {
        let x1 = l1.start.x;
        let y1 = l1.start.y;
        let x2 = l1.end.x;
        let y2 = l1.end.y;
        let x3 = l2.start.x;
        let y3 = l2.start.y;
        let x4 = l2.end.x;
        let y4 = l2.end.y;

        let denom = (y4 - y3) * (x2 - x1) - (x4 - x3) * (y2 - y1);
        if denom.abs() < 1e-10 {
            return None;
        }

        let ua = ((x4 - x3) * (y1 - y3) - (y4 - y3) * (x1 - x3)) / denom;

        if !extend {
            if ua < 0.0 || ua > 1.0 {
                return None;
            }
        }

        let ub = ((x2 - x1) * (y1 - y3) - (y2 - y1) * (x1 - x3)) / denom;

        if !extend {
            if ub < 0.0 || ub > 1.0 {
                return None;
            }
        }

        Some(Point::new(
            x1 + ua * (x2 - x1),
            y1 + ua * (y2 - y1),
        ))
    }

    /// 计算点在直线段上的投影点。
    ///
    /// - `point`：待投影的点；`line`：目标线段。
    /// 返回：投影被限制在线段范围内，点落在线段之外时给出较近的端点；线段退化为一点时返回其起点。
    pub fn point_line_projection(point: &Point, line: &Line) -> Point {
        let dx = line.end.x - line.start.x;
        let dy = line.end.y - line.start.y;
        let len_sq = dx * dx + dy * dy;

        if len_sq < 1e-10 {
            return line.start;
        }

        let t = ((point.x - line.start.x) * dx + (point.y - line.start.y) * dy) / len_sq;
        let t = t.clamp(0.0, 1.0);

        Point::new(
            line.start.x + t * dx,
            line.start.y + t * dy,
        )
    }

    /// 计算点到直线段的最短距离。
    ///
    /// 返回：点到线段的欧氏距离，恒为非负；点在线段之外时取到较近端点的距离。
    pub fn point_line_distance(point: &Point, line: &Line) -> f64 {
        point.distance_to(&point_line_projection(point, line))
    }

    /// 计算两条线段的最短距离。
    ///
    /// 返回：两条直线（含延长线）相交时为 0，否则为四个端点到对侧线段距离中的最小值。
    pub fn line_line_distance(l1: &Line, l2: &Line) -> f64 {
        let intersection = line_intersection(l1, l2, true);
        if intersection.is_some() {
            return 0.0;
        }

        let d1 = point_line_distance(&l1.start, l2);
        let d2 = point_line_distance(&l1.end, l2);
        let d3 = point_line_distance(&l2.start, l1);
        let d4 = point_line_distance(&l2.end, l1);

        d1.min(d2).min(d3).min(d4)
    }

    /// 判断点是否落在线段上（含两个端点）。
    ///
    /// - `tolerance`：距离容差，单位为图形单位；距离必须严格小于该值才算命中。
    /// 返回：命中为 `true`；点在线段外或距离恰好等于容差时为 `false`。
    pub fn point_on_line(point: &Point, line: &Line, tolerance: f64) -> bool {
        point_line_distance(point, line) < tolerance
    }

    /// 计算多边形面积（鞋带公式，取绝对值，与顶点绕向无关）。
    ///
    /// 顶点按给定顺序首尾相连；返回：非负面积，顶点数少于 3 时返回 0。
    pub fn polygon_area(points: &[Point]) -> f64 {
        if points.len() < 3 {
            return 0.0;
        }

        let mut area = 0.0;
        for i in 0..points.len() {
            let j = (i + 1) % points.len();
            area += points[i].x * points[j].y;
            area -= points[j].x * points[i].y;
        }

        area.abs() / 2.0
    }

    /// 计算多边形的形心（面积质心）。
    ///
    /// 返回：顶点数不少于 3 且面积大于 1e-10 时给出形心；顶点过少或面积退化为 0 时为 `None`。
    pub fn polygon_centroid(points: &[Point]) -> Option<Point> {
        if points.len() < 3 {
            return None;
        }

        let area = polygon_area(points);
        if area < 1e-10 {
            return None;
        }

        let mut cx = 0.0;
        let mut cy = 0.0;

        for i in 0..points.len() {
            let j = (i + 1) % points.len();
            let factor = points[i].x * points[j].y - points[j].x * points[i].y;
            cx += (points[i].x + points[j].x) * factor;
            cy += (points[i].y + points[j].y) * factor;
        }

        cx /= 6.0 * area;
        cy /= 6.0 * area;

        Some(Point::new(cx, cy))
    }

    /// 判断点是否在多边形内部（射线法）。
    ///
    /// 顶点需按顺序给出并首尾相连。返回：内部为 `true`、外部为 `false`；
    /// 恰好落在边界上的点结果不确定，需要精确判边界时请配合 [`point_on_line`]。
    pub fn point_in_polygon(point: &Point, points: &[Point]) -> bool {
        let mut inside = false;
        let n = points.len();

        for i in 0..n {
            let j = (i + 1) % n;
            let xi = points[i].x;
            let yi = points[i].y;
            let xj = points[j].x;
            let yj = points[j].y;

            if ((yi > point.y) != (yj > point.y)) &&
               (point.x < (xj - xi) * (point.y - yi) / (yj - yi) + xi) {
                inside = !inside;
            }
        }

        inside
    }

    /// 把角度值从「度」换算为「弧度」。
    #[inline]
    pub fn deg_to_rad(degrees: f64) -> f64 {
        degrees * std::f64::consts::PI / 180.0
    }

    /// 把角度值从「弧度」换算为「度」。
    #[inline]
    pub fn rad_to_deg(radians: f64) -> f64 {
        radians * 180.0 / std::f64::consts::PI
    }

    /// 把以「度」为单位的角度规范化到 [0, 360) 区间。
    ///
    /// 返回：与输入等价的角度；负值加 360 后落入区间，输入 360 返回 0。
    #[inline]
    pub fn normalize_angle(angle: f64) -> f64 {
        let mut angle = angle % 360.0;
        if angle < 0.0 {
            angle += 360.0;
        }
        angle
    }

    /// 把以「弧度」为单位的角度规范化到 [0, 2π) 区间。
    ///
    /// 返回：与输入等价的弧度值；负值加 2π 后落入区间。
    #[inline]
    pub fn normalize_angle_rad(angle: f64) -> f64 {
        let mut angle = angle % (2.0 * std::f64::consts::PI);
        if angle < 0.0 {
            angle += 2.0 * std::f64::consts::PI;
        }
        angle
    }

    /// 在 `a` 与 `b` 之间做线性插值。
    ///
    /// - `t`：插值系数，会被截断到 [0, 1]，因此结果不会越出 `a`、`b` 之间。
    /// 返回：`t = 0` 时为 `a`，`t = 1` 时为 `b`，中间值按比例过渡。
    #[inline]
    pub fn lerp(a: f64, b: f64, t: f64) -> f64 {
        a + (b - a) * t.clamp(0.0, 1.0)
    }

    /// 对两个点做线性插值，坐标逐分量使用 [`lerp`]。
    ///
    /// - `t`：插值系数，同样被截断到 [0, 1]。
    /// 返回：位于 `p1` 与 `p2` 连线上的点。
    #[inline]
    pub fn point_lerp(p1: &Point, p2: &Point, t: f64) -> Point {
        Point::new(
            lerp(p1.x, p2.x, t),
            lerp(p1.y, p2.y, t),
        )
    }

    /// 计算两条直线的夹角。
    ///
    /// 返回：以弧度表示的夹角，范围 [0, π]；与直线方向无关，取夹角与补角中的较小者。
    pub fn angle_between_lines(l1: &Line, l2: &Line) -> f64 {
        let a1 = l1.start.angle_to(l1.end);
        let a2 = l2.start.angle_to(l2.end);
        let diff = a2 - a1;
        diff.abs().min(2.0 * std::f64::consts::PI - diff.abs())
    }

    /// 计算点到圆周的距离（点到圆心的距离减去半径）。
    ///
    /// 返回：带符号的径向距离——点在圆外为正、圆上为 0、圆内为负。
    pub fn point_circle_distance(point: &Point, circle: &Circle) -> f64 {
        point.distance_to(&circle.center) - circle.radius
    }

    /// 计算点到圆弧的最短距离。
    ///
    /// 圆弧起止角以弧度表示并按逆时针方向展开：点的方位角落在弧内时取径向距离的
    /// 绝对值，落在弧外时取到较近端点的距离。
    /// 返回：最短欧氏距离，恒为非负。
    pub fn point_arc_distance(point: &Point, arc: &Arc) -> f64 {
        let center_dist = point.distance_to(&arc.center);
        let radial_dist = center_dist - arc.radius;

        let angle = arc.center.angle_to(*point);
        let angle = normalize_angle_rad(angle);

        let start = normalize_angle_rad(arc.start_angle);
        let end = normalize_angle_rad(arc.end_angle);

        let on_arc = if start <= end {
            angle >= start && angle <= end
        } else {
            angle >= start || angle <= end
        };

        if on_arc {
            radial_dist.abs()
        } else {
            let start_dist = point.distance_to(&Point::new(
                arc.center.x + arc.radius * start.cos(),
                arc.center.y + arc.radius * start.sin(),
            ));
            let end_dist = point.distance_to(&Point::new(
                arc.center.x + arc.radius * end.cos(),
                arc.center.y + arc.radius * end.sin(),
            ));

            start_dist.min(end_dist)
        }
    }

    /// 判断两条直线是否平行。
    ///
    /// - `tolerance`：方向夹角容差，单位为弧度；方向差折算到 [0, π] 后严格小于该值才算平行。
    /// 返回：平行为 `true`；共线也算平行。
    pub fn lines_parallel(l1: &Line, l2: &Line, tolerance: f64) -> bool {
        let angle1 = l1.start.angle_to(l1.end);
        let angle2 = l2.start.angle_to(l2.end);
        let diff = (angle1 - angle2).abs();
        diff.min(2.0 * std::f64::consts::PI - diff) < tolerance
    }

    /// 判断两条直线是否垂直。
    ///
    /// - `tolerance`：角度容差，单位为弧度；方向差与 π/2 的偏差严格小于该值才算垂直。
    /// 返回：垂直为 `true`。
    pub fn lines_perpendicular(l1: &Line, l2: &Line, tolerance: f64) -> bool {
        let angle1 = l1.start.angle_to(l1.end);
        let angle2 = l2.start.angle_to(l2.end);
        let diff = ((angle1 - angle2) % std::f64::consts::PI).abs();
        (diff - std::f64::consts::PI / 2.0).abs() < tolerance
    }
}

/// 单位转换工具：以毫米为基准在各图形单位之间换算。
///
/// 长度单位使用固定因子（1 厘米 = 10 毫米、1 米 = 1000 毫米、1 英寸 = 25.4 毫米、
/// 1 英尺 = 304.8 毫米）；角度及其他未列出的单位因子为 1，即不参与换算。
pub mod unit_conversion {
    use super::*;

    /// 单位转换因子（相对于毫米）
    const MM_FACTOR: f64 = 1.0;
    const CM_FACTOR: f64 = 10.0;
    const M_FACTOR: f64 = 1000.0;
    const INCH_FACTOR: f64 = 25.4;
    const FT_FACTOR: f64 = 304.8;

    /// 把数值从一个单位换算为另一个单位。
    ///
    /// - `value`：按 `from` 单位解释的数值；`from`、`to`：源单位与目标单位。
    /// 返回：按 `to` 单位表示的等值数值；角度类单位因子为 1，原值返回。
    pub fn convert(value: f64, from: DrawingUnits, to: DrawingUnits) -> f64 {
        let mm_value = value * get_factor(from);
        mm_value / get_factor(to)
    }

    /// 获取单位转换因子
    fn get_factor(unit: DrawingUnits) -> f64 {
        match unit {
            DrawingUnits::Millimeters => MM_FACTOR,
            DrawingUnits::Centimeters => CM_FACTOR,
            DrawingUnits::Meters => M_FACTOR,
            DrawingUnits::Inches => INCH_FACTOR,
            DrawingUnits::Feet => FT_FACTOR,
            _ => MM_FACTOR,
        }
    }

    /// 毫米转英寸（除以 25.4）。
    #[inline]
    pub fn mm_to_inch(mm: f64) -> f64 {
        mm / INCH_FACTOR
    }

    /// 英寸转毫米（乘以 25.4）。
    #[inline]
    pub fn inch_to_mm(inch: f64) -> f64 {
        inch * INCH_FACTOR
    }

    /// 毫米转英尺（除以 304.8）。
    #[inline]
    pub fn mm_to_foot(mm: f64) -> f64 {
        mm / FT_FACTOR
    }

    /// 英尺转毫米（乘以 304.8）。
    #[inline]
    pub fn foot_to_mm(foot: f64) -> f64 {
        foot * FT_FACTOR
    }
}

/// 再导出 `geom_tools` 的全部公开项。
pub use geom_tools::*;
/// 再导出 `unit_conversion` 的全部公开项。
pub use unit_conversion::*;
