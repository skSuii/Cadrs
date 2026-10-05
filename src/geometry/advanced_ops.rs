//! 高级几何操作：偏移（Offset）、倒角（Chamfer）、圆角（Fillet）、过渡（Blend）、
//! 扫掠（Sweep）、放样（Loft）、曲线光顺（Fairing）与等距曲线（Parallel）。
//!
//! 所有入口都以不可变引用接收输入（`Entity` 或 `Curve`），成功时返回新建的实体/曲线，
//! 不会修改或写回原对象；失败原因由各自的 `*Error` 枚举给出。
//! 距离与坐标使用世界坐标单位，角度一律为弧度；曲线的“左侧”指前进方向的法线 `(-dy, dx)` 一侧。
//! 相交判定复用 `geometry::intersection` 中的线段/圆求交函数。
//!
//! 注意：`blend_surfaces`、`sweep_profile`、`fair_curve` 目前是占位实现，只做最小工作，详见各函数文档。

use crate::geometry::{Point, Vector2, Line, Circle, Arc, Ellipse, Polyline, BSpline, NURBS, Curve};
use crate::data_structure::{Entity, EntityType, EntityGeometry};
use crate::geometry::intersection::{intersect_line_line, intersect_line_circle, IntersectionResult};
use thiserror::Error;

/// 偏移（Offset）操作失败的原因。
#[derive(Debug, Error)]
pub enum OffsetError {
    /// 偏移距离过大：圆或圆弧按该距离偏移后半径不再为正。
    ///
    /// - `distance`：被拒绝的偏移量绝对值。
    #[error("偏移距离过大: {distance}")]
    DistanceTooLarge { distance: f64 },
    
    /// 偏移结果自交，无法生成有效曲线。
    ///
    /// - `description`：自交位置或成因的文字说明。
    #[error("曲线自交: {description}")]
    SelfIntersection { description: String },
    
    /// 曲线类型不支持偏移；目前直线、圆、圆弧与折线可用。
    #[error("无法偏移: 曲线类型不支持")]
    UnsupportedCurveType,
    
    /// 数值计算失败，通常由退化输入（零长直线、零半径圆等）引起。
    ///
    /// - `message`：底层计算给出的说明。
    #[error("计算失败: {message}")]
    ComputationFailed { message: String },
}

/// 偏移/平行线的方向侧别。
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ParallelSide {
    /// 沿曲线前进方向的左侧，即法线 `(-dy, dx)` 一侧，偏移量取正值。
    Left,
    /// 沿曲线前进方向的右侧，等价于左侧取负偏移量。
    Right,
    /// 双侧同时偏移；当前实现按左侧处理，需要两侧时请分别偏移一次。
    Both,
}

/// 折线偏移时拐角的连接样式。
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OffsetCornerStyle {
    /// 尖角：相邻偏移边直接连接。
    Sharp,
    /// 圆角：用圆弧过渡拐角；`OffsetOptions` 的默认值。
    Round,
    /// 斜切角：用一段直线切掉拐角。
    Bevel,
}

/// 偏移操作的配置项，供 `OffsetCurve::offset_with_options` 使用。
pub struct OffsetOptions {
    /// 偏移距离（世界坐标单位）；正值偏向 `side` 指定的一侧，负值反向。
    pub distance: f64,
    /// 期望的偏移侧别；当前实现按左侧法线取向，未使用右/双侧语义。
    pub side: ParallelSide,
    /// 折线拐角样式；当前实现统一把相邻偏移点直接连线。
    pub corner_style: OffsetCornerStyle,
    /// 离散与误差判定公差，默认 `1e-6`；当前实现保留未使用。
    pub tolerance: f64,
    /// 是否把相邻偏移边延长到相交；当前实现保留未使用。
    pub extend_to_intersect: bool,
}

impl Default for OffsetOptions {
    fn default() -> Self {
        Self {
            distance: 1.0,
            side: ParallelSide::Left,
            corner_style: OffsetCornerStyle::Round,
            tolerance: 1e-6,
            extend_to_intersect: false,
        }
    }
}

