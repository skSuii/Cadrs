//! 几何约束求解：约束类型定义、约束记录与迭代式求解器。
//!
//! 核心概念：
//! - `GeometricConstraint`：约束种类（重合、平行、垂直、相切、等长等），决定求解时采用的残差度量。
//! - `ConstraintEntity`：一条约束记录，把约束种类绑定到一或两个实体 id 上，可附带参考点。
//! - `GeometricSolver`：按容差迭代消除残差的求解器，维护实体状态（`EntityState`）与求解快照（`SolverState`）。
//!
//! 与其他模块的关系：只依赖 `geometry::Point` 表示世界坐标点（`Point` 为本文件的类型别名），
//! 不直接操作 Document，实体 id 由调用方与文档中的 Entity 自行对应。坐标为世界坐标，长度为无单位数值，角度一律用弧度。

use serde::{Serialize, Deserialize};
use std::collections::{HashMap, HashSet};
use std::fmt;
use std::f64::consts::PI;

/// 几何约束的种类。
///
/// 每个变体对应一种残差度量；求解时按种类选择评估与修正方式，
/// 因而决定了约束需要几个实体（见 `GeometricSolver::validate_constraint`）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GeometricConstraint {
    /// 重合：两实体位置相同，残差为两点距离；也是 `Default` 的取值。
    Coincident,
    /// 垂直：两实体方向成 90°，残差为夹角与 π/2 之差的绝对值。
    Perpendicular,
    /// 平行：两实体方向相同或相反，残差为夹角相对 0 或 π 的最小偏差。
    Parallel,
    /// 相切：两曲线相切，残差取最近点距离；缺少圆心信息时退化为夹角偏差的 100 倍惩罚。
    Tangent,
    /// 水平：单实体方向沿 X 轴，残差为方向向量 y 分量的绝对值，修正时把角度置 0。
    Horizontal,
    /// 竖直：单实体方向沿 Y 轴，残差为方向向量 x 分量的绝对值，修正时把角度置 π/2。
    Vertical,
    /// 等长：两实体长度相等，残差为长度之差的绝对值。
    EqualLength,
    /// 等半径：两实体半径相等，残差为半径之差的绝对值。
    EqualRadius,
    /// 对称：两实体中点重合，残差为两中点距离（不建模真实对称轴）。
    Symmetric,
    /// 中点：参考点落在线段中点，残差为中点到该点的距离，需要 `point_on_first` 或 `point_on_second`。
    Midpoint,
    /// 中心：参考点落在实体中心，残差为中心到该点的距离；实体数量校验恒为通过。
    Center,
    /// 固定：锁定实体，残差取该实体状态的 `drag_distance`，由调用方写入拖动偏移。
    Fix,
}

impl Default for GeometricConstraint {
    fn default() -> Self {
        GeometricConstraint::Coincident
    }
}

impl GeometricConstraint {
    /// 返回约束的显示名（英文），供界面标签使用；等长、等半径名称中带空格。
    pub fn name(&self) -> &str {
        match self {
            GeometricConstraint::Coincident => "Coincident",
            GeometricConstraint::Perpendicular => "Perpendicular",
            GeometricConstraint::Parallel => "Parallel",
            GeometricConstraint::Tangent => "Tangent",
            GeometricConstraint::Horizontal => "Horizontal",
            GeometricConstraint::Vertical => "Vertical",
            GeometricConstraint::EqualLength => "Equal Length",
            GeometricConstraint::EqualRadius => "Equal Radius",
            GeometricConstraint::Symmetric => "Symmetric",
            GeometricConstraint::Midpoint => "Midpoint",
            GeometricConstraint::Center => "Center",
            GeometricConstraint::Fix => "Fix",
        }
    }

    /// 返回约束的界面图标字符（Unicode 符号，非 ASCII），供工具栏按钮使用。
    pub fn icon(&self) -> &str {
        match self {
            GeometricConstraint::Coincident => "⭕",
            GeometricConstraint::Perpendicular => "⊥",
            GeometricConstraint::Parallel => "∥",
            GeometricConstraint::Tangent => "◎",
            GeometricConstraint::Horizontal => "—",
            GeometricConstraint::Vertical => "|",
            GeometricConstraint::EqualLength => "=",
            GeometricConstraint::EqualRadius => "≅",
            GeometricConstraint::Symmetric => "◈",
            GeometricConstraint::Midpoint => "◉",
            GeometricConstraint::Center => "⊕",
            GeometricConstraint::Fix => "📌",
        }
    }
}

