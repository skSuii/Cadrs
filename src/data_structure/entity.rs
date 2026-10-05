//! 实体定义：图纸中的最小可绘制单元及其几何数据。
//!
//! [`Entity`] 由"通用属性 + 几何数据"两部分组成：通用属性（[`ObjectId`]、图层、可见性、
//! [`Transform`]）对所有实体一致，具体形状则存放在 [`EntityGeometry`] 枚举中，并用
//! [`EntityType`] 标记其类别。文本、标注、块参照、填充、实心体等复合几何以结构体变体形式
//! 内联在枚举里，字段含义见各变体说明。长度单位为文档单位，角度字段一律使用弧度。

use crate::geometry::{Point, Line, Circle, Arc, Ellipse, Polyline, BSpline, NURBS};
use crate::data_structure::ObjectId;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::any::Any;

/// 实体的几何类别标记，用于在不展开 [`EntityGeometry`] 的情况下做快速分类与分发。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum EntityType {
    /// 点实体。
    Point,
    /// 直线段实体。
    Line,
    /// 圆实体。
    Circle,
    /// 圆弧实体。
    Arc,
    /// 椭圆（弧）实体。
    Ellipse,
    /// 折线实体，可开可闭。
    Polyline,
    /// B 样条曲线实体。
    BSpline,
    /// NURBS 曲线实体。
    NURBS,
    /// 标注（尺寸）实体。
    Dimension,
    /// 单行/多行文本实体。
    Text,
    /// 块参照实体，指向块定义的一次插入。
    BlockRef,
    /// 光栅图像实体。
    Image,
    /// 图案填充实体。
    Hatch,
    /// 实心填充多边形实体。
    Solid,
}

/// 图纸中的一个可绘制实体：通用属性（标识、图层、可见性、变换）加一份几何数据。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entity {
    /// 全局唯一标识，由 [`Entity::new`] 自动生成，作为文档哈希表的键。
    pub id: ObjectId,
    /// 几何类别标记，应与 `geometry` 的实际变体一致（由工厂函数或 [`Entity::new`] 的调用方保证）。
    pub entity_type: EntityType,
    /// 所属图层的标识；新建实体时为空标识 [`ObjectId::nil`]，表示尚未归入任何图层。
    pub layer_id: ObjectId,
    /// 自定义扩展属性键值对，不参与几何运算。
    pub properties: HashMap<String, String>,
    /// 显示与编辑状态，默认 [`Visibility::Visible`]。
    pub visibility: Visibility,
    /// 施加于几何数据的平移/旋转/缩放，默认单位变换。
    pub transform: Transform,
    /// 具体几何数据及其全部形状参数。
    pub geometry: EntityGeometry,
}

/// 实体的显示与编辑状态。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Visibility {
    /// 正常显示且可编辑。
    Visible,
    /// 不显示，但仍可被选择与编辑。
    Hidden,
    /// 冻结：既不显示也不参与重生成等计算。
    Frozen,
    /// 锁定：显示但不可编辑。
    Locked,
}

/// 作用在实体几何上的刚体/缩放变换。
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Transform {
    /// 平移量，三个分量与文档单位一致。
    pub translation: Point,
    /// 绕原点旋转的角度，单位为弧度。
    pub rotation: f64,
    /// 各向同性缩放系数，1.0 表示不缩放。
    pub scale: f64,
}

