//! 多重引线（Multileader）：由引线折线、箭头与内容（多行文字或块）组成的注释对象。
//!
//! - `LeaderLine`：一条引线的顶点序列，并缓存末段方向角，供分支与狗腿线计算使用；
//! - `Multileader`：引线集合 + 箭头集合 + 内容，附带狗腿长度、分支夹角、基线间隙等排版参数；
//! - `MultileaderStyle`：命名样式模板；`MultileaderManager` 按样式创建对象并按 `ObjectId` 检索。
//!
//! 所有点均为世界坐标；角度一律为弧度（默认分支夹角 90° 在代码中写作 `90.0_f64.to_radians()`）。
//! 管理器只维护内存中的对象表与样式表，不读写磁盘。

use serde::{Serialize, Deserialize};
use std::fmt;

/// 引线的几何类型。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LeaderType {
    /// 直线段引线：相邻顶点之间用直线连接。
    Straight,
    /// 样条曲线引线：顶点作为样条控制点。
    Spline,
}

impl Default for LeaderType {
    fn default() -> Self {
        LeaderType::Straight
    }
}

/// 内容相对引线末端的附着位置。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LeaderAttachmentType {
    /// 内容顶部与引线末端对齐。
    AttachmentTop,
    /// 内容垂直中部与引线末端对齐。
    AttachmentMiddle,
    /// 内容底部与引线末端对齐。
    AttachmentBottom,
}

impl Default for LeaderAttachmentType {
    fn default() -> Self {
        LeaderAttachmentType::AttachmentTop
    }
}

/// 引线分支方向标记。
///
/// 仅作数据保存：`Multileader::rebuild_leaders` 目前不读取该值，狗腿线恒按
/// `branch_angle / 2`（弧度）偏移生成。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LeaderBranchType {
    /// 向左分支。
    Left,
    /// 向右分支。
    Right,
}

impl Default for LeaderBranchType {
    fn default() -> Self {
        LeaderBranchType::Left
    }
}

/// 一条引线：按顺序排列的顶点折线及缓存的末段方向角。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LeaderLine {
    /// 引线顶点，按绘制顺序排列，世界坐标；至少 2 个点才能定义方向与长度。
    pub points: Vec<crate::geometry::Point>,
    /// 引线几何类型（直线或样条）。
    pub leader_type: LeaderType,
    /// 分支方向标记。
    pub branch_type: LeaderBranchType,
    /// 末段方向角，弧度，由最后两个顶点按 `atan2(dy, dx)` 求得；顶点不足 2 个时为 0.0。
    pub last_segment_angle: f64,
}

impl Default for LeaderLine {
    fn default() -> Self {
        Self::new()
    }
}

impl LeaderLine {
    /// 创建一条空引线（无顶点）。
    ///
    /// 类型为直线、分支向左、末段方向角 0.0。
    #[inline]
    pub fn new() -> Self {
        Self {
            points: Vec::new(),
            leader_type: LeaderType::Straight,
            branch_type: LeaderBranchType::Left,
            last_segment_angle: 0.0,
        }
    }

    /// 由顶点序列构造引线，并立即推算末段方向角。
    ///
    /// - `points`：世界坐标顶点，按顺序排列，会被复制进引线。
    ///
    /// 返回的引线类型为直线、分支向左；`points` 少于 2 个点时 `last_segment_angle`
    /// 为 0.0（其余字段取默认值）。
    #[inline]
    pub fn with_points(points: &[crate::geometry::Point]) -> Self {
        let last_segment_angle = if points.len() >= 2 {
            let dx = points[points.len() - 1].x - points[points.len() - 2].x;
            let dy = points[points.len() - 1].y - points[points.len() - 2].y;
            dy.atan2(dx)
        } else {
            0.0
        };

        Self {
            points: points.to_vec(),
            leader_type: LeaderType::Straight,
            branch_type: LeaderBranchType::Left,
            last_segment_angle,
        }
    }

    /// 在引线末端追加一个顶点，并刷新末段方向角。
    ///
    /// - `point`：世界坐标顶点。
    ///
    /// 只有当追加前已有至少 2 个顶点时，才用「新点 − 倒数第二点」的 `atan2` 结果
    /// （弧度）更新 `last_segment_angle`。原地修改 `self`，无返回值。
    #[inline]
    pub fn add_point(&mut self, point: crate::geometry::Point) {
        if self.points.len() >= 2 {
            let last_idx = self.points.len() - 1;
            let dx = point.x - self.points[last_idx].x;
            let dy = point.y - self.points[last_idx].y;
            self.last_segment_angle = dy.atan2(dx);
        }
        self.points.push(point);
    }

    /// 返回引线顶点数量（含首末两端）。
    #[inline]
    pub fn point_count(&self) -> usize {
        self.points.len()
    }

    /// 引线是否没有任何顶点。
    ///
    /// 只有 1 个顶点时仍返回 `false`。
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.points.is_empty()
    }

    /// 返回折线总长，即相邻顶点欧氏距离之和，单位为世界坐标单位。
    ///
    /// 顶点少于 2 个时返回 0.0。
    #[inline]
    pub fn length(&self) -> f64 {
        if self.points.len() < 2 {
            return 0.0;
        }
        let mut total = 0.0;
        for i in 1..self.points.len() {
            total += self.points[i].distance_to(&self.points[i - 1]);
        }
        total
    }

    /// 返回首个顶点（引线起点）的引用；引线为空时返回 `None`。
    #[inline]
    pub fn start_point(&self) -> Option<&crate::geometry::Point> {
        self.points.first()
    }

    /// 返回最后一个顶点（引线末端，通常指向被标注对象）的引用；引线为空时返回 `None`。
    #[inline]
    pub fn end_point(&self) -> Option<&crate::geometry::Point> {
        self.points.last()
    }
}

