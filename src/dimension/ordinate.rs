//! 坐标（Ordinate）尺寸标注：以基准原点为参照，标注特征点的 X 或 Y 坐标。
//! `OrdinateDimension` 描述单条坐标标注（特征点、引出点与标注方向）；`OrdinateDimensionSet` 用同一组特征点
//! 批量生成水平（X）与垂直（Y）两个方向的标注。两者都复用 `dimension::linear` 的 `DimensionGeometry` 与
//! `DimensionStyle`，并可经 `From` 转换为 `Entity`，从而进入统一的渲染、拾取与导出链路。
//! 所有坐标均为世界坐标，测量值是线性长度（`from_baseline` 生成时为相对基准原点的坐标差）。
use crate::geometry::{Point, Vector2, Line};
use crate::data_structure::{Entity, EntityType, EntityGeometry};
use super::linear::{DimensionGeometry, DimensionStyle, DimensionType};
use serde::{Serialize, Deserialize};
use std::fmt;

/// 单条坐标标注：记录被标注的特征点、引出点与标注方向。
/// 由 `new_x` / `new_y` / `new_horizontal` / `new_vertical` 构造，或用 `from_baseline` 按基准原点批量生成；
/// 坐标为世界坐标，`geometry.measurement` 为对应方向的坐标值（基准系列中为相对基准原点的差值）。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OrdinateDimension {
    /// 标注通用数据：`definition_points` 为 [特征点, 引出点]，`measurement` 为该方向的坐标值。
    pub geometry: DimensionGeometry,
    /// 被标注的特征点，世界坐标。
    pub feature_point: Point,
    /// 引出线端点，决定文字放置的一侧，世界坐标。
    pub leader_point: Point,
    /// 标注线：由特征点指向引出点。
    pub dimension_line: Line,
    /// 标注方向，决定测量取 x 还是 y，以及水平 / 垂直的分类。
    pub orientation: OrdinateOrientation,
    /// 是否按 X 方向测量与显示；`X` / `Horizontal` 为 true，其余为 false。
    pub use_x_axis: bool,
    /// 是否按 Y 方向测量与显示；`Y` / `Vertical` 为 true，其余为 false。
    pub use_y_axis: bool,
    /// 基准原点；`new_x` / `new_y` 由调用者给出（允许为 `None`），`from_baseline` 生成时必为 `Some`。
    pub baseline_origin: Option<Point>,
    /// 是否为基准标注；`from_baseline` 生成的序列中只有第一个元素为 true。
    pub is_baseline: bool,
}

/// 坐标标注的测量方向。
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum OrdinateOrientation {
    /// 沿 X 方向：测量特征点的 x 坐标。
    X,
    /// 沿 Y 方向：测量特征点的 y 坐标。
    Y,
    /// 水平标注，语义与 `X` 相同，用于基准标注系列。
    Horizontal,
    /// 垂直标注，语义与 `Y` 相同，用于基准标注系列。
    Vertical,
}

impl OrdinateDimension {
    /// 构造 X 方向坐标标注。
    /// - `feature_point`：被标注的特征点，测量值直接取它的 x 坐标（绝对值，不是相对基准的差值）。
    /// - `leader_point`：引出线端点，决定文字放置的一侧。
    /// - `style`：标注样式，用于生成 `geometry` 与显示文字。
    /// - `baseline_origin`：基准原点，只记录到字段，不参与测量值计算；无基准时传 `None`。
    /// 生成的标注 `use_x_axis` 为 true、`is_baseline` 为 false；不修改入参、不写盘。
    pub fn new_x(
        feature_point: Point,
        leader_point: Point,
        style: DimensionStyle,
        baseline_origin: Option<Point>,
    ) -> Self {
        let mut geometry = DimensionGeometry::new(DimensionType::Ordinate, style.clone());
        geometry.definition_points = vec![feature_point, leader_point];
        
        geometry.measurement = feature_point.x;
        geometry.actual_measurement = geometry.measurement;
        geometry.update_text();
        
        let direction = (leader_point.to_vector2() - feature_point.to_vector2()).normalize();
        
        let dimension_line = Line::new(feature_point, leader_point);
        
        Self {
            geometry,
            feature_point,
            leader_point,
            dimension_line,
            orientation: OrdinateOrientation::X,
            use_x_axis: true,
            use_y_axis: false,
            baseline_origin,
            is_baseline: false,
        }
    }
    
