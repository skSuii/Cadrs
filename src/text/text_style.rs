//! 文字样式与字体度量。
//!
//! [`TextStyle`] 描述一套命名的字体设置（字体、字高、宽度因子、倾斜角、镜像与颜色等），
//! [`FontMetrics`] 给出排版的尺寸基准，[`TextStyleManager`] 按名称集中管理样式并提供当前样式。
//! 本模块只管样式定义，不修改 Document 文档：把样式应用到 Text/MText 上由调用者完成。
//! 字高与各度量使用文档单位（通常为毫米），倾斜角一律为弧度，颜色为 RGB 三元组（0~255）。

use serde::{Serialize, Deserialize};
use std::fmt;

/// 一套命名的文字样式：字体、字高、变形与颜色等绘制属性的集合。
///
/// `PartialEq` 只比较 [`TextStyle::name`]，因此同名的两个样式即被视为相等。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextStyle {
    /// 样式名，在样式管理器中唯一（比较相等时只用该字段）。
    pub name: String,
    /// 主字体名，例如 `"Arial"`。
    pub font_name: String,
    /// 大字体名，用于中日韩等双字节字符；`None` 表示不使用大字体。
    pub big_font_name: Option<String>,
    /// 固定字高（文档单位）；为 0 表示不固定，由文字对象各自指定字高。
    pub height: f64,
    /// 宽度因子，1.0 为不缩放，小于 1 则文字变窄。
    pub width_factor: f64,
    /// 倾斜角，单位弧度，正值向右倾斜。
    pub oblique_angle: f64,
    /// 是否反写（左右镜像）。
    pub is_backwards: bool,
    /// 是否倒写（上下镜像）。
    pub is_upside_down: bool,
    /// 是否竖排文字。
    pub is_vertical: bool,
    /// 文字颜色，RGB 三元组，各分量取值 0~255。
    pub color: (u8, u8, u8),
    /// 样式所属图层名；`None` 表示跟随文字对象所在图层。
    pub layer: Option<String>,
    /// 是否随注释比例缩放，使图纸空间与模型空间的注释大小一致。
    pub annotation_scaling: bool,
    /// 是否允许样式固定字高；为 false 时字高由各文字对象自行指定。
    pub allow_fixed_height: bool,
    /// 字体是否已加载；未加载的样式需先加载字体才能正确排版与渲染。
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
    /// 用样式名与字体名创建样式，其余字段取默认值（字高 2.5、宽度因子 1.0、倾斜角 0、黑色）。
    ///
    /// - `name`：样式名，需在管理器中保持唯一；
    /// - `font_name`：主字体名。
    ///
    /// # 示例
    /// `TextStyle::new("MyStyle", "Arial").with_height(3.0).with_width_factor(0.8)`
    pub fn new(name: impl Into<String>, font_name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            font_name: font_name.into(),
            ..Default::default()
        }
    }

    /// 构建器：设置固定字高（文档单位）后返回自身。
    pub fn with_height(mut self, height: f64) -> Self {
        self.height = height;
        self
    }

    /// 构建器：设置宽度因子后返回自身；1.0 表示不缩放。
    pub fn with_width_factor(mut self, factor: f64) -> Self {
        self.width_factor = factor;
        self
    }

    /// 构建器：设置倾斜角（弧度，正值向右倾斜）后返回自身。
    pub fn with_oblique_angle(mut self, angle: f64) -> Self {
        self.oblique_angle = angle;
        self
    }

    /// 就地修改固定字高（文档单位）；副作用：修改 `self`。
    pub fn set_height(&mut self, height: f64) {
        self.height = height;
    }

    /// 就地修改文字颜色；副作用：修改 `self`。
    ///
    /// - `color`：RGB 三元组，各分量取值 0~255。
    pub fn set_color(&mut self, color: (u8, u8, u8)) {
        self.color = color;
    }

    /// 设置是否反写（左右镜像）；副作用：修改 `self`。
    pub fn set_backwards(&mut self, is_backwards: bool) {
        self.is_backwards = is_backwards;
    }

    /// 设置是否倒写（上下镜像）；副作用：修改 `self`。
    pub fn set_upside_down(&mut self, is_upside_down: bool) {
        self.is_upside_down = is_upside_down;
    }

    /// 设置是否竖排文字；副作用：修改 `self`。
    pub fn set_vertical(&mut self, is_vertical: bool) {
        self.is_vertical = is_vertical;
    }
}

