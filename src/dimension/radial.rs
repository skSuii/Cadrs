//! 半径与直径尺寸标注：以圆或圆弧为测量对象，算出箭头、引出线与标注文字所需的几何。
//! 提供三种形态：`RadialDimension`（半径 / 直径标注，可选折弯 joggle）、
//! `SmallRadialDimension`（圆内空间不足时用一小段圆弧引出文字）、`DiameterDimension`（穿过圆心的直径标注）。
//! 三者都复用 `dimension::linear` 的 `DimensionGeometry` 与 `DimensionStyle`，并可经 `From` 转换为 `Entity`，
//! 从而进入统一的渲染、拾取与导出链路。所有坐标均为世界坐标（世界单位与图纸单位一致），
//! 几何在构造时一次性算出，之后不再随圆或样式变化而自动更新。
use crate::geometry::{Point, Vector2, Circle, Arc, Line};
use crate::data_structure::{Entity, EntityType, EntityGeometry};
use super::linear::{DimensionGeometry, DimensionStyle, DimensionType};
use serde::{Serialize, Deserialize};
use std::fmt;

/// 半径或直径标注的完整几何描述。
/// 由 `new_radial` / `new_diameter` 构造，`is_diameter` 决定 `geometry.measurement` 是半径还是直径；
/// 全部坐标为世界坐标，各几何字段在构造后不会自动跟随被测圆变化。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RadialDimension {
    /// 标注通用数据：`definition_points` 依次为圆心、测量点（直径标注再附第二个端点），`measurement` 为半径或直径。
    pub geometry: DimensionGeometry,
    /// 圆心处的十字标记，尺寸取样式箭头大小的两倍。
    pub center_mark: CenterMark,
    /// 标注线：自测量点沿径向向外引出的短线段，只用于承载箭头与文字，不参与测长。
    pub dimension_line: Line,
    /// 引出线：半径标注为「测量点 → 圆心」，直径标注为「p1 → p2」的直径连线。
    pub extension_line: Line,
    /// 箭头锚点：由测量点沿指向朝圆心一侧偏移一个箭头尺寸得到，世界坐标。
    /// 转为 `Entity` 时该点被当作文字位置传入（见 `impl From<RadialDimension> for Entity`），因此也决定标注文字的落点。
    pub arrow: Point,
    /// 被测圆的圆心，世界坐标。
    pub center_point: Point,
    /// 测量点：半径标注为圆周上的取点，直径标注为构造时的 `p1`。
    pub arc_point: Point,
    /// 是否为直径标注；为 true 时测量值是两倍半径，为 false 时就是半径。
    pub is_diameter: bool,
    /// 是否启用折弯（joggle），只有调用 `with_joggle` 后才会变为 true。
    pub joggle_enabled: bool,
    /// 折弯位置；未启用折弯时为 `None`，本模块不会自动推算该点。
    pub joggle_location: Option<Point>,
    /// 折弯偏移量，世界单位；未显式指定时取样式的文字高度。
    pub joggle_offset: f64,
}

/// 圆心标记：在被测圆的圆心处绘制的十字或中心点。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CenterMark {
    /// 标记中心，即被测圆的圆心，世界坐标。
    pub center: Point,
    /// 标记总尺寸（世界单位）：半径 / 直径标注取箭头两倍，小尺寸标注取一个箭头大小。
    pub size: f64,
    /// 标记形态，决定画十字还是只画中心点。
    pub mark_type: CenterMarkType,
    /// 中心线伸出圆周之外的长度；为 0 时不绘制中心线。
    pub extension_line_length: f64,
}

/// 圆心标记的形态。
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum CenterMarkType {
    /// 不绘制任何圆心标记。
    None,
    /// 只画中心点（小十字），用于小尺寸标注。
    Mark,
    /// 画十字中心线，常规半径 / 直径标注使用。
    Cross,
}

