//! 曲线分析模块：提供曲率、切线/法线、Frenet 标架、导数与弧长等分析能力。
//!
//! 分析结果以数据快照的形式返回（[`CurvatureInfo`]、[`FrenetFrame`]、[`CurveDerivatives`]、各类
//! `*Analysis` 结构），便于渲染、标注与质量检查直接消费，不需要再次调用几何求值。
//!
//! 参数约定：各分析方法的 `parameter` 一般归一化到 `[0, 1]`，对 [`Line`] 与 [`Arc`] 表示弧长
//! 比例，对 [`Circle`] 与 [`Ellipse`] 表示整周比例（乘 `2π` 得到圆心角）；`tolerance` 为弧长
//! 数值计算公差，单位与模型空间一致；半径、长度、曲率均为模型空间单位（曲率单位为 1/长度）。
//!
//! 与其他模块的关系：本模块为上层的 Tessellation 细分、Snap 捕捉与 Dimension 标注提供曲线
//! 局部几何属性，具体图元定义见同级的 `line`、`circle`、`arc`、`bspline` 等子模块。
//! 注意：本文件当前未被 `crate::geometry` 的模块树声明，其 `Curve` 使用方式与
//! [`crate::geometry::Curve`]（特征类型）不一致，编译前需先统一。

use crate::geometry::{Point, Vector2, Line, Circle, Arc, Ellipse, BSpline, NURBS, Curve};
use serde::{Serialize, Deserialize};
use std::fmt;

/// 曲率信息快照，描述曲线上某一点的弯曲程度与对应的曲率圆。
///
/// 用于把“曲率、曲率半径、曲率中心、法线”一次性交给调用者，避免重复求导。
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct CurvatureInfo {
    /// 曲线上的采样点。
    pub point: Point,
    /// 该点处的曲率 `κ`，单位 1/长度；直线为 `0.0`，圆上处处等于半径的倒数。
    pub curvature: f64,
    /// 曲率半径 `1/κ`，单位与模型空间一致；曲率为 `0.0` 时为 `INFINITY`。
    pub radius: f64,
    /// 曲率中心，即从 `point` 沿法线方向移动 `radius` 得到的点；直线无定义。
    pub center: Point,
    /// 指向曲率中心一侧的单位法向量（平面内）。
    pub normal: Vector2,
}

/// Frenet 标架快照，给出曲线在某点处的局部活动坐标系。
///
/// `tangent`、`normal` 位于 XY 平面内且相互垂直，`binormal` 当前恒为零向量（曲线按平面曲线处理）。
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct FrenetFrame {
    /// 标架原点，即曲线上的采样点。
    pub point: Point,
    /// 单位切向量，方向与参数增长方向一致。
    pub tangent: Vector2,
    /// 单位主法向量，指向曲线凹侧（曲率中心方向）。
    pub normal: Vector2,
    /// 副法向量，当前实现恒为 `(0.0, 0.0)`。
    pub binormal: Vector2,
}

/// 曲线导数快照，给出某点处的一至三阶导数向量。
///
/// 导数均对归一化参数求取（第一项即 `dC/dt`），因此模长依赖曲线的参数化方式，
/// 不等于对弧长求导；用于连续性判断与曲率计算。
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct CurveDerivatives {
    /// 曲线上的采样点。
    pub point: Point,
    /// 一阶导数向量，方向即切线方向。
    pub first_derivative: Vector2,
    /// 二阶导数向量，反映弯曲变化；直线恒为零向量。
    pub second_derivative: Vector2,
    /// 三阶导数向量，用于挠率等更高阶分析。
    pub third_derivative: Vector2,
}