/// 一条具体的约束记录：把约束种类绑定到参与约束的实体 id 上。
///
/// 实体 id 需与 `GeometricSolver::entities` 中的键一致；是否需要 `second_entity`
/// 由约束种类决定，可在加入求解器前用 `GeometricSolver::validate_constraint` 校验。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConstraintEntity {
    /// 约束的唯一标识，`new` 时由 UUID v4 生成，用于查找与删除。
    pub id: String,
    /// 约束种类，决定求解时采用的残差度量；默认 `Coincident`。
    pub constraint_type: GeometricConstraint,
    /// 参与约束的第一个实体 id。
    pub first_entity: String,
    /// 参与约束的第二个实体 id；仅重合、平行、垂直、相切、等长、等半径、对称需要。
    pub second_entity: Option<String>,
    /// 附加在第一个实体上的参考点（世界坐标），用于中点、中心等需要具体点的约束。
    pub point_on_first: Option<crate::geometry::Point>,
    /// 附加在第二个实体上的参考点（世界坐标）；求值时优先于 `point_on_first`。
    pub point_on_second: Option<crate::geometry::Point>,
    /// 该约束是否已生效，由调用方维护，求解器本身不修改。
    pub is_applied: bool,
    /// 关联实体是否正在被拖动，由调用方维护，可用于临时放宽约束。
    pub is_dragging: bool,
    /// 是否为参考（仅测量、不驱动几何）约束；为 true 时调用方可跳过几何修正。
    pub reference: bool,
}

impl Default for ConstraintEntity {
    fn default() -> Self {
        Self::new()
    }
}

impl ConstraintEntity {
    /// 创建一条默认的重合约束，`id` 由 UUID 生成；两个实体 id 均为空，需要用 `with_*` 补齐。
    pub fn new() -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            constraint_type: GeometricConstraint::Coincident,
            first_entity: String::new(),
            second_entity: None,
            point_on_first: None,
            point_on_second: None,
            is_applied: false,
            is_dragging: false,
            reference: false,
        }
    }

    /// 设置约束种类并返回自身，用于链式构建。
    pub fn with_type(mut self, constraint_type: GeometricConstraint) -> Self {
        self.constraint_type = constraint_type;
        self
    }

    /// 设置第一个实体 id（须与求解器中的实体键一致）并返回自身。
    pub fn with_first_entity(mut self, entity: &str) -> Self {
        self.first_entity = entity.to_string();
        self
    }

    /// 设置第二个实体 id 并返回自身；对单实体约束（水平/竖直/中点/固定）会使校验失败。
    pub fn with_second_entity(mut self, entity: &str) -> Self {
        self.second_entity = Some(entity.to_string());
        self
    }

    /// 设置第一个实体上的参考点（世界坐标）并返回自身。
    pub fn with_first_point(mut self, point: crate::geometry::Point) -> Self {
        self.point_on_first = Some(point);
        self
    }

    /// 设置第二个实体上的参考点（世界坐标）并返回自身。
    pub fn with_second_point(mut self, point: crate::geometry::Point) -> Self {
        self.point_on_second = Some(point);
        self
    }
}

impl fmt::Display for ConstraintEntity {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}: {} on {}",
            self.constraint_type.name(),
            self.first_entity,
            self.second_entity.as_ref().unwrap_or(&"None".to_string())
        )
    }
}

/// 一次求解的结果快照。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SolverState {
    /// 求解是否收敛，即最后一次 `solve` 退出时最大残差已低于容差。
    pub solved: bool,
    /// 实际迭代次数，`solve` 开始时清零。
    pub iterations: u32,
    /// 退出时的最大残差，度量单位与实体坐标一致。
    pub error: f64,
    /// 残差容忍上限，默认 1e-6；残差小于该值即认为收敛。
    pub max_error: f64,
    /// 被约束的自由度总数，为各实体 `constrained_dof` 之和。
    pub constrained_dof: u32,
    /// 剩余自由度数 = 实体数 × 3 − `constrained_dof`，为 0 表示完全约束。
    pub free_dof: u32,
}

impl Default for SolverState {
    fn default() -> Self {
        Self::new()
    }
}

