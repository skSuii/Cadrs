//! 约束求解（实体下标索引 + 数值迭代版）：约束表达式、约束系统与法方程迭代求解器。
//!
//! 核心概念：
//! - 约束表达式：`PointConstraint` / `LineConstraint` / `CircleConstraint` / `ArcConstraint` / `CurveConstraint`
//!   用实体下标（`ConstraintSystem::entities` 的位置）或显式参数引用几何；`GeometricConstraint`
//!   与 `DimensionalConstraint` 再把这些引用组合成几何约束与尺寸约束。
//! - `ConstraintSystem`：持有实体、约束与求解设置，`solve` 在内部迭代消除残差。
//! - `ConstraintBuilder`：以静态方法把常用约束直接写入系统，省去手工包装枚举。
//!
//! 与其他模块的关系：本文件与 `constraint` 模块顶层的字符串 id 版 `GeometricSolver` 是两套互不共享状态的实现；
//! 坐标为世界坐标，角度一律用弧度，实体参数按类型排列（点 2 个、线 4 个、圆 3 个、弧 5 个）。

use super::geometry::{Point, Line, Circle, Arc};
use std::collections::{HashMap, HashSet, VecDeque};

/// 几何约束：以残差形式表达的实体间几何关系，残差为 0 表示约束被满足。
///
/// 每个变体当前实现一种残差度量；`Angle` 的目标角与所有角度量均为弧度。
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GeometricConstraint {
    /// 重合：残差为两点的欧氏距离。
    Coincident(PointConstraint, PointConstraint),
    /// 水平：两点 y 坐标相同，残差为 y 之差（有符号）。
    Horizontal(PointConstraint, PointConstraint),
    /// 竖直：两点 x 坐标相同，残差为 x 之差（有符号）。
    Vertical(PointConstraint, PointConstraint),
    /// 平行：残差为两方向向量的叉积（越接近共线越接近 0）。
    Parallel(LineConstraint, LineConstraint),
    /// 垂直：残差为两方向向量的点积（越接近正交越接近 0）。
    Perpendicular(LineConstraint, LineConstraint),
    /// 相切：线与圆取「圆心到直线距离 − 半径」，圆与圆取「圆心距 − 半径和」；弧参与时残差按 0 处理。
    Tangent(CurveConstraint, CurveConstraint),
    /// 同心：残差为两圆心距离。
    Concentric(CircleConstraint, CircleConstraint),
    /// 等长：残差为两线长度之差。
    EqualLength(LineConstraint, LineConstraint),
    /// 等半径：残差为两圆半径之差。
    EqualRadius(CircleConstraint, CircleConstraint),
    /// 中点：点位于线段中点，残差为该点到中点（两端点坐标平均）的距离。
    Midpoint(PointConstraint, LineConstraint),
    /// 点在线上：残差为点到直线的垂距，直线按无限长处理。
    PointOnLine(PointConstraint, LineConstraint),
    /// 点在圆上：残差为「点到圆心距离 − 半径」。
    PointOnCircle(PointConstraint, CircleConstraint),
    /// 点在弧上：点角落在弧的起止角区间内时取径向偏差，区间外取到两端点距离的较小值。
    PointOnArc(PointConstraint, ArcConstraint),
    /// 对称：两点关于直线对称，残差为两点中点到直线的垂距。
    Symmetry(PointConstraint, PointConstraint, LineConstraint),
    /// 夹角：第三个字段为目标角（弧度），残差为实际有符号夹角减去目标角，夹角取值域为 (−π, π]。
    Angle(LineConstraint, LineConstraint, f64),
    /// 共线：与平行同形，残差为方向向量叉积，不校验两线是否分离。
    Collinear(LineConstraint, LineConstraint),
    /// 线方向沿 X 轴：残差为方向向量的 y 分量（应为 0）。
    ParallelX(LineConstraint),
    /// 线方向沿 Y 轴：残差为方向向量的 x 分量（应为 0）。
    ParallelY(LineConstraint),
}

/// 点的引用方式：指向某实体上的点，或一个尚未绑定的自由参数。
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PointConstraint {
    /// 实体上的点：`(实体下标, 点序号)`；线实体取 0 为起点、1 为终点，点实体忽略序号。
    EntityPoint(usize, usize),
    /// 自由点占位：当前实现一律按原点 (0, 0) 参与计算，尚未接入外部参数。
    FreePoint(usize),
}

/// 直线的引用方式。
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LineConstraint {
    /// 引用实体线段，取该实体的 4 个参数（起点 x、y，终点 x、y）。
    EntityLine(usize),
    /// 由两个点约束确定的直线。
    ThroughPoints(PointConstraint, PointConstraint),
}

/// 圆的引用方式。
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CircleConstraint {
    /// 引用实体圆，取该实体的 3 个参数（圆心 x、y，半径）。
    EntityCircle(usize),
    /// 由圆心点约束与显式半径定义。
    CenterRadius(PointConstraint, f64),
}

/// 圆弧的引用方式。
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ArcConstraint {
    /// 引用实体弧，取该实体的 5 个参数（圆心 x、y，半径，起始角，终止角），角度为弧度。
    EntityArc(usize),
    /// 由圆心点约束、半径与起止角定义，角度为弧度。
    CenterRadiusAngles(PointConstraint, f64, f64, f64),
}

/// 曲线的统一引用，供相切等跨类型约束使用。
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CurveConstraint {
    /// 直线或线段。
    Line(LineConstraint),
    /// 圆。
    Circle(CircleConstraint),
    /// 圆弧。
    Arc(ArcConstraint),
}

/// 尺寸约束：带目标数值的约束，和几何约束一样占用自由度并计入雅可比矩阵。
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DimensionalConstraint {
    /// 两点距离等于目标值；目标值应为有限非负数，否则 `is_valid` 判定为非法。
    Distance(PointConstraint, PointConstraint, f64),
    /// 两线夹角等于目标值，单位为弧度。
    Angle(LineConstraint, LineConstraint, f64),
    /// 圆半径等于目标值。
    Radius(CircleConstraint, f64),
    /// 圆直径等于目标值，求解时残差按 2 × 半径 − 目标直径 计算。
    Diameter(CircleConstraint, f64),
    /// 线长等于目标值。
    Length(LineConstraint, f64),
}

/// 约束系统：以实体下标为索引，集中保存实体、约束与求解设置。
///
/// 实体的 id 就是它在 `entities` 中的下标，因此实体只增不删，约束里保存的下标始终有效。
#[derive(Debug, Clone)]
pub struct ConstraintSystem {
    /// 几何约束列表，顺序决定雅可比矩阵的行顺序。
    pub geometric_constraints: Vec<GeometricConstraint>,
    /// 尺寸约束列表，其行排在几何约束之后。
    pub dimensional_constraints: Vec<DimensionalConstraint>,
    /// 被约束的实体，下标即约束中引用的实体 id。
    pub entities: Vec<ConstrainedEntity>,
    /// 迭代求解设置（迭代上限、容差、阻尼等）。
    pub solver_settings: SolverSettings,
}

impl ConstraintSystem {
    /// 创建空系统：无实体、无约束，求解设置取 `SolverSettings::default()`。
    pub fn new() -> Self {
        Self {
            geometric_constraints: Vec::new(),
            dimensional_constraints: Vec::new(),
            entities: Vec::new(),
            solver_settings: SolverSettings::default(),
        }
    }

    /// 追加一个实体并返回其下标，该下标即后续约束中使用的实体 id。
    /// - `entity`：实体及其初始参数，按值移入系统。
    /// 返回：新实体的下标，等于加入前的实体数量；实体不会被删除，下标因此保持稳定。
    pub fn add_entity(&mut self, entity: ConstrainedEntity) -> usize {
        let id = self.entities.len();
        self.entities.push(entity);
        id
    }

    /// 追加一条几何约束；不做有效性校验，可用 `validate_constraints` 事后检查。
    pub fn add_geometric_constraint(&mut self, constraint: GeometricConstraint) {
        self.geometric_constraints.push(constraint);
    }

    /// 追加一条尺寸约束；不做有效性校验，目标值非法时会在 `solve` 中产生残差。
    pub fn add_dimensional_constraint(&mut self, constraint: DimensionalConstraint) {
        self.dimensional_constraints.push(constraint);
    }

    /// 以当前参数为初值迭代求解，直到最大残差小于容差或达到迭代上限。
    /// 返回：`SolverResult`；失败时 `success` 为 false，`message` 给出迭代次数与残余残差。
    /// 副作用：求解得到的参数增量会累加到 `entities` 上，固定实体（`is_fixed`）不受影响。
    pub fn solve(&mut self) -> SolverResult {
        let mut solver = ConstraintSolver::new(self);
        solver.solve()
    }

    /// 按下标获取实体。
    /// 返回：下标有效时返回实体引用，越界返回 `None`。
    pub fn get_entity(&self, id: usize) -> Option<&ConstrainedEntity> {
        self.entities.get(id)
    }