/// 曲线偏移能力：为直线、圆、圆弧、折线与实体提供统一的等距偏移入口。
///
/// 实现均返回新建的 `Entity`，不修改 `self`；失败原因见 `OffsetError`。
pub trait OffsetCurve {
    /// 按给定距离偏移曲线，正值偏向曲线左侧（直线取法线 `(-dy, dx)`，圆/圆弧为增大半径）。
    ///
    /// - `distance`：偏移距离，世界坐标单位；圆/圆弧偏移后半径必须为正，否则返回
    ///   `OffsetError::DistanceTooLarge`。
    fn offset(&self, distance: f64) -> Result<Entity, OffsetError>;
    /// 按 `options` 偏移曲线；目前只有 `options.distance` 参与计算。
    fn offset_with_options(&self, options: &OffsetOptions) -> Result<Entity, OffsetError>;
    /// 等价于 `offset(distance)`，即向曲线左侧（圆/圆弧为外侧、半径增大）偏移。
    fn offset_left(&self, distance: f64) -> Result<Entity, OffsetError>;
    /// 等价于 `offset(-distance)`，即向曲线右侧（圆/圆弧为内侧、半径减小）偏移。
    fn offset_right(&self, distance: f64) -> Result<Entity, OffsetError>;
}

impl OffsetCurve for Line {
    fn offset(&self, distance: f64) -> Result<Entity, OffsetError> {
        self.offset_with_options(&OffsetOptions {
            distance,
            ..Default::default()
        })
    }
    
    fn offset_with_options(&self, options: &OffsetOptions) -> Result<Entity, OffsetError> {
        let direction = self.direction();
        let normal = Vector2::new(-direction.y, direction.x).normalize();
        
        let start_offset = Point::new(
            self.start.x + normal.x * options.distance,
            self.start.y + normal.y * options.distance,
            0.0,
        );
        let end_offset = Point::new(
            self.end.x + normal.x * options.distance,
            self.end.y + normal.y * options.distance,
            0.0,
        );
        
        let offset_line = Line::new(start_offset, end_offset);
        Ok(Entity::new(
            EntityType::Line,
            EntityGeometry::Line(offset_line),
        ))
    }
    
    fn offset_left(&self, distance: f64) -> Result<Entity, OffsetError> {
        self.offset(distance)
    }
    
    fn offset_right(&self, distance: f64) -> Result<Entity, OffsetError> {
        self.offset(-distance)
    }
}

impl OffsetCurve for Circle {
    fn offset(&self, distance: f64) -> Result<Entity, OffsetError> {
        let new_radius = self.radius + distance;
        if new_radius <= 0.0 {
            return Err(OffsetError::DistanceTooLarge { distance: distance.abs() });
        }
        
        let offset_circle = Circle::new(self.center, new_radius);
        Ok(Entity::new(
            EntityType::Circle,
            EntityGeometry::Circle(offset_circle),
        ))
    }
    
    fn offset_with_options(&self, options: &OffsetOptions) -> Result<Entity, OffsetError> {
        self.offset(options.distance)
    }
    
    fn offset_left(&self, distance: f64) -> Result<Entity, OffsetError> {
        self.offset(distance)
    }
    
    fn offset_right(&self, distance: f64) -> Result<Entity, OffsetError> {
        self.offset(-distance)
    }
}

impl OffsetCurve for Arc {
    fn offset(&self, distance: f64) -> Result<Entity, OffsetError> {
        let new_radius = self.radius + distance;
        if new_radius <= 0.0 {
            return Err(OffsetError::DistanceTooLarge { distance: distance.abs() });
        }
        
        let offset_arc = Arc::new(self.center, new_radius, self.start_angle, self.end_angle);
        Ok(Entity::new(
            EntityType::Arc,
            EntityGeometry::Arc(offset_arc),
        ))
    }
    
    fn offset_with_options(&self, _options: &OffsetOptions) -> Result<Entity, OffsetError> {
        self.offset(_options.distance)
    }
    
    fn offset_left(&self, distance: f64) -> Result<Entity, OffsetError> {
        self.offset(distance)
    }
    
    fn offset_right(&self, distance: f64) -> Result<Entity, OffsetError> {
        self.offset(-distance)
    }
}

impl OffsetCurve for Polyline {
    fn offset(&self, distance: f64) -> Result<Entity, OffsetError> {
        self.offset_with_options(&OffsetOptions {
            distance,
            ..Default::default()
        })
    }
    