impl SolverState {
    /// 创建初始状态：`solved` 为 true、迭代次数 0、残差 0、容差 1e-6，自由度计数均为 0。
    pub fn new() -> Self {
        Self {
            solved: true,
            iterations: 0,
            error: 0.0,
            max_error: 1e-6,
            constrained_dof: 0,
            free_dof: 0,
        }
    }
}

/// 基于迭代修正的几何约束求解器。
///
/// `solve` 反复评估所有约束的最大残差，未达 `tolerance` 时对重合/水平/竖直约束施加修正，
/// 直到收敛或达到 `max_iterations`。增删约束、清空约束时若 `auto_solve` 为 true 会自动求解。
#[derive(Debug, Clone)]
pub struct GeometricSolver {
    /// 已加入的约束列表，顺序即每轮修正的先后顺序。
    pub constraints: Vec<ConstraintEntity>,
    /// 实体 id → 简化状态（位置/角度/长度/半径）的映射，由 `add_entity` 登记。
    pub entities: HashMap<String, EntityState>,
    /// 残差收敛容差，默认 1e-6；越小迭代越久。
    pub tolerance: f64,
    /// 单次求解的最大迭代次数，默认 1000，达到上限仍未收敛则 `SolverState::solved` 为 false。
    pub max_iterations: u32,
    /// 最近一次求解的结果快照。
    pub state: SolverState,
    /// 为 true（默认）时 `add_constraint`/`remove_constraint`/`clear_constraints` 会立即触发 `solve`。
    pub auto_solve: bool,
    /// 是否启用约束推断；仅供调用方参考，求解器本身不读取该标志。
    pub inference_constraints: bool,
}

impl Default for GeometricSolver {
    fn default() -> Self {
        Self::new()
    }
}

impl GeometricSolver {
    /// 创建空求解器：无约束、无实体；容差 1e-6、最多 1000 次迭代，`auto_solve` 与约束推断默认开启。
    pub fn new() -> Self {
        Self {
            constraints: Vec::new(),
            entities: HashMap::new(),
            tolerance: 1e-6,
            max_iterations: 1000,
            state: SolverState::new(),
            auto_solve: true,
            inference_constraints: true,
        }
    }

    /// 加入一条约束，成功时将其复制进 `constraints`（不消耗传入值）。
    /// - `constraint`：待加入的约束；需通过 `validate_constraint` 的实体数量校验。
    /// 返回：加入成功返回 true；校验失败返回 false 且不修改求解器。`auto_solve` 为 true 时随后求解。
    pub fn add_constraint(&mut self, constraint: ConstraintEntity) -> bool {
        if self.validate_constraint(&constraint) {
            self.constraints.push(constraint.clone());
            if self.auto_solve {
                self.solve();
            }
            true
        } else {
            false
        }
    }

    /// 按 id 删除约束（会删除所有同 id 项）；`auto_solve` 为 true 时随后求解。
    /// 返回：删除了至少一条返回 true，未找到任何匹配返回 false。
    pub fn remove_constraint(&mut self, constraint_id: &str) -> bool {
        let original_len = self.constraints.len();
        self.constraints.retain(|c| c.id != constraint_id);
        if self.auto_solve {
            self.solve();
        }
        self.constraints.len() != original_len
    }

    /// 按 id 查找约束。
    /// 返回：命中返回约束引用；无匹配返回 `None`。
    pub fn get_constraint(&self, constraint_id: &str) -> Option<&ConstraintEntity> {
        self.constraints.iter().find(|c| c.id == constraint_id)
    }

    /// 列出引用指定实体的全部约束（`first_entity` 或 `second_entity` 与之字符串相等）。
    /// - `entity_id`：实体 id，精确匹配，不做归一化。
    /// 返回：匹配约束的引用列表；无匹配时为空向量。
    pub fn constraints_for_entity(&self, entity_id: &str) -> Vec<&ConstraintEntity> {
        self.constraints
            .iter()
            .filter(|c| c.first_entity == entity_id || c.second_entity.as_ref() == Some(&entity_id.to_string()))
            .collect()
    }

