//! 角度标注与弧长标注：由圆弧、三点或两条直线构造角度标注，并单独提供弧长标注。
//!
//! 三个构造入口 [`AngularDimension::new_from_arc`]、[`AngularDimension::new_from_3_point`] 与
//! [`AngularDimension::new_from_2_line`] 仅在尺寸界线与起止角的取法上不同，测量值统一为弧度；
//! 生成的标注同时保存圆弧、尺寸界线与文字位置等可直接绘制的几何，并可通过 `From` 转换为 [`Entity`]。
//!
//! 与 `linear` 模块共用 [`DimensionGeometry`]、[`DimensionStyle`] 与 [`DimensionType`]；
//! [`AngularUnit`] 负责把弧度值格式化为度、度分秒、百分度或弧度文本。

use crate::geometry::{Point, Vector2, Arc, Line};
use crate::data_structure::{Entity, EntityType, EntityGeometry};
use super::linear::{DimensionGeometry, DimensionStyle, DimensionType};
use serde::{Serialize, Deserialize};
use std::fmt;

/// 角度标注：同时保存可直接绘制的圆弧、尺寸界线与文字位置，测量值为弧度。
///
/// 三种构造函数对应不同输入方式，可用 [`AngularDimension::is_2_line`] 与
/// [`AngularDimension::is_3_point`] 判断来源。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AngularDimension {
    /// 定义点（圆心、两条边上的点）、角度测量值与显示文字，测量类型为 `Angular`。
    pub geometry: DimensionGeometry,
    /// 用于标示角度的圆弧，半径固定为样式箭头尺寸的 8 倍。
    pub arc: Arc,
    /// 第一条尺寸界线；由圆弧构造时自圆心沿起始边方向绘制。
    pub extension_line1: Line,
    /// 第二条尺寸界线。
    pub extension_line2: Line,
    /// 文字位置：位于角平分线方向上，偏移量为标注圆弧半径加文字高与尺寸线间隙。
    pub text_location: Point,
    /// 角的顶点，构造后不再随 `arc` 改变。
    pub arc_center: Point,
    /// 标注圆弧的半径，等于样式 `arrow_size` 的 8 倍。
    pub arc_radius: f64,
    /// 起始角（弧度），即标注圆弧的起始角；[`AngularDimension::flip`] 会交换它与 `end_angle`。
    pub start_angle: f64,
    /// 结束角（弧度）。
    pub end_angle: f64,
    /// 是否由两条直线构造（[`AngularDimension::new_from_2_line`]）。
    pub is_2_line: bool,
    /// 是否由三点构造；`new_from_arc` 与 `new_from_3_point` 均置为 `true`。
    pub is_3_point: bool,
}

impl AngularDimension {
    /// 依据角的顶点与一段参考圆弧生成角度标注，测量值为该弧的圆心角（弧度）。
    ///
    /// - `center`：角的顶点，通常与 `arc.center` 相同。
    /// - `arc`：参考圆弧，仅其起止角与半径用于确定两条边的方向，半径不参与测量。
    /// - `style`：标注样式，按值保存为副本。
    /// 尺寸界线自圆心沿两条边方向各延伸 `extension_line_extension` 的两倍，因此也能标注小圆角；
    /// 生成的标注 `is_3_point` 为 `true`、`is_2_line` 为 `false`。
    pub fn new_from_arc(
        center: Point,
        arc: Arc,
        style: DimensionStyle,
    ) -> Self {
        let mut geometry = DimensionGeometry::new(DimensionType::Angular, style.clone());
        geometry.definition_points = vec![
            center,
            Point::new(
                center.x + arc.radius * arc.start_angle.cos(),
                center.y + arc.radius * arc.start_angle.sin(),
                0.0,
            ),
            Point::new(
                center.x + arc.radius * arc.end_angle.cos(),
                center.y + arc.radius * arc.end_angle.sin(),
                0.0,
            ),
        ];
        
        let ext_length = style.extension_line_extension * 2.0;
        
        let v1 = Vector2::new(
            geometry.definition_points[1].x - center.x,
            geometry.definition_points[1].y - center.y,
        ).normalize();
        let v2 = Vector2::new(
            geometry.definition_points[2].x - center.x,
            geometry.definition_points[2].y - center.y,
        ).normalize();
        
        let ext1_end = Point::new(
            center.x + v1.x * ext_length,
            center.y + v1.y * ext_length,
            0.0,
        );
        let extension_line1 = Line::new(center, ext1_end);
        
        let ext2_end = Point::new(
            center.x + v2.x * ext_length,
            center.y + v2.y * ext_length,
            0.0,
        );
        let extension_line2 = Line::new(center, ext2_end);
        
        let arc_radius = style.arrow_size * 8.0;
        let mid_angle = (arc.start_angle + arc.end_angle) / 2.0;
        let text_location = Point::new(
            center.x + mid_angle.cos() * (arc_radius + style.text_height + style.dimension_line_gap),
            center.y + mid_angle.sin() * (arc_radius + style.text_height + style.dimension_line_gap),
            0.0,
        );
        
        geometry.calculate_measurement();
        
        Self {
            geometry,
            arc: Arc::new(center, arc_radius, arc.start_angle, arc.end_angle),
            extension_line1,
            extension_line2,
            text_location,
            arc_center: center,
            arc_radius,
            start_angle: arc.start_angle,
            end_angle: arc.end_angle,
            is_2_line: false,
            is_3_point: true,
        }
    }
    