/// 引线端部的箭头符号。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Arrowhead {
    /// 箭头位置，世界坐标。
    pub position: crate::geometry::Point,
    /// 箭头长度，世界坐标单位；默认 2.5。
    pub size: f64,
    /// 箭头形状。
    pub arrowhead_type: ArrowheadType,
    /// 箭头朝向角，弧度；默认 0.0，即指向 +X 方向。
    pub angle: f64,
}

impl Default for Arrowhead {
    fn default() -> Self {
        Self::new()
    }
}

impl Arrowhead {
    /// 创建默认箭头：位于原点、长度 2.5、闭合填充形、朝向角 0.0。
    #[inline]
    pub fn new() -> Self {
        Self {
            position: crate::geometry::Point::origin(),
            size: 2.5,
            arrowhead_type: ArrowheadType::ArrowClosedFilled,
            angle: 0.0,
        }
    }

    /// 设置箭头位置（世界坐标），链式返回修改后的自身。
    #[inline]
    pub fn with_position(mut self, position: crate::geometry::Point) -> Self {
        self.position = position;
        self
    }

    /// 设置箭头长度。
    ///
    /// - `size`：世界坐标单位，负值或 0 不做校验，由调用方保证合理。
    #[inline]
    pub fn with_size(mut self, size: f64) -> Self {
        self.size = size;
        self
    }

    /// 设置箭头形状，链式返回修改后的自身。
    #[inline]
    pub fn with_type(mut self, arrow_type: ArrowheadType) -> Self {
        self.arrowhead_type = arrow_type;
        self
    }

    /// 设置箭头朝向角，原地修改 `self`。
    ///
    /// - `angle`：弧度。
    #[inline]
    pub fn set_angle(&mut self, angle: f64) {
        self.angle = angle;
    }
}

/// 箭头形状。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ArrowheadType {
    /// 无箭头。
    None,
    /// 小圆点。
    DotSmall,
    /// 圆点。
    Dot,
    /// 开口箭头。
    ArrowOpen,
    /// 90° 开口箭头。
    ArrowOpen90,
    /// 45° 开口箭头。
    ArrowOpen45,
    /// 闭合（未填充）箭头。
    ArrowClosed,
    /// 闭合填充箭头，默认值。
    ArrowClosedFilled,
    /// 短斜线标记。
    Tick,
    /// 双倍长度斜线标记。
    Tick2x,
    /// 三角形。
    Triangle,
    /// 90° 三角形。
    Triangle90,
    /// 45° 三角形。
    Triangle45,
    /// 正方形。
    Square,
    /// 六边形。
    Hexagon,
    /// 空心圆。
    Circle,
    /// 实心圆。
    CircleFilled,
    /// 空心点。
    DotBlank,
    /// 原点标记。
    Origin,
    /// 第二种原点标记。
    Origin2,
    /// 无填充标记。
    NoneFilled,
    /// 用户自定义箭头。
    User,
}

impl Default for ArrowheadType {
    fn default() -> Self {
        ArrowheadType::ArrowClosedFilled
    }
}

impl ArrowheadType {
    /// 返回箭头形状的英文显示名，如 `"Closed Filled"`、`"Open 90"`。
    ///
    /// 仅供界面与导出文本使用，不参与几何计算。
    #[inline]
    pub fn name(&self) -> &str {
        match self {
            ArrowheadType::None => "None",
            ArrowheadType::DotSmall => "Dot Small",
            ArrowheadType::Dot => "Dot",
            ArrowheadType::ArrowOpen => "Open",
            ArrowheadType::ArrowOpen90 => "Open 90",
            ArrowheadType::ArrowOpen45 => "Open 45",
            ArrowheadType::ArrowClosed => "Closed",
            ArrowheadType::ArrowClosedFilled => "Closed Filled",
            ArrowheadType::Tick => "Tick",
            ArrowheadType::Tick2x => "Tick 2x",
            ArrowheadType::Triangle => "Triangle",
            ArrowheadType::Triangle90 => "Triangle 90",
            ArrowheadType::Triangle45 => "Triangle 45",
            ArrowheadType::Square => "Square",
            ArrowheadType::Hexagon => "Hexagon",
            ArrowheadType::Circle => "Circle",
            ArrowheadType::CircleFilled => "Circle Filled",
            ArrowheadType::DotBlank => "Dot Blank",
            ArrowheadType::Origin => "Origin",
            ArrowheadType::Origin2 => "Origin 2",
            ArrowheadType::NoneFilled => "None Filled",
            ArrowheadType::User => "User Defined",
        }
    }
}

/// 多重引线的多行文字内容。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MTextContent {
    /// 文字内容，可含 `\n` 表示换行；本模块只存储字符串，不做换行排版。
    pub text: String,
    /// 排版宽度，世界坐标单位；0.0 表示不限制宽度。
    pub width: f64,
    /// 文字样式。
    pub text_style: crate::text::TextStyle,
    /// 文字水平对齐方式。
    pub alignment: crate::text::TextAlignment,
    /// 行距系数，1.0 表示单倍行距。
    pub line_spacing: f64,
}

impl Default for MTextContent {
    fn default() -> Self {
        Self {
            text: String::new(),
            width: 0.0,
            text_style: crate::text::TextStyle::default(),
            alignment: crate::text::TextAlignment::Left,
            line_spacing: 1.0,
        }
    }
}

