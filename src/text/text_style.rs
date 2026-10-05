use serde::{Serialize, Deserialize};
use std::fmt;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextStyle {
    pub name: String,
    pub font_name: String,
    pub big_font_name: Option<String>,
    pub height: f64,
    pub width_factor: f64,
    pub oblique_angle: f64,
    pub is_backwards: bool,
    pub is_upside_down: bool,
    pub is_vertical: bool,
    pub color: (u8, u8, u8),
    pub layer: Option<String>,
    pub annotation_scaling: bool,
    pub allow_fixed_height: bool,
    pub is_loaded: bool,
}

impl Default for TextStyle {
    fn default() -> Self {
        Self {
            name: "Standard".to_string(),
            font_name: "Arial".to_string(),
            big_font_name: None,
            height: 2.5,
            width_factor: 1.0,
            oblique_angle: 0.0,
            is_backwards: false,
            is_upside_down: false,
            is_vertical: false,
            color: (0, 0, 0),
            layer: None,
            annotation_scaling: false,
            allow_fixed_height: false,
            is_loaded: true,
        }
    }
}

impl PartialEq for TextStyle {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl TextStyle {
    pub fn new(name: impl Into<String>, font_name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            font_name: font_name.into(),
            ..Default::default()
        }
    }

    pub fn with_height(mut self, height: f64) -> Self {
        self.height = height;
        self
    }

    pub fn with_width_factor(mut self, factor: f64) -> Self {
        self.width_factor = factor;
        self
    }

    pub fn with_oblique_angle(mut self, angle: f64) -> Self {
        self.oblique_angle = angle;
        self
    }

    pub fn set_height(&mut self, height: f64) {
        self.height = height;
    }

    pub fn set_color(&mut self, color: (u8, u8, u8)) {
        self.color = color;
    }

    pub fn set_backwards(&mut self, is_backwards: bool) {
        self.is_backwards = is_backwards;
    }

    pub fn set_upside_down(&mut self, is_upside_down: bool) {
        self.is_upside_down = is_upside_down;
    }

    pub fn set_vertical(&mut self, is_vertical: bool) {
        self.is_vertical = is_vertical;
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FontMetrics {
    pub font_name: String,
    pub baseline_to_baseline: f64,
    pub cap_height: f64,
    pub x_height: f64,
    pub descender_height: f64,
    pub italic_angle: f64,
    pub is_monospaced: bool,
    pub max_character_width: f64,
}

impl Default for FontMetrics {
    fn default() -> Self {
        Self {
            font_name: String::new(),
            baseline_to_baseline: 2.5,
            cap_height: 1.8,
            x_height: 1.3,
            descender_height: 0.5,
            italic_angle: 0.0,
            is_monospaced: false,
            max_character_width: 1.5,
        }
    }
}

impl FontMetrics {
    pub fn new(font_name: impl Into<String>) -> Self {
        Self {
            font_name: font_name.into(),
            ..Default::default()
        }
    }

    pub fn get_char_width(&self, _char: char) -> f64 {
        if self.is_monospaced {
            self.max_character_width
        } else {
            self.max_character_width * 0.6
        }
    }

    pub fn get_text_width(&self, text: &str) -> f64 {
        text.chars()
            .map(|c| self.get_char_width(c))
            .sum()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextStyleManager {
    styles: Vec<TextStyle>,
    current_style: Option<String>,
}

impl Default for TextStyleManager {
    fn default() -> Self {
        let mut manager = Self {
            styles: Vec::new(),
            current_style: None,
        };

        manager.add_style(TextStyle::default());
        manager.set_current_style("Standard");

        manager
    }
}

impl TextStyleManager {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_style(&mut self, style: TextStyle) -> bool {
        if self.styles.iter().any(|s| s.name == style.name) {
            return false;
        }
        self.styles.push(style);
        true
    }

    pub fn remove_style(&mut self, name: &str) -> bool {
        if name == "Standard" {
            return false;
        }
        let initial_len = self.styles.len();
        self.styles.retain(|s| s.name != name);
        if self.current_style == Some(name.to_string()) {
            self.current_style = Some("Standard".to_string());
        }
        self.styles.len() < initial_len
    }

    pub fn get_style(&self, name: &str) -> Option<&TextStyle> {
        self.styles.iter().find(|s| s.name == name)
    }

    pub fn get_style_mut(&mut self, name: &str) -> Option<&mut TextStyle> {
        self.styles.iter_mut().find(|s| s.name == name)
    }

    pub fn set_current_style(&mut self, name: &str) -> bool {
        if self.styles.iter().any(|s| s.name == name) {
            self.current_style = Some(name.to_string());
            true
        } else {
            false
        }
    }

    pub fn get_current_style(&self) -> Option<&TextStyle> {
        self.current_style
            .as_ref()
            .and_then(|name| self.get_style(name))
    }

    pub fn get_all_styles(&self) -> &[TextStyle] {
        &self.styles
    }

    pub fn get_style_names(&self) -> Vec<&str> {
        self.styles.iter().map(|s| s.name.as_str()).collect()
    }

    pub fn style_exists(&self, name: &str) -> bool {
        self.styles.iter().any(|s| s.name == name)
    }
}

impl fmt::Display for TextStyle {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "TextStyle(name=\"{}\", font=\"{}\", height={}, width_factor={})",
            self.name, self.font_name, self.height, self.width_factor
        )
    }
}

impl fmt::Display for TextStyleManager {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "TextStyleManager(styles={}, current=\"{}\")",
            self.styles.len(),
            self.current_style.as_ref().unwrap_or(&"None".to_string())
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_text_style_creation() {
        let style = TextStyle::new("MyStyle", "Arial")
            .with_height(3.0)
            .with_width_factor(0.8);

        assert_eq!(style.name, "MyStyle");
        assert_eq!(style.font_name, "Arial");
        assert_eq!(style.height, 3.0);
        assert_eq!(style.width_factor, 0.8);
    }

    #[test]
    fn test_text_style_manager() {
        let mut manager = TextStyleManager::new();

        let custom_style = TextStyle::new("Custom", "Verdana")
            .with_height(5.0);

        assert!(manager.add_style(custom_style.clone()));
        assert!(!manager.add_style(custom_style));

        assert!(manager.style_exists("Custom"));
        assert!(manager.set_current_style("Custom"));

        let current = manager.get_current_style().unwrap();
        assert_eq!(current.name, "Custom");
    }
}
