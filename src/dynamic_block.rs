//! 动态块（Dynamic Block）：由参数、动作、可见性状态与夹点构成的参数化块定义。
//!
//! 核心概念：
//! - 参数（`BlockParameter`）：可被驱动并带取值范围/表达式的量，距离类为世界单位，角度类为弧度；
//! - 动作（`BlockAction`）：由参数驱动、作用于一组实体的变换（移动/缩放/拉伸/旋转/镜像/阵列等）；
//! - 可见性状态（`VisibilitySetting`）：按实体 ID 决定该状态下哪些实体可见；
//! - 夹点（`GripPoint`）：参数在画布上的可拖拽控制点，由 `DynamicBlock::update_grips` 从参数派生。
//!
//! `DynamicBlock` 保存单个块的完整定义，`DynamicBlockManager` 按名称管理多个块并跟踪活动块；
//! 本模块只维护定义与规则，不直接修改文档中的实体，`BlockUnit` 提供块单位到毫米的换算系数。

use serde::{Serialize, Deserialize};
use std::collections::{HashMap, HashSet};
use std::fmt;

/// 参数类型：说明动态块参数驱动的是何种几何量，决定其取值单位与可搭配的动作。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ParameterType {
    /// 点位置参数，驱动实体的插入点或基点（世界坐标）。
    Point,
    /// 线性参数，沿 X 轴或 Y 轴度量并驱动距离。
    Linear,
    /// 对齐参数，沿两个定义点连线方向度量距离。
    Aligned,
    /// 角度参数，取值单位是弧度而非角度。
    Angle,
    /// 距离参数，取值为世界单位下的长度。
    Distance,
    /// 半径参数，驱动圆或圆弧的半径。
    Radius,
    /// 直径参数，驱动圆或圆弧的直径。
    Diameter,
    /// 弧长参数，驱动圆弧的弧线长度。
    ArcLength,
    /// 面积参数，驱动闭合区域的面积。
    Area,
    /// 可见性参数，用于切换到某个 `VisibilitySetting` 状态。
    Visibility,
    /// 查寻参数，按 `lookup_table` 把输入值组合映射为输出值。
    Lookup,
    /// 变换参数，把整体变换施加到被驱动的实体上。
    Xform,
    /// 块特性参数，把块特性暴露为可被动作读取的参数。
    BlockProperty,
    /// 用户自定义参数，语义由上层应用约定。
    UserParameter,
}

impl Default for ParameterType {
    fn default() -> Self {
        ParameterType::Point
    }
}

impl ParameterType {
    /// 返回该参数类型的英文显示名（如 `"Point"`、`"Arc Length"`），供界面与序列化展示使用。
    pub fn name(&self) -> &str {
        match self {
            ParameterType::Point => "Point",
            ParameterType::Linear => "Linear",
            ParameterType::Aligned => "Aligned",
            ParameterType::Angle => "Angle",
            ParameterType::Distance => "Distance",
            ParameterType::Radius => "Radius",
            ParameterType::Diameter => "Diameter",
            ParameterType::ArcLength => "Arc Length",
            ParameterType::Area => "Area",
            ParameterType::Visibility => "Visibility",
            ParameterType::Lookup => "Lookup",
            ParameterType::Xform => "Xform",
            ParameterType::BlockProperty => "Block Property",
            ParameterType::UserParameter => "User Parameter",
        }
    }

    /// 返回该参数类型在参数面板中使用的图标字符。
    pub fn icon(&self) -> &str {
        match self {
            ParameterType::Point => "📍",
            ParameterType::Linear => "↔",
            ParameterType::Aligned => "↗",
            ParameterType::Angle => "∠",
            ParameterType::Distance => "📏",
            ParameterType::Radius => "⭕",
            ParameterType::Diameter => "⊕",
            ParameterType::ArcLength => "⌒",
            ParameterType::Area => "⬜",
            ParameterType::Visibility => "👁",
            ParameterType::Lookup => "📋",
            ParameterType::Xform => "🔄",
            ParameterType::BlockProperty => "📦",
            ParameterType::UserParameter => "🔧",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
/// 单个动态块参数：定义、当前取值与可选的取值范围/表达式/查寻表。
pub struct BlockParameter {
    /// 参数名，块内唯一，动作通过它与参数关联。
    pub name: String,
    /// 参数类型，决定取值含义与可搭配的动作。
    pub parameter_type: ParameterType,
    /// 界面显示用的标签文本。
    pub label: String,
    /// 参数的说明文本。
    pub description: String,
    /// 当前取值；距离类为世界单位，角度类为弧度。
    pub value: f64,
    /// 默认取值，用于重置参数。
    pub default_value: f64,
    /// 取值下限（含端点）；`None` 表示不限制。
    pub minimum: Option<f64>,
    /// 取值上限（含端点）；`None` 表示不限制。
    pub maximum: Option<f64>,
    /// 交互拖动时的步长；`None` 表示连续取值。
    pub step: Option<f64>,
    /// 与该参数关联的表达式列表。
    pub expressions: Vec<Expression>,
    /// 是否为链式动作参数：取值变化会触发 `chain_actions` 中的动作。
    pub is_chain_action: bool,
    /// 是否为预设参数：可由 `ParameterSet` 一次性写入。
    pub is_preset: bool,
    /// 是否为查寻参数：为真时使用 `lookup_table` 做输入到输出的映射。
    pub is_lookup: bool,
    /// 查寻表，每行把一个输入值组合映射为一个输出值。
    pub lookup_table: Vec<LookupRow>,
    /// 参数在块坐标系中的定位点。
    pub position: crate::geometry::Point,
    /// 参数方向角，单位为弧度。
    pub angle: f64,
    /// 链式动作的 ID 列表，顺序即触发顺序。
    pub chain_actions: Vec<String>,
    /// 关联的块特性 ID；`None` 表示未关联。
    pub property_id: Option<String>,
}

impl Default for BlockParameter {
    fn default() -> Self {
        Self::new()
    }
}

impl BlockParameter {
    /// 创建默认参数：类型为 `Point`，取值为 0.0，无上下限、无表达式与查寻表。
    pub fn new() -> Self {
        Self {
            name: String::new(),
            parameter_type: ParameterType::Point,
            label: String::new(),
            description: String::new(),
            value: 0.0,
            default_value: 0.0,
            minimum: None,
            maximum: None,
            step: None,
            expressions: Vec::new(),
            is_chain_action: false,
            is_preset: false,
            is_lookup: false,
            lookup_table: Vec::new(),
            position: crate::geometry::Point::origin(),
            angle: 0.0,
            chain_actions: Vec::new(),
            property_id: None,
        }
    }

    /// 设置参数名并返回自身，用于链式构造。
    pub fn with_name(mut self, name: &str) -> Self {
        self.name = name.to_string();
        self
    }

    /// 设置参数类型并返回自身，用于链式构造。
    pub fn with_type(mut self, parameter_type: ParameterType) -> Self {
        self.parameter_type = parameter_type;
        self
    }

    /// 直接写入当前取值并返回自身；不校验 `minimum`/`maximum`。
    pub fn with_value(mut self, value: f64) -> Self {
        self.value = value;
        self
    }

    /// 同时设置取值上下限（均为含端点）并返回自身。
    pub fn with_range(mut self, min: f64, max: f64) -> Self {
        self.minimum = Some(min);
        self.maximum = Some(max);
        self
    }

    /// 校验后写入当前取值。
    ///
    /// - `value`：新取值，距离类为世界单位，角度类为弧度。
    ///
    /// 返回是否写入成功；超出 `minimum`/`maximum`（含端点比较）时保持不变并返回 `false`。
    pub fn set_value(&mut self, value: f64) -> bool {
        if let Some(min) = self.minimum {
            if value < min {
                return false;
            }
        }
        if let Some(max) = self.maximum {
            if value > max {
                return false;
            }
        }
        self.value = value;
        true
    }

    /// 追加一条表达式；不做去重，也不立即求值。
    pub fn add_expression(&mut self, expression: Expression) {
        self.expressions.push(expression);
    }

    /// 追加一个链式动作 ID；不去重，同一 ID 可重复加入。
    pub fn add_chain_action(&mut self, action_id: &str) {
        self.chain_actions.push(action_id.to_string());
    }

    /// 向查寻表末尾追加一行；按加入顺序参与匹配。
    pub fn add_lookup_row(&mut self, row: LookupRow) {
        self.lookup_table.push(row);
    }
}

/// 查寻表的一行：把一组输入值映射为一个输出值。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LookupRow {
    /// 输入值序列，顺序须与查寻参数的输入顺序一致。
    pub input_values: Vec<f64>,
    /// 命中该行时输出的值。
    pub output_value: f64,
}

impl Default for LookupRow {
    fn default() -> Self {
        Self::new()
    }
}

impl LookupRow {
    /// 创建空行：无输入值，输出值为 0.0。
    pub fn new() -> Self {
        Self {
            input_values: Vec::new(),
            output_value: 0.0,
        }
    }

    /// 一次性设置输入值序列与输出值并返回自身。
    pub fn with_values(mut self, inputs: &[f64], output: f64) -> Self {
        self.input_values = inputs.to_vec();
        self.output_value = output;
        self
    }
}

/// 参数表达式：保存表达式原文、最近一次求值结果与有效性标记。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Expression {
    /// 表达式名称，用于界面显示与外部引用。
    pub name: String,
    /// 表达式原文，例如 `"PI/2"`。
    pub expression: String,
    /// 最近一次求值结果；角度类表达式为弧度，未求值或解析失败时为 0.0。
    pub value: f64,
    /// 表达式是否有效的标记，由调用方维护，`evaluate` 不会改写它。
    pub is_valid: bool,
}

impl Default for Expression {
    fn default() -> Self {
        Self::new()
    }
}

impl Expression {
    /// 创建空表达式：原文为空、值为 0.0 且标记为有效。
    pub fn new() -> Self {
        Self {
            name: String::new(),
            expression: String::new(),
            value: 0.0,
            is_valid: true,
        }
    }