impl MTextContent {
    /// 用给定文字构造内容，其余字段取默认值（宽度 0.0、默认文字样式、左对齐、行距 1.0）。
    #[inline]
    pub fn new(text: &str) -> Self {
        Self {
            text: text.to_string(),
            ..Default::default()
        }
    }

    /// 设置排版宽度，链式返回修改后的自身。
    ///
    /// - `width`：世界坐标单位，0.0 表示不限制宽度。
    #[inline]
    pub fn with_width(mut self, width: f64) -> Self {
        self.width = width;
        self
    }

    /// 整体替换文字内容，原地修改 `self`。
    #[inline]
    pub fn set_text(&mut self, text: &str) {
        self.text = text.to_string();
    }

    /// 在现有文字末尾追加内容，不自动插入换行符。
    #[inline]
    pub fn append_text(&mut self, text: &str) {
        self.text.push_str(text);
    }

    /// 文字内容是否为空串（宽度、样式等其它字段不影响结果）。
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.text.is_empty()
    }
}

/// 多重引线的块内容（块参照）。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BlockContent {
    /// 块名，需与文档中的块定义一致；空串表示尚未指定。
    pub block_name: String,
    /// 块插入点，世界坐标。
    pub position: crate::geometry::Point,
    /// (x, y) 方向缩放因子，1.0 表示原始尺寸。
    pub scale: (f64, f64),
    /// 旋转角，弧度。
    pub rotation: f64,
    /// 属性标签与属性值的有序键值对，同一标签至多出现一次。
    pub attribute_values: Vec<(String, String)>,
}

impl Default for BlockContent {
    fn default() -> Self {
        Self::new()
    }
}

impl BlockContent {
    /// 创建空块内容：块名为空串、插入点为原点、缩放 (1.0, 1.0)、旋转角 0.0、无属性。
    #[inline]
    pub fn new() -> Self {
        Self {
            block_name: String::new(),
            position: crate::geometry::Point::origin(),
            scale: (1.0, 1.0),
            rotation: 0.0,
            attribute_values: Vec::new(),
        }
    }

    /// 设置块名，链式返回修改后的自身。
    #[inline]
    pub fn with_block(mut self, block_name: &str) -> Self {
        self.block_name = block_name.to_string();
        self
    }

    /// 写入或覆盖一个块属性。
    ///
    /// - `tag`：属性标签；已存在同名标签时覆盖其值，否则追加到末尾。
    /// - `value`：属性值，原样保存不做校验。
    ///
    /// 原地修改 `self`，不改变已有条目的相对顺序。
    #[inline]
    pub fn set_attribute(&mut self, tag: &str, value: &str) {
        if let Some((_, val)) = self.attribute_values.iter_mut().find(|(t, _)| t == tag) {
            *val = value.to_string();
        } else {
            self.attribute_values.push((tag.to_string(), value.to_string()));
        }
    }

    /// 按标签读取属性值。
    ///
    /// 返回首次匹配标签的值切片；标签不存在时返回 `None`。
    #[inline]
    pub fn get_attribute(&self, tag: &str) -> Option<&str> {
        self.attribute_values.iter()
            .find(|(t, _)| t == tag)
            .map(|(_, v)| v.as_str())
    }
}

/// 多重引线的内容：多行文字或块，二者互斥。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum MultileaderContent {
    /// 多行文字内容。
    MText(MTextContent),
    /// 块参照内容。
    Block(BlockContent),
}

impl Default for MultileaderContent {
    fn default() -> Self {
        MultileaderContent::MText(MTextContent::default())
    }
}

impl MultileaderContent {
    /// 用一段文字构造多行文字内容。
    #[inline]
    pub fn mtext(text: &str) -> Self {
        MultileaderContent::MText(MTextContent::new(text))
    }

    /// 用块名构造块内容，其余字段取默认值（原点插入、缩放 1、无旋转）。
    #[inline]
    pub fn block(block_name: &str) -> Self {
        MultileaderContent::Block(BlockContent::new().with_block(block_name))
    }

    /// 内容是否为多行文字。
    #[inline]
    pub fn is_mtext(&self) -> bool {
        matches!(self, MultileaderContent::MText(_))
    }

    /// 内容是否为块参照。
    #[inline]
    pub fn is_block(&self) -> bool {
        matches!(self, MultileaderContent::Block(_))
    }
}

/// 多重引线注释对象：引线 + 箭头 + 内容，并携带排版参数。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Multileader {
    /// 实体唯一标识；`MultileaderManager` 以此为键索引对象。
    pub object_id: super::data_structure::ObjectId,
    /// 引用的多重引线样式名，默认 `"Standard"`；本结构不校验样式是否存在。
    pub style_name: String,
    /// 引线集合，通常首条为主引线；为空时对象没有可见引线几何。
    pub leaders: Vec<LeaderLine>,
    /// 箭头集合，与 `leaders` 按下标对应，本模块不校验两者数量一致。
    pub arrowheads: Vec<Arrowhead>,
    /// 内容（多行文字或块）。
    pub content: MultileaderContent,
    /// 内容相对引线末端的附着位置。
    pub content_attachment: LeaderAttachmentType,
    /// 最近一次添加的引线端点，`rebuild_leaders` 以它计算狗腿方向。
    pub last_leader_point: crate::geometry::Point,
    /// 狗腿线方向向量，默认 (1.0, 0.0, 0.0)；`rebuild_leaders` 目前不使用该字段。
    pub dogleg_vector: crate::geometry::Point,
    /// 狗腿线长度，世界坐标单位，默认 8.0。
    pub dogleg_length: f64,
    /// 分支夹角，弧度，默认 90°。
    pub branch_angle: f64,
    /// 是否已设置基线（landing）。
    pub is_landing_set: bool,
    /// 基线与文字之间的间隙，世界坐标单位，默认 2.0。
    pub landing_gap: f64,
    /// 是否为内容绘制边框。
    pub enable_frame_text: bool,
    /// 文字高度，世界坐标单位，默认 2.5。
    pub text_height: f64,
    /// 文字旋转角，弧度。
    pub text_rotation: f64,
}