/// 字体度量：描述一种字体的排版尺寸基准，用于估算文字宽度与行距。
///
/// 各长度均为文档单位（通常为毫米），默认值对应字高 2.5 的常见西文字体。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FontMetrics {
    /// 字体名。
    pub font_name: String,
    /// 相邻两行基线之间的距离，即单倍行距。
    pub baseline_to_baseline: f64,
    /// 大写字母高度（基线到字母顶部）。
    pub cap_height: f64,
    /// 小写字母 x 的高度。
    pub x_height: f64,
    /// 降部深度：基线以下部分的高度（如 g、y 的下缘）。
    pub descender_height: f64,
    /// 斜体倾斜角，单位弧度。
    pub italic_angle: f64,
    /// 是否为等宽字体（决定 [`FontMetrics::get_char_width`] 的取值方式）。
    pub is_monospaced: bool,
    /// 单字符最大宽度，是字符宽度估算的基准值。
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
    /// 用字体名创建度量，其余尺寸取默认值（行距 2.5、大写高 1.8、x 高 1.3 等）。
    pub fn new(font_name: impl Into<String>) -> Self {
        Self {
            font_name: font_name.into(),
            ..Default::default()
        }
    }

    /// 估算单个字符的宽度。
    ///
    /// - `_char`：字符本身不参与计算，所有字符宽度相同；
    /// 返回：等宽字体返回 [`FontMetrics::max_character_width`]，非等宽字体取其 0.6 倍。
    pub fn get_char_width(&self, _char: char) -> f64 {
        if self.is_monospaced {
            self.max_character_width
        } else {
            self.max_character_width * 0.6
        }
    }

    /// 估算整串文本的宽度：逐字符累加 [`FontMetrics::get_char_width`]。
    ///
    /// 不区分字符、不处理换行与制表符，返回值同样为文档单位，仅供排版估算。
    pub fn get_text_width(&self, text: &str) -> f64 {
        text.chars()
            .map(|c| self.get_char_width(c))
            .sum()
    }
}

/// 文字样式管理器：按名称维护样式表，并记录当前样式。
///
/// 创建时自带名为 `Standard` 的默认样式且将其设为当前；`Standard` 不可删除。
/// 管理器不持有 Document 文档，样式改动需由调用者同步到文档中的文字对象。
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
    /// 创建管理器：写入 `Standard` 默认样式并将其设为当前样式。
    pub fn new() -> Self {
        Self::default()
    }

    /// 添加一个样式。
    ///
    /// 名称已存在时不覆盖原样式；返回：是否添加成功；副作用：成功时修改 `self`。
    pub fn add_style(&mut self, style: TextStyle) -> bool {
        if self.styles.iter().any(|s| s.name == style.name) {
            return false;
        }
        self.styles.push(style);
        true
    }

    /// 按名称删除样式。
    ///
    /// - `name`：样式名；`"Standard"` 永远不可删除，直接返回 false；
    /// 返回：是否真的删除了样式；若被删的是当前样式，当前样式回落为 `Standard`。
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

    /// 按名称查找样式；不存在返回 `None`。
    pub fn get_style(&self, name: &str) -> Option<&TextStyle> {
        self.styles.iter().find(|s| s.name == name)
    }

    /// 按名称获取可变样式引用，便于就地修改样式属性；不存在返回 `None`。
    ///
    /// 借用期间无法再访问管理器的其它方法。
    pub fn get_style_mut(&mut self, name: &str) -> Option<&mut TextStyle> {
        self.styles.iter_mut().find(|s| s.name == name)
    }

    /// 把已存在的样式设为当前样式；样式不存在时返回 false 且保持原状。
    pub fn set_current_style(&mut self, name: &str) -> bool {
        if self.styles.iter().any(|s| s.name == name) {
            self.current_style = Some(name.to_string());
            true
        } else {
            false
        }
    }

    /// 当前样式的只读引用；未设置或该样式已被删除时返回 `None`。
    pub fn get_current_style(&self) -> Option<&TextStyle> {
        self.current_style
            .as_ref()
            .and_then(|name| self.get_style(name))
    }

    /// 全部样式的切片，顺序与添加顺序一致。
    pub fn get_all_styles(&self) -> &[TextStyle] {
        &self.styles
    }

    /// 全部样式名，顺序与 [`TextStyleManager::get_all_styles`] 一致。
    pub fn get_style_names(&self) -> Vec<&str> {
        self.styles.iter().map(|s| s.name.as_str()).collect()
    }

    /// 判断是否存在指定名称的样式。
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