    /// 按下标获取实体可变引用，用于直接改写几何参数。
    /// 返回：越界返回 `None`。
    pub fn get_entity_mut(&mut self, id: usize) -> Option<&mut ConstrainedEntity> {
        self.entities.get_mut(id)
    }

    /// 自由度是否为 0：约束条数与实体自由度恰好抵消，几何被完全确定。
    pub fn is_fully_constrained(&self) -> bool {
        let degrees_of_freedom = self.calculate_degrees_of_freedom();
        degrees_of_freedom == 0
    }

    /// 自由度是否大于 0：约束不足，几何仍可变动。
    pub fn is_under_constrained(&self) -> bool {
        let degrees_of_freedom = self.calculate_degrees_of_freedom();
        degrees_of_freedom > 0
    }

    /// 自由度是否小于 0：约束冗余或互相冲突。
    pub fn is_over_constrained(&self) -> bool {
        let degrees_of_freedom = self.calculate_degrees_of_freedom();
        degrees_of_freedom < 0
    }

    /// 按「实体自由度之和 − 约束条数」估算系统自由度。
    /// 返回：正数表示欠约束、0 表示完全约束、负数表示过约束；固定实体贡献 0 个自由度，参考约束照常计数。
    pub fn calculate_degrees_of_freedom(&self) -> i32 {
        let mut dof = 0;

        for entity in &self.entities {
            dof += entity.degrees_of_freedom();
        }

        let num_constraints = self.geometric_constraints.len() + self.dimensional_constraints.len();
        dof -= num_constraints as i32;

        dof
    }

    /// 构造约束图：每个实体是一个节点，同一约束涉及的实体两两连无向边。
    /// 返回：可用于分析连通分量与冗余的 `ConstraintGraph`；`FreePoint` 等不含实体引用的约束不产生边。
    pub fn get_constraint_graph(&self) -> ConstraintGraph {
        let mut graph = ConstraintGraph::new();

        for (id, _) in self.entities.iter().enumerate() {
            graph.add_node(id);
        }

        for constraint in &self.geometric_constraints {
            let entities = constraint.get_constrained_entities();
            for &entity_id in &entities {
                for &other_id in &entities {
                    if entity_id != other_id {
                        graph.add_edge(entity_id, other_id);
                    }
                }
            }
        }

        for constraint in &self.dimensional_constraints {
            let entities = constraint.get_constrained_entities();
            for &entity_id in &entities {
                for &other_id in &entities {
                    if entity_id != other_id {
                        graph.add_edge(entity_id, other_id);
                    }
                }
            }
        }

        graph
    }

    /// 逐条检查几何约束与尺寸约束的形式合法性，并检查整体自由度是否超限。
    /// 返回：错误列表，按几何约束、尺寸约束、自由度的顺序排列；空列表表示全部通过。
    /// 自由度小于 0 时追加一条 `ValidationError::OverConstrained`，其 `excess_constraints` 为超出的条数。
    pub fn validate_constraints(&self) -> Vec<ValidationError> {
        let mut errors = Vec::new();

        for (i, constraint) in self.geometric_constraints.iter().enumerate() {
            if !constraint.is_valid() {
                errors.push(ValidationError::InvalidGeometricConstraint(i));
            }
        }

        for (i, constraint) in self.dimensional_constraints.iter().enumerate() {
            if !constraint.is_valid() {
                errors.push(ValidationError::InvalidDimensionalConstraint(i));
            }
        }

        let dof = self.calculate_degrees_of_freedom();
        if dof < 0 {
            errors.push(ValidationError::OverConstrained {
                excess_constraints: -dof as usize,
            });
        }

        errors
    }
}

impl Default for ConstraintSystem {
    fn default() -> Self {
        Self::new()
    }
}

/// 参与约束求解的实体：类型决定参数布局与自由度数。
#[derive(Debug, Clone)]
pub struct ConstrainedEntity {
    /// 实体下标，须与它在 `ConstraintSystem::entities` 中的位置一致。
    pub id: usize,
    /// 实体类型，决定 `parameters` 的含义与自由度数量。
    pub entity_type: EntityType,
    /// 几何参数向量，按类型排列：点 [x, y]、线 [x1, y1, x2, y2]、圆 [cx, cy, r]、弧 [cx, cy, r, start, end]。
    pub parameters: Vec<f64>,
    /// 是否固定：为 true 时自由度为 0，求解不会修改其参数。
    pub is_fixed: bool,
    /// 是否为参考实体（仅用于测量）；求解器不做特殊处理，语义由调用方决定。
    pub is_reference: bool,
}

impl ConstrainedEntity {
    /// 按类型创建实体，参数取该类型的默认初值（位于原点，长度/半径初始为 1）。
    /// - `id`：实体下标，需由调用方保证与它在系统中的位置一致。
    /// 返回的实体未固定、非参考，可再用 `as_fixed`/`as_reference` 调整。
    pub fn new(id: usize, entity_type: EntityType) -> Self {
        let parameters = entity_type.get_parameters();
        Self {
            id,
            entity_type,
            parameters,
            is_fixed: false,
            is_reference: false,
        }
    }

    /// 标记为固定并返回自身（链式构建）；固定实体自由度为 0，求解时参数保持不变。
    pub fn as_fixed(mut self) -> Self {
        self.is_fixed = true;
        self
    }

    /// 标记为参考实体并返回自身（链式构建）；当前求解流程不区分参考与非参考。
    pub fn as_reference(mut self) -> Self {
        self.is_reference = true;
        self
    }

    /// 该实体贡献的自由度数：固定实体恒为 0，否则点 2、线 4、圆 3、弧 5。
    pub fn degrees_of_freedom(&self) -> i32 {
        if self.is_fixed {
            return 0;
        }
        self.entity_type.degrees_of_freedom()
    }

    /// 按当前参数取点坐标。
    /// 返回：实体为点时返回该点；其它类型返回 `None`。
    pub fn get_point(&self) -> Option<Point> {
        self.entity_type.get_point(&self.parameters)
    }

    /// 按当前参数取线段两端点。
    /// 返回：实体为线时返回 `(起点, 终点)`，其它类型返回 `None`。
    pub fn get_line(&self) -> Option<(Point, Point)> {
        self.entity_type.get_line(&self.parameters)
    }

    /// 按当前参数取圆心与半径。
    /// 返回：实体为圆时返回 `(圆心, 半径)`，其它类型返回 `None`。
    pub fn get_circle(&self) -> Option<(Point, f64)> {
        self.entity_type.get_circle(&self.parameters)
    }

    /// 按当前参数取弧的圆心、半径与起止角（角度为弧度）。
    /// 返回：实体为弧时返回 `(圆心, 半径, 起始角, 终止角)`，其它类型返回 `None`。
    pub fn get_arc(&self) -> Option<(Point, f64, f64, f64)> {
        self.entity_type.get_arc(&self.parameters)
    }
}

/// 受约束实体的几何类型，决定参数布局与自由度数。
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EntityType {
    /// 点：参数 [x, y]，2 个自由度。
    Point,
    /// 线段：参数 [x1, y1, x2, y2]，4 个自由度。
    Line,
    /// 圆：参数 [cx, cy, r]，3 个自由度。
    Circle,
    /// 圆弧：参数 [cx, cy, r, start, end]，5 个自由度，起止角为弧度。
    Arc,
}

impl EntityType {
    fn get_parameters(&self) -> Vec<f64> {
        match self {
            EntityType::Point => vec![0.0, 0.0],
            EntityType::Line => vec![0.0, 0.0, 1.0, 0.0],
            EntityType::Circle => vec![0.0, 0.0, 1.0],
            EntityType::Arc => vec![0.0, 0.0, 1.0, 0.0, 1.0],
        }
    }

    fn degrees_of_freedom(&self) -> i32 {
        match self {
            EntityType::Point => 2,
            EntityType::Line => 4,
            EntityType::Circle => 3,
            EntityType::Arc => 5,
        }
    }

    fn get_point(&self, parameters: &[f64]) -> Option<Point> {
        match self {
            EntityType::Point => Some(Point::new(parameters[0], parameters[1])),
            _ => None,
        }
    }

    fn get_line(&self, parameters: &[f64]) -> Option<(Point, Point)> {
        match self {
            EntityType::Line => Some((
                Point::new(parameters[0], parameters[1]),
                Point::new(parameters[2], parameters[3]),
            )),
            _ => None,
        }
    }

    fn get_circle(&self, parameters: &[f64]) -> Option<(Point, f64)> {
        match self {
            EntityType::Circle => Some((
                Point::new(parameters[0], parameters[1]),
                parameters[2],
            )),
            _ => None,
        }
    }