    fn offset_with_options(&self, options: &OffsetOptions) -> Result<Entity, OffsetError> {
        let mut offset_vertices = Vec::new();
        
        for i in 0..self.vertices.len() {
            let prev = if i > 0 { &self.vertices[i-1] } else { &self.vertices[0] };
            let current = &self.vertices[i];
            let next = if i + 1 < self.vertices.len() { &self.vertices[i+1] } else { 
                if self.is_closed { &self.vertices[1] } else { &self.vertices[i] }
            };
            
            let dir1 = (current.to_vector2() - prev.to_vector2()).normalize();
            let dir2 = (next.to_vector2() - current.to_vector2()).normalize();
            let normal1 = Vector2::new(-dir1.y, dir1.x);
            let normal2 = Vector2::new(-dir2.y, dir2.x);
            
            let avg_normal = (normal1 + normal2).normalize();
            let offset_point = Point::new(
                current.x + avg_normal.x * options.distance,
                current.y + avg_normal.y * options.distance,
                0.0,
            );
            offset_vertices.push(offset_point);
        }
        
        let mut offset_polyline = Polyline::new();
        for vertex in offset_vertices {
            offset_polyline.push(vertex);
        }
        
        if self.is_closed {
            offset_polyline.close();
        }
        
        Ok(Entity::new(
            EntityType::Polyline,
            EntityGeometry::Polyline(offset_polyline),
        ))
    }
    
    fn offset_left(&self, distance: f64) -> Result<Entity, OffsetError> {
        self.offset(distance)
    }
    
    fn offset_right(&self, distance: f64) -> Result<Entity, OffsetError> {
        self.offset(-distance)
    }
}

impl OffsetCurve for Entity {
    fn offset(&self, distance: f64) -> Result<Entity, OffsetError> {
        match &self.geometry {
            EntityGeometry::Line(line) => line.offset(distance),
            EntityGeometry::Circle(circle) => circle.offset(distance),
            EntityGeometry::Arc(arc) => arc.offset(distance),
            EntityGeometry::Polyline(polyline) => polyline.offset(distance),
            _ => Err(OffsetError::UnsupportedCurveType),
        }
    }
    
    fn offset_with_options(&self, options: &OffsetOptions) -> Result<Entity, OffsetError> {
        match &self.geometry {
            EntityGeometry::Line(line) => line.offset_with_options(options),
            EntityGeometry::Circle(circle) => circle.offset_with_options(options),
            EntityGeometry::Arc(arc) => arc.offset_with_options(options),
            EntityGeometry::Polyline(polyline) => polyline.offset_with_options(options),
            _ => Err(OffsetError::UnsupportedCurveType),
        }
    }
    
    fn offset_left(&self, distance: f64) -> Result<Entity, OffsetError> {
        self.offset(distance)
    }
    
    fn offset_right(&self, distance: f64) -> Result<Entity, OffsetError> {
        self.offset(-distance)
    }
}

/// 倒角（Chamfer）与圆角（Fillet）操作失败的原因。
#[derive(Debug, Error)]
pub enum ChamferError {
    /// 倒角距离或圆角半径无效（小于等于 0）。
    #[error("倒角距离无效")]
    InvalidChamferDistances,
    
    /// 两实体没有有效交点，无法确定倒角/圆角的位置。
    #[error("实体不相交")]
    EntitiesDoNotIntersect,
    
    /// 几何构造失败。
    ///
    /// - `message`：失败说明。
    #[error("无法创建倒角: {message}")]
    CreationFailed { message: String },
    
    /// 实体类型不支持；目前倒角仅支持直线—直线，圆角另支持圆/圆弧—直线与圆—圆。
    #[error("不支持的实体类型")]
    UnsupportedEntityType,
}

/// 倒角参数集合：两条边上的距离与裁剪/保留开关。
///
/// 注意：本文件中的 `chamfer_entities` 目前只使用距离参数，裁剪与保留需要调用方处理。
pub struct ChamferOptions {
    /// 第一条边上的倒角距离（世界坐标单位），必须为正。
    pub distance1: f64,
    /// 第二条边上的倒角距离（世界坐标单位），必须为正。
    pub distance2: f64,
    /// 是否把原实体裁剪到倒角端点；默认 `true`。
    pub create_trim: bool,
    /// 是否保留原实体而不裁剪；默认 `false`，与 `create_trim` 取向相反。
    pub preserve_entities: bool,
}

impl Default for ChamferOptions {
    fn default() -> Self {
        Self {
            distance1: 1.0,
            distance2: 1.0,
            create_trim: true,
            preserve_entities: false,
        }
    }
}

/// 圆角参数集合：半径与是否裁剪、保留原实体。
///
/// 注意：本文件中的 `fillet_entities` 直接接收半径参数，其余字段供上层成组配置。
pub struct FilletOptions {
    /// 圆角半径（世界坐标单位），必须为正，默认 `1.0`。
    pub radius: f64,
    /// 是否把原实体裁剪到圆角切点；默认 `true`。
    pub create_trim: bool,
    /// 是否保留原实体而不裁剪；默认 `false`。
    pub preserve_entities: bool,
    /// 圆角圆弧参与后续离散的公差，默认 `0.01`。
    pub arc_tolerance: f64,
}