    /// 构造 Y 方向坐标标注。
    /// - `feature_point`：被标注的特征点，测量值直接取它的 y 坐标。
    /// - `leader_point`：引出线端点，决定文字放置的一侧。
    /// - `style`：标注样式。
    /// - `baseline_origin`：基准原点，只记录不使用；无基准时传 `None`。
    /// 生成的标注 `use_y_axis` 为 true、`is_baseline` 为 false。
    pub fn new_y(
        feature_point: Point,
        leader_point: Point,
        style: DimensionStyle,
        baseline_origin: Option<Point>,
    ) -> Self {
        let mut geometry = DimensionGeometry::new(DimensionType::Ordinate, style.clone());
        geometry.definition_points = vec![feature_point, leader_point];
        
        geometry.measurement = feature_point.y;
        geometry.actual_measurement = geometry.measurement;
        geometry.update_text();
        
        let direction = (leader_point.to_vector2() - feature_point.to_vector2()).normalize();
        
        let dimension_line = Line::new(feature_point, leader_point);
        
        Self {
            geometry,
            feature_point,
            leader_point,
            dimension_line,
            orientation: OrdinateOrientation::Y,
            use_x_axis: false,
            use_y_axis: true,
            baseline_origin,
            is_baseline: false,
        }
    }
    
    /// 构造水平坐标标注（语义与 `new_x` 相同，方向记作 `Horizontal`）。
    /// - `feature_point`：被标注的特征点，测量值取它的 x 坐标。
    /// - `leader_point`：引出线端点，决定文字位置。
    /// - `style`：标注样式。
    /// 不带基准原点：`baseline_origin` 为 `None`、`is_baseline` 为 false。
    pub fn new_horizontal(
        feature_point: Point,
        leader_point: Point,
        style: DimensionStyle,
    ) -> Self {
        let mut geometry = DimensionGeometry::new(DimensionType::Ordinate, style.clone());
        geometry.definition_points = vec![feature_point, leader_point];
        
        geometry.measurement = feature_point.x;
        geometry.actual_measurement = geometry.measurement;
        geometry.update_text();
        
        let dimension_line = Line::new(feature_point, leader_point);
        
        Self {
            geometry,
            feature_point,
            leader_point,
            dimension_line,
            orientation: OrdinateOrientation::Horizontal,
            use_x_axis: true,
            use_y_axis: false,
            baseline_origin: None,
            is_baseline: false,
        }
    }
    
    /// 构造垂直坐标标注（语义与 `new_y` 相同，方向记作 `Vertical`）。
    /// - `feature_point`：被标注的特征点，测量值取它的 y 坐标。
    /// - `leader_point`：引出线端点，决定文字位置。
    /// - `style`：标注样式。
    /// 不带基准原点：`baseline_origin` 为 `None`、`is_baseline` 为 false。
    pub fn new_vertical(
        feature_point: Point,
        leader_point: Point,
        style: DimensionStyle,
    ) -> Self {
        let mut geometry = DimensionGeometry::new(DimensionType::Ordinate, style.clone());
        geometry.definition_points = vec![feature_point, leader_point];
        
        geometry.measurement = feature_point.y;
        geometry.actual_measurement = geometry.measurement;
        geometry.update_text();
        
        let dimension_line = Line::new(feature_point, leader_point);
        
        Self {
            geometry,
            feature_point,
            leader_point,
            dimension_line,
            orientation: OrdinateOrientation::Vertical,
            use_x_axis: false,
            use_y_axis: true,
            baseline_origin: None,
            is_baseline: false,
        }
    }
    
