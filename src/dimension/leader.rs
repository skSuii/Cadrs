//! 引线（Leader）与多重引线（MultiLeader）：把文字、块或形位公差等注释连接到被标注对象。
//! `Leader` 由起点、落地段（landing）、内容、样式、可选注释与折弯（dogleg）组成；
//! `MultiLeader` 管理一组共享公共内容与样式的引线，并支持整体缩放与落地点对齐；
//! `LeaderAssociation` 记录引线与被标注对象的对应关系，供对象变化时更新箭头位置。
//! 引线经 `to_entity` 或 `From` 转成 `Entity`（Dimension 类型，线型固定为 Linear、测量值 0）后进入统一的渲染与导出链路。
//! 坐标为世界坐标；样式中的角度字段沿用 CAD 习惯以度为单位（`LeaderStyle::default` 取 0 与 90）。
use crate::geometry::{Point, Vector2, Line, Arc, Polyline};
use crate::data_structure::{Entity, EntityType, EntityGeometry, TextStyle};
use crate::geometric_tolerance::GeometricTolerance;
use serde::{Serialize, Deserialize};
use std::fmt;

/// 单条引线：从 `start_point` 指向被标注对象，另一端用落地段挂接注释。
/// 用 `new` 创建后再按需调用 `set_landing` / `set_annotation` / `flip_dogleg` 等就地修改的方法完善。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Leader {
    /// 引线起点（箭头端），通常指向被标注对象，世界坐标。
    pub start_point: Point,
    /// 落地段设置，决定注释的挂接位置与水平段长度。
    pub landing: LeaderLanding,
    /// 引线承载的内容：空 / 文字 / 块 / 形位公差。
    pub content: LeaderContent,
    /// 引线样式：箭头、线型、落地段与折弯尺寸等。
    pub style: LeaderStyle,
    /// 可选注释；`None` 时按 `content` 处理显示内容。
    pub annotation: Option<Annotation>,
    /// 是否属于多重引线；`new` 创建时为 false，供上层标记来源。
    pub is_mleader: bool,
    /// 折弯（dogleg）设置；`new` 按样式中的折弯参数初始化并默认启用。
    pub dogleg: DoglegSettings,
}

/// 引线落地段：注释内容之前的一段水平短段，决定文字的挂接位置。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LeaderLanding {
    /// 落地段端点，同时也是 `to_entity` 中文字位置，世界坐标。
    pub landing_point: Point,
    /// 落地段长度（世界单位）；`Default` 为 6.35。
    pub landing_length: f64,
    /// 是否绘制落地段；`remove_landing` 会把它置为 false 但保留其余字段。
    pub has_landing: bool,
}

/// 引线承载的内容类型。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum LeaderContent {
    /// 空内容：只画引线，不带注释。
    None,
    /// 文字内容，含文字样式与排版参数。
    Text(TextContent),
    /// 块（Block）引用，按块名与插入参数放置。
    Block(BlockContent),
    /// 形位公差标注；转换为实体时文字固定输出 "Tolerance"。
    Tolerance(GeometricTolerance),
}

/// 引线的文字内容及其排版参数。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TextContent {
    /// 文字正文；`Leader::add_segment` 会向其中追加换行与坐标文本。
    pub text: String,
    /// 文字样式（字体、字高、宽度因子等）。
    pub style: TextStyle,
    /// 文字宽度因子（相对字高的横向拉伸比例）；`Default` 为 1.0。
    pub width_factor: f64,
    /// 是否为多行文字（MText）；`Default` 为 false，仅作语义标记。
    pub is_mtext: bool,
}

/// 引线引用的块（Block）内容。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BlockContent {
    /// 块名；转换为实体时作为该实体的文字输出。
    pub block_name: String,
    /// 块插入比例，1.0 为原始大小。
    pub scale: f64,
    /// 块旋转角，按度给出。
    pub rotation: f64,
}