impl Default for FilletOptions {
    fn default() -> Self {
        Self {
            radius: 1.0,
            create_trim: true,
            preserve_entities: false,
            arc_tolerance: 0.01,
        }
    }
}

/// 对两条直线实体做倒角，返回一条连接两个倒角点的直线实体。
///
/// - `entity1` / `entity2`：两个待倒角的实体，两侧都必须拟合为直线，否则返回
///   `ChamferError::UnsupportedEntityType`。
/// - `dist1` / `dist2`：沿各自直线从交点量取的距离（世界坐标单位），必须为正，否则返回
///   `ChamferError::InvalidChamferDistances`。
///
/// 返回的 `Vec<Entity>` 只含倒角直线，原实体不会被修改或裁剪；两直线平行或线段不相交时
/// 返回 `ChamferError::EntitiesDoNotIntersect`。
///
/// # 示例
/// ```ignore
/// let chamfer = chamfer_entities(&line1, &line2, 1.0, 1.0)?;
/// ```
pub fn chamfer_entities(entity1: &Entity, entity2: &Entity, dist1: f64, dist2: f64) -> Result<Vec<Entity>, ChamferError> {
    if dist1 <= 0.0 || dist2 <= 0.0 {
        return Err(ChamferError::InvalidChamferDistances);
    }
    
    match (&entity1.geometry, &entity2.geometry) {
        (EntityGeometry::Line(line1), EntityGeometry::Line(line2)) => {
            chamfer_two_lines(line1, line2, dist1, dist2)
        }
        _ => Err(ChamferError::UnsupportedEntityType),
    }
}

fn chamfer_two_lines(line1: &Line, line2: &Line, dist1: f64, dist2: f64) -> Result<Vec<Entity>, ChamferError> {
    let intersection = intersect_line_line(line1.clone(), line2.clone());
    
    match intersection {
        IntersectionResult::Point(ip) => {
            let dir1 = line1.direction().normalize();
            let dir2 = line2.direction().normalize();
            
            let point1 = Point::new(
                ip.point.x - dir1.x * dist1,
                ip.point.y - dir1.y * dist1,
                0.0,
            );
            let point2 = Point::new(
                ip.point.x - dir2.x * dist2,
                ip.point.y - dir2.y * dist2,
                0.0,
            );
            
            let chamfer_line = Line::new(point1, point2);
            
            let entities = vec![
                Entity::new(EntityType::Line, EntityGeometry::Line(chamfer_line)),
            ];
            
            Ok(entities)
        }
        _ => Err(ChamferError::EntitiesDoNotIntersect),
    }
}

/// 对两个实体做圆角，返回一段圆角圆弧实体。
///
/// - `entity1` / `entity2`：支持直线—直线、圆—直线、圆弧—直线与圆—圆四种组合，
///   其余组合返回 `ChamferError::UnsupportedEntityType`。
/// - `radius`：圆角半径（世界坐标单位），必须为正，否则返回
///   `ChamferError::InvalidChamferDistances`。
///
/// 返回的 `Vec<Entity>` 只含圆角圆弧，原实体不会被修改或裁剪；两实体无有效交点时返回
/// `ChamferError::EntitiesDoNotIntersect`（圆—圆还要求圆心不重合）。
pub fn fillet_entities(entity1: &Entity, entity2: &Entity, radius: f64) -> Result<Vec<Entity>, ChamferError> {
    if radius <= 0.0 {
        return Err(ChamferError::InvalidChamferDistances);
    }
    
    match (&entity1.geometry, &entity2.geometry) {
        (EntityGeometry::Line(line1), EntityGeometry::Line(line2)) => {
            fillet_two_lines(line1, line2, radius)
        }
        (EntityGeometry::Circle(circle), EntityGeometry::Line(line)) => {
            fillet_circle_line(circle, line, radius)
        }
        (EntityGeometry::Arc(arc), EntityGeometry::Line(line)) => {
            fillet_arc_line(arc, line, radius)
        }
        (EntityGeometry::Circle(c1), EntityGeometry::Circle(c2)) => {
            fillet_two_circles(c1, c2, radius)
        }
        _ => Err(ChamferError::UnsupportedEntityType),
    }
}