    fn get_arc(&self, parameters: &[f64]) -> Option<(Point, f64, f64, f64)> {
        match self {
            EntityType::Arc => Some((
                Point::new(parameters[0], parameters[1]),
                parameters[2],
                parameters[3],
                parameters[4],
            )),
            _ => None,
        }
    }
}

/// 迭代求解器的设置。
#[derive(Debug, Clone)]
pub struct SolverSettings {
    /// 最大迭代次数，默认 100；达到上限仍未收敛即返回失败。
    pub max_iterations: usize,
    /// 收敛容差，默认 1e-6；最大残差绝对值小于该值即判定收敛。
    pub tolerance: f64,
    /// 阻尼系数，默认 0.5；仅在 `use_damping` 为 true 时有意义。
    pub damping_factor: f64,
    /// 是否启用阻尼，默认 true；当前求解流程未读取该标志，保留给调用方与后续实现。
    pub use_damping: bool,
    /// 约束权重，默认 1.0；当前求解流程未使用。
    pub constraint_weight: f64,
    /// 是否使用高斯-牛顿式法方程求解，默认 true；当前实现固定走该方法。
    pub use_gauss_newton: bool,
    /// 是否输出调试信息，默认 false；当前实现尚未打印任何内容。
    pub debug_output: bool,
}

impl SolverSettings {
    /// 返回默认设置：最多 100 次迭代、容差 1e-6、阻尼 0.5、启用阻尼与高斯-牛顿，权重 1.0，关闭调试输出。
    pub fn new() -> Self {
        Self {
            max_iterations: 100,
            tolerance: 1e-6,
            damping_factor: 0.5,
            use_damping: true,
            constraint_weight: 1.0,
            use_gauss_newton: true,
            debug_output: false,
        }
    }

    /// 设置最大迭代次数并返回自身（链式构建）。
    pub fn with_max_iterations(mut self, iterations: usize) -> Self {
        self.max_iterations = iterations;
        self
    }

    /// 设置收敛容差并返回自身（链式构建）；容差越小迭代次数越多。
    pub fn with_tolerance(mut self, tolerance: f64) -> Self {
        self.tolerance = tolerance;
        self
    }

    /// 设置阻尼系数并返回自身（链式构建）。
    pub fn with_damping(mut self, damping: f64) -> Self {
        self.damping_factor = damping;
        self
    }

    /// 打开调试输出并返回自身（链式构建）。
    pub fn with_debug(mut self) -> Self {
        self.debug_output = true;
        self
    }
}

impl Default for SolverSettings {
    fn default() -> Self {
        Self::new()
    }
}

/// 一次求解的结果。
#[derive(Debug, Clone)]
pub struct SolverResult {
    /// 是否求解成功（已收敛）。
    pub success: bool,
    /// 实际迭代次数；由 `failure` 构造时固定为 0。
    pub iterations: usize,
    /// 退出时的最大残差绝对值，越小越接近满足全部约束。
    pub residual_error: f64,
    /// 是否收敛；与 `success` 含义一致，失败结果为 false。
    pub converged: bool,
    /// 结果说明文字：成功为「求解成功」，失败为未收敛的迭代次数与残差。
    pub message: String,
}

impl SolverResult {
    /// 构造成功结果，`message` 固定为「求解成功」。
    /// - `iterations`：实际迭代次数。
    /// - `residual`：退出时的最大残差。
    pub fn success(iterations: usize, residual: f64) -> Self {
        Self {
            success: true,
            iterations,
            residual_error: residual,
            converged: true,
            message: String::from("求解成功"),
        }
    }

    /// 构造失败结果：`success` 与 `converged` 均为 false，迭代次数与残差置 0。
    /// - `message`：失败原因，通常是「在 N 次迭代后未收敛，残差：R」。
    pub fn failure(message: String) -> Self {
        Self {
            success: false,
            iterations: 0,
            residual_error: 0.0,
            converged: false,
            message,
        }
    }
}

struct ConstraintSolver<'a> {
    system: &'a mut ConstraintSystem,
    jacobian: Vec<Vec<f64>>,
    residuals: Vec<f64>,
    delta: Vec<f64>,
}

impl<'a> ConstraintSolver<'a> {
    fn new(system: &'a mut ConstraintSystem) -> Self {
        Self {
            system,
            jacobian: Vec::new(),
            residuals: Vec::new(),
            delta: Vec::new(),
        }
    }

    fn solve(&mut self) -> SolverResult {
        let total_dof = self.system.entities.iter()
            .map(|e| e.degrees_of_freedom() as usize)
            .sum();

        let num_constraints = self.system.geometric_constraints.len() + 
                            self.system.dimensional_constraints.len();

        if total_dof == 0 {
            return SolverResult::success(0, 0.0);
        }

        let mut iterations = 0;
        let mut max_error = f64::MAX;

        while iterations < self.system.solver_settings.max_iterations && 
              max_error > self.system.solver_settings.tolerance {

            self.build_system();

            if self.residuals.is_empty() {
                return SolverResult::success(iterations, 0.0);
            }

            max_error = self.residuals.iter()
                .map(|r| r.abs())
                .fold(0.0, f64::max);

            if max_error < self.system.solver_settings.tolerance {
                break;
            }

            self.solve_system();

            self.apply_deltas();

            iterations += 1;
        }

        if max_error < self.system.solver_settings.tolerance {
            SolverResult::success(iterations, max_error)
        } else {
            SolverResult::failure(format!("在{}次迭代后未收敛，残差：{}", iterations, max_error))
        }
    }

    fn build_system(&mut self) {
        let total_dof = self.system.entities.iter()
            .filter(|e| !e.is_fixed)
            .map(|e| e.degrees_of_freedom() as usize)
            .sum();

        let num_constraints = self.system.geometric_constraints.len() + 
                            self.system.dimensional_constraints.len();

        self.jacobian.clear();
        self.residuals.clear();
        self.jacobian.resize(num_constraints, vec![0.0; total_dof]);
        self.residuals.resize(num_constraints, 0.0);

        let mut row = 0;
        for constraint in &self.system.geometric_constraints {
            self.evaluate_geometric_constraint(row, constraint);
            row += 1;
        }

        for constraint in &self.system.dimensional_constraints {
            self.evaluate_dimensional_constraint(row, constraint);
            row += 1;
        }
    }

#[cfg(feature = "constraint")]
impl ConstraintSolver<'_> {
    fn evaluate_geometric_constraint(&mut self, row: usize, constraint: &GeometricConstraint) {
        match constraint {
            GeometricConstraint::Coincident(p1, p2) => {
                let pos1 = self.get_point_position(p1);
                let pos2 = self.get_point_position(p2);
                self.residuals[row] = (pos1 - pos2).norm();
                self.set_jacobian_coincident(row, p1, p2);
            }
            GeometricConstraint::Horizontal(p1, p2) => {
                let pos1 = self.get_point_position(p1);
                let pos2 = self.get_point_position(p2);
                self.residuals[row] = pos2.y - pos1.y;
                self.set_jacobian_horizontal(row, p1, p2);
            }
            GeometricConstraint::Vertical(p1, p2) => {
                let pos1 = self.get_point_position(p1);
                let pos2 = self.get_point_position(p2);
                self.residuals[row] = pos2.x - pos1.x;
                self.set_jacobian_vertical(row, p1, p2);
            }
            GeometricConstraint::Parallel(l1, l2) => {
                let dir1 = self.get_line_direction(l1);
                let dir2 = self.get_line_direction(l2);
                self.residuals[row] = dir1.cross(dir2);
                self.set_jacobian_parallel(row, l1, l2);
            }
            GeometricConstraint::Perpendicular(l1, l2) => {
                let dir1 = self.get_line_direction(l1);
                let dir2 = self.get_line_direction(l2);
                self.residuals[row] = dir1.dot(dir2);
                self.set_jacobian_perpendicular(row, l1, l2);
            }
            GeometricConstraint::Tangent(c1, c2) => {
                self.evaluate_tangent_constraint(row, c1, c2);
            }
            GeometricConstraint::Concentric(c1, c2) => {
                self.evaluate_concentric_constraint(row, c1, c2);
            }
            GeometricConstraint::EqualLength(l1, l2) => {
                let len1 = self.get_line_length(l1);
                let len2 = self.get_line_length(l2);
                self.residuals[row] = len1 - len2;
                self.set_jacobian_equal_length(row, l1, l2);
            }
            GeometricConstraint::EqualRadius(c1, c2) => {
                let r1 = self.get_circle_radius(c1);
                let r2 = self.get_circle_radius(c2);
                self.residuals[row] = r1 - r2;
                self.set_jacobian_equal_radius(row, c1, c2);
            }
            GeometricConstraint::Midpoint(p, l) => {
                self.evaluate_midpoint_constraint(row, p, l);
            }
            GeometricConstraint::PointOnLine(p, l) => {
                self.evaluate_point_on_line_constraint(row, p, l);
            }
            GeometricConstraint::PointOnCircle(p, c) => {
                self.evaluate_point_on_circle_constraint(row, p, c);
            }
            GeometricConstraint::PointOnArc(p, a) => {
                self.evaluate_point_on_arc_constraint(row, p, a);
            }
            GeometricConstraint::Symmetry(p1, p2, l) => {
                self.evaluate_symmetry_constraint(row, p1, p2, l);
            }
            GeometricConstraint::Angle(l1, l2, target) => {
                let dir1 = self.get_line_direction(l1);
                let dir2 = self.get_line_direction(l2);
                let angle = dir1.angle_between(dir2);
                self.residuals[row] = angle - target;
                self.set_jacobian_angle(row, l1, l2);
            }
            GeometricConstraint::Collinear(l1, l2) => {
                let dir1 = self.get_line_direction(l1);
                let dir2 = self.get_line_direction(l2);
                self.residuals[row] = dir1.cross(dir2);
                self.set_jacobian_collinear(row, l1, l2);
            }
            GeometricConstraint::ParallelX(l) => {
                let dir = self.get_line_direction(l);
                self.residuals[row] = dir.y;
                self.set_jacobian_parallel_x(row, l);
            }
            GeometricConstraint::ParallelY(l) => {
                let dir = self.get_line_direction(l);
                self.residuals[row] = dir.x;
                self.set_jacobian_parallel_y(row, l);
            }
        }
    }

