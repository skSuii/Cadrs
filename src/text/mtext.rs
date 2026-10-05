//! 多行文字（MText）：段落、特殊符号、分栏与文字流向。
//!
//! [`MTextParagraph`] 描述一段文字及其行高、缩进、项目符号与格式，若干段落组成 [`MTextBlock`]；
//! [`MTextWithColumns`] 在文字块之上提供分栏布局，可把主内容按栏高切分到各栏。
//! 位置为世界坐标，行高、缩进、栏宽等长度均为文档单位（通常为毫米）。
//! 本模块只描述排版数据，不生成 Tessellation 细分，也不直接写入 Document 文档。

use serde::{Serialize, Deserialize};
use std::fmt;

/// 多行文字中的一个段落：文字内容、起始位置、行高、缩进、项目符号与字符格式。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MTextParagraph {
    /// 段落文本，不含换行符——换行通过划分新段落表达。
    pub text: String,
    /// 段落起始位置（世界坐标，通常为文本基线左端）。
    pub start_position: crate::geometry::Point,
    /// 行高（文档单位），分栏流动时用于累计判断是否换栏。
    pub line_height: f64,
    /// 首行缩进量（文档单位），负值表示悬挂缩进。
    pub indentation: f64,
    /// 项目符号；`None` 表示该段无符号。
    pub bullet: Option<MTextBullet>,
    /// 段落内字符的格式（粗体、斜体、下划线、颜色、字体等）。
    pub formatting: super::TextFormatting,
}

/// 段落的项目符号类型。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MTextBullet {
    /// 圆点符号。
    Dot,
    /// 空心圆符号。
    Circle,
    /// 方块符号。
    Square,
    /// 数字编号，载荷为起始编号。
    Number(u32),
    /// 自定义符号，载荷为符号文本。
    Custom(String),
}

impl Default for MTextParagraph {
    fn default() -> Self {
        Self {
            text: String::new(),
            start_position: crate::geometry::Point::new(0.0, 0.0, 0.0),
            line_height: 1.0,
            indentation: 0.0,
            bullet: None,
            formatting: super::TextFormatting::default(),
        }
    }
}

impl MTextParagraph {
    /// 用文本创建段落，其余字段取默认值（起始位置原点、行高 1.0、无缩进、无符号、默认格式）。
    pub fn new(text: String) -> Self {
        Self {
            text,
            ..Default::default()
        }
    }

    /// 构建器：设置项目符号后返回自身。
    ///
    /// - `bullet`：符号类型，内部以 `Some` 保存。
    pub fn with_bullet(mut self, bullet: MTextBullet) -> Self {
        self.bullet = Some(bullet);
        self
    }

    /// 构建器：设置首行缩进（文档单位）后返回自身。
    pub fn with_indentation(mut self, indentation: f64) -> Self {
        self.indentation = indentation;
        self
    }

    /// 构建器：整体替换段落字符格式后返回自身。
    pub fn with_formatting(mut self, formatting: super::TextFormatting) -> Self {
        self.formatting = formatting;
        self
    }
}

/// 多行文字的特殊符号，可按文本形式插入到文字内容中。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MTextSymbol {
    /// 度符号 `°`。
    Degree,
    /// 正负号 `±`。
    PlusMinus,
    /// 直径符号，文本形式为 AutoCAD 控制码 `%%c`。
    Diameter,
    /// 约等于号 `≈`。
    Approximately,
    /// 省略号，文本形式为 `...`。
    Ellipsis,
    /// 一个空格。
    Space,
    /// 换行符。
    LineFeed,
    /// 自定义符号，载荷即其文本形式。
    Custom(String),
}