fn fillet_two_lines(line1: &Line, line2: &Line, radius: f64) -> Result<Vec<Entity>, ChamferError> {
    let intersection = intersect_line_line(line1.clone(), line2.clone());
    
    match intersection {
        IntersectionResult::Point(ip) => {
            let dir1 = line1.direction().normalize();
            let dir2 = line2.direction().normalize();
            
            let length1 = line1.length();
            let length2 = line2.length();
            
            let offset_dist1 = length1 * radius / (length1 + length2).max(1.0);
            let offset_dist2 = length2 * radius / (length1 + length2).max(1.0);
            
            let point1 = Point::new(
                ip.point.x - dir1.x * offset_dist1,
                ip.point.y - dir1.y * offset_dist1,
                0.0,
            );
            let point2 = Point::new(
                ip.point.x - dir2.x * offset_dist2,
                ip.point.y - dir2.y * offset_dist2,
                0.0,
            );
            
            let center = Point::new(
                (point1.x + point2.x) / 2.0,
                (point1.y + point2.y) / 2.0,
                0.0,
            );
            
            let start_angle = (point1 - center).to_vector2().angle();
            let end_angle = (point2 - center).to_vector2().angle();
            
            let fillet_arc = Arc::new(
                center,
                radius,
                start_angle,
                end_angle,
            );
            
            let entities = vec![
                Entity::new(EntityType::Arc, EntityGeometry::Arc(fillet_arc)),
            ];
            
            Ok(entities)
        }
        _ => Err(ChamferError::EntitiesDoNotIntersect),
    }
}

fn fillet_circle_line(circle: &Circle, line: &Line, radius: f64) -> Result<Vec<Entity>, ChamferError> {
    let result = intersect_line_circle(line.clone(), circle.clone());
    
    match result {
        IntersectionResult::Points(points) => {
            if points.len() >= 2 {
                let direction = line.direction().normalize();
                let perpendicular = Vector2::new(-direction.y, direction.x);
                
                let center = Point::new(
                    circle.center.x + perpendicular.x * radius,
                    circle.center.y + perpendicular.y * radius,
                    0.0,
                );
                
                let start_angle = (points[0].point - center).to_vector2().angle();
                let end_angle = (points[1].point - center).to_vector2().angle();
                
                let fillet_arc = Arc::new(
                    center,
                    radius,
                    start_angle,
                    end_angle,
                );
                
                Ok(vec![
                    Entity::new(EntityType::Arc, EntityGeometry::Arc(fillet_arc)),
                ])
            } else {
                Err(ChamferError::EntitiesDoNotIntersect)
            }
        }
        _ => Err(ChamferError::EntitiesDoNotIntersect),
    }
}

fn fillet_arc_line(arc: &Arc, line: &Line, radius: f64) -> Result<Vec<Entity>, ChamferError> {
    let circle = Circle::new(arc.center, arc.radius);
    let result = intersect_line_circle(line.clone(), circle);
    
    match result {
        IntersectionResult::Points(points) => {
            if !points.is_empty() {
                let direction = line.direction().normalize();
                let perpendicular = Vector2::new(-direction.y, direction.x);
                
                let center = Point::new(
                    arc.center.x + perpendicular.x * radius,
                    arc.center.y + perpendicular.y * radius,
                    0.0,
                );
                
                let start_angle = arc.start_angle;
                let end_angle = arc.end_angle;
                
                let fillet_arc = Arc::new(
                    center,
                    radius,
                    start_angle,
                    end_angle,
                );
                
                Ok(vec![
                    Entity::new(EntityType::Arc, EntityGeometry::Arc(fillet_arc)),
                ])
            } else {
                Err(ChamferError::EntitiesDoNotIntersect)
            }
        }
        _ => Err(ChamferError::EntitiesDoNotIntersect),
    }
}

fn fillet_two_circles(c1: &Circle, c2: &Circle, radius: f64) -> Result<Vec<Entity>, ChamferError> {
    let d = c1.center.distance_to(&c2.center);
    
    if d <= 1e-10 {
        return Err(ChamferError::EntitiesDoNotIntersect);
    }
    
    let r1 = c1.radius + radius;
    let r2 = c2.radius + radius;
    
    if d > r1 + r2 {
        return Err(ChamferError::EntitiesDoNotIntersect);
    }
    
    let center = Point::new(
        (c1.center.x + c2.center.x) / 2.0,
        (c1.center.y + c2.center.y) / 2.0,
        0.0,
    );
    
    let fillet_arc = Arc::new(
        center,
        radius,
        0.0,
        std::f64::consts::PI,
    );
    
    Ok(vec![
        Entity::new(EntityType::Arc, EntityGeometry::Arc(fillet_arc)),
    ])
}

/// 曲面过渡（Blend）失败的原因。
#[derive(Debug, Error)]
pub enum BlendError {
    /// 无法生成过渡曲面。
    #[error("无法创建过渡曲面")]
    CannotCreateBlend,
    
