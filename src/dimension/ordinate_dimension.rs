//! 坐标标注的另一种轻量实现：以基准原点为参照，给出特征点在某一方向上的坐标值。
//! 与 `dimension::ordinate` 不同，这里的 `OrdinateDimension` 自带文字、公差、文字高度与图层等字段，
//! 不依赖 `DimensionStyle` 与 `DimensionGeometry`，构造后可直接读取字段自行渲染或序列化。
//!
//! 注意：本文件当前未在 `dimension::mod` 中声明（生效的坐标标注实现是 `ordinate.rs`），
//! 且 `tolerance` 字段引用的 `super::linear_dimension::DimensionTolerance` 在模块树中不存在，因此不参与编译。
use crate::geometry::Point;
use std::fmt;
use serde::{Serialize, Deserialize};

/// 坐标标注的测量方向标记。
/// 只用于区分标注来源与显示，不参与数值计算：测量值由 `x_dimension` / `y_dimension` 各自算出。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OrdinateDimensionType {
    /// X 向坐标标注，由 `x_dimension` 生成。
    X,
    /// Y 向坐标标注，由 `y_dimension` 生成。
    Y,
}

/// 单条坐标标注的轻量描述：直接保存文字、公差与图层等渲染所需字段。
/// 由 `x_dimension` / `y_dimension` 构造，`measurement` 恒为非负的坐标差绝对值；
/// 坐标为世界坐标；本类型不生成箭头、引出线等几何，需由调用者按字段自行绘制。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OrdinateDimension {
    /// 测量方向标记。
    pub dim_type: OrdinateDimensionType,
    /// 基准原点（世界坐标），测量值的参照点。
    pub origin: Point,
    /// 被标注的特征点（世界坐标）。
    pub feature_point: Point,
    /// 引出线终点；由构造方法按基准坐标与测量值回代得出，当前实现下与 `feature_point` 重合。
    pub leader_end: Point,
    /// 显示文字，构造时为测量值保留两位小数的字符串。
    pub text: String,
    /// 文字高度（世界单位）；构造时固定为 2.5，`is_valid` 要求它大于 0。
    pub text_height: f64,
    /// 箭头尺寸（世界单位）；构造时固定为 2.5，`is_valid` 要求它大于 0。
    pub arrow_size: f64,
    /// 引出线长度（世界单位）；本文件只把它计入 `bounding_box` 的外扩量，不据此生成几何。
    pub leader_length: f64,
    /// 测量值：构造时取特征点与基准原点坐标差的绝对值，因此恒为非负。
    pub measurement: f64,
    /// 是否以 X 基准（datum）标注；构造时为 false，供上层自行标记语义。
    pub is_ordinate_from_xdatum: bool,
    /// 基准偏移量；构造时为 0.0，本文件不使用该字段。
    pub datum_offset: f64,
    /// 尺寸公差；`None` 表示不显示公差，可用 `with_tolerance` 设置。
    pub tolerance: Option<super::linear_dimension::DimensionTolerance>,
    /// 所属图层标识；`None` 表示未指定（沿用当前图层）。
    pub layer_id: Option<String>,
}

impl OrdinateDimension {
    /// 构造 X 向坐标标注。
    /// - `origin`：基准原点，被标注对象的参照点。
    /// - `feature_point`：特征点；测量值取 `feature_point.y - origin.y` 的绝对值，即两点在 y 方向的差（本实现按 y 差值计算，与方向命名并不一致）。
    /// - `leader_length`：引出线长度，只记录到字段，供 `bounding_box` 外扩使用。
    /// 文字为测量值保留两位小数，`text_height` 与 `arrow_size` 固定为 2.5，公差与图层为 `None`；不修改入参、不写盘。
    #[inline]
    pub fn x_dimension(
        origin: Point,
        feature_point: Point,
        leader_length: f64,
    ) -> Self {
        let measurement = feature_point.y - origin.y;
        let leader_end = Point::new(
            feature_point.x,
            origin.y + measurement,
            feature_point.z,
        );

        Self {
            dim_type: OrdinateDimensionType::X,
            origin,
            feature_point,
            leader_end,
            text: format!("{:.2}", measurement.abs()),
            text_height: 2.5,
            arrow_size: 2.5,
            leader_length,
            measurement: measurement.abs(),
            is_ordinate_from_xdatum: false,
            datum_offset: 0.0,
            tolerance: None,
            layer_id: None,
        }
    }

    /// 构造 Y 向坐标标注。
    /// - `origin`：基准原点。
    /// - `feature_point`：特征点；测量值取 `feature_point.x - origin.x` 的绝对值，即两点在 x 方向的差（本实现按 x 差值计算，与方向命名并不一致）。
    /// - `leader_length`：引出线长度，只记录到字段。
    /// 其余字段与 `x_dimension` 相同：文字保留两位小数，`text_height`、`arrow_size` 为 2.5，公差与图层为空。
    #[inline]
    pub fn y_dimension(
        origin: Point,
        feature_point: Point,
        leader_length: f64,
    ) -> Self {
        let measurement = feature_point.x - origin.x;
        let leader_end = Point::new(
            origin.x + measurement,
            feature_point.y,
            feature_point.z,
        );

        Self {
            dim_type: OrdinateDimensionType::Y,
            origin,
            feature_point,
            leader_end,
            text: format!("{:.2}", measurement.abs()),
            text_height: 2.5,
            arrow_size: 2.5,
            leader_length,
            measurement: measurement.abs(),
            is_ordinate_from_xdatum: false,
            datum_offset: 0.0,
            tolerance: None,
            layer_id: None,
        }
    }