/// 曲线分析特征，为各类曲线图元提供统一的曲率、标架、导数与弧长查询接口。
///
/// 参数 `parameter` 归一化到 `[0, 1]`，`tolerance` 为数值积分公差。
/// 调用者通常以 `&dyn CurveAnalyzer` 或泛型约束使用本特征。
pub trait CurveAnalyzer {
    /// 返回参数处曲率 `κ`，单位 1/长度；直线返回 `0.0`。
    fn curvature_at(&self, parameter: f64) -> f64;
    /// 返回给定点处曲率 `κ`；多数实现只按曲线类型返回常数，不按点位置求解。
    fn curvature_at_point(&self, point: Point) -> f64;
    /// 返回参数处曲率半径 `1/κ`，单位与模型空间一致；直线返回 `INFINITY`。
    fn radius_of_curvature(&self, parameter: f64) -> f64;
    /// 返回参数处的单位切向量。
    fn tangent_at(&self, parameter: f64) -> Vector2;
    /// 返回参数处的单位法向量，指向曲线凹侧。
    fn normal_at(&self, parameter: f64) -> Vector2;
    /// 返回参数处的 Frenet 标架快照。
    fn frenet_frame_at(&self, parameter: f64) -> FrenetFrame;
    /// 返回参数处的一至三阶导数快照。
    fn derivatives_at(&self, parameter: f64) -> CurveDerivatives;
    /// 返回整条曲线长度，按 `tolerance` 控制数值精度；直线与圆为解析解。
    fn arc_length(&self, tolerance: f64) -> f64;
    /// 返回弧长 `length` 处对应的参数，结果截断到 `[0, 1]`。
    fn parameter_at_arc_length(&self, length: f64) -> f64;
    /// 判断本曲线与另一条曲线是否 G1 连续（连接处切向一致）。
    fn is_g1_continuous(&self, other: &dyn Curve) -> bool;
    /// 判断本曲线与另一条曲线是否 G2 连续（切向与曲率均一致）。
    fn is_g2_continuous(&self, other: &dyn Curve) -> bool;
}

/// 直线分析器，只能通过 [`LineAnalyzer::analyze`] 使用，自身不保存状态。
pub struct LineAnalyzer;

impl LineAnalyzer {
    /// 分析直线段并返回属性快照。
    ///
    /// - `line`：待分析的线段，只读借用，不被修改。
    ///
    /// 返回的 [`LineAnalysis`] 中曲率恒为 `0.0`、曲率半径恒为 `INFINITY`、`is_linear`
    /// 恒为 `true`，包围盒为两个端点构成的轴对齐矩形。
    pub fn analyze(line: &Line) -> LineAnalysis {
        let length = line.length();
        let midpoint = line.midpoint();
        let direction = line.direction();
        let normal = Vector2::new(-direction.y, direction.x);
        
        LineAnalysis {
            line: line.clone(),
            length,
            midpoint,
            direction,
            normal,
            curvature: 0.0,
            radius_of_curvature: f64::INFINITY,
            is_linear: true,
            is_horizontal: line.is_horizontal(),
            is_vertical: line.is_vertical(),
            bounding_box: (line.start, line.end),
        }
    }
}

/// 直线段的属性快照，由 [`LineAnalyzer::analyze`] 生成。
#[derive(Debug, Clone)]
pub struct LineAnalysis {
    /// 被分析的线段副本，与传入对象相互独立。
    pub line: Line,
    /// 线段长度，模型空间单位。
    pub length: f64,
    /// 线段中点。
    pub midpoint: Point,
    /// 单位方向向量，由 `start` 指向 `end`；零长度线段为零向量。
    pub direction: Vector2,
    /// 单位法向量，由方向向量逆时针旋转 90° 得到。
    pub normal: Vector2,
    /// 曲率，对直线恒为 `0.0`。
    pub curvature: f64,
    /// 曲率半径，对直线恒为 `INFINITY`。
    pub radius_of_curvature: f64,
    /// 是否线性图元，对直线恒为 `true`。
    pub is_linear: bool,
    /// 是否水平，判定容差为 `1e-10`。
    pub is_horizontal: bool,
    /// 是否垂直，判定容差为 `1e-10`。
    pub is_vertical: bool,
    /// 轴对齐包围盒 `(min, max)`，由两个端点构成。
    pub bounding_box: (Point, Point),
}

/// 圆弧分析器，只能通过 [`ArcAnalyzer::analyze`] 使用，自身不保存状态。
pub struct ArcAnalyzer;