impl MTextSymbol {
    /// 把符号转换为其文本表示：直径输出控制码 `%%c`、省略号输出 `...`、换行输出 `\n`。
    ///
    /// 该方法是固有方法，`MTextSymbol` 并未实现 `Display`。
    pub fn to_string(&self) -> String {
        match self {
            MTextSymbol::Degree => "°".to_string(),
            MTextSymbol::PlusMinus => "±".to_string(),
            MTextSymbol::Diameter => "%%c".to_string(),
            MTextSymbol::Approximately => "≈".to_string(),
            MTextSymbol::Ellipsis => "...".to_string(),
            MTextSymbol::Space => " ".to_string(),
            MTextSymbol::LineFeed => "\n".to_string(),
            MTextSymbol::Custom(s) => s.clone(),
        }
    }
}

/// 多行文字块：若干段落及其基点、宽度限制与流向。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MTextBlock {
    /// 段落列表，顺序即排版顺序。
    pub paragraphs: Vec<MTextParagraph>,
    /// 文字块基点（世界坐标）。
    pub base_position: crate::geometry::Point,
    /// 段落宽度限制（文档单位）；`None` 表示不按宽度换行。
    pub width: Option<f64>,
    /// 段落排列方向。
    pub flow_direction: MTextFlowDirection,
}

/// 文字块内段落的排列方向。
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum MTextFlowDirection {
    /// 从左到右（默认）。
    LeftToRight,
    /// 从右到左。
    RightToLeft,
    /// 从上到下。
    TopToBottom,
}

impl Default for MTextBlock {
    fn default() -> Self {
        Self {
            paragraphs: Vec::new(),
            base_position: crate::geometry::Point::new(0.0, 0.0, 0.0),
            width: None,
            flow_direction: MTextFlowDirection::LeftToRight,
        }
    }
}

impl MTextBlock {
    /// 创建空文字块：无段落、无宽度限制、流向为从左到右、基点位于原点。
    pub fn new() -> Self {
        Self::default()
    }

    /// 在末尾追加一个段落；副作用：修改 `self`。
    pub fn add_paragraph(&mut self, paragraph: MTextParagraph) {
        self.paragraphs.push(paragraph);
    }

    /// 用纯文本构造段落（默认格式）并追加到末尾；副作用：修改 `self`。
    pub fn add_plain_paragraph(&mut self, text: &str) {
        self.paragraphs.push(MTextParagraph::new(text.to_string()));
    }

    /// 设置段落宽度限制（文档单位）；副作用：修改 `self`。
    pub fn set_width(&mut self, width: f64) {
        self.width = Some(width);
    }

    /// 设置段落排列方向；副作用：修改 `self`。
    pub fn set_flow_direction(&mut self, direction: MTextFlowDirection) {
        self.flow_direction = direction;
    }

    /// 把所有段落文本用换行符 `\n` 连接成纯文本。
    ///
    /// 忽略缩进、项目符号与字符格式，段落顺序保持不变。
    pub fn get_text(&self) -> String {
        self.paragraphs
            .iter()
            .map(|p| p.text.clone())
            .collect::<Vec<_>>()
            .join("\n")
    }
}

/// 分栏文字中的一栏：栏号、起始位置、栏尺寸、栏间距与栏内容。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MTextColumn {
    /// 栏号，从 1 开始。
    pub column_number: u32,
    /// 栏的起始位置（世界坐标）。
    pub start_position: crate::geometry::Point,
    /// 栏宽（文档单位）。
    pub width: f64,
    /// 栏高（文档单位），分栏流动时用于判断换栏。
    pub height: f64,
    /// 与后一栏之间的间距（文档单位），默认 0.5。
    pub gutter: f64,
    /// 本栏承载的文字块。
    pub contents: MTextBlock,
}

impl MTextColumn {
    /// 用栏号、起始位置与栏宽创建栏；高度初始为 0（需自行设置），栏间距 0.5，内容为空。
    pub fn new(column_number: u32, start_position: crate::geometry::Point, width: f64) -> Self {
        Self {
            column_number,
            start_position,
            width,
            height: 0.0,
            gutter: 0.5,
            contents: MTextBlock::default(),
        }
    }

    /// 设置栏高（文档单位）；副作用：修改 `self`。
    pub fn set_height(&mut self, height: f64) {
        self.height = height;
    }