/// 引线样式：控制箭头、线型、落地段与折弯尺寸；`Default` 给出名为 "Standard" 的常用值。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LeaderStyle {
    /// 样式标识，用于与文档中的样式表对应。
    pub id: String,
    /// 样式名称；`Default` 为 "Standard"。
    pub name: String,
    /// 箭头尺寸（世界单位）；`Default` 为 2.5。
    pub arrow_size: f64,
    /// 箭头形态。
    pub arrow_style: LeaderArrowStyle,
    /// 引线线型（直线或样条）。
    pub leader_line_type: LeaderLineType,
    /// 引线线宽；0.0 表示使用默认线宽。
    pub leader_line_weight: f64,
    /// 落地段与文字之间的间隙；`Default` 为 0.625。
    pub landing_gap: f64,
    /// 落地段长度；`Default` 为 6.35。
    pub landing_distance: f64,
    /// 文字高度（世界单位）；`Default` 为 2.5，转换为实体时写入 `text_height`。
    pub text_height: f64,
    /// 文字样式。
    pub text_style: TextStyle,
    /// 引线第一段的角度（度）；`Default` 为 0.0。
    pub first_segment_angle: f64,
    /// 引线第二段的角度（度）；`Default` 为 90.0。
    pub second_segment_angle: f64,
    /// 折弯段长度（世界单位）；`Default` 为 6.35，`Leader::new` 会复制到 `dogleg.length`。
    pub dogleg_length: f64,
}

/// 引线箭头形态。
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum LeaderArrowStyle {
    /// 闭合空心箭头。
    Closed,
    /// 闭合实心箭头，`LeaderStyle::default` 使用该形态。
    ClosedFilled,
    /// 实心圆点。
    Dot,
    /// 小实心圆点，用于密集标注。
    DotSmall,
    /// 开口箭头（两条线段）。
    Open,
    /// 原点标记。
    Origin,
    /// 原点标记的另一种画法。
    Origin02,
    /// 斜线箭头（建筑标记）。
    Oblique,
    /// 空心方框。
    Box,
    /// 实心方框。
    BoxFilled,
    /// 空心圆。
    Circle,
    /// 实心圆。
    CircleFilled,
}

/// 引线线型。
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum LeaderLineType {
    /// 直线段引线，`LeaderStyle::default` 使用该线型。
    Straight,
    /// 样条曲线引线。
    Splined,
}

/// 折弯（dogleg）设置：注释文字下方的一段水平短横线。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DoglegSettings {
    /// 是否启用折弯；`Leader::new` 创建时为 true。
    pub enabled: bool,
    /// 折弯方向，`flip_dogleg` 可就地翻转。
    pub direction: DoglegDirection,
    /// 折弯段长度（世界单位），默认取样式的 `dogleg_length`。
    pub length: f64,
    /// 第一段角度（度），默认取样式的同名值。
    pub first_segment_angle: f64,
    /// 第二段角度（度），默认取样式的同名值。
    pub second_segment_angle: f64,
}

/// 折弯方向。
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum DoglegDirection {
    /// 折弯向左。
    Left,
    /// 折弯向右。
    Right,
    /// 按引线方向自动选择；`flip_dogleg` 会把它变为 `Left`。
    Automatic,
}

/// 引线注释：注释内容及其在图纸上的放置信息。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Annotation {
    /// 注释内容（文字 / 多行文字 / 块 / 公差）。
    pub content: AnnotationContent,
    /// 注释位置，世界坐标。
    pub position: Point,
    /// 注释相对 `position` 的挂接点。
    pub attachment_point: AnnotationAttachment,
    /// 文字方向（从左到右 / 从右到左）。
    pub text_direction: TextDirection,
    /// 注释所在直线的旋转角，按度给出。
    pub line_rotation: f64,
    /// 折弯位置，世界坐标。
    pub dogleg_position: Point,
}

/// 注释挂接点：`position` 上的九宫格对齐位置。
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum AnnotationAttachment {
    /// 左上角对齐。
    TopLeft,
    /// 上边中点对齐。
    TopCenter,
    /// 右上角对齐。
    TopRight,
    /// 左边中点对齐。
    MiddleLeft,
    /// 正中对齐。
    MiddleCenter,
    /// 右边中点对齐。
    MiddleRight,
    /// 左下角对齐。
    BottomLeft,
    /// 下边中点对齐。
    BottomCenter,
    /// 右下角对齐。
    BottomRight,
}