/// 实体的几何数据，一个变体对应一种 [`EntityType`]。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EntityGeometry {
    /// 单个点，坐标为文档单位。
    Point(Point),
    /// 直线段，由起点与终点确定，含两端点。
    Line(Line),
    /// 圆，由圆心与半径确定。
    Circle(Circle),
    /// 圆弧，由圆心、半径与起止角确定，角度为弧度。
    Arc(Arc),
    /// 椭圆或椭圆弧，由圆心、长短半轴与旋转角确定。
    Ellipse(Ellipse),
    /// 折线，顶点序列加闭合标志。
    Polyline(Polyline),
    /// B 样条曲线，含控制点与阶数。
    BSpline(BSpline),
    /// NURBS 曲线，含控制点、节点向量与权重。
    NURBS(NURBS),
    /// 文本内容及其排版参数。
    Text {
        /// 显示的文本内容。
        content: String,
        /// 文本基线的插入点坐标。
        position: Point,
        /// 字高（大写字母高度），单位为文档单位。
        height: f64,
        /// 文本旋转角，单位为弧度。
        rotation: f64,
        /// 字宽因子，1.0 为不压缩不拉伸。
        width_factor: f64,
        /// 字体名，如 `"Arial"`。
        font_name: String,
        /// 加粗/斜体/下划线及对齐方式。
        style: TextStyle,
    },
    /// 尺寸标注数据，测量值由各定义点算出并缓存于 `measurement`。
    Dimension {
        /// 标注类型，决定各定义点的解释方式。
        dim_type: DimensionType,
        /// 测量结果，长度或角度（弧度），由定义点在创建时算得。
        measurement: f64,
        /// 标注文字的显示内容，可为已替换的自定义文本。
        text: String,
        /// 标注文字的放置点坐标。
        text_position: Point,
        /// 标注文字的字高，单位为文档单位。
        text_height: f64,
        /// 标注文字的旋转角，单位为弧度。
        text_rotation: f64,
        /// 标注线位置定义点，控制尺寸线与文字的落位。
        definition_point: Point,
        /// 第一条尺寸界线起点。
        def_point_1: Point,
        /// 第二条尺寸界线起点。
        def_point_2: Point,
        /// 第三条定义点，角标注等类型使用。
        def_point_3: Point,
        /// 第四条定义点，角标注等类型使用。
        def_point_4: Point,
        /// 角标注的夹角，单位为弧度。
        angle: f64,
        /// 是否绘制尺寸界线。
        extension_lines: bool,
        /// 是否绘制圆心标记（直径/半径标注使用）。
        center_marks: bool,
    },
    /// 块参照：对块定义的一次插入。
    BlockRef {
        /// 被引用块的名称，块删除后该引用将无法解析。
        block_name: String,
        /// 插入点坐标，对应块定义中的插入基点。
        position: Point,
        /// X 方向缩放系数。
        scale_x: f64,
        /// Y 方向缩放系数。
        scale_y: f64,
        /// Z 方向缩放系数。
        scale_z: f64,
        /// 插入旋转角，单位为弧度。
        rotation: f64,
        /// 阵列插入的列数，1 表示单列。
        column_count: u32,
        /// 阵列插入的行数，1 表示单行。
        row_count: u32,
        /// 阵列列间距，单位为文档单位。
        column_spacing: f64,
        /// 阵列行间距，单位为文档单位。
        row_spacing: f64,
    },
    /// 图案填充：一组边界路径加填充样式。
    Hatch {
        /// 图案名，`"SOLID"` 表示实心填充。
        pattern_name: String,
        /// 图案缩放系数，大于 1 疏化图案。
        pattern_scale: f64,
        /// 图案旋转角，单位为弧度。
        pattern_angle: f64,
        /// 为真时忽略图案名、直接按 `fill_color` 实心填充。
        solid_fill: bool,
        /// 填充颜色的 RGB 分量。
        fill_color: (u8, u8, u8),
        /// 边界路径集合，含外边界与孔洞。
        boundary_paths: Vec<HatchBoundary>,
        /// 为真时边界改变后填充自动重新生成。
        associativity: bool,
    },
    /// 实心填充多边形，固定四个顶点。
    Solid {
        /// 四个顶点，三角形填充时重复最后一个顶点。
        points: [Point; 4],
        /// 填充颜色的 RGB 分量。
        color: (u8, u8, u8),
    },
}

/// 标注的测量方式，决定标注定义点如何被解释与测量。
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum DimensionType {
    /// 线性标注：测量两点在指定方向上的距离。
    Linear,
    /// 对齐标注：测量两点间的真实距离，尺寸线与两点连线平行。
    Aligned,
    /// 角度标注：测量两线夹角，结果以弧度存入测量值。
    Angular,
    /// 直径标注：测量圆的直径并加注 `⌀` 前缀。
    Diameter,
    /// 半径标注：测量圆弧半径并加注 `R` 前缀。
    Radius,
    /// 坐标标注：标注点相对原点的 X 或 Y 坐标值。
    Ordinate,
    /// 弧长标注：测量圆弧长度并加注弧长符号。
    ArcLength,
    /// 坐标式标注：以坐标形式标注点位。
    Coordinate,
}

/// 文本的字符样式与对齐方式。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TextStyle {
    /// 是否加粗。
    pub bold: bool,
    /// 是否斜体。
    pub italic: bool,
    /// 是否加下划线。
    pub underline: bool,
    /// 相对于插入点的对齐方式。
    pub alignment: TextAlignment,
}

impl Default for TextStyle {
    fn default() -> Self {
        Self {
            bold: false,
            italic: false,
            underline: false,
            alignment: TextAlignment::Left,
        }
    }
}