    fn evaluate_tangent_constraint(&mut self, row: usize, c1: &CurveConstraint, c2: &CurveConstraint) {
        match (c1, c2) {
            (CurveConstraint::Line(l1), CurveConstraint::Circle(c2)) => {
                let line_p1 = self.get_line_endpoint(l1, 0);
                let line_p2 = self.get_line_endpoint(l1, 1);
                let center = self.get_circle_center(c2);
                let radius = self.get_circle_radius(c2);

                let to_center = center - line_p1;
                let line_dir = (line_p2 - line_p1).normalized();
                let proj = to_center.dot(line_dir);
                let closest = line_p1 + line_dir * proj;
                let to_closest = closest - center;

                self.residuals[row] = to_closest.norm() - radius;
            }
            (CurveConstraint::Circle(c1), CurveConstraint::Line(l2)) => {
                self.evaluate_tangent_constraint(row, c2, c1);
            }
            (CurveConstraint::Circle(c1), CurveConstraint::Circle(c2)) => {
                let center1 = self.get_circle_center(c1);
                let center2 = self.get_circle_center(c2);
                let r1 = self.get_circle_radius(c1);
                let r2 = self.get_circle_radius(c2);

                let dist = (center1 - center2).norm();
                self.residuals[row] = dist - (r1 + r2);
            }
            _ => {
                self.residuals[row] = 0.0;
            }
        }
    }

    fn evaluate_concentric_constraint(&mut self, row: usize, c1: &CircleConstraint, c2: &CircleConstraint) {
        let center1 = self.get_circle_center(c1);
        let center2 = self.get_circle_center(c2);
        self.residuals[row] = (center1 - center2).norm();
        self.set_jacobian_concentric(row, c1, c2);
    }

    fn evaluate_midpoint_constraint(&mut self, row: usize, p: &PointConstraint, l: &LineConstraint) {
        let point_pos = self.get_point_position(p);
        let line_p1 = self.get_line_endpoint(l, 0);
        let line_p2 = self.get_line_endpoint(l, 1);
        let midpoint = (line_p1 + line_p2) * 0.5;

        self.residuals[row] = (point_pos - midpoint).norm();
        self.set_jacobian_midpoint(row, p, l);
    }

    fn evaluate_point_on_line_constraint(&mut self, row: usize, p: &PointConstraint, l: &LineConstraint) {
        let point_pos = self.get_point_position(p);
        let line_p1 = self.get_line_endpoint(l, 0);
        let line_p2 = self.get_line_endpoint(l, 1);
        let line_dir = (line_p2 - line_p1).normalized();
        let to_point = point_pos - line_p1;
        let proj = to_point.dot(line_dir);
        let closest = line_p1 + line_dir * proj;

        self.residuals[row] = (point_pos - closest).norm();
        self.set_jacobian_point_on_line(row, p, l);
    }

    fn evaluate_point_on_circle_constraint(&mut self, row: usize, p: &PointConstraint, c: &CircleConstraint) {
        let point_pos = self.get_point_position(p);
        let center = self.get_circle_center(c);
        let radius = self.get_circle_radius(c);

        let dist = (point_pos - center).norm();
        self.residuals[row] = dist - radius;
        self.set_jacobian_point_on_circle(row, p, c);
    }

    fn evaluate_point_on_arc_constraint(&mut self, row: usize, p: &PointConstraint, a: &ArcConstraint) {
        let point_pos = self.get_point_position(p);
        let arc_center = self.get_arc_center(a);
        let arc_radius = self.get_arc_radius(a);
        let arc_start = self.get_arc_start_angle(a);
        let arc_end = self.get_arc_end_angle(a);

        let to_point = point_pos - arc_center;
        let point_angle = to_point.y.atan2(to_point.x);

        let normalized_point = if arc_end > arc_start {
            point_angle >= arc_start && point_angle <= arc_end
        } else {
            point_angle >= arc_start || point_angle <= arc_end
        };

        let radial_dist = (to_point.norm() - arc_radius).abs();

        if normalized_point {
            self.residuals[row] = radial_dist;
        } else {
            let start_dist = (point_pos - self.get_arc_point_at_angle(a, arc_start)).norm();
            let end_dist = (point_pos - self.get_arc_point_at_angle(a, arc_end)).norm();
            self.residuals[row] = radial_dist.min(start_dist).min(end_dist);
        }
    }

    fn evaluate_symmetry_constraint(&mut self, row: usize, p1: &PointConstraint, p2: &PointConstraint, l: &LineConstraint) {
        let pos1 = self.get_point_position(p1);
        let pos2 = self.get_point_position(p2);
        let line_p1 = self.get_line_endpoint(l, 0);
        let line_p2 = self.get_line_endpoint(l, 1);
        let line_dir = (line_p2 - line_p1).normalized();
        let normal = Vector2D::new(-line_dir.y, line_dir.x);

        let mid = (pos1 + pos2) * 0.5;
        let to_mid = mid - line_p1;
        let proj = to_mid.dot(line_dir);
        let closest = line_p1 + line_dir * proj;

        self.residuals[row] = (mid - closest).norm();
    }

    fn get_circle_center(&self, constraint: &CircleConstraint) -> Vector2D {
        match constraint {
            CircleConstraint::EntityCircle(entity_id) => {
                if let Some(entity) = self.system.entities.get(*entity_id) {
                    Vector2D::new(entity.parameters[0], entity.parameters[1])
                } else {
                    Vector2D::new(0.0, 0.0)
                }
            }
            CircleConstraint::CenterRadius(p, _) => self.get_point_position(p),
        }
    }

    fn get_arc_center(&self, constraint: &ArcConstraint) -> Vector2D {
        match constraint {
            ArcConstraint::EntityArc(entity_id) => {
                if let Some(entity) = self.system.entities.get(*entity_id) {
                    Vector2D::new(entity.parameters[0], entity.parameters[1])
                } else {
                    Vector2D::new(0.0, 0.0)
                }
            }
            ArcConstraint::CenterRadiusAngles(p, _, _, _) => self.get_point_position(p),
        }
    }

    fn get_arc_radius(&self, constraint: &ArcConstraint) -> f64 {
        match constraint {
            ArcConstraint::EntityArc(entity_id) => {
                if let Some(entity) = self.system.entities.get(*entity_id) {
                    entity.parameters[2]
                } else {
                    1.0
                }
            }
            ArcConstraint::CenterRadiusAngles(_, r, _, _) => *r,
        }
    }

    fn get_arc_start_angle(&self, constraint: &ArcConstraint) -> f64 {
        match constraint {
            ArcConstraint::EntityArc(entity_id) => {
                if let Some(entity) = self.system.entities.get(*entity_id) {
                    entity.parameters[3]
                } else {
                    0.0
                }
            }
            ArcConstraint::CenterRadiusAngles(_, _, start, _) => *start,
        }
    }

    fn get_arc_end_angle(&self, constraint: &ArcConstraint) -> f64 {
        match constraint {
            ArcConstraint::EntityArc(entity_id) => {
                if let Some(entity) = self.system.entities.get(*entity_id) {
                    entity.parameters[4]
                } else {
                    std::f64::consts::PI
                }
            }
            ArcConstraint::CenterRadiusAngles(_, _, _, end) => *end,
        }
    }

    fn get_arc_point_at_angle(&self, constraint: &ArcConstraint, angle: f64) -> Vector2D {
        let center = self.get_arc_center(constraint);
        let radius = self.get_arc_radius(constraint);
        center + Vector2D::new(angle.cos(), angle.sin()) * radius
    }