impl ArcAnalyzer {
    /// 分析圆弧并返回属性快照。
    ///
    /// - `arc`：待分析的圆弧，只读借用，不被修改。
    ///
    /// 长度按 `半径 × |end_angle - start_angle|` 计算，即直接取起止角之差的绝对值，
    /// 不区分顺逆时针；`sweep_angle` 使用同一算法。
    /// 包围盒按整圆外接矩形给出，因此可能大于弧实际覆盖的范围；返回点的 Z 坐标统一为 `0.0`。
    pub fn analyze(arc: &Arc) -> ArcAnalysis {
        let length = arc.radius * (arc.end_angle - arc.start_angle).abs();
        let midpoint_angle = (arc.start_angle + arc.end_angle) / 2.0;
        let midpoint = Point::new(
            arc.center.x + arc.radius * midpoint_angle.cos(),
            arc.center.y + arc.radius * midpoint_angle.sin(),
            0.0,
        );
        
        let curvature = 1.0 / arc.radius;
        let radius_of_curvature = arc.radius;
        
        let tangent = Vector2::new(-midpoint_angle.sin(), midpoint_angle.cos());
        let normal = Vector2::new(midpoint_angle.cos(), midpoint_angle.sin());
        
        let start_point = Point::new(
            arc.center.x + arc.radius * arc.start_angle.cos(),
            arc.center.y + arc.radius * arc.start_angle.sin(),
            0.0,
        );
        let end_point = Point::new(
            arc.center.x + arc.radius * arc.end_angle.cos(),
            arc.center.y + arc.radius * arc.end_angle.sin(),
            0.0,
        );
        
        let min_x = arc.center.x - arc.radius;
        let max_x = arc.center.x + arc.radius;
        let min_y = arc.center.y - arc.radius;
        let max_y = arc.center.y + arc.radius;
        
        ArcAnalysis {
            arc: arc.clone(),
            length,
            midpoint,
            direction: tangent,
            normal,
            curvature,
            radius_of_curvature,
            center: arc.center,
            diameter: arc.radius * 2.0,
            sweep_angle: (arc.end_angle - arc.start_angle).abs(),
            start_point,
            end_point,
            bounding_box: (
                Point::new(min_x, min_y, 0.0),
                Point::new(max_x, max_y, 0.0),
            ),
            is_clockwise: !arc.is_counter_clockwise,
        }
    }
}

/// 圆弧的属性快照，由 [`ArcAnalyzer::analyze`] 生成。
#[derive(Debug, Clone)]
pub struct ArcAnalysis {
    /// 被分析的圆弧副本。
    pub arc: Arc,
    /// 弧长，等于半径乘以起止角之差的绝对值。
    pub length: f64,
    /// 起止角算术平均值处的点（弧中点）。
    pub midpoint: Point,
    /// 中点处的单位切向量，指向角度增大的方向。
    pub direction: Vector2,
    /// 中点处的单位法向量，由圆心指向中点。
    pub normal: Vector2,
    /// 曲率 `1/radius`，单位 1/长度。
    pub curvature: f64,
    /// 曲率半径，等于圆弧半径。
    pub radius_of_curvature: f64,
    /// 圆心。
    pub center: Point,
    /// 直径，等于半径的两倍。
    pub diameter: f64,
    /// 扫掠角，单位为弧度，取起止角之差的绝对值。
    pub sweep_angle: f64,
    /// 起始角对应的点。
    pub start_point: Point,
    /// 终止角对应的点。
    pub end_point: Point,
    /// 轴对齐包围盒 `(min, max)`，按整圆外接矩形给出，可能大于弧实际范围。
    pub bounding_box: (Point, Point),
}

/// 圆分析器，只能通过 [`CircleAnalyzer::analyze`] 使用，自身不保存状态。
pub struct CircleAnalyzer;