impl RadialDimension {
    /// 由圆与圆周上的一个测量点构造半径标注。
    /// - `circle`：被测圆，只取圆心（其 `radius` 不参与测量）。
    /// - `p`：测量点，半径取 `circle.center` 到 `p` 的实际距离，同时决定引出方向。
    /// - `style`：标注样式，提供箭头尺寸、文字高度、尺寸线间隙与中心线延伸长度。
    /// 生成的标注 `is_diameter` 为 false、折弯关闭；不修改入参、不写盘。`p` 与圆心重合时方向向量归一化为零向量，标注退化为零长度且不 panic。
    pub fn new_radial(
        circle: Circle,
        p: Point,
        style: DimensionStyle,
    ) -> Self {
        let mut geometry = DimensionGeometry::new(DimensionType::Radial, style.clone());
        geometry.definition_points = vec![circle.center, p];
        
        let radius = circle.center.distance_to(&p);
        geometry.measurement = radius;
        geometry.actual_measurement = radius;
        geometry.update_text();
        
        let center_mark = CenterMark {
            center: circle.center,
            size: style.arrow_size * 2.0,
            mark_type: CenterMarkType::Cross,
            extension_line_length: style.extension_line_extension,
        };
        
        let direction = (p.to_vector2() - circle.center.to_vector2()).normalize();
        let offset = style.arrow_size;
        let arrow = Point::new(
            p.x - direction.x * offset,
            p.y - direction.y * offset,
            0.0,
        );
        
        let dim_line_end = Point::new(
            p.x + direction.x * (style.text_height + style.dimension_line_gap),
            p.y + direction.y * (style.text_height + style.dimension_line_gap),
            0.0,
        );
        let dimension_line = Line::new(p, dim_line_end);
        
        let extension_line = Line::new(p, circle.center);
        
        Self {
            geometry,
            center_mark,
            dimension_line,
            extension_line,
            arrow,
            center_point: circle.center,
            arc_point: p,
            is_diameter: false,
            joggle_enabled: false,
            joggle_location: None,
            joggle_offset: style.text_height,
        }
    }
    
    /// 由圆与直径两端点构造直径标注。
    /// - `circle`：被测圆，测量值取 `circle.radius * 2`，不校验两点是否真的落在圆周上。
    /// - `p1`：箭头所在端，决定箭头、标注线与文字的朝向。
    /// - `p2`：直径另一端，仅作为引出线终点。
    /// - `style`：标注样式，用法同 `new_radial`。
    /// 生成的标注 `is_diameter` 为 true、折弯关闭；不修改入参。
    pub fn new_diameter(
        circle: Circle,
        p1: Point,
        p2: Point,
        style: DimensionStyle,
    ) -> Self {
        let mut geometry = DimensionGeometry::new(DimensionType::Diameter, style.clone());
        geometry.definition_points = vec![circle.center, p1, p2];
        
        let diameter = circle.radius * 2.0;
        geometry.measurement = diameter;
        geometry.actual_measurement = diameter;
        geometry.update_text();
        
        let center_mark = CenterMark {
            center: circle.center,
            size: style.arrow_size * 2.0,
            mark_type: CenterMarkType::Cross,
            extension_line_length: style.extension_line_extension,
        };
        
        let direction = (p1.to_vector2() - circle.center.to_vector2()).normalize();
        let opposite_direction = Vector2::new(-direction.x, -direction.y);
        let offset = style.arrow_size;
        
        let arrow = Point::new(
            p1.x - direction.x * offset,
            p1.y - direction.y * offset,
            0.0,
        );
        
        let dim_line_end = Point::new(
            p1.x + direction.x * (style.text_height + style.dimension_line_gap),
            p1.y + direction.y * (style.text_height + style.dimension_line_gap),
            0.0,
        );
        let dimension_line = Line::new(p1, dim_line_end);
        
        let extension_line = Line::new(p1, p2);
        
        Self {
            geometry,
            center_mark,
            dimension_line,
            extension_line,
            arrow,
            center_point: circle.center,
            arc_point: p1,
            is_diameter: true,
            joggle_enabled: false,
            joggle_location: None,
            joggle_offset: style.text_height,
        }
    }
    
    /// 链式开启折弯（joggle）并设置折弯偏移量，返回设置后的标注。
    /// - `offset`：折弯处的偏移距离，世界单位。
    /// 按值消费 `self`，因此通常写成 `let dim = dim.with_joggle(2.0);`；只改 `joggle_enabled` 与 `joggle_offset`，不推算 `joggle_location`。
    pub fn with_joggle(mut self, offset: f64) -> Self {
        self.joggle_enabled = true;
        self.joggle_offset = offset;
        self
    }
    