    /// 由同一基准原点批量生成一族坐标标注，用于基准（baseline）标注。
    /// - `baseline_origin`：基准原点，测量值取「特征点坐标 − 基准坐标」。
    /// - `feature_points`：特征点列表，按顺序各生成一条标注，返回向量与它等长且顺序一致。
    /// - `orientation`：`X` / `Horizontal` 取 x 差值，`Y` / `Vertical` 取 y 差值。
    /// - `style`：标注样式，会克隆给每条标注。
    /// 每条标注的 `baseline_origin` 均为 `Some(baseline_origin)`，仅第一条的 `is_baseline` 为 true；
    /// 引出点按序号向外递增偏移 `(i + 1) * 3 * extension_line_offset`（X 方向朝 +y，Y 方向朝 +x）。
    pub fn from_baseline(
        baseline_origin: Point,
        feature_points: Vec<Point>,
        style: DimensionStyle,
        orientation: OrdinateOrientation,
    ) -> Vec<Self> {
        let mut dimensions = Vec::new();
        
        for (i, feature_point) in feature_points.iter().enumerate() {
            let is_baseline = i == 0;
            
            let mut geometry = DimensionGeometry::new(DimensionType::Ordinate, style.clone());
            geometry.definition_points = vec![*feature_point];
            
            match orientation {
                OrdinateOrientation::X | OrdinateOrientation::Horizontal => {
                    geometry.measurement = feature_point.x - baseline_origin.x;
                }
                OrdinateOrientation::Y | OrdinateOrientation::Vertical => {
                    geometry.measurement = feature_point.y - baseline_origin.y;
                }
            }
            
            geometry.actual_measurement = geometry.measurement;
            geometry.update_text();
            
            let offset = (i as f64 + 1.0) * style.extension_line_offset * 3.0;
            
            let direction = match orientation {
                OrdinateOrientation::X | OrdinateOrientation::Horizontal => Vector2::new(0.0, 1.0),
                _ => Vector2::new(1.0, 0.0),
            };
            
            let leader_point = Point::new(
                feature_point.x + direction.x * offset,
                feature_point.y + direction.y * offset,
                0.0,
            );
            
            let dimension_line = Line::new(*feature_point, leader_point);
            
            dimensions.push(Self {
                geometry,
                feature_point: *feature_point,
                leader_point,
                dimension_line,
                orientation,
                use_x_axis: matches!(orientation, OrdinateOrientation::X | OrdinateOrientation::Horizontal),
                use_y_axis: matches!(orientation, OrdinateOrientation::Y | OrdinateOrientation::Vertical),
                baseline_origin: Some(baseline_origin),
                is_baseline,
            });
        }
        
        dimensions
    }
    
    /// 交换特征点与引出点（就地修改），把标注翻到另一侧。
    /// 交换后调用 `geometry.calculate_measurement()`：对坐标标注该调用不会重算测量值，只按现有 `measurement` 刷新文字；
    /// `dimension_line` 与 `geometry.definition_points` 也不同步交换，如需保持一致请改用 `set_feature_point` / `set_leader_point`。
    pub fn flip(&mut self) {
        std::mem::swap(&mut self.feature_point, &mut self.leader_point);
        self.geometry.calculate_measurement();
    }
    
    /// 更新特征点并重算测量值（就地修改）。
    /// - `point`：新的特征点，世界坐标；同步写入 `geometry.definition_points[0]`。
    /// 测量值按当前 `orientation` 取 x（`X` / `Horizontal`）或 y（`Y` / `Vertical`），并刷新文字；引出点、标注线保持不变。
    pub fn set_feature_point(&mut self, point: Point) {
        self.feature_point = point;
        self.geometry.definition_points[0] = point;
        match self.orientation {
            OrdinateOrientation::X | OrdinateOrientation::Horizontal => {
                self.geometry.measurement = point.x;
            }
            _ => {
                self.geometry.measurement = point.y;
            }
        }
        self.geometry.actual_measurement = self.geometry.measurement;
        self.geometry.update_text();
    }
    
    /// 更新引出点（就地修改）。
    /// - `point`：新的引出点，世界坐标；同步写入 `geometry.definition_points[1]`，并以当前特征点为起点重建 `dimension_line`。
    /// 测量值与文字不受影响；调用前 `definition_points` 至少要有两个元素，否则会越界 panic。
    pub fn set_leader_point(&mut self, point: Point) {
        self.leader_point = point;
        self.geometry.definition_points[1] = point;
        self.dimension_line = Line::new(self.feature_point, point);
    }
}