impl CircleAnalyzer {
    /// 分析圆并返回属性快照。
    ///
    /// - `circle`：待分析的圆，只读借用，不被修改。
    ///
    /// 周长与面积为解析解；`circumference_points` 给出 0°、90°、180°、270° 四个采样点，
    /// 用于快速绘制或 Snap 捕捉候选，返回点 Z 坐标统一为 `0.0`。半径为 `0.0` 时曲率为无穷。
    pub fn analyze(circle: &Circle) -> CircleAnalysis {
        let circumference = 2.0 * std::f64::consts::PI * circle.radius;
        let area = std::f64::consts::PI * circle.radius * circle.radius;
        let curvature = 1.0 / circle.radius;
        
        let min_x = circle.center.x - circle.radius;
        let max_x = circle.center.x + circle.radius;
        let min_y = circle.center.y - circle.radius;
        let max_y = circle.center.y + circle.radius;
        
        let point1 = Point::new(circle.center.x + circle.radius, circle.center.y, 0.0);
        let point2 = Point::new(circle.center.x, circle.center.y + circle.radius, 0.0);
        let point3 = Point::new(circle.center.x - circle.radius, circle.center.y, 0.0);
        let point4 = Point::new(circle.center.x, circle.center.y - circle.radius, 0.0);
        
        CircleAnalysis {
            circle: circle.clone(),
            circumference,
            area,
            curvature,
            radius_of_curvature: circle.radius,
            diameter: circle.radius * 2.0,
            circumference_points: vec![point1, point2, point3, point4],
            bounding_box: (
                Point::new(min_x, min_y, 0.0),
                Point::new(max_x, max_y, 0.0),
            ),
        }
    }
}

/// 圆的属性快照，由 [`CircleAnalyzer::analyze`] 生成。
#[derive(Debug, Clone)]
pub struct CircleAnalysis {
    /// 被分析的圆副本。
    pub circle: Circle,
    /// 周长 `2πr`，模型空间单位。
    pub circumference: f64,
    /// 面积 `πr²`，模型空间单位的平方。
    pub area: f64,
    /// 曲率 `1/radius`，单位 1/长度。
    pub curvature: f64,
    /// 曲率半径，等于圆半径。
    pub radius_of_curvature: f64,
    /// 直径，等于半径的两倍。
    pub diameter: f64,
    /// 圆周上的四个象限采样点（0°、90°、180°、270°）。
    pub circumference_points: Vec<Point>,
    /// 轴对齐包围盒 `(min, max)`，外接于该圆。
    pub bounding_box: (Point, Point),
}

/// B 样条分析器，只能通过 [`BSplineAnalyzer::analyze`] 使用，自身不保存状态。
pub struct BSplineAnalyzer;

impl BSplineAnalyzer {
    /// 分析 B 样条曲线并返回属性快照。
    ///
    /// - `spline`：待分析的曲线，只读借用，不被修改。
    ///
    /// 弧长使用 `0.001` 的固定公差；曲率在参数 `[0, 1]` 上等距取 100 个样本统计最大、最小
    /// 与平均值，因此凸起处的真实极值可能被采样遗漏。`is_polynomial` 仅在次数不大于 1 时为
    /// `true`，次数更高时也会返回 `false`。
    pub fn analyze(spline: &BSpline) -> BSplineAnalysis {
        let length = spline.length(0.001);
        let degree = spline.degree;
        let num_control_points = spline.control_points.len();
        let num_knots = spline.knots.len();
        
        let mut max_curvature = 0.0;
        let mut min_curvature = f64::INFINITY;
        let mut total_curvature = 0.0;
        let mut curvature_samples = 0;
        
        for i in 0..100 {
            let t = i as f64 / 99.0;
            let curvature = spline.curvature_at(t);
            max_curvature = max_curvature.max(curvature);
            min_curvature = min_curvature.min(curvature);
            total_curvature += curvature;
            curvature_samples += 1;
        }
        
        let avg_curvature = total_curvature / curvature_samples as f64;
        
        BSplineAnalysis {
            spline: spline.clone(),
            length,
            degree,
            num_control_points,
            num_knots,
            max_curvature,
            min_curvature,
            avg_curvature,
            is_closed: spline.is_closed(),
            is_polynomial: degree <= 1,
        }
    }
}