/// 注释内容的轻量表示。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AnnotationContent {
    /// 单行文字。
    Text(String),
    /// 多行文字（MText）。
    MText(String),
    /// 块引用：块名与插入比例。
    Block(String, f64),
    /// 形位公差注释，具体公差由引线自身内容承载。
    Tolerance,
}

/// 注释文字方向。
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum TextDirection {
    /// 从左向右排布。
    LeftToRight,
    /// 从右向左排布，用于翻转或镜像的标注。
    RightToLeft,
}

impl Default for LeaderStyle {
    fn default() -> Self {
        Self {
            id: "Standard".to_string(),
            name: "Standard".to_string(),
            arrow_size: 2.5,
            arrow_style: LeaderArrowStyle::ClosedFilled,
            leader_line_type: LeaderLineType::Straight,
            leader_line_weight: 0.0,
            landing_gap: 0.625,
            landing_distance: 6.35,
            text_height: 2.5,
            text_style: TextStyle::default(),
            first_segment_angle: 0.0,
            second_segment_angle: 90.0,
            dogleg_length: 6.35,
        }
    }
}

impl Default for LeaderLanding {
    fn default() -> Self {
        Self {
            landing_point: Point::origin(),
            landing_length: 6.35,
            has_landing: true,
        }
    }
}

impl Default for TextContent {
    fn default() -> Self {
        Self {
            text: String::new(),
            style: TextStyle::default(),
            width_factor: 1.0,
            is_mtext: false,
        }
    }
}

impl Leader {
    /// 创建一条引线。
    /// - `start_point`：引线起点（箭头端），一般指向被标注对象。
    /// - `content`：引线内容（文字 / 块 / 公差 / 空）。
    /// - `style`：引线样式；其 `dogleg_length`、`first_segment_angle`、`second_segment_angle` 会复制到 `dogleg`。
    /// 落地段取 `LeaderLanding::default()`（`has_landing` 为 true），折弯默认启用且方向为 `Automatic`，
    /// `annotation` 为 `None`、`is_mleader` 为 false；不修改入参。
    pub fn new(start_point: Point, content: LeaderContent, style: LeaderStyle) -> Self {
        let dogleg_length = style.dogleg_length;
        let first_segment_angle = style.first_segment_angle;
        let second_segment_angle = style.second_segment_angle;
        
        Self {
            start_point,
            landing: LeaderLanding::default(),
            content,
            style,
            annotation: None,
            is_mleader: false,
            dogleg: DoglegSettings {
                enabled: true,
                direction: DoglegDirection::Automatic,
                length: dogleg_length,
                first_segment_angle,
                second_segment_angle,
            },
        }
    }
    
    /// 追加一个引线顶点（就地修改）。
    /// - `point`：顶点的世界坐标。
    /// 注意：本实现不保存折线路径，仅在内容为 `LeaderContent::Text` 时向文字末尾追加一行 `-> (x, y)`（坐标保留一位小数）；
    /// 内容为块、公差或空时该调用不做任何改动。
    pub fn add_segment(&mut self, point: Point) {
        match &mut self.content {
            LeaderContent::Text(text_content) => {
                text_content.text.push('\n');
                text_content.text.push_str(&format!("-> ({:.1}, {:.1})", point.x, point.y));
            }
            _ => {}
        }
    }
    
    /// 设置落地段端点并启用落地段（就地修改）。
    /// - `landing_point`：新的落地点（世界坐标），同时也是转换为实体时的文字位置。
    pub fn set_landing(&mut self, landing_point: Point) {
        self.landing.landing_point = landing_point;
        self.landing.has_landing = true;
    }
    
    /// 关闭落地段（就地修改）：只把 `has_landing` 置为 false，`landing_point` 与 `landing_length` 保留原值。
    pub fn remove_landing(&mut self) {
        self.landing.has_landing = false;
    }
    
    /// 翻转折弯方向（就地修改）：`Left` 与 `Right` 互换，`Automatic` 变为 `Left`；不会回到 `Automatic`。
    pub fn flip_dogleg(&mut self) {
        self.dogleg.direction = match self.dogleg.direction {
            DoglegDirection::Left => DoglegDirection::Right,
            DoglegDirection::Right => DoglegDirection::Left,
            DoglegDirection::Automatic => DoglegDirection::Left,
        };
    }
    