    /// 依据角的顶点与两条边上的点生成角度标注。
    ///
    /// - `center`：角的顶点。
    /// - `p1`、`p2`：两条边上的点，与顶点的距离可不等。
    /// - `style`：标注样式，按值保存为副本。
    /// 起止角取两条边的极角并按大小排序，测量值即二者之差（弧度，落在 0 至 π）。
    pub fn new_from_3_point(
        center: Point,
        p1: Point,
        p2: Point,
        style: DimensionStyle,
    ) -> Self {
        let mut geometry = DimensionGeometry::new(DimensionType::Angular, style.clone());
        geometry.definition_points = vec![center, p1, p2];
        
        let radius = center.distance_to(&p1).max(center.distance_to(&p2));
        let v1 = (p1 - center).to_vector2();
        let v2 = (p2 - center).to_vector2();
        let angle1 = v1.angle();
        let angle2 = v2.angle();
        
        let arc_radius = style.arrow_size * 8.0;
        let ext_length = style.extension_line_extension * 2.0;
        
        let ext1_end = Point::new(
            center.x + v1.normalize().x * ext_length,
            center.y + v1.normalize().y * ext_length,
            0.0,
        );
        let extension_line1 = Line::new(center, ext1_end);
        
        let ext2_end = Point::new(
            center.x + v2.normalize().x * ext_length,
            center.y + v2.normalize().y * ext_length,
            0.0,
        );
        let extension_line2 = Line::new(center, ext2_end);
        
        let start_angle = angle1.min(angle2);
        let end_angle = angle1.max(angle2);
        let mid_angle = (start_angle + end_angle) / 2.0;
        let text_location = Point::new(
            center.x + mid_angle.cos() * (arc_radius + style.text_height + style.dimension_line_gap),
            center.y + mid_angle.sin() * (arc_radius + style.text_height + style.dimension_line_gap),
            0.0,
        );
        
        geometry.calculate_measurement();
        
        Self {
            geometry,
            arc: Arc::new(center, arc_radius, start_angle, end_angle),
            extension_line1,
            extension_line2,
            text_location,
            arc_center: center,
            arc_radius,
            start_angle,
            end_angle,
            is_2_line: false,
            is_3_point: true,
        }
    }
    