    /// 设置表达式原文、立即求值一次并返回自身。
    pub fn with_expression(mut self, expr: &str) -> Self {
        self.expression = expr.to_string();
        self.evaluate();
        self
    }

    /// 就地重新求值并写入 `value`（不改写 `expression` 与 `is_valid`）。
    ///
    /// 先去除空格，仅识别 `PI`、`2*PI`、`PI/2`、`PI/4` 四个常量；其余表达式去掉圆括号后按十进制浮点数解析，
    /// 仍无法解析时结果为 0.0。角度常量结果单位为弧度。
    pub fn evaluate(&mut self) {
        let expr = self.expression.replace(" ", "");
        self.value = match expr.as_str() {
            "PI" => std::f64::consts::PI,
            "2*PI" => 2.0 * std::f64::consts::PI,
            "PI/2" => std::f64::consts::PI / 2.0,
            "PI/4" => std::f64::consts::PI / 4.0,
            _ => {
                let cleaned = expr.replace("(", "").replace(")", "");
                if let Ok(num) = cleaned.parse::<f64>() {
                    num
                } else {
                    0.0
                }
            }
        };
    }
}

/// 动作类型：参数取值变化时施加到目标实体上的变换方式。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ActionType {
    /// 平移：按 `displacement` 移动实体。
    Move,
    /// 缩放：以基点按 `scale` 缩放实体。
    Scale,
    /// 拉伸：按基点位移拉伸实体，保持与基点的相对关系。
    Stretch,
    /// 旋转：绕基点按 `angle` 旋转，角度为弧度。
    Rotate,
    /// 镜像：沿动作方向镜像实体。
    Mirror,
    /// 矩形阵列：按行列重复实体。
    Array,
    /// 极轴阵列：绕中心按角度重复实体。
    Polar,
    /// 查寻驱动：按查寻表选中的行施加变换。
    Lookup,
}

impl Default for ActionType {
    fn default() -> Self {
        ActionType::Move
    }
}

impl ActionType {
    /// 返回该动作类型的英文显示名（如 `"Move"`、`"Rotate"`）。
    pub fn name(&self) -> &str {
        match self {
            ActionType::Move => "Move",
            ActionType::Scale => "Scale",
            ActionType::Stretch => "Stretch",
            ActionType::Rotate => "Rotate",
            ActionType::Mirror => "Mirror",
            ActionType::Array => "Array",
            ActionType::Polar => "Polar",
            ActionType::Lookup => "Lookup",
        }
    }

    /// 返回该动作类型在动作面板中使用的图标字符。
    pub fn icon(&self) -> &str {
        match self {
            ActionType::Move => "➡",
            ActionType::Scale => "↔",
            ActionType::Stretch => "↗",
            ActionType::Rotate => "🔄",
            ActionType::Mirror => "🪞",
            ActionType::Array => "▦",
            ActionType::Polar => "◎",
            ActionType::Lookup => "📋",
        }
    }
}

/// 一条动态块动作：由某个参数驱动，把一种变换施加到一组实体上。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BlockAction {
    /// 动作唯一 ID（新建时生成 UUID 字符串），`dependent_actions` 等按此引用。
    pub id: String,
    /// 动作类型，决定使用哪些变换字段。
    pub action_type: ActionType,
    /// 动作名，用于界面显示。
    pub name: String,
    /// 动作说明文本。
    pub description: String,
    /// 驱动本动作的参数名，对应 `BlockParameter::name`。
    pub parameter_name: String,
    /// 受本动作影响的实体 ID 列表。
    pub entities: Vec<String>,
    /// 动作的连接点，用于定位基点与拉伸边界。
    pub connection_points: Vec<ConnectionPoint>,
    /// 平移向量，世界坐标。
    pub displacement: crate::geometry::Point,
    /// 旋转角，单位为弧度。
    pub angle: f64,
    /// 缩放比例，默认 1.0。
    pub scale: f64,
    /// 是否为依赖动作：由其他动作触发，而非直接由参数驱动。
    pub is_dependent: bool,
    /// 是否启用；为 `false` 时本动作不参与求值。
    pub is_enabled: bool,
    /// 是否翻转动作方向。
    pub flip_action: bool,
    /// 动作基点；`None` 表示未设置，取参数位置为基点。
    pub base_point: Option<ConnectionPoint>,
    /// 动作方向角，单位为弧度。
    pub action_direction: f64,
    /// 动作的附加数值（例如阵列角度增量或查寻输出值）。
    pub action_value: f64,
    /// 依赖本动作的其他动作 ID 列表。
    pub dependent_actions: Vec<String>,
}