    /// 引导曲线无效（自交、退化或与两侧曲面不匹配）。
    #[error("引导曲线无效")]
    InvalidGuideCurve,
    
    /// 曲面类型不支持；目前仅支持两个 NURBS 曲面。
    #[error("不支持的曲面类型")]
    UnsupportedSurfaceType,
}

/// 曲面过渡的配置项。
pub struct BlendOptions {
    /// 期望的连续性等级，默认 `G1`。
    pub continuity: BlendContinuity,
    /// 过渡张紧度，默认 `1.0`；越大过渡面越贴近两侧曲面。
    pub tension: f64,
    /// 可选的引导曲线，用于约束过渡方向；为 `None` 时不做引导。
    pub guide_curve: Option<Box<dyn Curve>>,
    /// 是否生成对称过渡；默认 `false`。
    pub symmetry: bool,
}

/// 过渡曲面与两侧曲面的连续性等级。
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BlendContinuity {
    /// 仅位置连续（相接）。
    G0,
    /// 位置与切线连续。
    G1,
    /// 位置、切线与曲率连续。
    G2,
}

impl Default for BlendOptions {
    fn default() -> Self {
        Self {
            continuity: BlendContinuity::G1,
            tension: 1.0,
            guide_curve: None,
            symmetry: false,
        }
    }
}

/// 在两个曲面之间生成过渡曲面。
///
/// - `surface1` / `surface2`：两侧曲面实体；当前只支持两个 `NURBS` 曲面，其他几何类型返回
///   `BlendError::UnsupportedSurfaceType`。
/// - `guide`：可选引导曲线，用于约束过渡走向。
///
/// 返回新建的过渡曲面实体，输入实体不被修改。当前实现以第一个曲面的控制点重建 NURBS，
/// `guide` 暂不参与计算，等价于返回第一个曲面的副本。
pub fn blend_surfaces(surface1: &Entity, surface2: &Entity, guide: Option<Box<dyn Curve>>) -> Result<Entity, BlendError> {
    match (&surface1.geometry, &surface2.geometry) {
        (EntityGeometry::NURBS(nurbs1), EntityGeometry::NURBS(nurbs2)) => {
            blend_two_nurbs(nurbs1, nurbs2, guide)
        }
        _ => Err(BlendError::UnsupportedSurfaceType),
    }
}

fn blend_two_nurbs(nurbs1: &NURBS, nurbs2: &NURBS, _guide: Option<Box<dyn Curve>>) -> Result<Entity, BlendError> {
    let blended = NURBS::from_points(
        nurbs1.control_points.clone(),
        nurbs1.degree,
    );
    
    Ok(Entity::new(
        EntityType::NURBS,
        EntityGeometry::NURBS(blended),
    ))
}

/// 扫掠（Sweep）失败的原因。
#[derive(Debug, Error)]
pub enum SweepError {
    /// 扫掠路径无效（自交、退化或点数不足）。
    #[error("扫掠路径无效")]
    InvalidSweepPath,
    
    /// 扫掠轮廓无效（自交或退化）。
    #[error("扫掠轮廓无效")]
    InvalidSweepProfile,
    
    /// 无法生成扫掠曲面。
    #[error("无法创建扫掠曲面")]
    CannotCreateSweep,
    
    /// 轮廓无法调整到垂直于路径。
    #[error("轮廓无法垂直于路径")]
    ProfileNotPerpendicularToPath,
}

/// 扫掠的配置项。
pub struct SweepOptions {
    /// 扫掠方式，默认 `Constant`（等截面）。
    pub sweep_type: SweepType,
    /// 是否沿路径扭转轮廓；默认 `false`。
    pub twist: bool,
    /// 是否沿路径缩放轮廓；默认 `false`。
    pub scale: bool,
    /// 拔模角（弧度），默认 `0.0`；为 0 表示截面沿路径不变形。
    pub draft_angle: f64,
    /// 路径是否跟随轮廓法向；默认 `true`。
    pub path_follows_profile_normal: bool,
}

/// 扫掠方式。
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SweepType {
    /// 等截面扫掠：轮廓沿路径平移，形状不变。
    Constant,
    /// 变截面扫掠：轮廓随路径位置改变（扭转/缩放/拔模）。
    Variable,
    /// 跟随扫掠：轮廓姿态跟随路径切线方向。
    Follow,
}

impl Default for SweepOptions {
    fn default() -> Self {
        Self {
            sweep_type: SweepType::Constant,
            twist: false,
            scale: false,
            draft_angle: 0.0,
            path_follows_profile_normal: true,
        }
    }
}