    fn get_line_endpoint(&self, constraint: &LineConstraint, index: usize) -> Vector2D {
        match constraint {
            LineConstraint::EntityLine(entity_id) => {
                if let Some(entity) = self.system.entities.get(*entity_id) {
                    Vector2D::new(entity.parameters[index * 2], entity.parameters[index * 2 + 1])
                } else {
                    Vector2D::new(0.0, 0.0)
                }
            }
            LineConstraint::ThroughPoints(p1, p2) => {
                if index == 0 {
                    self.get_point_position(p1)
                } else {
                    self.get_point_position(p2)
                }
            }
        }
    }

    fn set_jacobian_coincident(&mut self, _row: usize, _p1: &PointConstraint, _p2: &PointConstraint) {}
    fn set_jacobian_horizontal(&mut self, _row: usize, _p1: &PointConstraint, _p2: &PointConstraint) {}
    fn set_jacobian_vertical(&mut self, _row: usize, _p1: &PointConstraint, _p2: &PointConstraint) {}
    fn set_jacobian_parallel(&mut self, _row: usize, _l1: &LineConstraint, _l2: &LineConstraint) {}
    fn set_jacobian_perpendicular(&mut self, _row: usize, _l1: &LineConstraint, _l2: &LineConstraint) {}
    fn set_jacobian_equal_length(&mut self, _row: usize, _l1: &LineConstraint, _l2: &LineConstraint) {}
    fn set_jacobian_equal_radius(&mut self, _row: usize, _c1: &CircleConstraint, _c2: &CircleConstraint) {}
    fn set_jacobian_concentric(&mut self, _row: usize, _c1: &CircleConstraint, _c2: &CircleConstraint) {}
    fn set_jacobian_midpoint(&mut self, _row: usize, _p: &PointConstraint, _l: &LineConstraint) {}
    fn set_jacobian_point_on_line(&mut self, _row: usize, _p: &PointConstraint, _l: &LineConstraint) {}
    fn set_jacobian_point_on_circle(&mut self, _row: usize, _p: &PointConstraint, _c: &CircleConstraint) {}
    fn set_jacobian_angle(&mut self, _row: usize, _l1: &LineConstraint, _l2: &LineConstraint) {}
    fn set_jacobian_collinear(&mut self, _row: usize, _l1: &LineConstraint, _l2: &LineConstraint) {}
    fn set_jacobian_parallel_x(&mut self, _row: usize, _l: &LineConstraint) {}
    fn set_jacobian_parallel_y(&mut self, _row: usize, _l: &LineConstraint) {}
}

#[cfg(not(feature = "constraint"))]
impl ConstraintSolver<'_> {
    fn evaluate_geometric_constraint(&self, row: usize, constraint: &GeometricConstraint) {
        match constraint {
            GeometricConstraint::Coincident(p1, p2) => {
                let pos1 = self.get_point_position(p1);
                let pos2 = self.get_point_position(p2);
                self.residuals[row] = (pos1 - pos2).norm();
            }
            GeometricConstraint::Horizontal(p1, p2) => {
                let pos1 = self.get_point_position(p1);
                let pos2 = self.get_point_position(p2);
                self.residuals[row] = pos2.y - pos1.y;
            }
            GeometricConstraint::Vertical(p1, p2) => {
                let pos1 = self.get_point_position(p1);
                let pos2 = self.get_point_position(p2);
                self.residuals[row] = pos2.x - pos1.x;
            }
            _ => {
                self.residuals[row] = 0.0;
            }
        }
    }
}

    fn evaluate_dimensional_constraint(&self, row: usize, constraint: &DimensionalConstraint) {
        match constraint {
            DimensionalConstraint::Distance(p1, p2, target) => {
                let pos1 = self.get_point_position(p1);
                let pos2 = self.get_point_position(p2);
                let dist = (pos1 - pos2).norm();
                self.residuals[row] = dist - target;
            }
            DimensionalConstraint::Angle(l1, l2, target) => {
                let dir1 = self.get_line_direction(l1);
                let dir2 = self.get_line_direction(l2);
                let angle = dir1.angle_between(dir2);
                self.residuals[row] = angle - target;
            }
            DimensionalConstraint::Radius(c, target) => {
                let radius = self.get_circle_radius(c);
                self.residuals[row] = radius - target;
            }
            DimensionalConstraint::Diameter(c, target) => {
                let radius = self.get_circle_radius(c);
                self.residuals[row] = 2.0 * radius - target;
            }
            DimensionalConstraint::Length(l, target) => {
                let length = self.get_line_length(l);
                self.residuals[row] = length - target;
            }
        }
    }

    fn get_point_position(&self, constraint: &PointConstraint) -> Vector2D {
        match constraint {
            PointConstraint::EntityPoint(entity_id, point_index) => {
                if let Some(entity) = self.system.entities.get(*entity_id) {
                    match entity.entity_type {
                        EntityType::Point => Vector2D::new(entity.parameters[0], entity.parameters[1]),
                        EntityType::Line => {
                            if *point_index == 0 {
                                Vector2D::new(entity.parameters[0], entity.parameters[1])
                            } else {
                                Vector2D::new(entity.parameters[2], entity.parameters[3])
                            }
                        }
                        _ => Vector2D::new(0.0, 0.0),
                    }
                } else {
                    Vector2D::new(0.0, 0.0)
                }
            }
            PointConstraint::FreePoint(id) => Vector2D::new(0.0, 0.0),
        }
    }

    fn get_line_direction(&self, constraint: &LineConstraint) -> Vector2D {
        match constraint {
            LineConstraint::EntityLine(entity_id) => {
                if let Some(entity) = self.system.entities.get(*entity_id) {
                    let dir = Vector2D::new(
                        entity.parameters[2] - entity.parameters[0],
                        entity.parameters[3] - entity.parameters[1],
                    );
                    dir.normalized()
                } else {
                    Vector2D::new(1.0, 0.0)
                }
            }
            LineConstraint::ThroughPoints(p1, p2) => {
                let pos1 = self.get_point_position(p1);
                let pos2 = self.get_point_position(p2);
                let dir = pos2 - pos1;
                dir.normalized()
            }
        }
    }

    fn get_circle_radius(&self, constraint: &CircleConstraint) -> f64 {
        match constraint {
            CircleConstraint::EntityCircle(entity_id) => {
                if let Some(entity) = self.system.entities.get(*entity_id) {
                    entity.parameters[2]
                } else {
                    1.0
                }
            }
            CircleConstraint::CenterRadius(_, radius) => *radius,
        }
    }

    fn get_line_length(&self, constraint: &LineConstraint) -> f64 {
        match constraint {
            LineConstraint::EntityLine(entity_id) => {
                if let Some(entity) = self.system.entities.get(*entity_id) {
                    let dx = entity.parameters[2] - entity.parameters[0];
                    let dy = entity.parameters[3] - entity.parameters[1];
                    (dx * dx + dy * dy).sqrt()
                } else {
                    1.0
                }
            }
            LineConstraint::ThroughPoints(p1, p2) => {
                let pos1 = self.get_point_position(p1);
                let pos2 = self.get_point_position(p2);
                (pos1 - pos2).norm()
            }
        }
    }

    fn solve_system(&mut self) {
        let n = self.jacobian.len();
        let m = self.jacobian[0].len();

        if n == 0 || m == 0 {
            self.delta.clear();
            return;
        }

        let jtj: Vec<Vec<f64>> = (0..m)
            .map(|i| (0..m).map(|j| {
                (0..n).map(|k| self.jacobian[k][i] * self.jacobian[k][j]).sum()
            }).collect();

        let jtr: Vec<f64> = (0..m).map(|i| {
            (0..n).map(|k| self.jacobian[k][i] * self.residuals[k]).sum()
        }).collect();

        self.delta = self.solve_linear_system(&jtj, &jtr);
    }

    fn solve_linear_system(&self, a: &[Vec<f64>], b: &[f64]) -> Vec<f64> {
        let n = a.len();
        if n == 0 {
            return Vec::new();
        }

        let mut aug: Vec<Vec<f64>> = a.iter()
            .enumerate()
            .map(|(i, row)| {
                let mut new_row = row.clone();
                new_row.push(b[i]);
                new_row
            })
            .collect();

        for k in 0..n {
            let pivot = aug[k][k].abs();
            let mut pivot_row = k;
            for i in (k + 1)..n {
                if aug[i][k].abs() > pivot {
                    pivot = aug[i][k].abs();
                    pivot_row = i;
                }
            }

            if pivot < 1e-12 {
                continue;
            }

            if pivot_row != k {
                aug.swap(k, pivot_row);
            }

            for i in (k + 1)..n {
                let factor = aug[i][k] / aug[k][k];
                for j in k..=n {
                    aug[i][j] -= factor * aug[k][j];
                }
            }
        }

        let mut x = vec![0.0; n];
        for i in (0..n).rev() {
            let mut sum = 0.0;
            for j in (i + 1)..n {
                sum += aug[i][j] * x[j];
            }
            if aug[i][i].abs() < 1e-12 {
                x[i] = 0.0;
            } else {
                x[i] = (aug[i][n] - sum) / aug[i][i];
            }
        }

        x
    }

    fn apply_deltas(&mut self) {
        let mut param_index = 0;

        for entity in &mut self.system.entities {
            if entity.is_fixed {
                continue;
            }

            let dof = entity.degrees_of_freedom() as usize;
            if dof > 0 && param_index < self.delta.len() {
                for i in 0..dof {
                    if param_index + i < self.delta.len() {
                        entity.parameters[i] += self.delta[param_index + i];
                    }
                }
                param_index += dof;
            }
        }
    }
}