impl Default for BlockAction {
    fn default() -> Self {
        Self::new()
    }
}

impl BlockAction {
    /// 创建默认动作：类型为 `Move`、scale 为 1.0、启用且非依赖动作，并生成新的 UUID 作为 `id`。
    pub fn new() -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            action_type: ActionType::Move,
            name: String::new(),
            description: String::new(),
            parameter_name: String::new(),
            entities: Vec::new(),
            connection_points: Vec::new(),
            displacement: crate::geometry::Point::origin(),
            angle: 0.0,
            scale: 1.0,
            is_dependent: false,
            is_enabled: true,
            flip_action: false,
            base_point: None,
            action_direction: 0.0,
            action_value: 0.0,
            dependent_actions: Vec::new(),
        }
    }

    /// 设置动作类型并返回自身，用于链式构造。
    pub fn with_type(mut self, action_type: ActionType) -> Self {
        self.action_type = action_type;
        self
    }

    /// 设置动作名并返回自身，用于链式构造。
    pub fn with_name(mut self, name: &str) -> Self {
        self.name = name.to_string();
        self
    }

    /// 绑定驱动参数（按参数名匹配）并返回自身。
    pub fn with_parameter(mut self, param_name: &str) -> Self {
        self.parameter_name = param_name.to_string();
        self
    }

    /// 追加一个受本动作影响的实体 ID；不去重。
    pub fn add_entity(&mut self, entity_id: &str) {
        self.entities.push(entity_id.to_string());
    }

    /// 追加一个动作连接点；按加入顺序保存，不去重。
    pub fn add_connection_point(&mut self, point: ConnectionPoint) {
        self.connection_points.push(point);
    }

    /// 覆盖平移向量（世界坐标）；不触碰其他变换字段。
    pub fn set_displacement(&mut self, displacement: crate::geometry::Point) {
        self.displacement = displacement;
    }

    /// 设置旋转角；单位为弧度，不做归一化。
    pub fn set_rotation(&mut self, angle: f64) {
        self.angle = angle;
    }

    /// 设置缩放比例；不做正值校验，0 会导致实体退化。
    pub fn set_scale(&mut self, scale: f64) {
        self.scale = scale;
    }

    /// 启用或停用本动作；停用后求值阶段会跳过它。
    pub fn set_enabled(&mut self, enabled: bool) {
        self.is_enabled = enabled;
    }

    /// 追加一个依赖本动作的动作 ID；不去重。
    pub fn add_dependent_action(&mut self, action_id: &str) {
        self.dependent_actions.push(action_id.to_string());
    }

    /// 移除对指定动作的全部依赖记录；不存在时静默无操作。
    pub fn remove_dependent_action(&mut self, action_id: &str) {
        self.dependent_actions.retain(|id| id != action_id);
    }
}

/// 动作连接点：实体上的一个可抓取位置，用于确定动作基点或拉伸边界。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConnectionPoint {
    /// 连接点的世界坐标位置。
    pub position: crate::geometry::Point,
    /// 所属实体 ID；为空字符串表示不绑定实体。
    pub entity_id: String,
    /// 该实体上的夹点序号。
    pub grip_index: u32,
    /// 是否为块或动作的基点。
    pub is_base: bool,
}

impl Default for ConnectionPoint {
    fn default() -> Self {
        Self::new()
    }
}

impl ConnectionPoint {
    /// 创建连接点：原点、不绑定实体、夹点序号 0、非基点。
    pub fn new() -> Self {
        Self {
            position: crate::geometry::Point::origin(),
            entity_id: String::new(),
            grip_index: 0,
            is_base: false,
        }
    }

    /// 在指定世界坐标处创建不绑定实体的连接点。
    pub fn at_position(position: crate::geometry::Point) -> Self {
        Self {
            position,
            entity_id: String::new(),
            grip_index: 0,
            is_base: false,
        }
    }

    /// 在指定实体上创建连接点。
    ///
    /// - `entity_id`：所属实体 ID；
    /// - `position`：世界坐标位置，需与实体几何一致。
    pub fn on_entity(entity_id: &str, position: crate::geometry::Point) -> Self {
        Self {
            position,
            entity_id: entity_id.to_string(),
            grip_index: 0,
            is_base: false,
        }
    }

    /// 把本连接点标记为基点并返回自身（消费式调用，用于链式构造）。
    pub fn as_base(mut self) -> Self {
        self.is_base = true;
        self
    }
}

/// 可见性状态：实体在动态块中的显示方式。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VisibilityState {
    /// 可见：正常绘制并参与拾取。
    Visible,
    /// 不可见：不绘制，但实体仍保留在块定义中。
    Invisible,
    /// 隐藏：与不可见分开的一种隐藏标记，具体语义由调用方解释。
    Hidden,
}

impl Default for VisibilityState {
    fn default() -> Self {
        VisibilityState::Visible
    }
}

/// 一个可见性状态：记录该状态下哪些实体可见、哪些实体被隐藏。
///
/// 由 `DynamicBlock::set_visibility_state` 按 `name` 查找并据此改写各实体的可见标志。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VisibilitySetting {
    /// 状态名，在块内唯一，作为切换时的查找键。
    pub name: String,
    /// 状态的说明文本。
    pub description: String,
    /// 该状态下可见的实体 ID 列表。
    pub visible_entities: Vec<String>,
    /// 该状态下隐藏的实体 ID 列表。
    pub hidden_entities: Vec<String>,
    /// 状态在界面上使用的图标字符。
    pub icon: String,
}

impl Default for VisibilitySetting {
    fn default() -> Self {
        Self::new()
    }
}

impl VisibilitySetting {
    /// 创建空状态：名称、说明、图标均为空，可见与隐藏实体列表为空。
    pub fn new() -> Self {
        Self {
            name: String::new(),
            description: String::new(),
            visible_entities: Vec::new(),
            hidden_entities: Vec::new(),
            icon: String::new(),
        }
    }

    /// 设置状态名并返回自身，用于链式构造。
    pub fn with_name(mut self, name: &str) -> Self {
        self.name = name.to_string();
        self
    }

    /// 把实体标记为本状态下的可见或隐藏；两个列表互斥，重复调用不会产生重复项。
    ///
    /// - `entity_id`：目标实体 ID；
    /// - `visible`：`true` 表示加入可见列表并移出隐藏列表，`false` 反之。
    pub fn set_entity_visible(&mut self, entity_id: &str, visible: bool) {
        if visible {
            if !self.visible_entities.contains(&entity_id.to_string()) {
                self.visible_entities.push(entity_id.to_string());
            }
            self.hidden_entities.retain(|e| e != entity_id);
        } else {
            if !self.hidden_entities.contains(&entity_id.to_string()) {
                self.hidden_entities.push(entity_id.to_string());
            }
            self.visible_entities.retain(|e| e != entity_id);
        }
    }