/// 沿路径扫掠轮廓生成曲面。
///
/// - `profile`：被扫掠的轮廓曲线。
/// - `path`：扫掠路径曲线。
///
/// 返回新建的曲面实体，输入曲线不被修改。当前实现只返回一条由 `(0,0,0)` 与 `(1,0,0)`
/// 生成的 NURBS 占位结果，`path` 未参与计算。
pub fn sweep_profile(profile: &dyn Curve, path: &dyn Curve) -> Result<Entity, SweepError> {
    sweep_general_along_path(profile, path)
}

fn sweep_general_along_path(profile: &dyn Curve, _path: &dyn Curve) -> Result<Entity, SweepError> {
    let nurbs = NURBS::from_points(
        vec![Point::origin(), Point::new(1.0, 0.0, 0.0)],
        1,
    );
    
    Ok(Entity::new(
        EntityType::NURBS,
        EntityGeometry::NURBS(nurbs),
    ))
}

/// 放样（Loft）失败的原因。
#[derive(Debug, Error)]
pub enum LoftError {
    /// 参与放样的轮廓少于两个。
    #[error("放样轮廓不足")]
    InsufficientProfiles,
    
    /// 相邻轮廓之间不存在可用的对应关系。
    #[error("轮廓不相交")]
    ProfilesDoNotIntersect,
    
    /// 无法生成放样曲面。
    #[error("无法创建放样曲面")]
    CannotCreateLoft,
    
    /// 引导曲线数量不足，无法按引导放样。
    #[error("引导曲线数量不足")]
    InsufficientGuideCurves,
}

/// 放样的配置项。
pub struct LoftOptions {
    /// 放样方式，默认 `Smooth`。
    pub loft_type: LoftType,
    /// 引导曲线集合；为空表示不使用引导。
    pub guide_curves: Vec<Box<dyn Curve>>,
    /// 起始端张紧度，默认 `0.5`。
    pub start_tension: f64,
    /// 结束端张紧度，默认 `0.5`。
    pub end_tension: f64,
    /// 是否生成闭合放样曲面（首尾轮廓相连）；默认 `false`。
    pub closed: bool,
    /// 是否简化结果控制点；默认 `false`。
    pub simplify: bool,
    /// 拟合公差，默认 `0.01`。
    pub tolerance: f64,
}

/// 放样方式。
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LoftType {
    /// 直纹放样：相邻轮廓间线性过渡。
    Linear,
    /// 三次放样：按三次曲线过渡。
    Cubic,
    /// 平滑放样：整体光顺过渡；`LoftOptions` 的默认值。
    Smooth,
    /// 由引导曲线控制的放样。
    ByGuideCurves,
}

impl Default for LoftOptions {
    fn default() -> Self {
        Self {
            loft_type: LoftType::Smooth,
            guide_curves: Vec::new(),
            start_tension: 0.5,
            end_tension: 0.5,
            closed: false,
            simplify: false,
            tolerance: 0.01,
        }
    }
}

/// 按一组轮廓放样生成曲面。
///
/// - `profiles`：参与放样的轮廓；少于 2 个时返回 `LoftError::InsufficientProfiles`。
/// - `_guide_curves`：可选的引导曲线集合，当前实现未参与计算。
///
/// 返回新建的 NURBS 曲面实体：直线轮廓取其两个端点，圆/圆弧轮廓取圆心，其他曲线类型取
/// 原点，再以二次 NURBS 拟合，因此轮廓的完整形状不会保留。输入轮廓不被修改。
pub fn loft_profiles(profiles: Vec<Box<dyn Curve>>, _guide_curves: Option<Vec<Box<dyn Curve>>>) -> Result<Entity, LoftError> {
    if profiles.len() < 2 {
        return Err(LoftError::InsufficientProfiles);
    }
    
    let mut all_points = Vec::new();
    for profile in &profiles {
        match profile {
            Curve::Line(line) => {
                all_points.push(line.start);
                all_points.push(line.end);
            }
            Curve::Circle(circle) => {
                all_points.push(circle.center);
            }
            Curve::Arc(arc) => {
                all_points.push(arc.center);
            }
            _ => {
                all_points.push(Point::origin());
            }
        }
    }
    
    let nurbs = NURBS::from_points(all_points, 2);
    
    Ok(Entity::new(
        EntityType::NURBS,
        EntityGeometry::NURBS(nurbs),
    ))
}

/// 曲线光顺（Fairing）失败的原因。
#[derive(Debug, Error)]
pub enum FairingError {
    /// 曲线过于平坦（曲率接近 0），无需也无法光顺。
    #[error("曲线太平坦")]
    CurveTooFlat,
    