    /// 依据两条直线上的点及它们的交点生成角度标注。
    ///
    /// - `p1`、`p2`：两条直线上的点，边方向由交点指向它们。
    /// - `center`：两条直线的交点，作为角的顶点；由调用方保证其正确性。
    /// - `style`：标注样式，按值保存为副本。
    /// 尺寸界线自 `p1`、`p2` 沿各自方向向外延伸，而非自交点起始；`is_2_line` 置为 `true`。
    pub fn new_from_2_line(
        p1: Point,
        p2: Point,
        center: Point,
        style: DimensionStyle,
    ) -> Self {
        let mut geometry = DimensionGeometry::new(DimensionType::Angular, style.clone());
        geometry.definition_points = vec![center, p1, p2];
        
        let arc_radius = style.arrow_size * 8.0;
        let ext_length = style.extension_line_extension * 2.0;
        
        let v1 = (p1 - center).to_vector2().normalize();
        let v2 = (p2 - center).to_vector2().normalize();
        
        let ext1_end = Point::new(
            p1.x + v1.x * ext_length,
            p1.y + v1.y * ext_length,
            0.0,
        );
        let extension_line1 = Line::new(p1, ext1_end);
        
        let ext2_end = Point::new(
            p2.x + v2.x * ext_length,
            p2.y + v2.y * ext_length,
            0.0,
        );
        let extension_line2 = Line::new(p2, ext2_end);
        
        let angle1 = v1.angle();
        let angle2 = v2.angle();
        let start_angle = angle1.min(angle2);
        let end_angle = angle1.max(angle2);
        let mid_angle = (start_angle + end_angle) / 2.0;
        let text_location = Point::new(
            center.x + mid_angle.cos() * (arc_radius + style.text_height + style.dimension_line_gap),
            center.y + mid_angle.sin() * (arc_radius + style.text_height + style.dimension_line_gap),
            0.0,
        );
        
        geometry.calculate_measurement();
        
        Self {
            geometry,
            arc: Arc::new(center, arc_radius, start_angle, end_angle),
            extension_line1,
            extension_line2,
            text_location,
            arc_center: center,
            arc_radius,
            start_angle,
            end_angle,
            is_2_line: true,
            is_3_point: false,
        }
    }
    
    /// 交换起始角与结束角，用于改为标注补角一侧，同时交换两条尺寸界线。
    ///
    /// 会重新计算测量值与显示文字（就地修改 `self`）；`text_location` 不参与交换，
    /// 需要时请再调用 [`AngularDimension::set_text_location`] 指定文字位置。
    pub fn flip(&mut self) {
        std::mem::swap(&mut self.start_angle, &mut self.end_angle);
        std::mem::swap(&mut self.extension_line1, &mut self.extension_line2);
        self.geometry.calculate_measurement();
    }
    
    /// 设置文字位置，并写入 `geometry.user_text_location`，使
    /// [`crate::dimension::linear::text_position_for`] 采用该点。
    ///
    /// - `location`：文字定位点（世界坐标）。
    /// 只修改 `self`，不重新计算测量值与文字。
    pub fn set_text_location(&mut self, location: Point) {
        self.text_location = location;
        self.geometry.user_text_location = Some(location);
    }
}

impl From<AngularDimension> for Entity {
    fn from(dim: AngularDimension) -> Self {
        let text_location = dim.text_location;
        let angle = dim.end_angle - dim.start_angle;
        let geometry = dim.geometry;
        Entity::new(
            EntityType::Dimension,
            super::linear::entity_geometry_from_dimension(&geometry, text_location, angle, true, false),
        )
    }
}

/// 弧长标注：测量值为圆弧上起止角之间的弧长，等于半径与圆心角（弧度）之积。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ArcLengthDimension {
    /// 定义点（圆心、弧的两个端点）、弧长测量值与显示文字，测量类型为 `ArcLength`。
    pub geometry: DimensionGeometry,
    /// 用于标示弧长的圆弧，半径固定为样式箭头尺寸的 8 倍。
    pub arc: Arc,
    /// 起始端尺寸界线，自弧端点沿半径方向向外延伸。
    pub extension_line1: Line,
    /// 终止端尺寸界线。
    pub extension_line2: Line,
    /// 文字位置：位于弧的角平分线方向上，偏移量为标注圆弧半径加文字高与尺寸线间隙。
    pub text_location: Point,
}