impl From<OrdinateDimension> for Entity {
    fn from(dim: OrdinateDimension) -> Self {
        let leader_point = dim.leader_point;
        let geometry = dim.geometry;
        Entity::new(
            EntityType::Dimension,
            super::linear::entity_geometry_from_dimension(&geometry, leader_point, 0.0, false, false),
        )
    }
}

/// 坐标标注集合：对同一组特征点同时生成水平与垂直两个方向的坐标标注。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OrdinateDimensionSet {
    /// 水平（X）方向标注列表，每条对应一个特征点投影到基准轴后的 x 坐标。
    pub x_dimensions: Vec<OrdinateDimension>,
    /// 垂直（Y）方向标注列表，每条对应一个特征点投影到基准轴后的 y 坐标。
    pub y_dimensions: Vec<OrdinateDimension>,
    /// 两个方向共用的基准原点，世界坐标。
    pub baseline_origin: Point,
    /// 生成标注所用样式的副本；`add_feature_point` 追加新标注时继续复用它。
    pub style: DimensionStyle,
}

impl OrdinateDimensionSet {
    /// 由基准原点与一组特征点构造成对的坐标标注集合。
    /// - `origin`：基准原点；水平标注用 (特征点.x, origin.y)、垂直标注用 (origin.x, 特征点.y)，即先投影到基准轴上再作差。
    /// - `feature_points`：特征点列表，每个点各产生一条水平与一条垂直标注，两个列表长度都等于它。
    /// - `style`：标注样式，会克隆给每条标注。
    /// 所有标注的 `baseline_origin` 均为 `Some(origin)`，两个列表中各自只有第一条的 `is_baseline` 为 true；不修改入参、不写盘。
    pub fn new(
        origin: Point,
        feature_points: Vec<Point>,
        style: DimensionStyle,
    ) -> Self {
        let x_points: Vec<Point> = feature_points.iter().map(|p| Point::new(p.x, origin.y, p.z)).collect();
        let y_points: Vec<Point> = feature_points.iter().map(|p| Point::new(origin.x, p.y, p.z)).collect();
        
        let x_dimensions = OrdinateDimension::from_baseline(
            origin,
            x_points,
            style.clone(),
            OrdinateOrientation::Horizontal,
        );
        let y_dimensions = OrdinateDimension::from_baseline(
            origin,
            y_points,
            style.clone(),
            OrdinateOrientation::Vertical,
        );
        
        Self {
            x_dimensions,
            y_dimensions,
            baseline_origin: origin,
            style,
        }
    }
    
    /// 追加一个特征点，并在水平与垂直方向各生成一条新标注（就地修改）。
    /// - `point`：新特征点，世界坐标；同样先投影到基准轴再测量。
    /// 新标注的 `is_baseline` 恒为 true（单点序列序号为 0），引出点偏移固定为 `3 * extension_line_offset`，不随已有标注数量递增。
    pub fn add_feature_point(&mut self, point: Point) {
        let x_point = Point::new(point.x, self.baseline_origin.y, point.z);
        let y_point = Point::new(self.baseline_origin.x, point.y, point.z);
        
        self.x_dimensions.extend(OrdinateDimension::from_baseline(
            self.baseline_origin,
            vec![x_point],
            self.style.clone(),
            OrdinateOrientation::Horizontal,
        ));
        
        self.y_dimensions.extend(OrdinateDimension::from_baseline(
            self.baseline_origin,
            vec![y_point],
            self.style.clone(),
            OrdinateOrientation::Vertical,
        ));
    }
    
    /// 把集合内全部标注转换为 `Entity`，顺序为「先全部水平、后全部垂直」，用于加入文档或渲染。
    /// 不修改自身；转换基于克隆，返回的实体与集合不再保持关联。
    pub fn to_entities(&self) -> Vec<Entity> {
        let mut entities = Vec::new();
        
        for dim in &self.x_dimensions {
            entities.push(dim.clone().into());
        }
        
        for dim in &self.y_dimensions {
            entities.push(dim.clone().into());
        }
        
        entities
    }
}