    /// 设置与后一栏的间距（文档单位）；副作用：修改 `self`。
    pub fn set_gutter(&mut self, gutter: f64) {
        self.gutter = gutter;
    }
}

/// 带分栏的多行文字：主内容加若干等宽栏。
///
/// 只有 [`MTextWithColumns::column_flow`] 为 true 时才会把主内容分配到各栏。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MTextWithColumns {
    /// 主文字块，是分栏流动时的内容来源，流动不会修改它。
    pub main_content: MTextBlock,
    /// 已创建的栏，顺序即排版顺序。
    pub columns: Vec<MTextColumn>,
    /// 栏数，[`MTextWithColumns::create_columns`] 按它重建栏。
    pub column_count: u32,
    /// 是否按内容自动调整高度（当前实现只保存该标志，未参与计算）。
    pub auto_height: bool,
    /// 是否启用内容分栏流动；为 false 时 [`MTextWithColumns::flow_content_to_columns`] 直接返回。
    pub column_flow: bool,
}

impl Default for MTextWithColumns {
    fn default() -> Self {
        Self {
            main_content: MTextBlock::default(),
            columns: Vec::new(),
            column_count: 1,
            auto_height: true,
            column_flow: false,
        }
    }
}

impl MTextWithColumns {
    /// 用栏数创建：主内容与栏列表为空、auto_height 为 true、column_flow 为 false。
    pub fn new(column_count: u32) -> Self {
        Self {
            column_count,
            ..Default::default()
        }
    }

    /// 按 [`MTextWithColumns::column_count`] 重建等宽栏。
    ///
    /// - `start_position`：第一栏的起始位置（世界坐标）；
    /// - `total_width`：所有栏加栏间距的总宽度（文档单位）；
    /// - `gutter`：相邻栏之间的间距（文档单位）；
    /// 副作用：清空原有栏；栏宽为 `(total_width - gutter × (栏数 - 1)) / 栏数`，
    /// 各栏高度保持 0，需再调用 [`MTextColumn::set_height`] 才能用于分栏流动。
    pub fn create_columns(&mut self, start_position: crate::geometry::Point, total_width: f64, gutter: f64) {
        self.columns.clear();
        let column_width = (total_width - gutter * (self.column_count as f64 - 1.0)) / self.column_count as f64;

        for i in 0..self.column_count {
            let x = start_position.x + (column_width + gutter) * i as f64;
            let position = crate::geometry::Point::new(x, start_position.y, 0.0);
            self.columns.push(MTextColumn::new(i + 1, position, column_width));
        }
    }

    /// 把主内容的段落按栏高依次分配到各栏。
    ///
    /// 仅当 column_flow 为 true 且已存在栏时生效，否则直接返回且不做任何修改；
    /// 逐段累加行高，超出当前栏高且尚未到最后一栏时切换到下一栏；
    /// 段落被**复制**进栏内容，主内容保持不变；已存在的栏内容不会被清空或累加。
    pub fn flow_content_to_columns(&mut self) {
        if !self.column_flow || self.columns.is_empty() {
            return;
        }

        let total_capacity: usize = self.columns.iter().map(|c| c.height as usize).sum();
        let mut current_column = 0;
        let mut current_height = 0.0;

        for paragraph in &mut self.main_content.paragraphs {
            if current_height + paragraph.line_height > self.columns[current_column as usize].height
                && current_column < self.column_count - 1
            {
                current_column += 1;
                current_height = 0.0;
            }

            self.columns[current_column as usize]
                .contents
                .add_paragraph(paragraph.clone());
            current_height += paragraph.line_height;
        }
    }
}

impl fmt::Display for MTextParagraph {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Paragraph(text=\"{}\", line_height={})",
            self.text, self.line_height
        )
    }
}

impl fmt::Display for MTextBlock {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "MTextBlock(paragraphs={}, width={})",
            self.paragraphs.len(),
            self.width.unwrap_or(0.0)
        )
    }
}