impl ArcLengthDimension {
    /// 依据一段圆弧生成弧长标注，测量值取半径与圆心角（弧度）之积。
    ///
    /// - `arc`：被标注的圆弧，使用其圆心与起止角；`arc.radius` 不参与测量。
    /// - `style`：标注样式，按值保存为副本。
    /// 起止尺寸界线自弧的两个端点沿半径方向向外延伸 `extension_line_extension` 的两倍，
    /// 文字按样式的单位格式与公差设置立即生成。
    pub fn new(arc: Arc, style: DimensionStyle) -> Self {
        let mut geometry = DimensionGeometry::new(DimensionType::ArcLength, style.clone());
        geometry.definition_points = vec![
            arc.center,
            Point::new(
                arc.center.x + arc.radius * arc.start_angle.cos(),
                arc.center.y + arc.radius * arc.start_angle.sin(),
                0.0,
            ),
            Point::new(
                arc.center.x + arc.radius * arc.end_angle.cos(),
                arc.center.y + arc.radius * arc.end_angle.sin(),
                0.0,
            ),
        ];
        
        let arc_length = arc.radius * (arc.end_angle - arc.start_angle).abs();
        geometry.measurement = arc_length;
        geometry.actual_measurement = arc_length;
        geometry.update_text();
        
        let radius = style.arrow_size * 8.0;
        let mid_angle = (arc.start_angle + arc.end_angle) / 2.0;
        let text_location = Point::new(
            arc.center.x + mid_angle.cos() * (radius + style.text_height + style.dimension_line_gap),
            arc.center.y + mid_angle.sin() * (radius + style.text_height + style.dimension_line_gap),
            0.0,
        );
        
        let ext_length = style.extension_line_extension * 2.0;
        let v1 = Vector2::new(
            geometry.definition_points[1].x - arc.center.x,
            geometry.definition_points[1].y - arc.center.y,
        ).normalize();
        let v2 = Vector2::new(
            geometry.definition_points[2].x - arc.center.x,
            geometry.definition_points[2].y - arc.center.y,
        ).normalize();
        
        let ext1_end = Point::new(
            geometry.definition_points[1].x + v1.x * ext_length,
            geometry.definition_points[1].y + v1.y * ext_length,
            0.0,
        );
        let extension_line1 = Line::new(geometry.definition_points[1], ext1_end);
        
        let ext2_end = Point::new(
            geometry.definition_points[2].x + v2.x * ext_length,
            geometry.definition_points[2].y + v2.y * ext_length,
            0.0,
        );
        let extension_line2 = Line::new(geometry.definition_points[2], ext2_end);
        
        Self {
            geometry,
            arc: Arc::new(arc.center, radius, arc.start_angle, arc.end_angle),
            extension_line1,
            extension_line2,
            text_location,
        }
    }
}

impl From<ArcLengthDimension> for Entity {
    fn from(dim: ArcLengthDimension) -> Self {
        let text_location = dim.text_location;
        let angle = dim.arc.end_angle - dim.arc.start_angle;
        let geometry = dim.geometry;
        Entity::new(
            EntityType::Dimension,
            super::linear::entity_geometry_from_dimension(&geometry, text_location, angle, true, false),
        )
    }
}

/// 角度的显示单位，决定 [`AngularUnit::format_angle`] 的输出形式，默认按度显示。
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum AngularUnit {
    /// 十进制度，形如 45.00°。
    Degrees,
    /// 度分秒，形如 45°30'0"。
    DegreesMinutesSeconds,
    /// 百分度（gon），形如 50.00g。
    Gradians,
    /// 弧度，形如 0.7854 rad。
    Radians,
}

impl Default for AngularUnit {
    fn default() -> Self {
        AngularUnit::Degrees
    }
}

impl AngularUnit {
    /// 把弧度值格式化为当前单位的文本，供标注文字显示。
    ///
    /// - `angle_radians`：角度值，输入单位固定为弧度。
    /// 返回固定精度的文本：度与百分度保留两位小数，度分秒的秒四舍五入到整数，弧度保留四位小数。
    /// 只读，不修改 `self`。
    ///
    /// # 示例
    /// ```text
    /// assert_eq!(AngularUnit::Degrees.format_angle(std::f64::consts::FRAC_PI_2), "90.00°");
    /// assert_eq!(AngularUnit::Radians.format_angle(std::f64::consts::PI), "3.1416 rad");
    /// ```
    pub fn format_angle(&self, angle_radians: f64) -> String {
        match self {
            AngularUnit::Degrees => {
                let degrees = angle_radians.to_degrees();
                format!("{:.2}°", degrees)
            }
            AngularUnit::DegreesMinutesSeconds => {
                let degrees = angle_radians.to_degrees();
                let d = degrees.floor() as i32;
                let minutes = ((degrees - d as f64) * 60.0).floor() as i32;
                let seconds = ((degrees - d as f64 - minutes as f64 / 60.0) * 3600.0).round();
                format!("{}°{}'{:.0}\"", d, minutes, seconds)
            }
            AngularUnit::Gradians => {
                let gradians = angle_radians.to_degrees() * 10.0 / 9.0;
                format!("{:.2}g", gradians)
            }
            AngularUnit::Radians => {
                format!("{:.4} rad", angle_radians)
            }
        }
    }
}