    /// 设置圆心标记的形态（就地修改，无返回值）。
    /// - `mark_type`：新的标记形态，只影响 `center_mark.mark_type`，标记的中心与尺寸保持不变。
    pub fn set_center_mark_type(&mut self, mark_type: CenterMarkType) {
        self.center_mark.mark_type = mark_type;
    }
    
    /// 交换标注线的起点与终点（就地修改），把箭头与文字翻到另一端。
    /// 交换后按定义点重算测量值（半径标注得到圆心到测量点的距离）；不改变 `arrow`、`extension_line` 等其它字段。
    pub fn flip(&mut self) {
        std::mem::swap(&mut self.dimension_line.start, &mut self.dimension_line.end);
        self.geometry.calculate_measurement();
    }
    
    /// 覆盖测量值并同步刷新显示文字（就地修改）。
    /// - `measurement`：新的测量值，会同时写入 `measurement` 与 `actual_measurement`。
    /// 用于按比例换算或手工改值，不改动任何几何点，也不校验该值与圆心、测量点是否自洽。
    pub fn set_measurement(&mut self, measurement: f64) {
        self.geometry.measurement = measurement;
        self.geometry.actual_measurement = measurement;
        self.geometry.update_text();
    }
}

impl From<RadialDimension> for Entity {
    fn from(dim: RadialDimension) -> Self {
        let arrow = dim.arrow;
        let geometry = dim.geometry;
        Entity::new(
            EntityType::Dimension,
            super::linear::entity_geometry_from_dimension(&geometry, arrow, 0.0, true, true),
        )
    }
}

/// 小尺寸半径标注：当圆内放不下箭头与文字时，改用绕圆心的一小段圆弧引出文字。
/// 由 `new` 构造，测量值与半径标注一致（圆心到测量点的距离），但中心标记只画中心点、不带中心线。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SmallRadialDimension {
    /// 标注通用数据：`definition_points` 为 [圆心, 测量点]，`measurement` 为半径。
    pub geometry: DimensionGeometry,
    /// 圆心标记：取 `Mark` 形态且延伸长度为 0，尺寸为一个箭头大小。
    pub center_mark: CenterMark,
    /// 圆弧尺寸线：绕圆心、以测量点方向为中心跨 ±π/6（约 ±30°）的圆弧，半径取圆心到 `arrow1` 的距离。
    pub dimension_line: Arc,
    /// 圆弧起点侧的箭头位置，位于测量点沿指向反方向半个箭头尺寸处。
    pub arrow1: Point,
    /// 圆弧终点侧的箭头位置，位于测量点沿指向正方向半个箭头尺寸处。
    pub arrow2: Point,
    /// 文字锚点：测量点沿径向外侧偏移「文字高度 + 尺寸线间隙」后的位置，世界坐标。
    pub text_location: Point,
    /// 被测圆的圆心，世界坐标。
    pub center_point: Point,
}

impl SmallRadialDimension {
    /// 由圆与圆周上的一个测量点构造小尺寸半径标注。
    /// - `circle`：被测圆，只取圆心。
    /// - `p`：测量点，半径取圆心到 `p` 的实际距离，并决定圆弧与文字的引出一侧。
    /// - `style`：标注样式，提供箭头尺寸、文字高度与尺寸线间隙。
    /// 尺寸线是绕圆心、以圆心指向 `p` 的方向为中心跨 ±π/6 弧度的圆弧；不修改入参，`p` 与圆心重合时方向退化为零向量且不 panic。
    pub fn new(
        circle: Circle,
        p: Point,
        style: DimensionStyle,
    ) -> Self {
        let mut geometry = DimensionGeometry::new(DimensionType::Radial, style.clone());
        geometry.definition_points = vec![circle.center, p];
        
        let radius = circle.center.distance_to(&p);
        geometry.measurement = radius;
        geometry.actual_measurement = radius;
        geometry.update_text();
        
        let center_mark = CenterMark {
            center: circle.center,
            size: style.arrow_size,
            mark_type: CenterMarkType::Mark,
            extension_line_length: 0.0,
        };
        
        let direction = (p.to_vector2() - circle.center.to_vector2()).normalize();
        let arrow_size = style.arrow_size;
        
        let arrow1 = Point::new(
            p.x - direction.x * arrow_size * 0.5,
            p.y - direction.y * arrow_size * 0.5,
            0.0,
        );
        let arrow2 = Point::new(
            p.x + direction.x * arrow_size * 0.5,
            p.y + direction.y * arrow_size * 0.5,
            0.0,
        );
        
        let text_location = Point::new(
            p.x + direction.x * (style.text_height + style.dimension_line_gap),
            p.y + direction.y * (style.text_height + style.dimension_line_gap),
            0.0,
        );
        
        let dim_line_radius = circle.center.distance_to(&arrow1);
        let angle = (p - circle.center).to_vector2().angle();
        let start_angle = angle - std::f64::consts::PI / 6.0;
        let end_angle = angle + std::f64::consts::PI / 6.0;
        let dimension_line = Arc::new(circle.center, dim_line_radius, start_angle, end_angle);
        
        Self {
            geometry,
            center_mark,
            dimension_line,
            arrow1,
            arrow2,
            text_location,
            center_point: circle.center,
        }
    }
}