    /// 挂接注释（就地修改），直接覆盖已有注释。
    /// - `annotation`：注释内容与放置信息；设置后 `annotation` 恒为 `Some`，本类型没有清除接口。
    pub fn set_annotation(&mut self, annotation: Annotation) {
        self.annotation = Some(annotation);
    }
    
    /// 转换为 `Entity`，用于加入文档、渲染与导出。
    /// 生成的实体为 Dimension 类型、线型固定 `Linear`、测量值恒为 0：文字按内容取自身文本 / 块名 / 固定串 "Tolerance" / 空串，
    /// 文字位置取 `landing.landing_point`，定义点取 `start_point` 与落地点，不绘制延伸线与圆心标记。
    /// 不修改自身；`annotation`、`dogleg` 等字段不参与转换。
    pub fn to_entity(&self) -> Entity {
        let text = match &self.content {
            LeaderContent::Text(text_content) => text_content.text.clone(),
            LeaderContent::Block(block_content) => block_content.block_name.clone(),
            LeaderContent::Tolerance(_) => "Tolerance".to_string(),
            LeaderContent::None => String::new(),
        };
        Entity::new(
            EntityType::Dimension,
            EntityGeometry::Dimension {
                dim_type: crate::data_structure::DimensionType::Linear,
                measurement: 0.0,
                text,
                text_position: self.landing.landing_point,
                text_height: self.style.text_height,
                text_rotation: 0.0,
                definition_point: self.start_point,
                def_point_1: self.landing.landing_point,
                def_point_2: Point::origin(),
                def_point_3: Point::origin(),
                def_point_4: Point::origin(),
                angle: 0.0,
                extension_lines: false,
                center_marks: false,
            },
        )
    }
}

impl From<Leader> for Entity {
    fn from(leader: Leader) -> Self {
        leader.to_entity()
    }
}

/// 多重引线：一组共享公共内容与样式的引线，可统一缩放与对齐落地点。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MultiLeader {
    /// 引线列表，条目由 `add_leader` 创建、`remove_leader` 删除。
    pub leaders: Vec<Leader>,
    /// 公共内容，`add_leader` 时克隆给新引线；修改它不影响已创建的引线。
    pub common_content: LeaderContent,
    /// 公共样式模板，`add_leader` 时克隆；每条引线持有自己的样式副本。
    pub style: LeaderStyle,
    /// 整体比例，`new` 为 1.0；`set_overall_scale` 会用它乘算各引线的样式尺寸。
    pub overall_scale: f64,
    /// 落地点的对齐方式，由 `align_landings` 执行。
    pub landing_alignment: LandingAlignment,
}

/// 多重引线的落地点对齐方式。
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum LandingAlignment {
    /// 所有引线的落地点都与第一条对齐（`align_landings` 中与 `AlignFirst` 行为相同）。
    AlignAll,
    /// 只把后续引线的落地点对齐到第一条。
    AlignFirst,
    /// 把全部引线落地点的 y 统一到其中的最小值。
    Distribute,
}

impl MultiLeader {
    /// 创建多重引线容器，初始不含任何引线。
    /// - `style`：公共样式模板；`content`：公共内容。
    /// 返回值的 `overall_scale` 为 1.0、`landing_alignment` 为 `AlignAll`；两个入参只作为后续 `add_leader` 的模板。
    pub fn new(style: LeaderStyle, content: LeaderContent) -> Self {
        Self {
            leaders: Vec::new(),
            common_content: content,
            style,
            overall_scale: 1.0,
            landing_alignment: LandingAlignment::AlignAll,
        }
    }
    
    /// 用公共内容与样式新建一条引线并追加到末尾（就地修改）。
    /// - `start_point`：新引线的起点（箭头端）。
    /// 返回新引线的可变引用，可直接链式调用 `set_landing` 等；借用期间无法再访问 `MultiLeader` 的其它成员。
    /// 新引线持有 `common_content` 与 `style` 的克隆，此后修改二者不会影响已创建的引线。
    pub fn add_leader(&mut self, start_point: Point) -> &mut Leader {
        let leader = Leader::new(start_point, self.common_content.clone(), self.style.clone());
        self.leaders.push(leader);
        self.leaders.last_mut().unwrap()
    }
    