    /// 链式替换显示文字，返回设置后的标注。
    /// - `text`：完整文字内容，原样写入，不做单位换算或小数位处理。
    /// 按值消费 `self`；测量值与几何字段不受影响。
    #[inline]
    pub fn with_text(mut self, text: String) -> Self {
        self.text = text;
        self
    }

    /// 链式设置尺寸公差，返回设置后的标注。
    /// - `upper`：上偏差；`lower`：下偏差。
    /// 两者相差小于 1e-6 时公差记为对称公差（`is_symmetrical`），小数位固定为 2；按值消费 `self`，再次调用会覆盖原公差。
    #[inline]
    pub fn with_tolerance(mut self, upper: f64, lower: f64) -> Self {
        self.tolerance = Some(super::linear_dimension::DimensionTolerance {
            upper_tolerance: upper,
            lower_tolerance: lower,
            is_symmetrical: (upper - lower).abs() < 1e-6,
            decimal_places: 2,
        });
        self
    }

    /// 覆盖测量值并重新格式化文字（就地修改）。
    /// - `value`：新的测量值；`measurement` 取其绝对值，而文字按原值保留两位小数。
    /// 因此传入负数时文字带负号、`measurement` 仍为非负，两者可能不一致；几何点与公差不受影响。
    #[inline]
    pub fn set_measurement(&mut self, value: f64) {
        self.measurement = value.abs();
        self.text = format!("{:.2}", value);
    }

    /// 返回标注的三个定义点，顺序固定为 [基准原点, 特征点, 引出线终点]，供上层序列化或渲染使用。
    /// 不修改自身；返回的是当前字段的副本。
    #[inline]
    pub fn definition_points(&self) -> [Point; 3] {
        [
            self.origin,
            self.feature_point,
            self.leader_end,
        ]
    }

    /// 判断标注是否可用于渲染：要求 `measurement` 非负、`arrow_size` 与 `text_height` 均大于 0。
    /// 不校验文字是否为空、`leader_length` 是否为正，也不检查特征点是否与基准重合。
    #[inline]
    pub fn is_valid(&self) -> bool {
        self.measurement >= 0.0 &&
        self.arrow_size > 0.0 &&
        self.text_height > 0.0
    }

    /// 返回标注的包围盒，形式为 `(最小点, 最大点)`，两端点均含在盒内。
    /// 先取基准原点与特征点的坐标范围，再向四周各外扩 `leader_length`（负值会收缩），z 恒为 0。
    /// 未计入文字宽度、箭头实际尺寸与引出线方向，仅适合粗略的显示范围估算。
    #[inline]
    pub fn bounding_box(&self) -> (Point, Point) {
        let mut min_x = self.origin.x.min(self.feature_point.x);
        let mut min_y = self.origin.y.min(self.feature_point.y);
        let mut max_x = self.origin.x.max(self.feature_point.x);
        let mut max_y = self.origin.y.max(self.feature_point.y);

        min_x -= self.leader_length;
        min_y -= self.leader_length;
        max_x += self.leader_length;
        max_y += self.leader_length;

        (
            Point::new(min_x, min_y, 0.0),
            Point::new(max_x, max_y, 0.0)
        )
    }
}

impl fmt::Display for OrdinateDimension {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "OrdinateDimension(type: {:?}, measurement: {}, text: '{}')",
            self.dim_type,
            self.measurement,
            self.text
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_x_dimension() {
        let origin = Point::origin();
        let feature = Point::new(0.0, 10.0, 0.0);
        let dim = OrdinateDimension::x_dimension(origin, feature, 5.0);
        
        assert!(dim.is_valid());
        assert!((dim.measurement - 10.0).abs() < 1e-6);
        assert_eq!(dim.dim_type, OrdinateDimensionType::X);
    }

    #[test]
    fn test_y_dimension() {
        let origin = Point::origin();
        let feature = Point::new(15.0, 0.0, 0.0);
        let dim = OrdinateDimension::y_dimension(origin, feature, 5.0);
        
        assert!((dim.measurement - 15.0).abs() < 1e-6);
        assert_eq!(dim.dim_type, OrdinateDimensionType::Y);
    }

    #[test]
    fn test_negative_coordinate() {
        let origin = Point::new(0.0, 0.0, 0.0);
        let feature = Point::new(-8.0, -5.0, 0.0);
        let dim = OrdinateDimension::x_dimension(origin, feature, 3.0);
        
        assert!((dim.measurement - 5.0).abs() < 1e-6);
    }
}