impl Default for Multileader {
    fn default() -> Self {
        Self::new()
    }
}

impl Multileader {
    /// 创建默认多重引线。
    ///
    /// 样式名 `"Standard"`，无引线与箭头，内容为空的多行文字，狗腿长 8.0，
    /// 分支夹角 90°（弧度），`is_landing_set` 为 true，基线间隙 2.0，文字高 2.5。
    /// 每次调用都会生成新的 `object_id`。
    #[inline]
    pub fn new() -> Self {
        Self {
            object_id: super::data_structure::ObjectId::new(),
            style_name: "Standard".to_string(),
            leaders: Vec::new(),
            arrowheads: Vec::new(),
            content: MultileaderContent::MText(MTextContent::new("")),
            content_attachment: LeaderAttachmentType::AttachmentTop,
            last_leader_point: crate::geometry::Point::origin(),
            dogleg_vector: crate::geometry::Point::new(1.0, 0.0, 0.0),
            dogleg_length: 8.0,
            branch_angle: 90.0_f64.to_radians(),
            is_landing_set: true,
            landing_gap: 2.0,
            enable_frame_text: false,
            text_height: 2.5,
            text_rotation: 0.0,
        }
    }

    /// 替换内容（多行文字或块），链式返回修改后的自身。
    #[inline]
    pub fn with_content(mut self, content: MultileaderContent) -> Self {
        self.content = content;
        self
    }

    /// 追加一条引线到集合末尾。
    ///
    /// - `leader`：引线值，按移动语义存入；不校验顶点数量。
    #[inline]
    pub fn add_leader(&mut self, leader: LeaderLine) {
        self.leaders.push(leader);
    }

    /// 同时追加一条引线和一个箭头，两者不校验配对关系。
    ///
    /// - `leader`：引线会被克隆后存入 `leaders`，原值仍归调用方所有。
    /// - `arrowhead`：箭头按移动语义存入 `arrowheads`。
    #[inline]
    pub fn add_leader_with_arrowhead(&mut self, leader: LeaderLine, arrowhead: Arrowhead) {
        self.leaders.push(leader.clone());
        self.arrowheads.push(arrowhead);
    }

    /// 返回引线数量。
    #[inline]
    pub fn leader_count(&self) -> usize {
        self.leaders.len()
    }

    /// 是否没有任何引线；有内容但没有引线时仍返回 `true`。
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.leaders.is_empty()
    }

    /// 设置样式名，原地修改 `self`；不校验该样式是否已注册。
    #[inline]
    pub fn set_style(&mut self, style_name: &str) {
        self.style_name = style_name.to_string();
    }

    /// 替换多行文字内容，原地修改 `self`。
    ///
    /// 仅当 `content` 为多行文字时生效；内容是块参照时静默忽略，不报错也不改类型。
    #[inline]
    pub fn set_text(&mut self, text: &str) {
        if let MultileaderContent::MText(mtext) = &mut self.content {
            mtext.set_text(text);
        }
    }

    /// 读取多行文字内容。
    ///
    /// 内容为多行文字时返回其文字；内容为块参照时返回 `None`。
    #[inline]
    pub fn get_text(&self) -> Option<&str> {
        match &self.content {
            MultileaderContent::MText(mtext) => Some(&mtext.text),
            _ => None,
        }
    }

    /// 设置狗腿线长度，原地修改 `self`。
    ///
    /// - `length`：世界坐标单位，负值不做校验。
    #[inline]
    pub fn set_dogleg_length(&mut self, length: f64) {
        self.dogleg_length = length;
    }

    /// 设置分支夹角，原地修改 `self`。
    ///
    /// - `angle`：弧度；`rebuild_leaders` 取其一半作为狗腿偏移角。
    #[inline]
    pub fn set_branch_angle(&mut self, angle: f64) {
        self.branch_angle = angle;
    }

    /// 设置内容相对引线末端的附着位置。
    #[inline]
    pub fn set_content_attachment(&mut self, attachment: LeaderAttachmentType) {
        self.content_attachment = attachment;
    }

    /// 按 `last_leader_point` 与各引线末顶点重建狗腿（landing）线段。
    ///
    /// 对每条引线取末顶点，按 `atan2(dy, dx) + branch_angle / 2`（弧度）求偏移方向，
    /// 再沿该方向前进 `dogleg_length` 得到新顶点：首条引线插在末顶点之前，
    /// 其余引线插在下标 1 处。原地修改 `self.leaders`；引线为空、或某条引线无顶点时跳过。
    /// 重复调用会不断插入新的狗腿顶点。
    #[inline]
    pub fn rebuild_leaders(&mut self) {
        for (idx, leader) in self.leaders.iter_mut().enumerate() {
            if let Some(end_point) = leader.points.last() {
                let dx = end_point.x - self.last_leader_point.x;
                let dy = end_point.y - self.last_leader_point.y;
                let base_angle = dy.atan2(dx);

                let dogleg_angle = base_angle + self.branch_angle / 2.0;

                let new_point = crate::geometry::Point::new(
                    end_point.x + dogleg_angle.cos() * self.dogleg_length,
                    end_point.y + dogleg_angle.sin() * self.dogleg_length,
                    end_point.z,
                );

                if idx == 0 {
                    leader.points.insert(leader.points.len() - 1, new_point);
                } else {
                    leader.points.insert(1, new_point);
                }
            }
        }
    }
}

