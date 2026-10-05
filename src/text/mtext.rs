use serde::{Serialize, Deserialize};
use std::fmt;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MTextParagraph {
    pub text: String,
    pub start_position: crate::geometry::Point,
    pub line_height: f64,
    pub indentation: f64,
    pub bullet: Option<MTextBullet>,
    pub formatting: super::TextFormatting,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MTextBullet {
    Dot,
    Circle,
    Square,
    Number(u32),
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
    pub fn new(text: String) -> Self {
        Self {
            text,
            ..Default::default()
        }
    }

    pub fn with_bullet(mut self, bullet: MTextBullet) -> Self {
        self.bullet = Some(bullet);
        self
    }

    pub fn with_indentation(mut self, indentation: f64) -> Self {
        self.indentation = indentation;
        self
    }

    pub fn with_formatting(mut self, formatting: super::TextFormatting) -> Self {
        self.formatting = formatting;
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MTextSymbol {
    Degree,
    PlusMinus,
    Diameter,
    Approximately,
    Ellipsis,
    Space,
    LineFeed,
    Custom(String),
}

impl MTextSymbol {
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MTextBlock {
    pub paragraphs: Vec<MTextParagraph>,
    pub base_position: crate::geometry::Point,
    pub width: Option<f64>,
    pub flow_direction: MTextFlowDirection,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum MTextFlowDirection {
    LeftToRight,
    RightToLeft,
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
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_paragraph(&mut self, paragraph: MTextParagraph) {
        self.paragraphs.push(paragraph);
    }

    pub fn add_plain_paragraph(&mut self, text: &str) {
        self.paragraphs.push(MTextParagraph::new(text.to_string()));
    }

    pub fn set_width(&mut self, width: f64) {
        self.width = Some(width);
    }

    pub fn set_flow_direction(&mut self, direction: MTextFlowDirection) {
        self.flow_direction = direction;
    }

    pub fn get_text(&self) -> String {
        self.paragraphs
            .iter()
            .map(|p| p.text.clone())
            .collect::<Vec<_>>()
            .join("\n")
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MTextColumn {
    pub column_number: u32,
    pub start_position: crate::geometry::Point,
    pub width: f64,
    pub height: f64,
    pub gutter: f64,
    pub contents: MTextBlock,
}

impl MTextColumn {
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

    pub fn set_height(&mut self, height: f64) {
        self.height = height;
    }

    pub fn set_gutter(&mut self, gutter: f64) {
        self.gutter = gutter;
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MTextWithColumns {
    pub main_content: MTextBlock,
    pub columns: Vec<MTextColumn>,
    pub column_count: u32,
    pub auto_height: bool,
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
    pub fn new(column_count: u32) -> Self {
        Self {
            column_count,
            ..Default::default()
        }
    }

    pub fn create_columns(&mut self, start_position: crate::geometry::Point, total_width: f64, gutter: f64) {
        self.columns.clear();
        let column_width = (total_width - gutter * (self.column_count as f64 - 1.0)) / self.column_count as f64;

        for i in 0..self.column_count {
            let x = start_position.x + (column_width + gutter) * i as f64;
            let position = crate::geometry::Point::new(x, start_position.y, 0.0);
            self.columns.push(MTextColumn::new(i + 1, position, column_width));
        }
    }

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