/// B 样条曲线的属性快照，由 [`BSplineAnalyzer::analyze`] 生成。
#[derive(Debug, Clone)]
pub struct BSplineAnalysis {
    /// 被分析的曲线副本。
    pub spline: BSpline,
    /// 曲线长度，按 `0.001` 公差数值计算。
    pub length: f64,
    /// 曲线次数。
    pub degree: usize,
    /// 控制点数量。
    pub num_control_points: usize,
    /// 节点向量长度，合法值应为控制点数加次数加一。
    pub num_knots: usize,
    /// 100 个采样点中的最大曲率。
    pub max_curvature: f64,
    /// 100 个采样点中的最小曲率；无采样时保持 `INFINITY`。
    pub min_curvature: f64,
    /// 100 个采样点曲率的算术平均值。
    pub avg_curvature: f64,
    /// 曲线是否闭合。
    pub is_closed: bool,
    /// 是否按多项式曲线处理，仅当次数不大于 1 时为 `true`。
    pub is_polynomial: bool,
}

impl CurveAnalyzer for Line {
    fn curvature_at(&self, _parameter: f64) -> f64 {
        0.0
    }
    
    fn curvature_at_point(&self, _point: Point) -> f64 {
        0.0
    }
    
    fn radius_of_curvature(&self, _parameter: f64) -> f64 {
        f64::INFINITY
    }
    
    fn tangent_at(&self, parameter: f64) -> Vector2 {
        self.direction()
    }
    
    fn normal_at(&self, parameter: f64) -> Vector2 {
        let dir = self.direction();
        Vector2::new(-dir.y, dir.x)
    }
    
    fn frenet_frame_at(&self, _parameter: f64) -> FrenetFrame {
        let point = self.point_at_parameter(_parameter);
        let tangent = self.direction();
        let normal = Vector2::new(-tangent.y, tangent.x);
        let binormal = Vector2::new(0.0, 0.0);
        
        FrenetFrame { point, tangent, normal, binormal }
    }
    
    fn derivatives_at(&self, _parameter: f64) -> CurveDerivatives {
        let point = self.point_at_parameter(_parameter);
        let first = self.direction();
        let second = Vector2::new(0.0, 0.0);
        let third = Vector2::new(0.0, 0.0);
        
        CurveDerivatives { point, first_derivative: first, second_derivative: second, third_derivative: third }
    }
    
    fn arc_length(&self, _tolerance: f64) -> f64 {
        self.length()
    }
    
    fn parameter_at_arc_length(&self, length: f64) -> f64 {
        (length / self.length()).clamp(0.0, 1.0)
    }
    
    fn is_g1_continuous(&self, other: &dyn Curve) -> bool {
        match other {
            Curve::Line(other_line) => {
                let end_dir = self.direction();
                let start_dir = other_line.direction();
                (end_dir - start_dir).magnitude() < 1e-6
            }
            _ => false,
        }
    }
    
    fn is_g2_continuous(&self, other: &dyn Curve) -> bool {
        self.is_g1_continuous(other)
    }
}

impl CurveAnalyzer for Circle {
    fn curvature_at(&self, _parameter: f64) -> f64 {
        1.0 / self.radius
    }
    
    fn curvature_at_point(&self, _point: Point) -> f64 {
        1.0 / self.radius
    }
    
    fn radius_of_curvature(&self, _parameter: f64) -> f64 {
        self.radius
    }
    
    fn tangent_at(&self, parameter: f64) -> Vector2 {
        let angle = parameter * 2.0 * std::f64::consts::PI;
        Vector2::new(-angle.sin(), angle.cos())
    }
    
    fn normal_at(&self, parameter: f64) -> Vector2 {
        let angle = parameter * 2.0 * std::f64::consts::PI;
        Vector2::new(angle.cos(), angle.sin())
    }
    