/// 文本相对插入点的水平对齐方式。
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum TextAlignment {
    /// 左对齐：插入点在文本左端。
    Left,
    /// 居中对齐：插入点在文本水平中点。
    Center,
    /// 右对齐：插入点在文本右端。
    Right,
    /// 垂直居中：插入点在文本高度中点。
    Middle,
}

/// 填充的一条边界路径，由若干首尾相接的边构成。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HatchBoundary {
    /// 边界类型，决定该路径是外边界还是孔洞。
    pub boundary_type: BoundaryType,
    /// 构成路径的边，按顺序首尾相接；最后一条边的终点回到第一条边的起点时形成闭合环。
    pub edges: Vec<HatchEdge>,
    /// 是否为最外层边界；外边界为真，孔洞为假。
    pub is_outer: bool,
    /// 为真时路径本身即一条闭合折线，边列表只作几何近似使用。
    pub is_polyline: bool,
}

/// 填充边界的语义分类。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BoundaryType {
    /// 外部边界：最外围的填充范围。
    External,
    /// 外边界：与外层边界平行的次级外边界。
    Outer,
    /// 孔洞边界：从填充区域中挖去的部分，填充时按奇偶规则排除。
    Hole,
    /// 派生边界：由其他图元计算得出、非用户直接指定的边界。
    Derived,
}

/// 填充边界上的一条边。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HatchEdge {
    /// 边的几何类型，决定哪些可选字段有效。
    pub edge_type: EdgeType,
    /// 起点坐标。
    pub start_point: Point,
    /// 终点坐标。
    pub end_point: Point,
    /// 圆弧/椭圆弧的圆心，直线边为 [`None`]。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub center_point: Option<Point>,
    /// 圆弧半径，直线边为 [`None`]。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub radius: Option<f64>,
    /// 起始角（弧度），直线边为 [`None`]。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_angle: Option<f64>,
    /// 终止角（弧度），直线边为 [`None`]。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_angle: Option<f64>,
    /// 凸度：弧段圆心角四分之一的正切值，0 表示直线边。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bulge: Option<f64>,
}

/// 填充边的几何类型。
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum EdgeType {
    /// 直线边。
    Line,
    /// 圆弧边，使用圆心、半径与起止角字段。
    Arc,
    /// 椭圆弧边，使用圆心与起止角字段。
    Ellipse,
}

impl Default for Visibility {
    fn default() -> Self {
        Visibility::Visible
    }
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            translation: Point::origin(),
            rotation: 0.0,
            scale: 1.0,
        }
    }
}

impl Entity {
    /// 新建实体：自动生成新 [`ObjectId`]，图层置为空标识，属性表为空，可见性与变换取默认值。
    ///
    /// - `entity_type`：几何类别，必须与 `geometry` 的实际变体一致；
    /// - `geometry`：实体几何数据。
    #[inline]
    pub fn new(entity_type: EntityType, geometry: EntityGeometry) -> Self {
        Self {
            id: ObjectId::new(),
            entity_type,
            layer_id: ObjectId::nil(),
            properties: HashMap::new(),
            visibility: Visibility::default(),
            transform: Transform::default(),
            geometry,
        }
    }

    /// 实体标识，可复制后作为文档中各表的查询键。
    #[inline]
    pub fn id(&self) -> &ObjectId {
        &self.id
    }

    /// 实体的几何类别，仅作分类用，不反映几何参数。
    #[inline]
    pub fn entity_type(&self) -> &EntityType {
        &self.entity_type
    }

    /// 所属图层的标识；返回空标识 [`ObjectId::nil`] 表示该实体尚未归入任何图层。
    #[inline]
    pub fn layer_id(&self) -> &ObjectId {
        &self.layer_id
    }

    /// 将实体改挂到指定图层，会修改 `self`；图层不存在时文档侧查询会失败。
    #[inline]
    pub fn set_layer_id(&mut self, layer_id: ObjectId) {
        self.layer_id = layer_id;
    }

    /// 自定义属性表的只读视图，属性不参与几何运算。
    #[inline]
    pub fn properties(&self) -> &HashMap<String, String> {
        &self.properties
    }

    /// 写入或覆盖一个自定义属性，会修改 `self`。
    ///
    /// - `key`：属性名，同名时旧值被覆盖；
    /// - `value`：属性值，以字符串保存。
    #[inline]
    pub fn set_property(&mut self, key: String, value: String) {
        self.properties.insert(key, value);
    }