    /// 校验约束给出的实体数量是否与其种类匹配。
    /// 重合/垂直/平行/相切/等长/等半径/对称必须给出 `second_entity`；水平/竖直/中点/固定必须不给出；中心恒为合法。
    /// 返回：结构合法返回 true。只检查字段组合，不检查实体是否真的存在于 `entities` 中。
    pub fn validate_constraint(&self, constraint: &ConstraintEntity) -> bool {
        match constraint.constraint_type {
            GeometricConstraint::Coincident
            | GeometricConstraint::Perpendicular
            | GeometricConstraint::Parallel
            | GeometricConstraint::Tangent
            | GeometricConstraint::EqualLength
            | GeometricConstraint::EqualRadius
            | GeometricConstraint::Symmetric => constraint.second_entity.is_some(),
            GeometricConstraint::Horizontal
            | GeometricConstraint::Vertical
            | GeometricConstraint::Midpoint
            | GeometricConstraint::Fix => constraint.second_entity.is_none(),
            GeometricConstraint::Center => true,
        }
    }

    /// 迭代求解：每轮取所有约束残差的最大值，小于 `tolerance` 即收敛，否则施加修正，最多 `max_iterations` 轮。
    /// 返回：本次求解后的 `SolverState` 引用，含迭代次数、最大残差、是否收敛与自由度统计。
    /// 副作用：就地修改 `entities`——重合约束让两实体各移动一半距离，水平/竖直约束直接改写实体角度；
    /// 达到迭代上限仍未收敛时 `solved` 为 false，实体停留在最后一次修正的结果上。
    pub fn solve(&mut self) -> &SolverState {
        self.state.iterations = 0;
        self.state.error = 0.0;
        self.state.solved = true;

        let mut converged = false;
        let mut error = f64::MAX;

        for _ in 0..self.max_iterations {
            self.state.iterations += 1;
            error = self.evaluate_constraints();

            if error < self.tolerance {
                converged = true;
                break;
            }

            self.apply_corrections(error);
        }

        self.state.error = error;
        self.state.solved = converged;

        self.calculate_dof();
        &self.state
    }

    fn evaluate_constraints(&self) -> f64 {
        let mut max_error: f64 = 0.0;

        for constraint in &self.constraints {
            let error = match constraint.constraint_type {
                GeometricConstraint::Coincident => self.evaluate_coincident(constraint),
                GeometricConstraint::Perpendicular => self.evaluate_perpendicular(constraint),
                GeometricConstraint::Parallel => self.evaluate_parallel(constraint),
                GeometricConstraint::Tangent => self.evaluate_tangent(constraint),
                GeometricConstraint::Horizontal => self.evaluate_horizontal(constraint),
                GeometricConstraint::Vertical => self.evaluate_vertical(constraint),
                GeometricConstraint::EqualLength => self.evaluate_equal_length(constraint),
                GeometricConstraint::EqualRadius => self.evaluate_equal_radius(constraint),
                GeometricConstraint::Symmetric => self.evaluate_symmetric(constraint),
                GeometricConstraint::Midpoint => self.evaluate_midpoint(constraint),
                GeometricConstraint::Center => self.evaluate_center(constraint),
                GeometricConstraint::Fix => self.evaluate_fix(constraint),
            };
            max_error = max_error.max(error);
        }

        max_error
    }

    fn evaluate_coincident(&self, constraint: &ConstraintEntity) -> f64 {
        if let (Some(first), Some(second)) = (
            self.get_entity_position(&constraint.first_entity),
            constraint.second_entity.as_ref().and_then(|e| self.get_entity_position(e)),
        ) {
            first.distance_to(&second)
        } else {
            0.0
        }
    }

    fn evaluate_perpendicular(&self, constraint: &ConstraintEntity) -> f64 {
        if let (Some(first), Some(second)) = (
            self.get_entity_direction(&constraint.first_entity),
            self.get_entity_direction(&constraint.second_entity.as_ref().unwrap()),
        ) {
            let dot = first.x * second.x + first.y * second.y;
            let angle = dot.abs().acos();
            let target = PI / 2.0;
            (angle - target).abs()
        } else {
            0.0
        }
    }

    fn evaluate_parallel(&self, constraint: &ConstraintEntity) -> f64 {
        if let (Some(first), Some(second)) = (
            self.get_entity_direction(&constraint.first_entity),
            self.get_entity_direction(&constraint.second_entity.as_ref().unwrap()),
        ) {
            let dot = first.x * second.x + first.y * second.y;
            let angle = dot.abs().acos();
            angle.min(PI - angle)
        } else {
            0.0
        }
    }