    /// 判断实体在本状态下是否可见。
    ///
    /// 返回仅在实体同时出现在 `visible_entities` 且不在 `hidden_entities` 中时为 `true`；
    /// 未登记过的实体返回 `false`。
    pub fn is_entity_visible(&self, entity_id: &str) -> bool {
        self.visible_entities.contains(&entity_id.to_string())
            && !self.hidden_entities.contains(&entity_id.to_string())
    }
}

/// 参数集：把若干参数与动作打包成一组，供一次性切换或应用。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ParameterSet {
    /// 参数集名称，在块内唯一。
    pub name: String,
    /// 参数集说明文本。
    pub description: String,
    /// 包含的参数名列表，按 `BlockParameter::name` 引用。
    pub parameters: Vec<String>,
    /// 包含的动作 ID 列表，按 `BlockAction::id` 引用。
    pub actions: Vec<String>,
    /// 参数集在界面上使用的图标字符。
    pub icon: String,
}

impl Default for ParameterSet {
    fn default() -> Self {
        Self::new()
    }
}

impl ParameterSet {
    /// 创建空参数集：名称、说明、图标为空，参数与动作列表为空。
    pub fn new() -> Self {
        Self {
            name: String::new(),
            description: String::new(),
            parameters: Vec::new(),
            actions: Vec::new(),
            icon: String::new(),
        }
    }

    /// 设置参数集名称并返回自身，用于链式构造。
    pub fn with_name(mut self, name: &str) -> Self {
        self.name = name.to_string();
        self
    }

    /// 向集合中加入一个参数名；不去重，也不校验该参数是否存在。
    pub fn add_parameter(&mut self, param_name: &str) {
        self.parameters.push(param_name.to_string());
    }

    /// 向集合中加入一个动作 ID；不去重，也不校验该动作是否存在。
    pub fn add_action(&mut self, action_id: &str) {
        self.actions.push(action_id.to_string());
    }
}

/// 夹点：参数在画布上的可拖拽控制点，拖动即修改关联参数的取值。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GripPoint {
    /// 被驱动的参数名，对应 `BlockParameter::name`。
    pub parameter_name: String,
    /// 夹点当前对应的参数值；角度类为弧度。
    pub parameter_value: f64,
    /// 夹点的世界坐标位置。
    pub position: crate::geometry::Point,
    /// 夹点类型，决定拖动交互方式。
    pub grip_type: GripType,
    /// 是否显示该夹点。
    pub is_visible: bool,
    /// 是否可交互；为 `false` 时仍可显示但不可拖动。
    pub is_enabled: bool,
    /// 是否处于悬停高亮状态，属于瞬态界面状态。
    pub is_hovered: bool,
    /// 悬停提示文本。
    pub tooltip: String,
}

/// 夹点类型：决定夹点被拖动时的交互方式。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GripType {
    /// 线性夹点：沿单轴改变距离。
    Linear,
    /// 角度夹点：绕基点改变角度（弧度）。
    Angular,
    /// 半径夹点：沿径向改变半径。
    Radial,
    /// 移动夹点：整体平移所驱动的实体。
    Mover,
    /// 对齐夹点：沿两个定义点的连线方向移动。
    Aligned,
}

impl Default for GripType {
    fn default() -> Self {
        GripType::Mover
    }
}

impl GripPoint {
    /// 创建空夹点：无关联参数、值为 0.0、位于原点、类型为 `Mover` 且可见可用。
    pub fn new() -> Self {
        Self {
            parameter_name: String::new(),
            parameter_value: 0.0,
            position: crate::geometry::Point::origin(),
            grip_type: GripType::Mover,
            is_visible: true,
            is_enabled: true,
            is_hovered: false,
            tooltip: String::new(),
        }
    }

    /// 为指定参数创建移动类夹点，并自动生成 `"参数名: 值"` 形式的提示文本。
    ///
    /// - `param_name`：被驱动的参数名；
    /// - `value`：夹点对应的参数值（角度为弧度）；
    /// - `position`：夹点的世界坐标位置。
    pub fn for_parameter(param_name: &str, value: f64, position: crate::geometry::Point) -> Self {
        Self {
            parameter_name: param_name.to_string(),
            parameter_value: value,
            position,
            grip_type: GripType::Mover,
            is_visible: true,
            is_enabled: true,
            is_hovered: false,
            tooltip: format!("{}: {}", param_name, value),
        }
    }

    /// 更新夹点的世界坐标位置。
    pub fn set_position(&mut self, position: crate::geometry::Point) {
        self.position = position;
    }

    /// 设置夹点是否可交互。
    pub fn set_enabled(&mut self, enabled: bool) {
        self.is_enabled = enabled;
    }

    /// 设置夹点是否处于悬停高亮状态。
    pub fn set_hover(&mut self, hovered: bool) {
        self.is_hovered = hovered;
    }
}

/// 动态块的完整定义：实体集合、参数、动作、可见性状态、参数集与夹点。
///
/// 块内的 `entities` 保存在块坐标系中，`scale` 与 `rotation` 为插入时的整体变换；
/// 本结构只保存定义，参数求值与实体改写由上层（编辑/渲染流程）驱动。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DynamicBlock {
    /// 块名，在 `DynamicBlockManager` 中作为唯一键。
    pub name: String,
    /// 块说明文本。
    pub description: String,
    /// 块单位，用于与其他单位换算（换算系数以毫米为基准）。
    pub block_units: BlockUnit,
    /// 块内的实体列表（块坐标系）。
    pub entities: Vec<DynamicBlockEntity>,
    /// 参数列表，参数名在块内唯一。
    pub parameters: Vec<BlockParameter>,
    /// 动作列表，动作 ID 在块内唯一。
    pub actions: Vec<BlockAction>,
    /// 可见性状态列表，状态名在块内唯一。
    pub visibility_states: Vec<VisibilitySetting>,
    /// 参数集列表。
    pub parameter_sets: Vec<ParameterSet>,
    /// 夹点列表，通常由 `update_grips` 从参数派生。
    pub grips: Vec<GripPoint>,
    /// 是否为动态块；`convert_to_static` 会置为 `false` 并清空参数化数据。
    pub is_dynamic: bool,
    /// 是否同时作为块表记录存在。
    pub is_block_table_record: bool,
    /// 插入比例，默认 1.0，需为正数。
    pub scale: f64,
    /// 插入旋转角，单位为弧度，默认 0.0。
    pub rotation: f64,
    /// 是否允许分解；为 `false` 时块参照不可被炸开。
    pub allows_exploding: bool,
    /// 关联的块表记录 ID；为空字符串表示未关联。
    pub block_table_record_id: String,
}

impl Default for DynamicBlock {
    fn default() -> Self {
        Self::new()
    }
}

impl DynamicBlock {
    /// 创建空动态块：无名、无实体与参数，单位无单位，比例 1.0，允许分解。
    pub fn new() -> Self {
        Self {
            name: String::new(),
            description: String::new(),
            block_units: BlockUnit::Unitless,
            entities: Vec::new(),
            parameters: Vec::new(),
            actions: Vec::new(),
            visibility_states: Vec::new(),
            parameter_sets: Vec::new(),
            grips: Vec::new(),
            is_dynamic: true,
            is_block_table_record: true,
            scale: 1.0,
            rotation: 0.0,
            allows_exploding: true,
            block_table_record_id: String::new(),
        }
    }

