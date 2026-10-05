//! 图层：实体的组织与样式容器。
//!
//! 每个 [`Layer`] 持有自己的颜色、线宽与 [`LayerVisibility`] 状态，实体通过 `layer_id`
//! 关联到图层并由图层间接决定显示样式；文档新建时会预置 `"ModelSpace"` 与 `"PaperSpace"`
//! 两个图层。可见性与锁定是两组正交的判断：[`Layer::is_visible`] 与 [`Layer::is_locked`]
//! 对 `Locked` 状态的解读不同，调用者应按用途选用。线宽以毫米计，不随文档单位换算。

use uuid::Uuid;
use std::collections::HashMap;
use super::entity_id::ObjectId;

/// 图层：一组实体的样式与可见性设置。
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Layer {
    id: ObjectId,
    name: String,
    description: String,
    color: Color,
    line_type: (),
    line_weight: f64,
    visibility: LayerVisibility,
    plot_style: String,
    properties: HashMap<String, String>,
}

/// 图层颜色，24 位 RGB 分量。
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Color {
    /// 红色分量，0~255。
    pub red: u8,
    /// 绿色分量，0~255。
    pub green: u8,
    /// 蓝色分量，0~255。
    pub blue: u8,
}

/// 图层的显示与编辑状态。
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum LayerVisibility {
    /// 正常显示且可编辑。
    Visible,
    /// 隐藏：不显示，但仍可编辑。
    Hidden,
    /// 冻结：不显示且不参与重生成等计算。
    Frozen,
    /// 锁定：显示但不可编辑。
    Locked,
}

impl Layer {
    /// 新建图层，属性取默认值。
    ///
    /// 默认颜色为白色 `(255, 255, 255)`、线宽 `0.25`、状态 [`LayerVisibility::Visible`]、
    /// 描述与打印样式为空字符串；图层 id 自动生成，加入文档后才会生效。
    ///
    /// - `name`：图层名，本方法不做重名校验。
    pub fn new(name: String) -> Self {
        Self {
            id: ObjectId::new(),
            name,
            description: String::new(),
            color: Color { red: 255, green: 255, blue: 255 },
            line_type: (),
            line_weight: 0.25,
            visibility: LayerVisibility::Visible,
            plot_style: String::new(),
            properties: HashMap::new(),
        }
    }
    
    /// 图层的唯一标识，作为文档图层表的键，也是实体 `layer_id` 的取值。
    pub fn id(&self) -> &ObjectId {
        &self.id
    }
    
    /// 设置图层描述，会修改 `self`；描述仅用于展示，不影响显示样式。
    pub fn set_description(&mut self, description: String) {
        self.description = description;
    }

    /// 图层名，未做唯一性保证，重名图层可能同时存在。
    #[inline]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// 重命名图层，会修改 `self`；仅改变名称，实体仍按 id 关联本图层。
    #[inline]
    pub fn set_name(&mut self, name: String) {
        self.name = name;
    }

    /// 图层描述，未设置时为空字符串。
    #[inline]
    pub fn description(&self) -> &str {
        &self.description
    }

    /// 图层颜色，默认白色。
    #[inline]
    pub fn color(&self) -> Color {
        self.color
    }

    /// 设置图层颜色，会修改 `self`；只影响按图层取色的实体。
    #[inline]
    pub fn set_color(&mut self, color: Color) {
        self.color = color;
    }

    /// 线宽，单位为毫米，默认 `0.25`，不受文档单位设置影响。
    #[inline]
    pub fn line_weight(&self) -> f64 {
        self.line_weight
    }

    /// 设置线宽，会修改 `self`；负值不合法，本方法不做校验。
    #[inline]
    pub fn set_line_weight(&mut self, line_weight: f64) {
        self.line_weight = line_weight;
    }

    /// 当前显示与编辑状态。
    #[inline]
    pub fn visibility(&self) -> LayerVisibility {
        self.visibility
    }

    /// 设置显示与编辑状态，会修改 `self`，影响该图层下全部实体的显示。
    #[inline]
    pub fn set_visibility(&mut self, visibility: LayerVisibility) {
        self.visibility = visibility;
    }

    /// 图层内容是否参与显示（Visible 与 Locked 均视为可见）
    #[inline]
    pub fn is_visible(&self) -> bool {
        matches!(self.visibility, LayerVisibility::Visible | LayerVisibility::Locked)
    }

    /// 图层是否处于隐藏状态；仅 [`LayerVisibility::Hidden`] 为真，冻结不算隐藏。
    #[inline]
    pub fn is_hidden(&self) -> bool {
        matches!(self.visibility, LayerVisibility::Hidden)
    }

    /// 图层是否处于冻结状态；仅 [`LayerVisibility::Frozen`] 为真。
    #[inline]
    pub fn is_frozen(&self) -> bool {
        matches!(self.visibility, LayerVisibility::Frozen)
    }

    /// 图层内容是否被锁定不可编辑（Locked 与 Frozen 均视为锁定）
    #[inline]
    pub fn is_locked(&self) -> bool {
        matches!(self.visibility, LayerVisibility::Locked | LayerVisibility::Frozen)
    }
}
