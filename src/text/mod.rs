//! 文字层：单行文字（Text）、多行文字（MText）与文字样式。
//!
//! 文字对象以世界坐标 [`Point`](crate::geometry::Point) 定位：字高与宽度因子使用文档单位（通常为毫米），
//! 旋转角与倾斜角一律为弧度；多行文字由若干带格式的段落组成，可设置行距、栏宽与分栏。
//! [`TextStyleManager`] 按名称管理文字样式，创建时自带名为 `Standard` 的默认样式。
//!
//! 与其它模块的关系：文字最终作为 Entity 实体加入 Document 文档，渲染与 IO 模块依据本模块的
//! 对齐方式与样式信息生成 Tessellation 细分与文件输出。

/// 单行文字 [`Text`]、对齐方式与文字构造器 [`TextBuilder`]。
pub mod text;
/// 多行文字的段落、特殊符号、分栏与流向。
pub mod mtext;
/// 文字样式 [`TextStyle`] 与字体度量 [`FontMetrics`]。
pub mod text_style;

pub use text::{Text, TextAlignment, TextVerticalAlignment, TextFormatting, FormattedText, TextBuilder, MText};
pub use mtext::{MTextParagraph, MTextSymbol, MTextBlock, MTextColumn, MTextWithColumns, MTextFlowDirection, MTextBullet};
pub use text_style::{TextStyle, FontMetrics, TextStyleManager};