    fn evaluate_tangent(&self, constraint: &ConstraintEntity) -> f64 {
        if let (Some(first), Some(second)) = (
            self.get_entity_direction(&constraint.first_entity),
            self.get_entity_direction(&constraint.second_entity.as_ref().unwrap()),
        ) {
            let center1 = self.get_entity_center(&constraint.first_entity);
            let center2 = self.get_entity_center(&constraint.second_entity.as_ref().unwrap());

            if let (Some(c1), Some(c2)) = (center1, center2) {
                let point1 = self.get_entity_closest_point(&constraint.first_entity, &c2);
                let point2 = self.get_entity_closest_point(&constraint.second_entity.as_ref().unwrap(), &c1);

                point1.distance_to(&point2)
            } else {
                let dot = first.x * second.x + first.y * second.y;
                let angle = dot.abs().acos();
                angle.min(PI - angle) * 100.0
            }
        } else {
            0.0
        }
    }

    fn evaluate_horizontal(&self, constraint: &ConstraintEntity) -> f64 {
        if let Some(direction) = self.get_entity_direction(&constraint.first_entity) {
            direction.y.abs()
        } else {
            0.0
        }
    }

    fn evaluate_vertical(&self, constraint: &ConstraintEntity) -> f64 {
        if let Some(direction) = self.get_entity_direction(&constraint.first_entity) {
            direction.x.abs()
        } else {
            0.0
        }
    }

    fn evaluate_equal_length(&self, constraint: &ConstraintEntity) -> f64 {
        let len1 = self.get_entity_length(&constraint.first_entity);
        let len2 = self.get_entity_length(constraint.second_entity.as_ref().unwrap());
        (len1 - len2).abs()
    }

    fn evaluate_equal_radius(&self, constraint: &ConstraintEntity) -> f64 {
        let r1 = self.get_entity_radius(&constraint.first_entity);
        let r2 = self.get_entity_radius(constraint.second_entity.as_ref().unwrap());
        (r1 - r2).abs()
    }

    fn evaluate_symmetric(&self, constraint: &ConstraintEntity) -> f64 {
        if let (Some(line1), Some(line2)) = (
            self.get_entity_points(&constraint.first_entity),
            constraint.second_entity.as_ref().and_then(|e| self.get_entity_points(e)),
        ) {
            let mid1 = Point::new(
                (line1.0.x + line1.1.x) / 2.0,
                (line1.0.y + line1.1.y) / 2.0,
                0.0,
            );
            let mid2 = Point::new(
                (line2.0.x + line2.1.x) / 2.0,
                (line2.0.y + line2.1.y) / 2.0,
                0.0,
            );
            mid1.distance_to(&mid2)
        } else {
            0.0
        }
    }

    fn evaluate_midpoint(&self, constraint: &ConstraintEntity) -> f64 {
        if let (Some(points), Some(entity_point)) = (
            self.get_entity_points(&constraint.first_entity),
            constraint.point_on_second.as_ref().or(constraint.point_on_first.as_ref()),
        ) {
            let mid = Point::new(
                (points.0.x + points.1.x) / 2.0,
                (points.0.y + points.1.y) / 2.0,
                0.0,
            );
            mid.distance_to(entity_point)
        } else {
            0.0
        }
    }

    fn evaluate_center(&self, constraint: &ConstraintEntity) -> f64 {
        if let Some(center) = self.get_entity_center(&constraint.first_entity) {
            if let Some(point) = constraint.point_on_second.as_ref().or(constraint.point_on_first.as_ref()) {
                center.distance_to(point)
            } else {
                0.0
            }
        } else {
            0.0
        }
    }

    fn evaluate_fix(&self, constraint: &ConstraintEntity) -> f64 {
        if let Some(state) = self.entities.get(&constraint.first_entity) {
            state.drag_distance
        } else {
            0.0
        }
    }

    fn apply_corrections(&mut self, _error: f64) {
        let constraints: Vec<_> = self.constraints.clone();
        for constraint in constraints {
            match constraint.constraint_type {
                GeometricConstraint::Coincident => self.apply_coincident_correction(&constraint),
                GeometricConstraint::Horizontal => self.apply_horizontal_correction(&constraint),
                GeometricConstraint::Vertical => self.apply_vertical_correction(&constraint),
                _ => {}
            }
        }
    }