    /// 设置块名并返回自身，用于链式构造。
    pub fn with_name(mut self, name: &str) -> Self {
        self.name = name.to_string();
        self
    }

    /// 追加一个块内实体；按加入顺序保存，不校验 ID 重复。
    pub fn add_entity(&mut self, entity: DynamicBlockEntity) {
        self.entities.push(entity);
    }

    /// 追加一个参数；不校验参数名是否已存在。
    pub fn add_parameter(&mut self, parameter: BlockParameter) {
        self.parameters.push(parameter);
    }

    /// 追加一个动作；不校验动作 ID 是否已存在。
    pub fn add_action(&mut self, action: BlockAction) {
        self.actions.push(action);
    }

    /// 追加一个可见性状态；不校验状态名是否已存在。
    pub fn add_visibility_state(&mut self, state: VisibilitySetting) {
        self.visibility_states.push(state);
    }

    /// 追加一个参数集；不校验名称是否已存在。
    pub fn add_parameter_set(&mut self, set: ParameterSet) {
        self.parameter_sets.push(set);
    }

    /// 按名称查找参数，返回首个同名参数的只读引用；不存在时为 `None`。
    pub fn get_parameter(&self, name: &str) -> Option<&BlockParameter> {
        self.parameters.iter().find(|p| p.name == name)
    }

    /// 按 ID 查找动作，返回首个匹配动作的只读引用；不存在时为 `None`。
    pub fn get_action(&self, id: &str) -> Option<&BlockAction> {
        self.actions.iter().find(|a| a.id == id)
    }

    /// 按参数名写入取值（经参数自身的上下限校验）。
    ///
    /// 返回是否写入成功：参数不存在或取值越界时返回 `false` 且不修改块。
    pub fn set_parameter_value(&mut self, name: &str, value: f64) -> bool {
        if let Some(param) = self.parameters.iter_mut().find(|p| p.name == name) {
            param.set_value(value)
        } else {
            false
        }
    }

    /// 切换到指定可见性状态：按状态内容改写块内每个实体的 `is_visible` 标志。
    ///
    /// 返回是否切换成功；状态名不存在时返回 `false` 且所有实体保持原样。
    /// 副作用：直接修改 `self.entities`（状态列表中未登记的实体将被置为不可见）。
    pub fn set_visibility_state(&mut self, state_name: &str) -> bool {
        if let Some(state) = self.visibility_states.iter().find(|s| s.name == state_name) {
            for entity in &mut self.entities {
                entity.is_visible = state.is_entity_visible(&entity.id);
            }
            true
        } else {
            false
        }
    }

    /// 依据当前参数重建夹点列表。
    ///
    /// 先清空 `grips`，再为每个参数生成一个移动类夹点（位置取参数定位点、提示文本自动生成），
    /// 因此原有夹点的启用/悬停等定制状态会丢失。
    pub fn update_grips(&mut self) {
        self.grips.clear();
        for param in &self.parameters {
            let grip = GripPoint::for_parameter(
                &param.name,
                param.value,
                param.position,
            );
            self.grips.push(grip);
        }
    }

    /// 动态块是否被锁定；当前实现恒返回 `false`（块定义本身不提供锁定语义）。
    pub fn is_locked(&self) -> bool {
        false
    }

    /// 设置是否允许分解；为 `false` 时块参照不可被炸开。
    pub fn set_allows_exploding(&mut self, allows: bool) {
        self.allows_exploding = allows;
    }
}

/// 动态块内部的实体记录：几何引用加显示属性，坐标位于块坐标系。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DynamicBlockEntity {
    /// 实体唯一 ID（新建时生成 UUID 字符串），供参数与动作引用。
    pub id: String,
    /// 实体类型名，例如 `"LINE"`、`"CIRCLE"`。
    pub entity_type: String,
    /// 几何数据的字符串表示，含义由 `entity_type` 决定。
    pub geometry: String,
    /// 在块坐标系中的插入点/基点（世界坐标，z 通常为 0）。
    pub position: crate::geometry::Point,
    /// 在块坐标系中的旋转角，单位为弧度。
    pub rotation: f64,
    /// X/Y 方向的缩放比例，通常为 (1.0, 1.0)。
    pub scale: (f64, f64),
    /// 所属图层名。
    pub layer: String,
    /// RGB 颜色分量，每个分量取值 0~255。
    pub color: (u8, u8, u8),
    /// 线型名，对应线型表条目。
    pub linetype: String,
    /// 线宽，单位为 1/100 毫米；0 表示使用默认线宽。
    pub lineweight: i32,
    /// 是否可见；`DynamicBlock::set_visibility_state` 会改写该标志。
    pub is_visible: bool,
    /// 是否锁定，锁定的实体不参与编辑操作。
    pub is_locked: bool,
    /// 依次施加的变换列表，`Transformation::order` 决定顺序。
    pub transformations: Vec<Transformation>,
}

impl Default for DynamicBlockEntity {
    fn default() -> Self {
        Self::new()
    }
}

impl DynamicBlockEntity {
    /// 创建空实体：生成新 UUID、类型与几何为空、比例 (1.0, 1.0)、颜色黑色、可见未锁定。
    pub fn new() -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            entity_type: String::new(),
            geometry: String::new(),
            position: crate::geometry::Point::origin(),
            rotation: 0.0,
            scale: (1.0, 1.0),
            layer: String::new(),
            color: (0, 0, 0),
            linetype: String::new(),
            lineweight: 0,
            is_visible: true,
            is_locked: false,
            transformations: Vec::new(),
        }
    }

    /// 设置实体类型名（如 `"LINE"`）并返回自身，用于链式构造。
    pub fn with_type(mut self, entity_type: &str) -> Self {
        self.entity_type = entity_type.to_string();
        self
    }

    /// 设置实体在块坐标系中的插入点并返回自身。
    pub fn at_position(mut self, position: crate::geometry::Point) -> Self {
        self.position = position;
        self
    }

    /// 设置实体是否可见；不影响所属块的可见性状态列表。
    pub fn set_visibility(&mut self, visible: bool) {
        self.is_visible = visible;
    }

    /// 追加一个变换；不会清空已有变换，最终顺序由 `Transformation::order` 决定。
    pub fn add_transformation(&mut self, transformation: Transformation) {
        self.transformations.push(transformation);
    }

    /// 清空全部变换，使实体的 `transformations` 变为空列表。
    pub fn clear_transformations(&mut self) {
        self.transformations.clear();
    }
}

/// 单个几何变换记录：类型、参数表与应用顺序。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Transformation {
    /// 变换类型，决定 `parameters` 中需要哪些键。
    pub transformation_type: TransformationType,
    /// 变换参数表，键名依类型而定（旋转用 `"angle"`，单位为弧度；缩放用比例键）。
    pub parameters: HashMap<String, f64>,
    /// 应用顺序，数值小者先应用。
    pub order: u32,
}