impl fmt::Display for Multileader {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "MLEADER: {} leaders, {} arrowheads", self.leaders.len(), self.arrowheads.len())
    }
}

/// 多重引线的链式构造器，最终通过 `build` 产出对象。
#[derive(Debug, Clone)]
pub struct MultileaderBuilder {
    multileader: Multileader,
}

impl Default for MultileaderBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl MultileaderBuilder {
    /// 创建持有默认多重引线的构造器。
    #[inline]
    pub fn new() -> Self {
        Self {
            multileader: Multileader::new(),
        }
    }

    /// 设置样式名，链式返回 `self`。
    #[inline]
    pub fn style(mut self, style_name: &str) -> Self {
        self.multileader.set_style(style_name);
        self
    }

    /// 设置内容（多行文字或块），链式返回 `self`。
    #[inline]
    pub fn content(mut self, content: MultileaderContent) -> Self {
        self.multileader.content = content;
        self
    }

    /// 设置多行文字内容，链式返回 `self`；内容为块参照时不生效。
    #[inline]
    pub fn text(mut self, text: &str) -> Self {
        self.multileader.set_text(text);
        self
    }

    /// 把顶点追加到当前（最后一条）引线，并把 `last_leader_point` 更新为该点。
    ///
    /// - `point`：世界坐标顶点。
    ///
    /// 尚无引线时自动新建一条直线引线再加入该点。原地修改 `self`，无返回值。
    #[inline]
    pub fn add_leader_point(&mut self, point: crate::geometry::Point) {
        if let Some(last_leader) = self.multileader.leaders.last_mut() {
            last_leader.add_point(point);
        } else {
            let mut leader = LeaderLine::new();
            leader.add_point(point);
            self.multileader.leaders.push(leader);
        }
        self.multileader.last_leader_point = point;
    }

    /// 开始一条新的空引线，作为后续 `add_leader_point` 的目标。
    ///
    /// - `point`：仅写入 `last_leader_point`，不会成为引线顶点；随后需用
    ///   `add_leader_point` 显式添加顶点。
    #[inline]
    pub fn start_leader(&mut self, point: crate::geometry::Point) {
        let leader = LeaderLine::new();
        self.multileader.leaders.push(leader);
        self.multileader.last_leader_point = point;
    }

    /// 追加一个箭头，不自动与引线配对。
    #[inline]
    pub fn add_arrowhead(&mut self, arrowhead: Arrowhead) {
        self.multileader.arrowheads.push(arrowhead);
    }

    /// 设置狗腿线长度，链式返回 `self`。
    ///
    /// - `length`：世界坐标单位。
    #[inline]
    pub fn set_dogleg_length(mut self, length: f64) -> Self {
        self.multileader.set_dogleg_length(length);
        self
    }

    /// 设置分支夹角，链式返回 `self`。
    ///
    /// - `angle`：弧度。
    #[inline]
    pub fn set_branch_angle(mut self, angle: f64) -> Self {
        self.multileader.set_branch_angle(angle);
        self
    }

    /// 设置内容附着位置，链式返回 `self`。
    #[inline]
    pub fn set_content_attachment(mut self, attachment: LeaderAttachmentType) -> Self {
        self.multileader.set_content_attachment(attachment);
        self
    }

    /// 结束构造，返回内部的多重引线对象（构造器被消费）。
    #[inline]
    pub fn build(self) -> Multileader {
        self.multileader
    }

    /// 借出内部对象的可变引用，便于在构造过程中调用 `rebuild_leaders` 等方法。
    #[inline]
    pub fn build_mut(&mut self) -> &mut Multileader {
        &mut self.multileader
    }
}

/// 多重引线样式：一组可复用的排版与外观参数模板。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MultileaderStyle {
    /// 样式名，管理器以此索引；空名无法注册。
    pub name: String,
    /// 样式说明，仅用于界面展示。
    pub description: String,
    /// 箭头形状。
    pub arrowhead_type: ArrowheadType,
    /// 箭头长度，世界坐标单位，默认 2.5。
    pub arrowhead_size: f64,
    /// 引线几何类型。
    pub leader_type: LeaderType,
    /// 内容类型（多行文字/块/复制内容）。
    pub content_type: ContentType,
    /// 文字样式。
    pub text_style: crate::text::TextStyle,
    /// 文字高度，世界坐标单位，默认 2.5。
    pub text_height: f64,
    /// 文字旋转角，弧度。
    pub text_rotation: f64,
    /// 文字水平对齐方式。
    pub text_alignment: crate::text::TextAlignment,
    /// 文字相对引线的附着位置。
    pub text_attachment: TextAttachment,
    /// 是否为文字绘制边框。
    pub text_frame_enabled: bool,
    /// 基线与文字之间的间隙，世界坐标单位，默认 2.0。
    pub landing_gap: f64,
    /// 狗腿线长度，世界坐标单位，默认 8.0。
    pub dogleg_length: f64,
    /// 分支夹角，弧度，默认 90°。
    pub branch_angle: f64,
    /// 是否为内容绘制边框的另一个开关，与 `text_frame_enabled` 并存。
    pub enable_frame_text: bool,
    /// 整体缩放系数，1.0 表示不缩放。
    pub scale_factor: f64,
}