impl From<SmallRadialDimension> for Entity {
    fn from(dim: SmallRadialDimension) -> Self {
        let text_location = dim.text_location;
        let geometry = dim.geometry;
        Entity::new(
            EntityType::Dimension,
            super::linear::entity_geometry_from_dimension(&geometry, text_location, 0.0, false, true),
        )
    }
}

/// 直径标注（带中心线）：尺寸线自圆心指向圆周一点，文字默认落在圆外。
/// 由 `new_with_center_line` 构造；与 `RadialDimension` 的直径模式不同，这里不接收第二个端点，方向固定沿 +x。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DiameterDimension {
    /// 标注通用数据：`definition_points` 只含圆心，`measurement` 为 `2 * radius`。
    pub geometry: DimensionGeometry,
    /// 圆心处的十字标记，尺寸取样式箭头大小的两倍。
    pub center_mark: CenterMark,
    /// 直径尺寸线：由圆心指向 `far_point`，仅覆盖一个半径的长度。
    pub dimension_line: Line,
    /// 文字放置点，世界坐标；构造时可指定，未指定时默认在圆心右侧 1.5 倍半径处。
    pub text_location: Point,
    /// 被测圆的圆心，世界坐标。
    pub center_point: Point,
    /// 尺寸线终点：圆心沿 +x 方向一个半径处的圆周点，z 与圆心一致（构造时写 0.0）。
    pub far_point: Point,
}

impl DiameterDimension {
    /// 由圆构造带中心线的直径标注，方向固定沿 +x 轴。
    /// - `circle`：被测圆，测量值取 `radius * 2`，`far_point` 取圆心右侧一个半径处。
    /// - `style`：标注样式，提供箭头尺寸、中心线延伸长度等。
    /// - `text_location`：文字位置；传 `None` 时默认落在圆心右侧 1.5 倍半径处。
    /// 不修改入参；与 `RadialDimension::new_diameter` 的区别是不需要也不能指定第二个端点。
    pub fn new_with_center_line(
        circle: Circle,
        style: DimensionStyle,
        text_location: Option<Point>,
    ) -> Self {
        let mut geometry = DimensionGeometry::new(DimensionType::Diameter, style.clone());
        geometry.definition_points = vec![circle.center];
        
        let diameter = circle.radius * 2.0;
        geometry.measurement = diameter;
        geometry.actual_measurement = diameter;
        geometry.update_text();
        
        let center_mark = CenterMark {
            center: circle.center,
            size: style.arrow_size * 2.0,
            mark_type: CenterMarkType::Cross,
            extension_line_length: style.extension_line_extension,
        };
        
        let far_point = Point::new(
            circle.center.x + circle.radius,
            circle.center.y,
            0.0,
        );
        
        let text_location = text_location.unwrap_or_else(|| {
            Point::new(
                circle.center.x + circle.radius * 1.5,
                circle.center.y,
                0.0,
            )
        });
        
        let dimension_line = Line::new(circle.center, far_point);
        
        Self {
            geometry,
            center_mark,
            dimension_line,
            text_location,
            center_point: circle.center,
            far_point,
        }
    }
}

impl From<DiameterDimension> for Entity {
    fn from(dim: DiameterDimension) -> Self {
        let text_location = dim.text_location;
        let geometry = dim.geometry;
        Entity::new(
            EntityType::Dimension,
            super::linear::entity_geometry_from_dimension(&geometry, text_location, 0.0, false, true),
        )
    }
}