    /// 按键查询自定义属性，键不存在时返回 [`None`]。
    #[inline]
    pub fn get_property(&self, key: &str) -> Option<&String> {
        self.properties.get(key)
    }

    /// 当前显示与编辑状态。
    #[inline]
    pub fn visibility(&self) -> Visibility {
        self.visibility
    }

    /// 设置显示与编辑状态，会修改 `self`；冻结或隐藏的实体不参与显示。
    #[inline]
    pub fn set_visibility(&mut self, visibility: Visibility) {
        self.visibility = visibility;
    }

    /// 当前变换（平移、旋转、缩放）。
    #[inline]
    pub fn transform(&self) -> Transform {
        self.transform
    }

    /// 整体替换变换，会修改 `self`。
    #[inline]
    pub fn set_transform(&mut self, transform: Transform) {
        self.transform = transform;
    }

    /// 借用变换的可变引用以便就地修改，会修改 `self`。
    #[inline]
    pub fn transform_mut(&mut self) -> &mut Transform {
        &mut self.transform
    }

    /// 几何的特征位置：点取自身、直线取起点、圆/圆弧/椭圆取圆心、文本与标注取插入点、块参照取插入点。
    ///
    /// 返回几何类型没有特征位置（折线、样条、填充、实心体等）时返回 [`None`]。
    #[inline]
    pub fn get_position(&self) -> Option<Point> {
        match &self.geometry {
            EntityGeometry::Point(p) => Some(*p),
            EntityGeometry::Line(l) => Some(l.start),
            EntityGeometry::Circle(c) => Some(c.center),
            EntityGeometry::Arc(a) => Some(a.center),
            EntityGeometry::Ellipse(e) => Some(e.center),
            EntityGeometry::Text { position, .. } => Some(*position),
            EntityGeometry::Dimension { definition_point, .. } => Some(*definition_point),
            EntityGeometry::BlockRef { position, .. } => Some(*position),
            _ => None,
        }
    }

    /// 几何数据的只读引用，配合 [`Entity::entity_type`] 判类后使用。
    #[inline]
    pub fn geometry(&self) -> &EntityGeometry {
        &self.geometry
    }

    /// 几何数据的可变引用，就地改写后实体的 [`Entity::bounding_box`] 等派生结果随之变化。
    #[inline]
    pub fn geometry_mut(&mut self) -> &mut EntityGeometry {
        &mut self.geometry
    }

    /// 轴对齐包围盒，返回 `(最小角点, 最大角点)` 两点，Z 分量固定为 0。
    ///
    /// 只对点、直线、圆实现：点返回退化为一点的一对坐标；直线按两端点取最小/最大值；
    /// 圆按正外接正方形计算。其余几何类型（圆弧、椭圆、折线、样条、文本、标注、填充等）
    /// 返回 [`None`]，调用者需自行处理或改用细分结果。
    ///
    /// # 示例
    /// ```text
    /// let line = Line::new(Point::origin(), Point::new(1.0, 0.0, 0.0));
    /// let entity = Entity::new(EntityType::Line, EntityGeometry::Line(line));
    /// let (min, max) = entity.bounding_box().unwrap(); // min = (0,0,0), max = (1,0,0)
    /// ```
    #[inline]
    pub fn bounding_box(&self) -> Option<(Point, Point)> {
        match &self.geometry {
            EntityGeometry::Point(p) => Some((*p, *p)),
            EntityGeometry::Line(l) => {
                let min = Point::new(l.start.x.min(l.end.x), l.start.y.min(l.end.y), 0.0);
                let max = Point::new(l.start.x.max(l.end.x), l.start.y.max(l.end.y), 0.0);
                Some((min, max))
            }
            EntityGeometry::Circle(c) => {
                let min = Point::new(c.center.x - c.radius, c.center.y - c.radius, 0.0);
                let max = Point::new(c.center.x + c.radius, c.center.y + c.radius, 0.0);
                Some((min, max))
            }
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_entity_creation() {
        let line = Line::new(Point::origin(), Point::new(1.0, 0.0, 0.0));
        let entity = Entity::new(EntityType::Line, EntityGeometry::Line(line));
        
        assert_eq!(entity.entity_type(), &EntityType::Line);
        assert_eq!(entity.visibility(), Visibility::Visible);
    }

    #[test]
    fn test_entity_properties() {
        let point = Point::origin();
        let mut entity = Entity::new(EntityType::Point, EntityGeometry::Point(point));

        entity.set_property("color".to_string(), "red".to_string());
        assert_eq!(entity.get_property("color"), Some(&"red".to_string()));
    }
}