/// 多重引线内容类型。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ContentType {
    /// 多行文字。
    MText,
    /// 块参照。
    Block,
    /// 复制已有引线的内容。
    CopyContent,
}

impl Default for ContentType {
    fn default() -> Self {
        ContentType::MText
    }
}

/// 文字相对引线的附着位置（比 `LeaderAttachmentType` 划分更细）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TextAttachment {
    /// 附着的顶部。
    TopOfTop,
    /// 附着顶部的中间。
    MiddleOfTop,
    /// 垂直居中。
    Middle,
    /// 附着顶部的底部。
    BottomOfTop,
    /// 附着的底部。
    BottomOfBottom,
    /// 附着底部的中间。
    MiddleOfBottom,
    /// 附着底部。
    AttachmentBottom,
}

impl Default for TextAttachment {
    fn default() -> Self {
        TextAttachment::TopOfTop
    }
}

impl Default for MultileaderStyle {
    fn default() -> Self {
        Self::new("Standard")
    }
}

impl MultileaderStyle {
    /// 按名称创建样式，其余参数取默认值。
    ///
    /// 默认：闭合填充箭头、箭头长 2.5、直线引线、多行文字内容、文字高 2.5、
    /// 文字旋转 0.0、左对齐、`TopOfTop` 附着、狗腿长 8.0、分支夹角 90°、
    /// 缩放系数 1.0、不启用文字边框。
    #[inline]
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            description: String::new(),
            arrowhead_type: ArrowheadType::ArrowClosedFilled,
            arrowhead_size: 2.5,
            leader_type: LeaderType::Straight,
            content_type: ContentType::MText,
            text_style: crate::text::TextStyle::default(),
            text_height: 2.5,
            text_rotation: 0.0,
            text_alignment: crate::text::TextAlignment::Left,
            text_attachment: TextAttachment::TopOfTop,
            text_frame_enabled: false,
            landing_gap: 2.0,
            dogleg_length: 8.0,
            branch_angle: 90.0_f64.to_radians(),
            enable_frame_text: false,
            scale_factor: 1.0,
        }
    }

    /// 设置箭头形状，原地修改 `self`。
    #[inline]
    pub fn set_arrowhead_type(&mut self, arrow_type: ArrowheadType) {
        self.arrowhead_type = arrow_type;
    }

    /// 设置箭头长度，原地修改 `self`。
    ///
    /// - `size`：世界坐标单位。
    #[inline]
    pub fn set_arrowhead_size(&mut self, size: f64) {
        self.arrowhead_size = size;
    }

    /// 设置文字高度，原地修改 `self`。
    ///
    /// - `height`：世界坐标单位。
    #[inline]
    pub fn set_text_height(&mut self, height: f64) {
        self.text_height = height;
    }

    /// 设置狗腿线长度，原地修改 `self`。
    ///
    /// - `length`：世界坐标单位。
    #[inline]
    pub fn set_dogleg_length(&mut self, length: f64) {
        self.dogleg_length = length;
    }

    /// 设置分支夹角，原地修改 `self`。
    ///
    /// - `angle`：弧度。
    #[inline]
    pub fn set_branch_angle(&mut self, angle: f64) {
        self.branch_angle = angle;
    }

    /// 按样式创建一个多重引线。
    ///
    /// 返回的对象来自 `Multileader::new()`，仅把内容置为空的多行文字；
    /// **不会**套用本样式的箭头、文字高度、狗腿长度等参数。
    #[inline]
    pub fn create_multileader(&self) -> Multileader {
        Multileader::new()
            .with_content(MultileaderContent::MText(MTextContent::new("")))
    }
}

/// 多重引线管理器：维护对象表与样式表，并跟踪当前样式和激活对象。
///
/// 仅供内存使用，不负责序列化或写盘；对象以自身的 `object_id` 为键存放。
#[derive(Debug, Clone)]
pub struct MultileaderManager {
    multileaders: std::collections::HashMap<super::data_structure::ObjectId, Multileader>,
    styles: std::collections::HashMap<String, MultileaderStyle>,
    current_style: String,
    active_multileader: Option<super::data_structure::ObjectId>,
}

impl Default for MultileaderManager {
    fn default() -> Self {
        Self::new()
    }
}

impl MultileaderManager {
    /// 创建管理器并注册内置样式 `"Standard"` 与 `"StandardWithFrame"`。
    ///
    /// 初始当前样式为 `"Standard"`，对象表为空，无激活对象。
    #[inline]
    pub fn new() -> Self {
        let mut manager = Self {
            multileaders: std::collections::HashMap::new(),
            styles: std::collections::HashMap::new(),
            current_style: "Standard".to_string(),
            active_multileader: None,
        };
        manager.register_builtin_styles();
        manager
    }

    fn register_builtin_styles(&mut self) {
        self.styles.insert("Standard".to_string(), MultileaderStyle::new("Standard"));
        self.styles.insert("StandardWithFrame".to_string(), {
            let mut style = MultileaderStyle::new("StandardWithFrame");
            style.text_frame_enabled = true;
            style
        });
    }

    /// 登记一个多重引线，并返回它的 `object_id`。
    ///
    /// 以对象自身的 `object_id` 为键写入对象表；键相同则覆盖已有对象
    /// （旧对象被丢弃，可能造成引用该 id 的外部数据失效）。
    #[inline]
    pub fn add(&mut self, multileader: Multileader) -> super::data_structure::ObjectId {
        let object_id = multileader.object_id.clone();
        self.multileaders.insert(object_id.clone(), multileader);
        object_id
    }

