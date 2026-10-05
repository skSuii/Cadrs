use uuid::Uuid;
use std::collections::HashMap;
use super::entity_id::ObjectId;

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

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Color {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum LayerVisibility {
    Visible,
    Hidden,
    Frozen,
    Locked,
}

impl Layer {
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
    
    pub fn id(&self) -> &ObjectId {
        &self.id
    }
    
    pub fn set_description(&mut self, description: String) {
        self.description = description;
    }

    #[inline]
    pub fn name(&self) -> &str {
        &self.name
    }

    #[inline]
    pub fn set_name(&mut self, name: String) {
        self.name = name;
    }

    #[inline]
    pub fn description(&self) -> &str {
        &self.description
    }

    #[inline]
    pub fn color(&self) -> Color {
        self.color
    }

    #[inline]
    pub fn set_color(&mut self, color: Color) {
        self.color = color;
    }

    #[inline]
    pub fn line_weight(&self) -> f64 {
        self.line_weight
    }

    #[inline]
    pub fn set_line_weight(&mut self, line_weight: f64) {
        self.line_weight = line_weight;
    }

    #[inline]
    pub fn visibility(&self) -> LayerVisibility {
        self.visibility
    }

    #[inline]
    pub fn set_visibility(&mut self, visibility: LayerVisibility) {
        self.visibility = visibility;
    }

    /// 图层内容是否参与显示（Visible 与 Locked 均视为可见）
    #[inline]
    pub fn is_visible(&self) -> bool {
        matches!(self.visibility, LayerVisibility::Visible | LayerVisibility::Locked)
    }

    #[inline]
    pub fn is_hidden(&self) -> bool {
        matches!(self.visibility, LayerVisibility::Hidden)
    }

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