    /// 达到最大迭代次数仍未满足公差。
    #[error("超过最大迭代次数")]
    MaxIterationsExceeded,
    
    /// 无法对曲线执行光顺计算。
    #[error("无法平滑曲线")]
    CannotFairCurve,
}

/// 曲线光顺的配置项。
pub struct FairingOptions {
    /// 光顺收敛公差，默认 `0.001`。
    pub tolerance: f64,
    /// 最大迭代次数，默认 `100`；超出即返回 `FairingError::MaxIterationsExceeded`。
    pub max_iterations: u32,
    /// 光顺权重，默认 `1.0`；越大越接近原曲线。
    pub weight: f64,
    /// 是否保持两端点不动；默认 `true`。
    pub preserve_ends: bool,
}

impl Default for FairingOptions {
    fn default() -> Self {
        Self {
            tolerance: 0.001,
            max_iterations: 100,
            weight: 1.0,
            preserve_ends: true,
        }
    }
}

/// 对曲线做光顺处理，返回光顺后的曲线。
///
/// - `curve`：待光顺的曲线，不会被修改。
/// - `_tolerance`：收敛公差，当前实现未使用。
/// - `_max_iterations`：最大迭代次数，当前实现未使用。
///
/// 当前实现直接克隆输入曲线返回，因此结果与输入相同，也不会返回错误。
pub fn fair_curve(curve: &dyn Curve, _tolerance: f64, _max_iterations: u32) -> Result<Box<dyn Curve>, FairingError> {
    Ok(Box::new(curve.clone()))
}

/// 等距曲线（Parallel）失败的原因。
#[derive(Debug, Error)]
pub enum ParallelError {
    /// 等距结果自交。
    #[error("曲线自交")]
    SelfIntersection,
    
    /// 无法生成等距曲线，例如向内偏移后半径不为正。
    #[error("无法创建等距曲线")]
    CannotCreateParallel,
    
    /// 曲线类型不支持；目前仅支持直线、圆与圆弧。
    #[error("不支持的曲线类型")]
    UnsupportedCurveType,
}

/// 生成曲线的等距（平行）曲线。
///
/// - `curve`：支持直线、圆与圆弧，其他类型返回 `ParallelError::UnsupportedCurveType`。
/// - `distance`：偏移距离（世界坐标单位），非负即可，方向由 `side` 决定。
/// - `side`：`Left`/`Both` 沿法线正向偏移（圆、圆弧为增大半径），`Right` 反向偏移。
///
/// 返回新的曲线，输入曲线不被修改；圆或圆弧反向偏移后半径不为正时返回
/// `ParallelError::CannotCreateParallel`。
pub fn parallel_curve(curve: &dyn Curve, distance: f64, side: ParallelSide) -> Result<Box<dyn Curve>, ParallelError> {
    match curve {
        Curve::Line(line) => {
            let dir = line.direction().normalize();
            let normal = Vector2::new(-dir.y, dir.x);
            
            let offset_factor = match side {
                ParallelSide::Left => 1.0,
                ParallelSide::Right => -1.0,
                ParallelSide::Both => 1.0,
            };
            
            let parallel_line = Line::new(
                Point::new(line.start.x + normal.x * distance * offset_factor, 
                           line.start.y + normal.y * distance * offset_factor, 0.0),
                Point::new(line.end.x + normal.x * distance * offset_factor, 
                           line.end.y + normal.y * distance * offset_factor, 0.0),
            );
            Ok(Curve::Line(parallel_line))
        }
        Curve::Circle(circle) => {
            let new_radius = match side {
                ParallelSide::Left => circle.radius + distance,
                ParallelSide::Right => circle.radius - distance,
                ParallelSide::Both => circle.radius + distance,
            };
            if new_radius <= 0.0 {
                return Err(ParallelError::CannotCreateParallel);
            }
            let parallel_circle = Circle::new(circle.center, new_radius);
            Ok(Curve::Circle(parallel_circle))
        }
        Curve::Arc(arc) => {
            let new_radius = match side {
                ParallelSide::Left => arc.radius + distance,
                ParallelSide::Right => arc.radius - distance,
                ParallelSide::Both => arc.radius + distance,
            };
            if new_radius <= 0.0 {
                return Err(ParallelError::CannotCreateParallel);
            }
            let parallel_arc = Arc::new(arc.center, new_radius, arc.start_angle, arc.end_angle);
            Ok(Curve::Arc(parallel_arc))
        }
        _ => Err(ParallelError::UnsupportedCurveType),
    }
}