    /// 按当前样式创建一个空多重引线并登记，返回其 `object_id`。
    ///
    /// 当前样式名在样式表中找不到时回退到 `MultileaderStyle::default()`（`"Standard"`）。
    #[inline]
    pub fn create(&mut self) -> super::data_structure::ObjectId {
        let style = self.styles.get(&self.current_style).cloned().unwrap_or_default();
        let multileader = style.create_multileader();
        self.add(multileader)
    }

    /// 按 id 读取多重引线的只读引用；不存在时返回 `None`。
    #[inline]
    pub fn get(&self, object_id: &super::data_structure::ObjectId) -> Option<&Multileader> {
        self.multileaders.get(object_id)
    }

    /// 按 id 读取多重引线的可变引用；不存在时返回 `None`。
    #[inline]
    pub fn get_mut(&mut self, object_id: &super::data_structure::ObjectId) -> Option<&mut Multileader> {
        self.multileaders.get_mut(object_id)
    }

    /// 移除指定对象。
    ///
    /// 返回 `true` 表示确实移除了对象，`false` 表示该 id 不存在；
    /// 若被移除的对象正是激活对象，`active` 系列方法随后会返回 `None`。
    #[inline]
    pub fn remove(&mut self, object_id: &super::data_structure::ObjectId) -> bool {
        self.multileaders.remove(object_id).is_some()
    }

    /// 返回已登记的多重引线数量（不含样式数量）。
    #[inline]
    pub fn count(&self) -> usize {
        self.multileaders.len()
    }

    /// 清空所有多重引线对象，但保留样式表与当前样式设置。
    #[inline]
    pub fn clear(&mut self) {
        self.multileaders.clear();
    }

    /// 切换当前样式。
    ///
    /// 返回 `true` 表示切换成功；样式名未注册时返回 `false`，且当前样式保持不变。
    #[inline]
    pub fn set_style(&mut self, style_name: &str) -> bool {
        if self.styles.contains_key(style_name) {
            self.current_style = style_name.to_string();
            true
        } else {
            false
        }
    }

    /// 返回当前样式名。
    #[inline]
    pub fn current_style(&self) -> &str {
        &self.current_style
    }

    /// 注册一个样式。
    ///
    /// 名称非空时写入样式表（同名样式被覆盖）并返回 `true`；名称为空串时不做任何事并返回 `false`。
    #[inline]
    pub fn add_style(&mut self, style: MultileaderStyle) -> bool {
        if style.name.is_empty() {
            return false;
        }
        self.styles.insert(style.name.clone(), style);
        true
    }

    /// 按名称读取样式；不存在时返回 `None`。
    #[inline]
    pub fn get_style(&self, name: &str) -> Option<&MultileaderStyle> {
        self.styles.get(name)
    }

    /// 返回所有已注册样式名，顺序不保证稳定（取决于哈希表遍历顺序）。
    #[inline]
    pub fn style_names(&self) -> Vec<&str> {
        self.styles.keys().map(|s| s.as_str()).collect()
    }

    /// 设置激活的多重引线；传 `None` 表示取消激活。
    ///
    /// 只记录 id，不校验对象是否存在。
    #[inline]
    pub fn set_active(&mut self, object_id: Option<super::data_structure::ObjectId>) {
        self.active_multileader = object_id;
    }

    /// 读取激活对象的只读引用；未设置激活对象或该对象已被移除时返回 `None`。
    #[inline]
    pub fn active(&self) -> Option<&Multileader> {
        self.active_multileader.as_ref().and_then(|id| self.get(id))
    }

    /// 读取激活对象的可变引用；未设置激活对象或该对象已被移除时返回 `None`。
    #[inline]
    pub fn active_mut(&mut self) -> Option<&mut Multileader> {
        let id = self.active_multileader.clone()?;
        self.get_mut(&id)
    }

    /// 向激活对象追加一个引线顶点（世界坐标），并更新其 `last_leader_point`。
    ///
    /// 返回 `true` 表示写入成功；无激活对象（或对象已失效）时返回 `false` 且不做任何修改。
    /// 激活对象尚无引线时会自动新建一条。
    #[inline]
    pub fn add_leader_to_active(&mut self, point: crate::geometry::Point) -> bool {
        if let Some(mleader) = self.active_mut() {
            if let Some(last_leader) = mleader.leaders.last_mut() {
                last_leader.add_point(point);
            } else {
                let mut leader = LeaderLine::new();
                leader.add_point(point);
                mleader.leaders.push(leader);
            }
            mleader.last_leader_point = point;
            true
        } else {
            false
        }
    }

    /// 向指定对象追加一个引线顶点（世界坐标），并更新其 `last_leader_point`。
    ///
    /// - `object_id`：目标对象 id。
    /// - `point`：世界坐标顶点。
    ///
    /// 返回 `true` 表示写入成功；id 不存在时返回 `false` 且不新建对象。
    /// 目标对象尚无引线时会自动新建一条。
    #[inline]
    pub fn add_leader_point(&mut self, object_id: &super::data_structure::ObjectId, point: crate::geometry::Point) -> bool {
        if let Some(mleader) = self.get_mut(object_id) {
            if let Some(last_leader) = mleader.leaders.last_mut() {
                last_leader.add_point(point);
            } else {
                let mut leader = LeaderLine::new();
                leader.add_point(point);
                mleader.leaders.push(leader);
            }
            mleader.last_leader_point = point;
            true
        } else {
            false
        }
    }