    fn apply_coincident_correction(&mut self, constraint: &ConstraintEntity) {
        if let (Some(first_pos), Some(second_pos)) = (
            self.get_entity_position(&constraint.first_entity),
            constraint.second_entity.as_ref().and_then(|e| self.get_entity_position(e)),
        ) {
            let correction = Point::new(
                (second_pos.x - first_pos.x) / 2.0,
                (second_pos.y - first_pos.y) / 2.0,
                0.0,
            );
            self.move_entity(&constraint.first_entity, &correction);
            self.move_entity(constraint.second_entity.as_ref().unwrap(), &Point::new(-correction.x, -correction.y, 0.0));
        }
    }

    fn apply_horizontal_correction(&mut self, constraint: &ConstraintEntity) {
        if let Some(state) = self.entities.get_mut(&constraint.first_entity) {
            state.angle = 0.0;
        }
    }

    fn apply_vertical_correction(&mut self, constraint: &ConstraintEntity) {
        if let Some(state) = self.entities.get_mut(&constraint.first_entity) {
            state.angle = PI / 2.0;
        }
    }

    fn get_entity_position(&self, entity_id: &str) -> Option<Point> {
        self.entities.get(entity_id).map(|s| s.position)
    }

    fn get_entity_direction(&self, entity_id: &str) -> Option<Point> {
        self.entities.get(entity_id).map(|s| {
            Point::new(s.angle.cos(), s.angle.sin(), 0.0)
        })
    }

    fn get_entity_center(&self, entity_id: &str) -> Option<Point> {
        self.entities.get(entity_id).map(|s| s.center)
    }

    fn get_entity_points(&self, entity_id: &str) -> Option<(Point, Point)> {
        self.entities.get(entity_id).map(|s| {
            let start = Point::new(
                s.position.x + (s.angle - s.length / 2.0).cos() * s.length,
                s.position.y + (s.angle - s.length / 2.0).sin() * s.length,
                0.0,
            );
            let end = Point::new(
                s.position.x + (s.angle + s.length / 2.0).cos() * s.length,
                s.position.y + (s.angle + s.length / 2.0).sin() * s.length,
                0.0,
            );
            (start, end)
        })
    }

    fn get_entity_length(&self, entity_id: &str) -> f64 {
        self.entities.get(entity_id).map(|s| s.length).unwrap_or(0.0)
    }

    fn get_entity_radius(&self, entity_id: &str) -> f64 {
        self.entities.get(entity_id).map(|s| s.radius).unwrap_or(0.0)
    }

    fn get_entity_closest_point(&self, entity_id: &str, point: &Point) -> Point {
        if let Some(state) = self.entities.get(entity_id) {
            let dx = point.x - state.position.x;
            let dy = point.y - state.position.y;
            let angle = dy.atan2(dx);
            Point::new(
                state.position.x + angle.cos() * state.length / 2.0,
                state.position.y + angle.sin() * state.length / 2.0,
                0.0,
            )
        } else {
            *point
        }
    }

    fn move_entity(&mut self, entity_id: &str, delta: &Point) {
        if let Some(state) = self.entities.get_mut(entity_id) {
            state.position.x += delta.x;
            state.position.y += delta.y;
            state.center.x += delta.x;
            state.center.y += delta.y;
        }
    }

    fn calculate_dof(&mut self) {
        let mut total_dof = 0;
        let mut constrained_dof = 0;

        for entity in self.entities.values() {
            total_dof += 3;
            constrained_dof += entity.constrained_dof;
        }

        self.state.constrained_dof = constrained_dof;
        self.state.free_dof = total_dof - constrained_dof;
    }

    /// 登记一个受约束实体，已存在同 id 时直接覆盖其状态。
    /// - `entity_id`：实体标识，必须与约束里使用的 id 一致。
    /// - `position`：实体位置（世界坐标），同时作为初始中心。
    /// - `angle`：方位角，弧度。
    /// - `length`：长度，用于等长约束与端点推算。
    /// 新状态的半径与已约束自由度均置 0，拖动偏移置 0。
    pub fn add_entity(&mut self, entity_id: &str, position: Point, angle: f64, length: f64) {
        self.entities.insert(
            entity_id.to_string(),
            EntityState {
                position,
                center: position,
                angle,
                length,
                radius: 0.0,
                constrained_dof: 0,
                drag_distance: 0.0,
            },
        );
    }

