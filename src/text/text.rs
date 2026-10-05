use serde::{Serialize, Deserialize};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum TextAlignment {
    Left,
    Center,
    Right,
    Middle,
    TopLeft,
    TopCenter,
    TopRight,
    MiddleLeft,
    MiddleCenter,
    MiddleRight,
    BottomLeft,
    BottomCenter,
    BottomRight,
    Aligned,
    MiddleOfField,
}

impl Default for TextAlignment {
    fn default() -> Self {
        TextAlignment::Left
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum TextVerticalAlignment {
    Top,
    Middle,
    Bottom,
}

impl Default for TextVerticalAlignment {
    fn default() -> Self {
        TextVerticalAlignment::Bottom
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextStyle {
    pub name: String,
    pub font_name: String,
    pub height: f64,
    pub width_factor: f64,
    pub oblique_angle: f64,
    pub is_backwards: bool,
    pub is_upside_down: bool,
    pub is_vertical: bool,
    pub color: (u8, u8, u8),
    pub layer: Option<String>,
}

impl Default for TextStyle {
    fn default() -> Self {
        Self {
            name: "Standard".to_string(),
            font_name: "Arial".to_string(),
            height: 2.5,
            width_factor: 1.0,
            oblique_angle: 0.0,
            is_backwards: false,
            is_upside_down: false,
            is_vertical: false,
            color: (0, 0, 0),
            layer: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Text {
    pub content: String,
    pub position: crate::geometry::Point,
    pub height: f64,
    pub rotation: f64,
    pub width_factor: f64,
    pub oblique_angle: f64,
    pub style: TextStyle,
    pub horizontal_alignment: TextAlignment,
    pub vertical_alignment: TextVerticalAlignment,
    pub color: (u8, u8, u8),
    pub layer: Option<String>,
    pub visibility: crate::data_structure::Visibility,
}

impl Default for Text {
    fn default() -> Self {
        Self {
            content: String::new(),
            position: crate::geometry::Point::new(0.0, 0.0, 0.0),
            height: 2.5,
            rotation: 0.0,
            width_factor: 1.0,
            oblique_angle: 0.0,
            style: TextStyle::default(),
            horizontal_alignment: TextAlignment::Left,
            vertical_alignment: TextVerticalAlignment::Bottom,
            color: (0, 0, 0),
            layer: None,
            visibility: crate::data_structure::Visibility::Visible,
        }
    }
}

impl Text {
    pub fn new(content: String, position: crate::geometry::Point, height: f64) -> Self {
        Self {
            content,
            position,
            height,
            ..Default::default()
        }
    }

    pub fn with_style(content: String, position: crate::geometry::Point, height: f64, style: TextStyle) -> Self {
        Self {
            content,
            position,
            height,
            style,
            ..Default::default()
        }
    }

    pub fn set_alignment(&mut self, horizontal: TextAlignment, vertical: TextVerticalAlignment) {
        self.horizontal_alignment = horizontal;
        self.vertical_alignment = vertical;
    }

    pub fn set_rotation(&mut self, angle: f64) {
        self.rotation = angle;
    }

    pub fn get_bounding_box(&self) -> (crate::geometry::Point, crate::geometry::Point) {
        let width = self.content.len() as f64 * self.height * self.width_factor * 0.6;
        let half_height = self.height / 2.0;

        let min = crate::geometry::Point::new(
            self.position.x - width / 2.0,
            self.position.y - half_height,
            0.0,
        );
        let max = crate::geometry::Point::new(
            self.position.x + width / 2.0,
            self.position.y + half_height,
            0.0,
        );

        (min, max)
    }
}

impl fmt::Display for Text {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Text(content=\"{}\", position={}, height={})",
            self.content, self.position, self.height
        )
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TextJustification {
    Left,
    Center,
    Right,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextFormatting {
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
    pub color: Option<(u8, u8, u8)>,
    pub font_size: Option<f64>,
    pub font_name: Option<String>,
}

impl Default for TextFormatting {
    fn default() -> Self {
        Self {
            bold: false,
            italic: false,
            underline: false,
            color: None,
            font_size: None,
            font_name: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormattedText {
    pub text: String,
    pub formatting: TextFormatting,
}

impl FormattedText {
    pub fn new(text: String) -> Self {
        Self {
            text,
            formatting: TextFormatting::default(),
        }
    }

    pub fn bold(mut self) -> Self {
        self.formatting.bold = true;
        self
    }

    pub fn italic(mut self) -> Self {
        self.formatting.italic = true;
        self
    }

    pub fn with_color(mut self, color: (u8, u8, u8)) -> Self {
        self.formatting.color = Some(color);
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MText {
    pub contents: Vec<FormattedText>,
    pub position: crate::geometry::Point,
    pub height: f64,
    pub rotation: f64,
    pub style: TextStyle,
    pub line_spacing: f64,
    pub width: Option<f64>,
    pub color: (u8, u8, u8),
    pub layer: Option<String>,
    pub visibility: crate::data_structure::Visibility,
}

impl Default for MText {
    fn default() -> Self {
        Self {
            contents: Vec::new(),
            position: crate::geometry::Point::new(0.0, 0.0, 0.0),
            height: 2.5,
            rotation: 0.0,
            style: TextStyle::default(),
            line_spacing: 1.0,
            width: None,
            color: (0, 0, 0),
            layer: None,
            visibility: crate::data_structure::Visibility::Visible,
        }
    }
}

impl MText {
    pub fn new(contents: Vec<FormattedText>, position: crate::geometry::Point, height: f64) -> Self {
        Self {
            contents,
            position,
            height,
            ..Default::default()
        }
    }

    pub fn add_text(&mut self, text: FormattedText) {
        self.contents.push(text);
    }

    pub fn add_plain_text(&mut self, text: &str) {
        self.contents.push(FormattedText::new(text.to_string()));
    }

    pub fn set_width(&mut self, width: f64) {
        self.width = Some(width);
    }

    pub fn set_line_spacing(&mut self, spacing: f64) {
        self.line_spacing = spacing;
    }

    pub fn to_plain_text(&self) -> String {
        self.contents
            .iter()
            .map(|ft| ft.text.clone())
            .collect()
    }
}

impl fmt::Display for MText {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "MText(position={}, height={}, lines={})",
            self.position,
            self.height,
            self.contents.len()
        )
    }
}

pub struct TextBuilder {
    content: String,
    position: crate::geometry::Point,
    height: f64,
    rotation: f64,
    style: TextStyle,
    horizontal_alignment: TextAlignment,
    vertical_alignment: TextVerticalAlignment,
    width_factor: f64,
    oblique_angle: f64,
}

impl Default for TextBuilder {
    fn default() -> Self {
        Self {
            content: String::new(),
            position: crate::geometry::Point::new(0.0, 0.0, 0.0),
            height: 2.5,
            rotation: 0.0,
            style: TextStyle::default(),
            horizontal_alignment: TextAlignment::Left,
            vertical_alignment: TextVerticalAlignment::Bottom,
            width_factor: 1.0,
            oblique_angle: 0.0,
        }
    }
}

impl TextBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn content(mut self, content: impl Into<String>) -> Self {
        self.content = content.into();
        self
    }

    pub fn position(mut self, x: f64, y: f64) -> Self {
        self.position = crate::geometry::Point::new(x, y, 0.0);
        self
    }

    pub fn position_point(mut self, point: crate::geometry::Point) -> Self {
        self.position = point;
        self
    }

    pub fn height(mut self, height: f64) -> Self {
        self.height = height;
        self
    }

    pub fn rotation(mut self, angle: f64) -> Self {
        self.rotation = angle;
        self
    }

    pub fn style(mut self, style: TextStyle) -> Self {
        self.style = style;
        self
    }

    pub fn alignment(mut self, horizontal: TextAlignment, vertical: TextVerticalAlignment) -> Self {
        self.horizontal_alignment = horizontal;
        self.vertical_alignment = vertical;
        self
    }

    pub fn width_factor(mut self, factor: f64) -> Self {
        self.width_factor = factor;
        self
    }

    pub fn oblique_angle(mut self, angle: f64) -> Self {
        self.oblique_angle = angle;
        self
    }

    pub fn build(self) -> Text {
        Text {
            content: self.content,
            position: self.position,
            height: self.height,
            rotation: self.rotation,
            width_factor: self.width_factor,
            oblique_angle: self.oblique_angle,
            style: self.style,
            horizontal_alignment: self.horizontal_alignment,
            vertical_alignment: self.vertical_alignment,
            color: (0, 0, 0),
            layer: None,
            visibility: crate::data_structure::Visibility::Visible,
        }
    }
}

impl fmt::Display for TextBuilder {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "TextBuilder(content=\"{}\", position={}, height={})",
            self.content, self.position, self.height
        )
    }
}