    /// 借出整个对象表（键为 `object_id`）的只读引用，便于遍历或统计。
    #[inline]
    pub fn all(&self) -> &std::collections::HashMap<super::data_structure::ObjectId, Multileader> {
        &self.multileaders
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometry::Point;

    #[test]
    fn test_leader_line() {
        let leader = LeaderLine::new();
        assert!(leader.is_empty());

        let points = vec![
            Point::new(0.0, 0.0, 0.0),
            Point::new(10.0, 0.0, 0.0),
            Point::new(10.0, 10.0, 0.0),
        ];
        let leader = LeaderLine::with_points(&points);
        assert_eq!(leader.point_count(), 3);
        assert!((leader.length() - 20.0).abs() < 1e-10);
    }

    #[test]
    fn test_leader_line_add_point() {
        let mut leader = LeaderLine::new();
        leader.add_point(Point::new(0.0, 0.0, 0.0));
        leader.add_point(Point::new(10.0, 0.0, 0.0));
        assert_eq!(leader.point_count(), 2);
        assert!((leader.length() - 10.0).abs() < 1e-10);
    }

    #[test]
    fn test_arrowhead() {
        let arrowhead = Arrowhead::new()
            .with_position(Point::new(100.0, 100.0, 0.0))
            .with_size(3.0)
            .with_type(ArrowheadType::ArrowClosedFilled);

        assert!((arrowhead.position.x - 100.0).abs() < 1e-10);
        assert_eq!(arrowhead.size, 3.0);
    }

    #[test]
    fn test_multileader() {
        let mut mleader = Multileader::new();
        let leader = LeaderLine::with_points(&[
            Point::new(0.0, 0.0, 0.0),
            Point::new(5.0, 0.0, 0.0),
            Point::new(10.0, 10.0, 0.0),
        ]);
        mleader.add_leader(leader);
        assert_eq!(mleader.leader_count(), 1);
        assert!(!mleader.is_empty());
    }

    #[test]
    fn test_multileader_builder() {
        let builder = MultileaderBuilder::new()
            .text("Test Label")
            .set_dogleg_length(10.0)
            .set_branch_angle(90.0_f64.to_radians());

        let mleader = builder.build();
        assert_eq!(mleader.get_text(), Some("Test Label"));
        assert!((mleader.dogleg_length - 10.0).abs() < 1e-10);
    }

    #[test]
    fn test_multileader_content() {
        assert!(MultileaderContent::mtext("Test").is_mtext());
        assert!(!MultileaderContent::mtext("Test").is_block());

        assert!(MultileaderContent::block("MyBlock").is_block());
        assert!(!MultileaderContent::block("MyBlock").is_mtext());
    }

    #[test]
    fn test_multileader_style() {
        let style = MultileaderStyle::new("TestStyle");
        assert_eq!(style.name, "TestStyle");
        assert_eq!(style.arrowhead_size, 2.5);
    }

    #[test]
    fn test_multileader_style_operations() {
        let mut style = MultileaderStyle::new("TestStyle");
        style.set_arrowhead_type(ArrowheadType::Dot);
        style.set_arrowhead_size(5.0);
        style.set_text_height(3.0);
        assert_eq!(style.arrowhead_type, ArrowheadType::Dot);
        assert!((style.arrowhead_size - 5.0).abs() < 1e-10);
        assert!((style.text_height - 3.0).abs() < 1e-10);
    }

    #[test]
    fn test_multileader_manager() {
        let manager = MultileaderManager::new();
        assert!(manager.count() == 0);
        assert!(manager.get_style("Standard").is_some());
    }

    #[test]
    fn test_multileader_manager_operations() {
        let mut manager = MultileaderManager::new();
        let object_id = manager.create();

        assert_eq!(manager.count(), 1);
        assert!(manager.get(&object_id).is_some());

        assert!(manager.remove(&object_id));
        assert_eq!(manager.count(), 0);
    }

    #[test]
    fn test_multileader_style_management() {
        let mut manager = MultileaderManager::new();
        let mut style = MultileaderStyle::new("Custom");
        style.description = "Custom multileader style".to_string();
        assert!(manager.add_style(style));
        assert!(manager.get_style("Custom").is_some());

        assert_eq!(manager.set_style("Custom"), true);
        assert_eq!(manager.current_style(), "Custom");
    }

    #[test]
    fn test_multileader_text_content() {
        let mut mleader = Multileader::new();
        mleader.set_text("Hello\nWorld");
        assert_eq!(mleader.get_text(), Some("Hello\nWorld"));

        mleader.set_text("Updated");
        assert_eq!(mleader.get_text(), Some("Updated"));
    }

    #[test]
    fn test_leader_types() {
        let straight_leader = LeaderLine::with_points(&[
            Point::new(0.0, 0.0, 0.0),
            Point::new(10.0, 0.0, 0.0),
        ]);
        assert_eq!(straight_leader.leader_type, LeaderType::Straight);
    }

    #[test]
    fn test_multileader_rebuild() {
        let mut mleader = Multileader::new();
        let leader = LeaderLine::with_points(&[
            Point::new(0.0, 0.0, 0.0),
            Point::new(10.0, 0.0, 0.0),
        ]);
        mleader.add_leader(leader);
        mleader.dogleg_length = 8.0;
        mleader.branch_angle = 90.0_f64.to_radians();
        mleader.last_leader_point = Point::new(10.0, 0.0, 0.0);

        mleader.rebuild_leaders();

        if let Some(leader) = mleader.leaders.first() {
            assert!(leader.point_count() >= 3);
        }
    }
}