impl Default for Transformation {
    fn default() -> Self {
        Self::new()
    }
}

impl Transformation {
    /// 创建恒等变换：类型为 `Identity`、参数表为空、顺序为 0。
    pub fn new() -> Self {
        Self {
            transformation_type: TransformationType::Identity,
            parameters: HashMap::new(),
            order: 0,
        }
    }

    /// 设置变换类型并返回自身，用于链式构造。
    pub fn with_type(mut self, transformation_type: TransformationType) -> Self {
        self.transformation_type = transformation_type;
        self
    }

    /// 插入或覆盖一个变换参数并返回自身；同名键以本次写入为准。
    pub fn with_param(mut self, name: &str, value: f64) -> Self {
        self.parameters.insert(name.to_string(), value);
        self
    }
}

/// 变换类型：实体变换的种类，决定 `Transformation::parameters` 的键含义。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TransformationType {
    /// 恒等变换，不改变几何。
    Identity,
    /// 平移，参数为各轴位移量（世界单位）。
    Translation,
    /// 旋转，参数 `"angle"` 以弧度给出。
    Rotation,
    /// 缩放，参数为各轴比例。
    Scaling,
    /// 镜像，参数描述镜像轴。
    Mirror,
    /// 错切，参数为各轴错切因子。
    Shear,
}

impl Default for TransformationType {
    fn default() -> Self {
        TransformationType::Identity
    }
}

/// 块单位：块定义使用的长度单位，`conversion_factor` 给出到毫米的换算系数。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BlockUnit {
    /// 无单位：数值原样使用，换算系数为 1.0。
    Unitless,
    /// 英寸（1 in = 25.4 mm）。
    Inches,
    /// 英尺（1 ft = 304.8 mm）。
    Feet,
    /// 英里（1 mi = 1609344 mm）。
    Miles,
    /// 毫米（基准单位，系数 1.0）。
    Millimeters,
    /// 厘米（1 cm = 10 mm）。
    Centimeters,
    /// 米（1 m = 1000 mm）。
    Meters,
    /// 千米（1 km = 10^6 mm）。
    Kilometers,
    /// 微英寸（1 µin = 2.54e-5 mm）。
    Microinches,
    /// 密尔，即千分之一英寸（1 mil = 0.0254 mm）。
    Mils,
    /// 码（1 yd = 914.4 mm）。
    Yards,
    /// 埃（1 Å = 10^-7 mm）。
    Angstroms,
    /// 纳米（1 nm = 10^-6 mm）。
    Nanometers,
    /// 微米（1 µm = 0.001 mm）。
    Microns,
    /// 分米（1 dm = 100 mm）。
    Decimeters,
    /// 十米（1 dam = 10^4 mm）。
    Decameters,
    /// 百米（1 hm = 10^5 mm）。
    Hectometers,
    /// 吉米（1 Gm = 10^12 mm）。
    Gigameters,
    /// 天文单位（≈ 1.495978707e14 mm）。
    AstronomicalUnits,
    /// 光年（≈ 9.4607304725808e18 mm）。
    LightYears,
    /// 秒差距（≈ 3.085677581491367e19 mm）。
    Parsecs,
}

impl Default for BlockUnit {
    fn default() -> Self {
        BlockUnit::Unitless
    }
}

impl BlockUnit {
    /// 返回块单位的英文显示名（如 `"Millimeters"`、`"Astronomical Units"`）。
    pub fn name(&self) -> &str {
        match self {
            BlockUnit::Unitless => "Unitless",
            BlockUnit::Inches => "Inches",
            BlockUnit::Feet => "Feet",
            BlockUnit::Miles => "Miles",
            BlockUnit::Millimeters => "Millimeters",
            BlockUnit::Centimeters => "Centimeters",
            BlockUnit::Meters => "Meters",
            BlockUnit::Kilometers => "Kilometers",
            BlockUnit::Microinches => "Microinches",
            BlockUnit::Mils => "Mils",
            BlockUnit::Yards => "Yards",
            BlockUnit::Angstroms => "Angstroms",
            BlockUnit::Nanometers => "Nanometers",
            BlockUnit::Microns => "Microns",
            BlockUnit::Decimeters => "Decimeters",
            BlockUnit::Decameters => "Decameters",
            BlockUnit::Hectometers => "Hectometers",
            BlockUnit::Gigameters => "Gigameters",
            BlockUnit::AstronomicalUnits => "Astronomical Units",
            BlockUnit::LightYears => "Light Years",
            BlockUnit::Parsecs => "Parsecs",
        }
    }

    /// 返回该单位到毫米的换算系数：`Unitless` 与 `Millimeters` 均为 1.0，`Meters` 为 1000.0。
    ///
    /// 调用方用它把块单位长度换算为毫米，例如英寸长度乘以 25.4。
    pub fn conversion_factor(&self) -> f64 {
        match self {
            BlockUnit::Unitless => 1.0,
            BlockUnit::Inches => 25.4,
            BlockUnit::Feet => 304.8,
            BlockUnit::Miles => 1609344.0,
            BlockUnit::Millimeters => 1.0,
            BlockUnit::Centimeters => 10.0,
            BlockUnit::Meters => 1000.0,
            BlockUnit::Kilometers => 1000000.0,
            BlockUnit::Microinches => 0.0000254,
            BlockUnit::Mils => 0.0254,
            BlockUnit::Yards => 914.4,
            BlockUnit::Angstroms => 0.0000001,
            BlockUnit::Nanometers => 0.000001,
            BlockUnit::Microns => 0.001,
            BlockUnit::Decimeters => 100.0,
            BlockUnit::Decameters => 10000.0,
            BlockUnit::Hectometers => 100000.0,
            BlockUnit::Gigameters => 1000000000000.0,
            BlockUnit::AstronomicalUnits => 149597870700000.0,
            BlockUnit::LightYears => 9460730472580800000.0,
            BlockUnit::Parsecs => 30856775814913670000.0,
        }
    }
}

/// 动态块管理器：按名称保存块定义，并跟踪当前活动块与编辑/面板显示状态。
#[derive(Debug, Clone)]
pub struct DynamicBlockManager {
    blocks: HashMap<String, DynamicBlock>,
    active_block: Option<String>,
    is_editing: bool,
    show_grips: bool,
    show_palette: bool,
}

impl Default for DynamicBlockManager {
    fn default() -> Self {
        Self::new()
    }
}

impl DynamicBlockManager {
    /// 创建空管理器：无块、无活动块、非编辑态，夹点与参数面板默认显示。
    pub fn new() -> Self {
        Self {
            blocks: HashMap::new(),
            active_block: None,
            is_editing: false,
            show_grips: true,
            show_palette: true,
        }
    }

    /// 新建同名块并设为活动块，返回其可变引用供继续填充内容。
    ///
    /// - `name`：块名，同名旧块会被覆盖；
    ///
    /// 副作用：写入管理器并把 `active_block` 指向新块。
    pub fn create_block(&mut self, name: &str) -> &mut DynamicBlock {
        let block = DynamicBlock::new().with_name(name);
        self.blocks.insert(name.to_string(), block);
        self.active_block = Some(name.to_string());
        self.blocks.get_mut(name).unwrap()
    }