/// 轻量二维向量，供约束残差评估与雅可比计算内部使用（世界坐标，无单位）。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vector2D {
    /// X 分量。
    pub x: f64,
    /// Y 分量。
    pub y: f64,
}

impl Vector2D {
    /// 按分量构造向量。
    /// - `x`：X 分量。
    /// - `y`：Y 分量。
    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    /// 返回向量模长（欧氏长度），零向量返回 0。
    pub fn norm(&self) -> f64 {
        (self.x * self.x + self.y * self.y).sqrt()
    }

    /// 返回单位向量；模长小于 1e-12 时返回 (1, 0)，以免产生 NaN。
    pub fn normalized(&self) -> Self {
        let norm = self.norm();
        if norm < 1e-12 {
            Self { x: 1.0, y: 0.0 }
        } else {
            Self {
                x: self.x / norm,
                y: self.y / norm,
            }
        }
    }

    /// 返回与另一向量的点积：同向为正、垂直为 0，用于垂直约束的残差。
    pub fn dot(&self, other: Vector2D) -> f64 {
        self.x * other.x + self.y * other.y
    }

    /// 返回二维叉积的 z 分量（`x·other.y − y·other.x`）：共线时为 0，用于平行与共线约束的残差。
    pub fn cross(&self, other: Vector2D) -> f64 {
        self.x * other.y - self.y * other.x
    }

    /// 返回从 self 转到 other 的有符号夹角，取 `atan2(叉积, 点积)`，范围为 (−π, π]，单位弧度。
    pub fn angle_between(&self, other: Vector2D) -> f64 {
        let dot = self.dot(other);
        let det = self.cross(other);
        det.atan2(dot)
    }
}

impl std::ops::Sub for Vector2D {
    type Output = Self;

    fn sub(self, other: Self) -> Self::Output {
        Self {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }
}

impl std::ops::Add for Vector2D {
    type Output = Self;