    fn frenet_frame_at(&self, parameter: f64) -> FrenetFrame {
        let angle = parameter * 2.0 * std::f64::consts::PI;
        let point = Point::new(
            self.center.x + self.radius * angle.cos(),
            self.center.y + self.radius * angle.sin(),
            0.0,
        );
        let tangent = Vector2::new(-angle.sin(), angle.cos());
        let normal = Vector2::new(angle.cos(), angle.sin());
        let binormal = Vector2::new(0.0, 0.0);
        
        FrenetFrame { point, tangent, normal, binormal }
    }
    
    fn derivatives_at(&self, parameter: f64) -> CurveDerivatives {
        let angle = parameter * 2.0 * std::f64::consts::PI;
        let point = Point::new(
            self.center.x + self.radius * angle.cos(),
            self.center.y + self.radius * angle.sin(),
            0.0,
        );
        let first = Vector2::new(
            -self.radius * 2.0 * std::f64::consts::PI * angle.sin(),
            self.radius * 2.0 * std::f64::consts::PI * angle.cos(),
        );
        let second = Vector2::new(
            -self.radius * (2.0 * std::f64::consts::PI).powi(2) * angle.cos(),
            -self.radius * (2.0 * std::f64::consts::PI).powi(2) * angle.sin(),
        );
        let third = Vector2::new(
            self.radius * (2.0 * std::f64::consts::PI).powi(3) * angle.sin(),
            -self.radius * (2.0 * std::f64::consts::PI).powi(3) * angle.cos(),
        );
        
        CurveDerivatives { point, first_derivative: first, second_derivative: second, third_derivative: third }
    }
    
    fn arc_length(&self, _tolerance: f64) -> f64 {
        2.0 * std::f64::consts::PI * self.radius
    }
    
    fn parameter_at_arc_length(&self, length: f64) -> f64 {
        (length / (2.0 * std::f64::consts::PI * self.radius)).clamp(0.0, 1.0)
    }
    
    fn is_g1_continuous(&self, _other: &dyn Curve) -> bool {
        true
    }
    
    fn is_g2_continuous(&self, _other: &dyn Curve) -> bool {
        true
    }
}

impl CurveAnalyzer for Arc {
    fn curvature_at(&self, _parameter: f64) -> f64 {
        1.0 / self.radius
    }
    
    fn curvature_at_point(&self, _point: Point) -> f64 {
        1.0 / self.radius
    }
    
    fn radius_of_curvature(&self, _parameter: f64) -> f64 {
        self.radius
    }
    
    fn tangent_at(&self, parameter: f64) -> Vector2 {
        let angle = self.start_angle + parameter * (self.end_angle - self.start_angle);
        Vector2::new(-angle.sin(), angle.cos())
    }
    
    fn normal_at(&self, parameter: f64) -> Vector2 {
        let angle = self.start_angle + parameter * (self.end_angle - self.start_angle);
        Vector2::new(angle.cos(), angle.sin())
    }
    
    fn frenet_frame_at(&self, parameter: f64) -> FrenetFrame {
        let angle = self.start_angle + parameter * (self.end_angle - self.start_angle);
        let point = Point::new(
            self.center.x + self.radius * angle.cos(),
            self.center.y + self.radius * angle.sin(),
            0.0,
        );
        let tangent = Vector2::new(-angle.sin(), angle.cos());
        let normal = Vector2::new(angle.cos(), angle.sin());
        let binormal = Vector2::new(0.0, 0.0);
        
        FrenetFrame { point, tangent, normal, binormal }
    }
    
    fn derivatives_at(&self, parameter: f64) -> CurveDerivatives {
        let angle = self.start_angle + parameter * (self.end_angle - self.start_angle);
        let d_angle = self.end_angle - self.start_angle;
        let point = Point::new(
            self.center.x + self.radius * angle.cos(),
            self.center.y + self.radius * angle.sin(),
            0.0,
        );
        let first = Vector2::new(
            -self.radius * d_angle * angle.sin(),
            self.radius * d_angle * angle.cos(),
        );
        let second = Vector2::new(
            -self.radius * d_angle.powi(2) * angle.cos(),
            -self.radius * d_angle.powi(2) * angle.sin(),
        );
        let third = Vector2::new(
            self.radius * d_angle.powi(3) * angle.sin(),
            -self.radius * d_angle.powi(3) * angle.cos(),
        );
        
        CurveDerivatives { point, first_derivative: first, second_derivative: second, third_derivative: third }
    }
    