    /// 把实体位置设为给定点，并按新旧位置之差同步平移中心。
    /// - `position`：新的世界坐标位置。
    /// 实体不存在时不做任何修改；角度、长度与半径保持不变。
    pub fn set_entity_position(&mut self, entity_id: &str, position: Point) {
        if let Some(state) = self.entities.get_mut(entity_id) {
            let dx = position.x - state.position.x;
            let dy = position.y - state.position.y;
            state.position = position;
            state.center.x += dx;
            state.center.y += dy;
        }
    }

    /// 清空全部约束，并把各实体的 `constrained_dof` 归零（保留实体本身）；`auto_solve` 为 true 时随后求解。
    pub fn clear_constraints(&mut self) {
        self.constraints.clear();
        for state in self.entities.values_mut() {
            state.constrained_dof = 0;
        }
        if self.auto_solve {
            self.solve();
        }
    }

    /// 返回当前约束条数（含尚未生效的约束）。
    pub fn constraint_count(&self) -> usize {
        self.constraints.len()
    }

    /// 草图是否已完全约束。
    ///
    /// 仅当至少存在一个受约束实体、且剩余自由度 `free_dof` 为 0 时返回 `true`。
    /// 空求解器（尚未加入任何实体）不算「完全约束」，避免刚创建就被判定为已约束。
    /// 是否已完全约束：最近一次求解后 `state.free_dof == 0`。
    /// 结论依赖 `state`，约束变更后未重新求解时可能过时。
    pub fn is_fully_constrained(&self) -> bool {
        !self.entities.is_empty() && self.state.free_dof == 0
    }

    /// 是否过约束：被约束自由度总数超过 实体数 × 3，视为存在冗余或冲突约束。
    pub fn is_over_constrained(&self) -> bool {
        (self.state.constrained_dof as usize) > self.entities.len() * 3
    }
}

/// 求解器内部维护的实体简化状态，每个实体按 3 个自由度计入统计。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EntityState {
    /// 实体参考位置，世界坐标。
    pub position: Point,
    /// 实体中心，随 `move_entity` 与 `set_entity_position` 和位置同步平移。
    pub center: Point,
    /// 方位角，弧度；水平修正置 0，竖直修正置 π/2。
    pub angle: f64,
    /// 长度，用于等长约束与端点推算。
    pub length: f64,
    /// 半径，`add_entity` 时固定为 0，由等半径等约束使用。
    pub radius: f64,
    /// 该实体已被约束的自由度计数，用于统计 `SolverState::free_dof`。
    pub constrained_dof: u32,
    /// 最近一次拖动偏移，`Fix` 约束以其为残差，由调用方写入。
    pub drag_distance: f64,
}

impl Default for EntityState {
    fn default() -> Self {
        Self::new()
    }
}

impl EntityState {
    /// 创建全零状态：位置与中心均在原点，角度、长度、半径与拖动距离均为 0。
    pub fn new() -> Self {
        Self {
            position: Point::origin(),
            center: Point::origin(),
            angle: 0.0,
            length: 0.0,
            radius: 0.0,
            constrained_dof: 0,
            drag_distance: 0.0,
        }
    }
}