    fn add(self, other: Self) -> Self::Output {
        Self {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}

impl std::ops::AddAssign for Vector2D {
    fn add_assign(&mut self, other: Self) {
        self.x += other.x;
        self.y += other.y;
    }
}

impl std::ops::SubAssign for Vector2D {
    fn sub_assign(&mut self, other: Self) {
        self.x -= other.x;
        self.y -= other.y;
    }
}

impl std::ops::Mul<f64> for Vector2D {
    type Output = Self;

    fn mul(self, scalar: f64) -> Self::Output {
        Self {
            x: self.x * scalar,
            y: self.y * scalar,
        }
    }
}

impl std::ops::Div<f64> for Vector2D {
    type Output = Self;

    fn div(self, scalar: f64) -> Self::Output {
        Self {
            x: self.x / scalar,
            y: self.y / scalar,
        }
    }
}

/// 约束图：实体为节点、同一约束涉及的实体两两连边，用于分析约束的连通性与冗余（本模块内部使用）。
struct ConstraintGraph {
    nodes: HashSet<usize>,
    edges: HashSet<(usize, usize)>,
}

impl ConstraintGraph {
    /// 创建空图：无节点、无边。
    pub fn new() -> Self {
        Self {
            nodes: HashSet::new(),
            edges: HashSet::new(),
        }
    }

    /// 加入一个实体节点；重复加入同一节点无副作用。
    pub fn add_node(&mut self, node: usize) {
        self.nodes.insert(node);
    }

    /// 在两个节点之间加一条无向边：自环（`node1 == node2`）被忽略，边按 (较小, 较大) 归一化后去重存储。
    pub fn add_edge(&mut self, node1: usize, node2: usize) {
        if node1 != node2 {
            self.edges.insert((node1.min(node2), node1.max(node2)));
        }
    }

    /// 返回全部连通分量，每个分量是节点下标列表（遍历起点取自哈希集合，顺序不保证稳定）。
    pub fn get_connected_components(&self) -> Vec<Vec<usize>> {
        let mut visited = HashSet::new();
        let mut components = Vec::new();

        for &node in &self.nodes {
            if !visited.contains(&node) {
                let component = self.bfs(node, &mut visited);
                components.push(component);
            }
        }

        components
    }

    fn bfs(&self, start: usize, visited: &mut HashSet<usize>) -> Vec<usize> {
        let mut component = Vec::new();
        let mut queue = VecDeque::new();

        queue.push_back(start);
        visited.insert(start);

        while let Some(node) = queue.pop_front() {
            component.push(node);

            for &neighbor in self.get_neighbors(node) {
                if !visited.contains(&neighbor) {
                    visited.insert(neighbor);
                    queue.push_back(neighbor);
                }
            }
        }

        component
    }

    fn get_neighbors(&self, node: usize) -> Vec<usize> {
        let mut neighbors = Vec::new();

        for &(a, b) in &self.edges {
            if a == node {
                neighbors.push(b);
            } else if b == node {
                neighbors.push(a);
            }
        }

        neighbors
    }

    /// 是否存在环（有环通常意味着约束冗余）。
    /// 注意当前实现把无向边按「小下标 → 大下标」建成有向图再做拓扑排序，该有向图必然无环，
    /// 因此实际上总是返回 false，仅作为待完善的占位判断。
    pub fn has_cycles(&self) -> bool {
        let components = self.get_connected_components();

        for component in components {
            if self.detect_cycle_in_component(&component) {
                return true;
            }
        }

        false
    }

    fn detect_cycle_in_component(&self, component: &[usize]) -> bool {
        let mut in_degree = HashMap::new();
        let mut adjacency = HashMap::new();

        for &node in component {
            in_degree.insert(node, 0);
            adjacency.insert(node, Vec::new());
        }

        for &(a, b) in &self.edges {
            if component.contains(&a) && component.contains(&b) {
                adjacency.get_mut(&a).unwrap().push(b);
                *in_degree.get_mut(&b).unwrap() += 1;
            }
        }

        let mut queue: VecDeque<usize> = in_degree.iter()
            .filter(|(_, &d)| d == 0)
            .map(|(&node, _)| node)
            .collect();

        let mut visited_count = 0;

        while let Some(node) = queue.pop_front() {
            visited_count += 1;

            for &neighbor in adjacency.get(&node).unwrap() {
                if let Some(&mut deg) = in_degree.get_mut(&neighbor) {
                    deg -= 1;
                    if deg == 0 {
                        queue.push_back(neighbor);
                    }
                }
            }
        }

        visited_count < component.len()
    }
}

/// 约束校验错误，由 `ConstraintSystem::validate_constraints` 产生。
#[derive(Debug, Clone)]
pub enum ValidationError {
    /// 第 n 条几何约束无效（引用形式非法）。
    InvalidGeometricConstraint(usize),
    /// 第 n 条尺寸约束无效（引用形式非法或目标值非有限）。
    InvalidDimensionalConstraint(usize),
    /// 过约束：`excess_constraints` 为超出实体自由度的约束条数。
    OverConstrained { excess_constraints: usize },
    /// 欠约束：`missing_constraints` 为仍缺少的约束条数；当前校验流程不会产生。
    UnderConstrained { missing_constraints: usize },
    /// 约束互相冲突：`constraints` 为相关约束的下标；当前校验流程不会产生。
    ConflictingConstraints { constraints: Vec<usize> },
}

impl GeometricConstraint {
    /// 返回该约束直接引用的实体下标，重复引用只保留一次。
    /// 返回：目前仅重合、水平、竖直、平行、垂直、相切、同心返回实体；等长、等半径、中点、点在线上、
    /// 点在圆/弧上、对称、角度、共线、平行 X/Y 等变体尚未实现，一律返回空向量。
    pub fn get_constrained_entities(&self) -> Vec<usize> {
        match self {
            GeometricConstraint::Coincident(p1, p2) => {
                let mut entities = Vec::new();
                if let PointConstraint::EntityPoint(id, _) = p1 {
                    entities.push(*id);
                }
                if let PointConstraint::EntityPoint(id, _) = p2 {
                    if !entities.contains(id) {
                        entities.push(*id);
                    }
                }
                entities
            }
            GeometricConstraint::Horizontal(p1, p2) => {
                let mut entities = Vec::new();
                if let PointConstraint::EntityPoint(id, _) = p1 {
                    entities.push(*id);
                }
                if let PointConstraint::EntityPoint(id, _) = p2 {
                    if !entities.contains(id) {
                        entities.push(*id);
                    }
                }
                entities
            }
            GeometricConstraint::Vertical(p1, p2) => {
                let mut entities = Vec::new();
                if let PointConstraint::EntityPoint(id, _) = p1 {
                    entities.push(*id);
                }
                if let PointConstraint::EntityPoint(id, _) = p2 {
                    if !entities.contains(id) {
                        entities.push(*id);
                    }
                }
                entities
            }
            GeometricConstraint::Parallel(l1, l2) => {
                let mut entities = Vec::new();
                if let LineConstraint::EntityLine(id) = l1 {
                    entities.push(*id);
                }
                if let LineConstraint::EntityLine(id) = l2 {
                    if !entities.contains(id) {
                        entities.push(*id);
                    }
                }
                entities
            }
            GeometricConstraint::Perpendicular(l1, l2) => {
                let mut entities = Vec::new();
                if let LineConstraint::EntityLine(id) = l1 {
                    entities.push(*id);
                }
                if let LineConstraint::EntityLine(id) = l2 {
                    if !entities.contains(id) {
                        entities.push(*id);
                    }
                }
                entities
            }
            GeometricConstraint::Tangent(c1, c2) => {
                let mut entities = Vec::new();
                match c1 {
                    CurveConstraint::Line(l) => {
                        if let LineConstraint::EntityLine(id) = l {
                            entities.push(*id);
                        }
                    }
                    CurveConstraint::Circle(c) => {
                        if let CircleConstraint::EntityCircle(id) = c {
                            entities.push(*id);
                        }
                    }
                    CurveConstraint::Arc(a) => {
                        if let ArcConstraint::EntityArc(id) = a {
                            entities.push(*id);
                        }
                    }
                }
                match c2 {
                    CurveConstraint::Line(l) => {
                        if let LineConstraint::EntityLine(id) = l {
                            if !entities.contains(id) {
                                entities.push(*id);
                            }
                        }
                    }
                    CurveConstraint::Circle(c) => {
                        if let CircleConstraint::EntityCircle(id) = c {
                            if !entities.contains(id) {
                                entities.push(*id);
                            }
                        }
                    }
                    CurveConstraint::Arc(a) => {
                        if let ArcConstraint::EntityArc(id) = a {
                            if !entities.contains(id) {
                                entities.push(*id);
                            }
                        }
                    }
                }
                entities
            }
            GeometricConstraint::Concentric(c1, c2) => {
                let mut entities = Vec::new();
                if let CircleConstraint::EntityCircle(id) = c1 {
                    entities.push(*id);
                }
                if let CircleConstraint::EntityCircle(id) = c2 {
                    if !entities.contains(id) {
                        entities.push(*id);
                    }
                }
                entities
            }
            _ => Vec::new(),
        }
    }

    /// 检查约束的引用形式是否合法：各端必须是实体引用（`EntityPoint`/`EntityLine`/`EntityCircle`/`EntityArc`），
    /// `Angle` 还要求目标角为有限值；`Tangent` 接受任意曲线引用。
    /// 返回：形式合法返回 true，此时才可安全加入系统求解。
    pub fn is_valid(&self) -> bool {
        match self {
            GeometricConstraint::Coincident(p1, p2) => {
                matches!(p1, PointConstraint::EntityPoint(_, _)) &&
                matches!(p2, PointConstraint::EntityPoint(_, _))
            }
            GeometricConstraint::Horizontal(p1, p2) => {
                matches!(p1, PointConstraint::EntityPoint(_, _)) &&
                matches!(p2, PointConstraint::EntityPoint(_, _))
            }
            GeometricConstraint::Vertical(p1, p2) => {
                matches!(p1, PointConstraint::EntityPoint(_, _)) &&
                matches!(p2, PointConstraint::EntityPoint(_, _))
            }
            GeometricConstraint::Parallel(l1, l2) => {
                matches!(l1, LineConstraint::EntityLine(_)) &&
                matches!(l2, LineConstraint::EntityLine(_))
            }
            GeometricConstraint::Perpendicular(l1, l2) => {
                matches!(l1, LineConstraint::EntityLine(_)) &&
                matches!(l2, LineConstraint::EntityLine(_))
            }
            GeometricConstraint::Tangent(c1, c2) => {
                matches!(c1, CurveConstraint::Line(_) | CurveConstraint::Circle(_) | CurveConstraint::Arc(_)) &&
                matches!(c2, CurveConstraint::Line(_) | CurveConstraint::Circle(_) | CurveConstraint::Arc(_))
            }
            GeometricConstraint::Concentric(c1, c2) => {
                matches!(c1, CircleConstraint::EntityCircle(_)) &&
                matches!(c2, CircleConstraint::EntityCircle(_))
            }
            GeometricConstraint::EqualLength(l1, l2) => {
                matches!(l1, LineConstraint::EntityLine(_)) &&
                matches!(l2, LineConstraint::EntityLine(_))
            }
            GeometricConstraint::EqualRadius(c1, c2) => {
                matches!(c1, CircleConstraint::EntityCircle(_)) &&
                matches!(c2, CircleConstraint::EntityCircle(_))
            }
            GeometricConstraint::Midpoint(p, l) => {
                matches!(p, PointConstraint::EntityPoint(_, _)) &&
                matches!(l, LineConstraint::EntityLine(_))
            }
            GeometricConstraint::PointOnLine(p, l) => {
                matches!(p, PointConstraint::EntityPoint(_, _)) &&
                matches!(l, LineConstraint::EntityLine(_))
            }
            GeometricConstraint::PointOnCircle(p, c) => {
                matches!(p, PointConstraint::EntityPoint(_, _)) &&
                matches!(c, CircleConstraint::EntityCircle(_))
            }
            GeometricConstraint::PointOnArc(p, a) => {
                matches!(p, PointConstraint::EntityPoint(_, _)) &&
                matches!(a, ArcConstraint::EntityArc(_))
            }
            GeometricConstraint::Symmetry(p1, p2, l) => {
                matches!(p1, PointConstraint::EntityPoint(_, _)) &&
                matches!(p2, PointConstraint::EntityPoint(_, _)) &&
                matches!(l, LineConstraint::EntityLine(_))
            }
            GeometricConstraint::Angle(l1, l2, angle) => {
                matches!(l1, LineConstraint::EntityLine(_)) &&
                matches!(l2, LineConstraint::EntityLine(_)) &&
                angle.is_finite()
            }
            GeometricConstraint::Collinear(l1, l2) => {
                matches!(l1, LineConstraint::EntityLine(_)) &&
                matches!(l2, LineConstraint::EntityLine(_))
            }
            GeometricConstraint::ParallelX(l) => {
                matches!(l, LineConstraint::EntityLine(_))
            }
            GeometricConstraint::ParallelY(l) => {
                matches!(l, LineConstraint::EntityLine(_))
            }
        }
    }
}

impl DimensionalConstraint {
    /// 返回该尺寸约束引用的实体下标（去重）。
    /// 返回：距离与角度返回两个实体，半径与直径返回一个圆，长度返回一条线；引用自由参数的一侧不计入。
    pub fn get_constrained_entities(&self) -> Vec<usize> {
        match self {
            DimensionalConstraint::Distance(p1, p2, _) => {
                let mut entities = Vec::new();
                if let PointConstraint::EntityPoint(id, _) = p1 {
                    entities.push(*id);
                }
                if let PointConstraint::EntityPoint(id, _) = p2 {
                    if !entities.contains(id) {
                        entities.push(*id);
                    }
                }
                entities
            }
            DimensionalConstraint::Angle(l1, l2, _) => {
                let mut entities = Vec::new();
                if let LineConstraint::EntityLine(id) = l1 {
                    entities.push(*id);
                }
                if let LineConstraint::EntityLine(id) = l2 {
                    if !entities.contains(id) {
                        entities.push(*id);
                    }
                }
                entities
            }
            DimensionalConstraint::Radius(c, _) => {
                if let CircleConstraint::EntityCircle(id) = c {
                    vec![*id]
                } else {
                    Vec::new()
                }
            }
            DimensionalConstraint::Diameter(c, _) => {
                if let CircleConstraint::EntityCircle(id) = c {
                    vec![*id]
                } else {
                    Vec::new()
                }
            }
            DimensionalConstraint::Length(l, _) => {
                if let LineConstraint::EntityLine(id) = l {
                    vec![*id]
                } else {
                    Vec::new()
                }
            }
        }
    }

    /// 检查尺寸约束是否合法：距离与角度要求引用实体且目标值为有限数（距离还须非负），
    /// 半径、直径、长度要求目标值为有限非负数（引用形式不参与校验）。
    /// 返回：合法返回 true。
    pub fn is_valid(&self) -> bool {
        match self {
            DimensionalConstraint::Distance(p1, p2, dist) => {
                matches!(p1, PointConstraint::EntityPoint(_, _)) &&
                matches!(p2, PointConstraint::EntityPoint(_, _)) &&
                dist.is_finite() && *dist >= 0.0
            }
            DimensionalConstraint::Angle(l1, l2, angle) => {
                matches!(l1, LineConstraint::EntityLine(_)) &&
                matches!(l2, LineConstraint::EntityLine(_)) &&
                angle.is_finite()
            }
            DimensionalConstraint::Radius(_, radius) => {
                radius.is_finite() && *radius >= 0.0
            }
            DimensionalConstraint::Diameter(_, diameter) => {
                diameter.is_finite() && *diameter >= 0.0
            }
            DimensionalConstraint::Length(_, length) => {
                length.is_finite() && *length >= 0.0
            }
        }
    }
}

/// 约束构造辅助：以静态方法把常用几何/尺寸约束写入 `ConstraintSystem`，省去手工包装枚举。
///
/// 所有方法都直接修改传入的系统、不返回结果；`*_id` 为实体下标（即 `ConstraintSystem::add_entity` 的返回值），
/// `point*_index` 为实体上的点序号（线取 0 起点、1 终点）。方法本身不做有效性校验。
pub struct ConstraintBuilder;

impl ConstraintBuilder {
    /// 让两个实体上的点重合。
    /// - `system`：目标约束系统。
    /// - `entity1_id`/`point1_index`：第一个实体及其点序号。
    /// - `entity2_id`/`point2_index`：第二个实体及其点序号。
    pub fn coincident(
        system: &mut ConstraintSystem,
        entity1_id: usize,
        point1_index: usize,
        entity2_id: usize,
        point2_index: usize,
    ) {
        let constraint = GeometricConstraint::Coincident(
            PointConstraint::EntityPoint(entity1_id, point1_index),
            PointConstraint::EntityPoint(entity2_id, point2_index),
        );
        system.add_geometric_constraint(constraint);
    }

    /// 让两个点处于同一水平位置（y 坐标相同），参数含义同 `coincident`。
    pub fn horizontal(
        system: &mut ConstraintSystem,
        entity1_id: usize,
        point1_index: usize,
        entity2_id: usize,
        point2_index: usize,
    ) {
        let constraint = GeometricConstraint::Horizontal(
            PointConstraint::EntityPoint(entity1_id, point1_index),
            PointConstraint::EntityPoint(entity2_id, point2_index),
        );
        system.add_geometric_constraint(constraint);
    }

    /// 让两个点处于同一竖直位置（x 坐标相同），参数含义同 `coincident`。
    pub fn vertical(
        system: &mut ConstraintSystem,
        entity1_id: usize,
        point1_index: usize,
        entity2_id: usize,
        point2_index: usize,
    ) {
        let constraint = GeometricConstraint::Vertical(
            PointConstraint::EntityPoint(entity1_id, point1_index),
            PointConstraint::EntityPoint(entity2_id, point2_index),
        );
        system.add_geometric_constraint(constraint);
    }

    /// 让两条实体线平行。
    /// - `line1_id`、`line2_id`：两条线的实体下标。
    pub fn parallel(
        system: &mut ConstraintSystem,
        line1_id: usize,
        line2_id: usize,
    ) {
        let constraint = GeometricConstraint::Parallel(
            LineConstraint::EntityLine(line1_id),
            LineConstraint::EntityLine(line2_id),
        );
        system.add_geometric_constraint(constraint);
    }

    /// 让两条实体线垂直。
    /// - `line1_id`、`line2_id`：两条线的实体下标。
    pub fn perpendicular(
        system: &mut ConstraintSystem,
        line1_id: usize,
        line2_id: usize,
    ) {
        let constraint = GeometricConstraint::Perpendicular(
            LineConstraint::EntityLine(line1_id),
            LineConstraint::EntityLine(line2_id),
        );
        system.add_geometric_constraint(constraint);
    }

    /// 让两条曲线相切。
    /// - `curve1_id`/`curve1_type`：第一条曲线的实体下标与类型。
    /// - `curve2_id`/`curve2_type`：第二条曲线的实体下标与类型。
    /// 注意求解器当前只支持线-圆与圆-圆相切，弧参与时残差按 0 处理（相当于不约束）。
    pub fn tangent(
        system: &mut ConstraintSystem,
        curve1_id: usize,
        curve1_type: CurveType,
        curve2_id: usize,
        curve2_type: CurveType,
    ) {
        let c1 = match curve1_type {
            CurveType::Line => CurveConstraint::Line(LineConstraint::EntityLine(curve1_id)),
            CurveType::Circle => CurveConstraint::Circle(CircleConstraint::EntityCircle(curve1_id)),
            CurveType::Arc => CurveConstraint::Arc(ArcConstraint::EntityArc(curve1_id)),
        };
        
        let c2 = match curve2_type {
            CurveType::Line => CurveConstraint::Line(LineConstraint::EntityLine(curve2_id)),
            CurveType::Circle => CurveConstraint::Circle(CircleConstraint::EntityCircle(curve2_id)),
            CurveType::Arc => CurveConstraint::Arc(ArcConstraint::EntityArc(curve2_id)),
        };

        let constraint = GeometricConstraint::Tangent(c1, c2);
        system.add_geometric_constraint(constraint);
    }

    /// 让两个实体圆同心（圆心重合，半径互不影响）。
    /// - `circle1_id`、`circle2_id`：两个圆的实体下标。
    pub fn concentric(
        system: &mut ConstraintSystem,
        circle1_id: usize,
        circle2_id: usize,
    ) {
        let constraint = GeometricConstraint::Concentric(
            CircleConstraint::EntityCircle(circle1_id),
            CircleConstraint::EntityCircle(circle2_id),
        );
        system.add_geometric_constraint(constraint);
    }

    /// 约束两点距离等于目标值。
    /// - `entity1_id`/`point1_index`、`entity2_id`/`point2_index`：两个点所在的实体与点序号。
    /// - `target_distance`：目标距离，非负，单位同实体坐标。
    pub fn distance(
        system: &mut ConstraintSystem,
        entity1_id: usize,
        point1_index: usize,
        entity2_id: usize,
        point2_index: usize,
        target_distance: f64,
    ) {
        let constraint = DimensionalConstraint::Distance(
            PointConstraint::EntityPoint(entity1_id, point1_index),
            PointConstraint::EntityPoint(entity2_id, point2_index),
            target_distance,
        );
        system.add_dimensional_constraint(constraint);
    }

    /// 约束两条线的夹角等于目标值。
    /// - `line1_id`、`line2_id`：两条线的实体下标。
    /// - `target_angle`：目标夹角，单位为弧度。
    pub fn angle(
        system: &mut ConstraintSystem,
        line1_id: usize,
        line2_id: usize,
        target_angle: f64,
    ) {
        let constraint = DimensionalConstraint::Angle(
            LineConstraint::EntityLine(line1_id),
            LineConstraint::EntityLine(line2_id),
            target_angle,
        );
        system.add_dimensional_constraint(constraint);
    }

    /// 约束圆的半径等于目标值。
    /// - `circle_id`：圆的实体下标。
    /// - `target_radius`：目标半径，非负。
    pub fn radius(
        system: &mut ConstraintSystem,
        circle_id: usize,
        target_radius: f64,
    ) {
        let constraint = DimensionalConstraint::Radius(
            CircleConstraint::EntityCircle(circle_id),
            target_radius,
        );
        system.add_dimensional_constraint(constraint);
    }

    /// 约束圆的直径等于目标值，求解时按 2 × 半径与目标值比较。
    /// - `circle_id`：圆的实体下标。
    /// - `target_diameter`：目标直径，非负。
    pub fn diameter(
        system: &mut ConstraintSystem,
        circle_id: usize,
        target_diameter: f64,
    ) {
        let constraint = DimensionalConstraint::Diameter(
            CircleConstraint::EntityCircle(circle_id),
            target_diameter,
        );
        system.add_dimensional_constraint(constraint);
    }

    /// 约束线的长度等于目标值（按两端点距离计算）。
    /// - `line_id`：线的实体下标。
    /// - `target_length`：目标长度，非负。
    pub fn length(
        system: &mut ConstraintSystem,
        line_id: usize,
        target_length: f64,
    ) {
        let constraint = DimensionalConstraint::Length(
            LineConstraint::EntityLine(line_id),
            target_length,
        );
        system.add_dimensional_constraint(constraint);
    }
}

/// 曲线类型标记：`ConstraintBuilder::tangent` 依据它把实体下标包装成对应的曲线引用。
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CurveType {
    /// 直线或线段。
    Line,
    /// 圆。
    Circle,
    /// 圆弧。
    Arc,
}