    /// 插入一个块，以 `block.name` 为键；同名块会被覆盖，且不改变活动块。
    pub fn add_block(&mut self, block: DynamicBlock) {
        self.blocks.insert(block.name.clone(), block);
    }

    /// 按名称获取块的只读引用；不存在时为 `None`。
    pub fn get_block(&self, name: &str) -> Option<&DynamicBlock> {
        self.blocks.get(name)
    }

    /// 按名称获取块的可变引用；不存在时为 `None`。
    pub fn get_block_mut(&mut self, name: &str) -> Option<&mut DynamicBlock> {
        self.blocks.get_mut(name)
    }

    /// 删除指定块，返回是否确实删除了一个块；不存在时为 `false`。
    pub fn remove_block(&mut self, name: &str) -> bool {
        self.blocks.remove(name).is_some()
    }

    /// 重命名块并同步其 `name` 字段。
    ///
    /// 返回是否成功：`old_name` 不存在时为 `false`；目标名已存在时会被覆盖。
    pub fn rename_block(&mut self, old_name: &str, new_name: &str) -> bool {
        if let Some(block) = self.blocks.remove(old_name) {
            let mut new_block = block;
            new_block.name = new_name.to_string();
            self.blocks.insert(new_name.to_string(), new_block);
            true
        } else {
            false
        }
    }

    /// 向指定块追加一个参数；块不存在时返回 `false`。
    pub fn add_parameter(&mut self, block_name: &str, parameter: BlockParameter) -> bool {
        if let Some(block) = self.blocks.get_mut(block_name) {
            block.parameters.push(parameter);
            true
        } else {
            false
        }
    }

    /// 向指定块追加一个动作；块不存在时返回 `false`。
    pub fn add_action(&mut self, block_name: &str, action: BlockAction) -> bool {
        if let Some(block) = self.blocks.get_mut(block_name) {
            block.actions.push(action);
            true
        } else {
            false
        }
    }

    /// 写入指定块的参数值（含参数自身的范围校验）。
    ///
    /// 返回是否成功：块不存在、参数不存在或取值越界时返回 `false` 且不做修改。
    pub fn set_parameter_value(&mut self, block_name: &str, param_name: &str, value: f64) -> bool {
        if let Some(block) = self.blocks.get_mut(block_name) {
            block.set_parameter_value(param_name, value)
        } else {
            false
        }
    }

    /// 切换指定块的可见性状态（会按状态改写块内各实体的可见标志）。
    ///
    /// 返回是否成功：块不存在或状态名不存在时返回 `false`。
    pub fn set_visibility_state(&mut self, block_name: &str, state_name: &str) -> bool {
        if let Some(block) = self.blocks.get_mut(block_name) {
            block.set_visibility_state(state_name)
        } else {
            false
        }
    }

    /// 返回已注册块的数量。
    pub fn block_count(&self) -> usize {
        self.blocks.len()
    }

    /// 返回全部块名；顺序取决于内部哈希表的键序，不保证稳定。
    pub fn block_names(&self) -> Vec<&str> {
        self.blocks.keys().map(|s| s.as_str()).collect()
    }

    /// 设置活动块；传 `None` 取消活动块，传不存在的名称也不会报错（后续读取得到 `None`）。
    pub fn set_active_block(&mut self, name: Option<&str>) {
        self.active_block = name.map(|s| s.to_string());
    }

    /// 返回活动块的只读引用；未设置活动块或该名称已失效时为 `None`。
    pub fn active_block(&self) -> Option<&DynamicBlock> {
        self.active_block.as_ref().and_then(|name| self.blocks.get(name))
    }

    /// 返回活动块的可变引用；未设置活动块或该名称已失效时为 `None`。
    pub fn active_block_mut(&mut self) -> Option<&mut DynamicBlock> {
        self.active_block.as_ref().and_then(|name| self.blocks.get_mut(name))
    }

    /// 进入或退出块编辑模式；仅改变标志，不修改任何块定义。
    pub fn set_editing(&mut self, editing: bool) {
        self.is_editing = editing;
    }

    /// 是否处于块编辑模式。
    pub fn is_editing(&self) -> bool {
        self.is_editing
    }

    /// 设置是否显示夹点。
    pub fn set_show_grips(&mut self, show: bool) {
        self.show_grips = show;
    }

    /// 是否显示夹点。
    pub fn show_grips(&self) -> bool {
        self.show_grips
    }

    /// 设置是否显示参数面板。
    pub fn set_show_palette(&mut self, show: bool) {
        self.show_palette = show;
    }

    /// 是否显示参数面板。
    pub fn show_palette(&self) -> bool {
        self.show_palette
    }

    /// 清空所有块，并把活动块与编辑标志复位（不影响夹点/面板显示开关）。
    pub fn clear(&mut self) {
        self.blocks.clear();
        self.active_block = None;
        self.is_editing = false;
    }

    /// 深拷贝一份块定义并以新名字注册。
    ///
    /// 返回是否成功：`source_name` 不存在时为 `false`；目标名已存在时会被覆盖。
    pub fn duplicate_block(&mut self, source_name: &str, new_name: &str) -> bool {
        if let Some(source_block) = self.blocks.get(source_name) {
            let mut new_block = source_block.clone();
            new_block.name = new_name.to_string();
            self.blocks.insert(new_name.to_string(), new_block);
            true
        } else {
            false
        }
    }

    /// 把动态块转为静态块：置 `is_dynamic` 为 `false`，并清空参数、动作、参数集与夹点。
    ///
    /// 返回是否成功；块不存在时为 `false`。副作用：实体的可见性标志与变换保持不变。
    pub fn convert_to_static(&mut self, block_name: &str) -> bool {
        if let Some(block) = self.blocks.get_mut(block_name) {
            block.is_dynamic = false;
            block.parameters.clear();
            block.actions.clear();
            block.parameter_sets.clear();
            block.grips.clear();
            true
        } else {
            false
        }
    }

    /// 统计指定块中各类元素的数量。
    ///
    /// 返回统计快照；块不存在时为 `None`。
    pub fn get_statistics(&self, block_name: &str) -> Option<BlockStatistics> {
        self.blocks.get(block_name).map(|block| BlockStatistics {
            parameter_count: block.parameters.len(),
            action_count: block.actions.len(),
            visibility_state_count: block.visibility_states.len(),
            entity_count: block.entities.len(),
            grip_count: block.grips.len(),
            is_dynamic: block.is_dynamic,
        })
    }
}