    /// 按下标删除一条引线（就地修改）。
    /// - `index`：从 0 开始的引线下标。
    /// 删除成功返回 true，后续引线下标前移；下标越界时不做任何改动并返回 false。
    pub fn remove_leader(&mut self, index: usize) -> bool {
        if index < self.leaders.len() {
            self.leaders.remove(index);
            true
        } else {
            false
        }
    }
    
    /// 设置整体比例，并按该比例就地放大 / 缩小每条引线的样式尺寸。
    /// - `scale`：比例因子，分别乘到各引线样式副本的 `arrow_size`、`landing_gap`、`landing_distance`、`text_height`、`dogleg_length` 上。
    /// 注意这是累乘而非重设：重复调用会叠加缩放；`overall_scale` 字段被直接覆盖为 `scale`，可能与样式的实际尺寸不一致。几何坐标不受影响。
    pub fn set_overall_scale(&mut self, scale: f64) {
        self.overall_scale = scale;
        for leader in &mut self.leaders {
            leader.style.arrow_size *= scale;
            leader.style.landing_gap *= scale;
            leader.style.landing_distance *= scale;
            leader.style.text_height *= scale;
            leader.style.dogleg_length *= scale;
        }
    }
    
    /// 按 `landing_alignment` 统一各引线落地点的位置（就地修改）。
    /// `AlignAll` 与 `AlignFirst` 都把后续引线的落地点整体移到第一条的落地点；`Distribute` 只把所有落地点的 y 压到其中的最小值（x 不变，并非等距分布）。
    /// 引线列表为空时直接返回，不做任何处理。
    pub fn align_landings(&mut self) {
        if self.leaders.is_empty() {
            return;
        }
        
        match self.landing_alignment {
            LandingAlignment::AlignFirst => {
                let first_landing = self.leaders[0].landing.landing_point;
                for leader in &mut self.leaders[1..] {
                    leader.landing.landing_point = first_landing;
                }
            }
            LandingAlignment::Distribute => {
                let min_y = self.leaders.iter()
                    .filter_map(|l| Some(l.landing.landing_point.y))
                    .fold(f64::MAX, f64::min);
                
                for leader in &mut self.leaders {
                    leader.landing.landing_point.y = min_y;
                }
            }
            _ => {}
        }
    }
    
    /// 把每条引线转换为 `Entity` 并按原顺序返回，用于批量加入文档或渲染。
    /// 不修改自身；返回的实体各自独立，与 `MultiLeader` 不再关联。
    pub fn to_entities(&self) -> Vec<Entity> {
        self.leaders.iter()
            .map(|l| l.to_entity())
            .collect()
    }
}

/// 引线与被标注对象的关联记录：对象几何变化时可据此重新定位引线箭头与折弯点。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LeaderAssociation {
    /// 关联的引线标识；本模块只保存不校验，是否存在对应引线由调用者保证。
    pub leader_id: usize,
    /// 引线箭头端的位置，世界坐标。
    pub arrow_point: Point,
    /// 被标注对象上的目标位置，世界坐标。
    pub target_point: Point,
    /// 折弯点；未调用 `with_dogleg` 时为 `None`。
    pub dogleg_point: Option<Point>,
}

impl LeaderAssociation {
    /// 创建一条关联记录，`dogleg_point` 初始为 `None`。
    /// - `leader_id`：引线标识，仅存储、不做存在性校验。
    /// - `arrow_point`：箭头端位置；`target_point`：被标注对象上的目标位置，均为世界坐标。
    pub fn new(leader_id: usize, arrow_point: Point, target_point: Point) -> Self {
        Self {
            leader_id,
            arrow_point,
            target_point,
            dogleg_point: None,
        }
    }
    
    /// 链式设置折弯点，返回设置后的关联记录。
    /// - `dogleg_point`：折弯位置（世界坐标）；设置后 `dogleg_point` 恒为 `Some`，没有清除接口。
    /// 按值消费 `self`。
    pub fn with_dogleg(mut self, dogleg_point: Point) -> Self {
        self.dogleg_point = Some(dogleg_point);
        self
    }
}