/// 本模块使用的点类型别名，直接复用 `geometry::Point`，表示世界坐标点。
pub type Point = crate::geometry::Point;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constraint_entity_creation() {
        let constraint = ConstraintEntity::new()
            .with_type(GeometricConstraint::Coincident)
            .with_first_entity("line1")
            .with_second_entity("line2");

        assert_eq!(constraint.constraint_type, GeometricConstraint::Coincident);
        assert_eq!(constraint.first_entity, "line1");
        assert_eq!(constraint.second_entity, Some("line2".to_string()));
    }

    #[test]
    fn test_solver_creation() {
        let solver = GeometricSolver::new();
        assert_eq!(solver.constraint_count(), 0);
        assert!(!solver.is_fully_constrained());
    }

    #[test]
    fn test_add_constraint() {
        let mut solver = GeometricSolver::new();
        solver.auto_solve = false;

        let constraint = ConstraintEntity::new()
            .with_type(GeometricConstraint::Coincident)
            .with_first_entity("line1")
            .with_second_entity("line2");

        assert!(solver.add_constraint(constraint));
        assert_eq!(solver.constraint_count(), 1);
    }

    #[test]
    fn test_validate_constraint() {
        let solver = GeometricSolver::new();

        let valid_constraint = ConstraintEntity::new()
            .with_type(GeometricConstraint::Coincident)
            .with_first_entity("line1")
            .with_second_entity("line2");

        let invalid_constraint = ConstraintEntity::new()
            .with_type(GeometricConstraint::Horizontal)
            .with_first_entity("line1")
            .with_second_entity("line2");

        assert!(solver.validate_constraint(&valid_constraint));
        assert!(!solver.validate_constraint(&invalid_constraint));
    }

    #[test]
    fn test_remove_constraint() {
        let mut solver = GeometricSolver::new();
        solver.auto_solve = false;

        let constraint = ConstraintEntity::new()
            .with_type(GeometricConstraint::Coincident)
            .with_first_entity("line1")
            .with_second_entity("line2");

        solver.add_constraint(constraint.clone());
        assert_eq!(solver.constraint_count(), 1);

        solver.remove_constraint(&constraint.id);
        assert_eq!(solver.constraint_count(), 0);
    }

    #[test]
    fn test_add_entity() {
        let mut solver = GeometricSolver::new();
        solver.add_entity("line1", Point::new(0.0, 0.0, 0.0), 0.0, 10.0);

        assert!(solver.entities.contains_key("line1"));
        let state = solver.entities.get("line1").unwrap();
        assert!((state.position.x - 0.0).abs() < 1e-10);
        assert!((state.length - 10.0).abs() < 1e-10);
    }

    #[test]
    fn test_constraint_types() {
        assert_eq!(GeometricConstraint::Coincident.name(), "Coincident");
        assert_eq!(GeometricConstraint::Perpendicular.name(), "Perpendicular");
        assert_eq!(GeometricConstraint::Parallel.name(), "Parallel");
        assert_eq!(GeometricConstraint::Tangent.name(), "Tangent");
        assert_eq!(GeometricConstraint::Horizontal.name(), "Horizontal");
        assert_eq!(GeometricConstraint::Vertical.name(), "Vertical");
    }

    #[test]
    fn test_constraint_icons() {
        assert_eq!(GeometricConstraint::Coincident.icon(), "⭕");
        assert_eq!(GeometricConstraint::Horizontal.icon(), "—");
        assert_eq!(GeometricConstraint::Vertical.icon(), "|");
        assert_eq!(GeometricConstraint::Fix.icon(), "📌");
    }

    #[test]
    fn test_solver_state() {
        let state = SolverState::new();
        assert!(state.solved);
        assert_eq!(state.iterations, 0);
        assert!((state.error - 0.0).abs() < 1e-10);
    }

    #[test]
    fn test_constraints_for_entity() {
        let mut solver = GeometricSolver::new();
        solver.auto_solve = false;

        solver.add_constraint(
            ConstraintEntity::new()
                .with_type(GeometricConstraint::Coincident)
                .with_first_entity("line1")
                .with_second_entity("line2"),
        );

        solver.add_constraint(
            ConstraintEntity::new()
                .with_type(GeometricConstraint::Horizontal)
                .with_first_entity("line1"),
        );

        let constraints = solver.constraints_for_entity("line1");
        assert_eq!(constraints.len(), 2);
    }

    #[test]
    fn test_clear_constraints() {
        let mut solver = GeometricSolver::new();
        solver.auto_solve = false;

        solver.add_constraint(ConstraintEntity::new()
            .with_type(GeometricConstraint::Coincident)
            .with_first_entity("line1")
            .with_second_entity("line2"));

        solver.add_constraint(ConstraintEntity::new()
            .with_type(GeometricConstraint::Horizontal)
            .with_first_entity("line1"));

        assert_eq!(solver.constraint_count(), 2);
        solver.clear_constraints();
        assert_eq!(solver.constraint_count(), 0);
    }

    #[test]
    fn test_dof_calculation() {
        let mut solver = GeometricSolver::new();
        solver.add_entity("line1", Point::new(0.0, 0.0, 0.0), 0.0, 10.0);
        solver.add_entity("line2", Point::new(10.0, 0.0, 0.0), PI / 2.0, 10.0);

        solver.auto_solve = false;
        solver.add_constraint(ConstraintEntity::new()
            .with_type(GeometricConstraint::Coincident)
            .with_first_entity("line1")
            .with_second_entity("line2"));

        solver.solve();

        assert!(solver.state.free_dof >= 0);
    }
}