/// 块统计快照：某一时刻块内各类元素的数量，仅用于展示，不反映后续变化。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockStatistics {
    /// 参数个数。
    pub parameter_count: usize,
    /// 动作个数。
    pub action_count: usize,
    /// 可见性状态个数。
    pub visibility_state_count: usize,
    /// 块内实体个数。
    pub entity_count: usize,
    /// 夹点个数。
    pub grip_count: usize,
    /// 该块是否为动态块。
    pub is_dynamic: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometry::Point;

    #[test]
    fn test_parameter_creation() {
        let param = BlockParameter::new()
            .with_name("Length")
            .with_type(ParameterType::Distance)
            .with_value(100.0)
            .with_range(0.0, 1000.0);

        assert_eq!(param.name, "Length");
        assert_eq!(param.parameter_type, ParameterType::Distance);
        assert!((param.value - 100.0).abs() < 1e-10);
    }

    #[test]
    fn test_parameter_range_validation() {
        let mut param = BlockParameter::new()
            .with_name("Length")
            .with_range(0.0, 100.0);

        assert!(param.set_value(50.0));
        assert!(!param.set_value(150.0));
        assert!(!param.set_value(-10.0));
    }

    #[test]
    fn test_action_creation() {
        let action = BlockAction::new()
            .with_type(ActionType::Move)
            .with_name("Move Action")
            .with_parameter("Length");

        assert_eq!(action.action_type, ActionType::Move);
        assert_eq!(action.name, "Move Action");
    }

    #[test]
    fn test_dynamic_block_creation() {
        let mut block = DynamicBlock::new().with_name("Door");

        let param = BlockParameter::new()
            .with_name("Width")
            .with_type(ParameterType::Linear)
            .with_value(900.0);

        block.add_parameter(param);

        assert_eq!(block.parameters.len(), 1);
        assert!(block.get_parameter("Width").is_some());
    }

    #[test]
    fn test_dynamic_block_parameter_value() {
        let mut block = DynamicBlock::new().with_name("Door");
        block.add_parameter(
            BlockParameter::new()
                .with_name("Width")
                .with_type(ParameterType::Linear)
                .with_range(600.0, 1200.0)
        );

        assert!(block.set_parameter_value("Width", 1000.0));
        assert!(!block.set_parameter_value("Width", 1500.0));
    }

    #[test]
    fn test_visibility_state() {
        let mut block = DynamicBlock::new().with_name("Door");

        let mut state = VisibilitySetting::new().with_name("Open");
        state.set_entity_visible("door_panel", false);
        state.set_entity_visible("hinge", true);

        block.add_visibility_state(state);

        assert_eq!(block.visibility_states.len(), 1);
    }

    #[test]
    fn test_grip_points() {
        let grip = GripPoint::for_parameter(
            "Width",
            1000.0,
            Point::new(100.0, 50.0, 0.0),
        );

        assert_eq!(grip.parameter_name, "Width");
        assert!((grip.parameter_value - 1000.0).abs() < 1e-10);
    }

    #[test]
    fn test_connection_point() {
        let point = ConnectionPoint::at_position(Point::new(100.0, 50.0, 0.0));
        assert!((point.position.x - 100.0).abs() < 1e-10);

        let base_point = ConnectionPoint::on_entity("line1", Point::new(0.0, 0.0, 0.0)).as_base();
        assert!(base_point.is_base);
    }

    #[test]
    fn test_expression() {
        let mut expr = Expression::new().with_expression("PI/2");
        assert!((expr.value - std::f64::consts::PI / 2.0).abs() < 1e-10);
    }

    #[test]
    fn test_lookup_row() {
        let row = LookupRow::new()
            .with_values(&[0.0, 50.0, 100.0], 25.0);

        assert_eq!(row.input_values.len(), 3);
        assert!((row.output_value - 25.0).abs() < 1e-10);
    }

    #[test]
    fn test_parameter_set() {
        let mut set = ParameterSet::new().with_name("Linear Set");
        set.add_parameter("Width");
        set.add_parameter("Height");
        set.add_action("move_action");

        assert_eq!(set.parameters.len(), 2);
        assert_eq!(set.actions.len(), 1);
    }

    #[test]
    fn test_dynamic_block_manager() {
        let mut manager = DynamicBlockManager::new();
        manager.create_block("Door");

        assert_eq!(manager.block_count(), 1);
        assert!(manager.get_block("Door").is_some());
    }

    #[test]
    fn test_dynamic_block_duplicate() {
        let mut manager = DynamicBlockManager::new();
        manager.create_block("Door");

        assert!(manager.duplicate_block("Door", "Door Copy"));
        assert_eq!(manager.block_count(), 2);
        assert!(manager.get_block("Door Copy").is_some());
    }

    #[test]
    fn test_dynamic_block_convert_to_static() {
        let mut manager = DynamicBlockManager::new();
        manager.create_block("Door");

        assert!(manager.convert_to_static("Door"));
        let block = manager.get_block("Door").unwrap();
        assert!(!block.is_dynamic);
        assert!(block.parameters.is_empty());
    }

    #[test]
    fn test_block_unit_conversion() {
        assert!((BlockUnit::Millimeters.conversion_factor() - 1.0).abs() < 1e-10);
        assert!((BlockUnit::Centimeters.conversion_factor() - 10.0).abs() < 1e-10);
        assert!((BlockUnit::Inches.conversion_factor() - 25.4).abs() < 1e-10);
    }

    #[test]
    fn test_parameter_type_names() {
        assert_eq!(ParameterType::Point.name(), "Point");
        assert_eq!(ParameterType::Linear.name(), "Linear");
        assert_eq!(ParameterType::Angle.name(), "Angle");
        assert_eq!(ParameterType::Visibility.name(), "Visibility");
    }

    #[test]
    fn test_action_type_names() {
        assert_eq!(ActionType::Move.name(), "Move");
        assert_eq!(ActionType::Rotate.name(), "Rotate");
        assert_eq!(ActionType::Scale.name(), "Scale");
        assert_eq!(ActionType::Stretch.name(), "Stretch");
    }

    #[test]
    fn test_dynamic_block_entity() {
        let entity = DynamicBlockEntity::new()
            .with_type("LINE")
            .at_position(Point::new(100.0, 100.0, 0.0));

        assert_eq!(entity.entity_type, "LINE");
        assert!((entity.position.x - 100.0).abs() < 1e-10);
    }

    #[test]
    fn test_transformation() {
        let transform = Transformation::new()
            .with_type(TransformationType::Rotation)
            .with_param("angle", 45.0);

        assert_eq!(transform.transformation_type, TransformationType::Rotation);
        assert!((transform.parameters["angle"] - 45.0).abs() < 1e-10);
    }

    #[test]
    fn test_block_statistics() {
        let mut manager = DynamicBlockManager::new();
        manager.create_block("Door");
        let block = manager.get_block_mut("Door").unwrap();

        block.add_parameter(BlockParameter::new().with_name("Width"));
        block.add_parameter(BlockParameter::new().with_name("Height"));
        block.add_action(BlockAction::new().with_name("Move"));

        let stats = manager.get_statistics("Door").unwrap();
        assert_eq!(stats.parameter_count, 2);
        assert_eq!(stats.action_count, 1);
    }

    #[test]
    fn test_visibility_setting() {
        let mut setting = VisibilitySetting::new().with_name("State1");
        setting.set_entity_visible("entity1", false);
        setting.set_entity_visible("entity2", true);

        assert!(!setting.is_entity_visible("entity1"));
        assert!(setting.is_entity_visible("entity2"));
    }

    #[test]
    fn test_grip_hover() {
        let mut grip = GripPoint::new();
        grip.set_hover(true);
        assert!(grip.is_hovered);
    }
}