    fn arc_length(&self, _tolerance: f64) -> f64 {
        self.radius * (self.end_angle - self.start_angle).abs()
    }
    
    fn parameter_at_arc_length(&self, length: f64) -> f64 {
        let total_length = self.arc_length(0.001);
        let t = (length / total_length).clamp(0.0, 1.0);
        let angle_span = self.end_angle - self.start_angle;
        if angle_span > 0.0 {
            t
        } else {
            1.0 - t
        }
    }
    
    fn is_g1_continuous(&self, _other: &dyn Curve) -> bool {
        true
    }
    
    fn is_g2_continuous(&self, _other: &dyn Curve) -> bool {
        true
    }
}

/// 计算任意曲线的总长度。
///
/// - `curve`：曲线对象，以特征对象传入，由内部按具体种类分派。
/// - `tolerance`：数值积分公差，单位与模型空间一致，仅对 B 样条与 NURBS 生效；直线、圆、
///   圆弧与多段线使用解析式或顶点累加。
///
/// 返回长度为模型空间单位。当前实现对椭圆分支直接返回 `0.0`（未实现），多段线按顶点逐段
/// 累加，闭合多段线会额外计入末顶点到首顶点的闭合段。
pub fn compute_curve_length(curve: &dyn Curve, tolerance: f64) -> f64 {
    match curve {
        Curve::Line(line) => line.length(),
        Curve::Circle(circle) => 2.0 * std::f64::consts::PI * circle.radius,
        Curve::Arc(arc) => arc.radius * (arc.end_angle - arc.start_angle).abs(),
        Curve::Ellipse(_) => 0.0,
        Curve::BSpline(spline) => spline.length(tolerance),
        Curve::NURBS(nurbs) => nurbs.length(tolerance),
        Curve::Polyline(polyline) => {
            let mut length = 0.0;
            for i in 0..polyline.vertices.len().saturating_sub(if polyline.is_closed { 0 } else { 1 }) {
                let next_i = if polyline.is_closed && i + 1 >= polyline.vertices.len() { 0 } else { i + 1 };
                if next_i < polyline.vertices.len() {
                    length += polyline.vertices[i].distance_to(&polyline.vertices[next_i]);
                }
            }
            length
        }
    }
}

/// 计算任意曲线在给定参数处的点。
///
/// - `curve`：曲线对象，以特征对象传入，由内部按具体种类分派。
/// - `parameter`：归一化参数，`0.0` 为首端、`1.0` 为末端；对圆与椭圆按整周比例解释
///   （内部乘以 `2π` 得到圆心角），对直线与圆弧按各自参数区间线性映射。
///
/// 不支持的曲线种类（B 样条、NURBS 等）返回坐标原点，调用者需自行区分；`parameter` 不做
/// 区间限制，超出范围会沿参数方向外推。
pub fn compute_point_on_curve(curve: &dyn Curve, parameter: f64) -> Point {
    match curve {
        Curve::Line(line) => line.point_at_parameter(parameter),
        Curve::Circle(circle) => {
            let angle = parameter * 2.0 * std::f64::consts::PI;
            Point::new(
                circle.center.x + circle.radius * angle.cos(),
                circle.center.y + circle.radius * angle.sin(),
                0.0,
            )
        }
        Curve::Arc(arc) => {
            let angle = arc.start_angle + parameter * (arc.end_angle - arc.start_angle);
            Point::new(
                arc.center.x + arc.radius * angle.cos(),
                arc.center.y + arc.radius * angle.sin(),
                0.0,
            )
        }
        Curve::Ellipse(ellipse) => {
            let angle = parameter * 2.0 * std::f64::consts::PI;
            Point::new(
                ellipse.center.x + ellipse.semi_major * angle.cos(),
                ellipse.center.y + ellipse.semi_minor * angle.sin(),
                0.0,
            )
        }
        _ => Point::origin(),
    }
}
