//! 布局与打印：图纸规格、打印设置、布局、视口与布局管理器。
//!
//! 核心概念：
//! - `PaperSize`/`PaperUnit`/`PaperOrientation`：图纸规格、计量单位与方向，规格尺寸以毫米给出；
//! - `PlotSettings`：一次打印的完整参数（设备、图纸、比例、旋转、打印样式表等）；
//! - `Layout`：一个布局（或模型空间页），含图纸尺寸、页边距与可打印区域，长度单位毫米；
//! - `Viewport`：布局中的视口，决定模型空间的取景范围、比例以及捕捉/栅格设置；
//! - `LayoutManager`：按下标管理布局（初始为 Layout1、Layout2）与全局视口表，并单独持有模型空间布局。
//!
//! 长度除特别说明外均为毫米；`PlotRotation` 用度，视口取景与捕捉使用模型空间的世界单位。

use serde::{Serialize, Deserialize};
use std::fmt;

/// 图纸方向：纵向或横向；只影响纸张宽高的取法（见 `Layout::paper_width`/`paper_height`）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PaperOrientation {
    /// 纵向：宽取短边、高取长边。
    Portrait,
    /// 横向：宽取长边、高取短边。
    Landscape,
}

impl Default for PaperOrientation {
    fn default() -> Self {
        PaperOrientation::Portrait
    }
}

/// 图纸单位：打印尺寸使用的计量单位。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PaperUnit {
    /// 英寸。
    Inches,
    /// 毫米（默认，与布局的毫米尺寸一致）。
    Millimeters,
    /// 像素：用于屏幕或位图输出，不对应物理尺寸。
    Pixels,
}

impl Default for PaperUnit {
    fn default() -> Self {
        PaperUnit::Millimeters
    }
}

/// 标准图纸规格；`width_mm`/`height_mm` 给出纵向（Portrait）下的短边与长边，单位毫米。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PaperSize {
    /// ISO A0（841 × 1189 mm）。
    A0,
    /// ISO A1（594 × 841 mm）。
    A1,
    /// ISO A2（420 × 594 mm）。
    A2,
    /// ISO A3（297 × 420 mm）。
    A3,
    /// ISO A4（210 × 297 mm），也是默认规格。
    A4,
    /// ISO A5（148 × 210 mm）。
    A5,
    /// 美制 Letter（8.5 × 11 in，215.9 × 279.4 mm）。
    Letter,
    /// 美制 Legal（8.5 × 14 in，215.9 × 355.6 mm）。
    Legal,
    /// 美制 Tabloid（11 × 17 in，279.4 × 431.8 mm）。
    Tabloid,
    /// 美制建筑 Arch A（9 × 12 in，228.6 × 304.8 mm）。
    ArchA,
    /// 美制建筑 Arch B（12 × 18 in，304.8 × 457.2 mm）。
    ArchB,
    /// 美制建筑 Arch C（18 × 24 in，457.2 × 609.6 mm）。
    ArchC,
    /// 美制建筑 Arch D（24 × 36 in，609.6 × 914.4 mm）。
    ArchD,
    /// 美制建筑 Arch E（36 × 48 in，914.4 × 1219.2 mm）。
    ArchE,
    /// 自定义规格：本实现回退为 210 × 297 mm（等同 A4），真实尺寸需由调用方另行维护。
    Custom,
}

impl PaperSize {
    /// 返回该规格在纵向（Portrait）下的宽度，单位为毫米；`Custom` 回退为 210.0。
    #[inline]
    pub fn width_mm(&self) -> f64 {
        match self {
            PaperSize::A0 => 841.0,
            PaperSize::A1 => 594.0,
            PaperSize::A2 => 420.0,
            PaperSize::A3 => 297.0,
            PaperSize::A4 => 210.0,
            PaperSize::A5 => 148.0,
            PaperSize::Letter => 215.9,
            PaperSize::Legal => 215.9,
            PaperSize::Tabloid => 279.4,
            PaperSize::ArchA => 228.6,
            PaperSize::ArchB => 304.8,
            PaperSize::ArchC => 457.2,
            PaperSize::ArchD => 609.6,
            PaperSize::ArchE => 914.4,
            PaperSize::Custom => 210.0,
        }
    }

    /// 返回该规格在纵向（Portrait）下的高度，单位为毫米；`Custom` 回退为 297.0。
    #[inline]
    pub fn height_mm(&self) -> f64 {
        match self {
            PaperSize::A0 => 1189.0,
            PaperSize::A1 => 841.0,
            PaperSize::A2 => 594.0,
            PaperSize::A3 => 420.0,
            PaperSize::A4 => 297.0,
            PaperSize::A5 => 210.0,
            PaperSize::Letter => 279.4,
            PaperSize::Legal => 355.6,
            PaperSize::Tabloid => 431.8,
            PaperSize::ArchA => 304.8,
            PaperSize::ArchB => 457.2,
            PaperSize::ArchC => 609.6,
            PaperSize::ArchD => 914.4,
            PaperSize::ArchE => 1219.2,
            PaperSize::Custom => 297.0,
        }
    }

    /// 返回带尺寸说明的显示名（如 `"ISO A4 (210 x 297 mm)"`、`"Letter (8.5 x 11 in)"`）。
    #[inline]
    pub fn name(&self) -> String {
        match self {
            PaperSize::A0 => "ISO A0 (841 x 1189 mm)".to_string(),
            PaperSize::A1 => "ISO A1 (594 x 841 mm)".to_string(),
            PaperSize::A2 => "ISO A2 (420 x 594 mm)".to_string(),
            PaperSize::A3 => "ISO A3 (297 x 420 mm)".to_string(),
            PaperSize::A4 => "ISO A4 (210 x 297 mm)".to_string(),
            PaperSize::A5 => "ISO A5 (148 x 210 mm)".to_string(),
            PaperSize::Letter => "Letter (8.5 x 11 in)".to_string(),
            PaperSize::Legal => "Legal (8.5 x 14 in)".to_string(),
            PaperSize::Tabloid => "Tabloid (11 x 17 in)".to_string(),
            PaperSize::ArchA => "Arch A (9 x 12 in)".to_string(),
            PaperSize::ArchB => "Arch B (12 x 18 in)".to_string(),
            PaperSize::ArchC => "Arch C (18 x 24 in)".to_string(),
            PaperSize::ArchD => "Arch D (24 x 36 in)".to_string(),
            PaperSize::ArchE => "Arch E (36 x 48 in)".to_string(),
            PaperSize::Custom => "Custom".to_string(),
        }
    }
}

impl Default for PaperSize {
    fn default() -> Self {
        PaperSize::A4
    }
}

/// 打印旋转：仅支持 0°、90°、180°、270° 四档。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PlotRotation {
    /// 不旋转（0°）。
    Degrees0,
    /// 逆时针旋转 90°。
    Degrees90,
    /// 旋转 180°。
    Degrees180,
    /// 旋转 270°（即顺时针 90°）。
    Degrees270,
}

impl Default for PlotRotation {
    fn default() -> Self {
        PlotRotation::Degrees0
    }
}

impl PlotRotation {
    /// 按角度就近取整到 0° / 90° / 180° / 270° 四档。
    ///
    /// - `degrees`：任意角度（度），负值与超过 360° 的值会先归一化到 `[0, 360)`。
    /// - 返回：最接近的打印旋转档位；边界值 45°/135°/225°/315° 取较大的档位。
    #[inline]
    pub fn from_degrees(degrees: f64) -> Self {
        // 归一化到 [0, 360)，注意 Rust 的 % 对负数返回负值
        let normalized = (degrees % 360.0 + 360.0) % 360.0;
        if normalized < 45.0 {
            PlotRotation::Degrees0
        } else if normalized < 135.0 {
            PlotRotation::Degrees90
        } else if normalized < 225.0 {
            PlotRotation::Degrees180
        } else if normalized < 315.0 {
            PlotRotation::Degrees270
        } else {
            PlotRotation::Degrees0
        }
    }

    /// 返回该档位对应的角度值（0.0、90.0、180.0 或 270.0），单位：度。
    #[inline]
    pub fn to_degrees(&self) -> f64 {
        match self {
            PlotRotation::Degrees0 => 0.0,
            PlotRotation::Degrees90 => 90.0,
            PlotRotation::Degrees180 => 180.0,
            PlotRotation::Degrees270 => 270.0,
        }
    }
}

/// 打印范围类型：决定打印时取哪一块区域。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PlotType {
    /// 按当前布局的图纸范围打印（默认）。
    Layout,
    /// 按图形实际外接范围打印。
    Extents,
    /// 按图形界限打印。
    Limits,
    /// 按命名视图的范围打印。
    View,
    /// 按用户框选窗口打印，窗口由 `PlotSettings::plot_window_area` 给出。
    Window,
    /// 按当前屏幕显示范围打印。
    Display,
}

impl Default for PlotType {
    fn default() -> Self {
        PlotType::Layout
    }
}

/// 着色打印方式：决定三维/实体内容在图纸上的显示效果。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ShadePlotMode {
    /// 按屏幕显示效果输出（默认）。
    AsDisplayed,
    /// 仅输出线框。
    Wireframe,
    /// 线框并消除隐藏线。
    Hidden,
    /// 按渲染结果输出。
    Rendered,
}

impl Default for ShadePlotMode {
    fn default() -> Self {
        ShadePlotMode::AsDisplayed
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ScaleType {
    ScaleToFit,
    Scale1To128,
    Scale1To64,
    Scale1To32,
    Scale1To16,
    Scale3To32,
    Scale1To8,
    Scale3To16,
    Scale1To4,
    Scale3To8,
    Scale1To2,
    Scale3To4,
    Scale1To1,
    Scale3To2,
    Scale2To1,
    Custom,
}

impl ScaleType {
    #[inline]
    pub fn from_ratio(numerator: f64, denominator: f64) -> Self {
        let ratio = numerator / denominator;
        let tolerance = 0.001;

        if (ratio - 1.0).abs() < tolerance {
            ScaleType::ScaleToFit
        } else if (ratio - 1.0 / 128.0).abs() < tolerance {
            ScaleType::Scale1To128
        } else if (ratio - 1.0 / 64.0).abs() < tolerance {
            ScaleType::Scale1To64
        } else if (ratio - 1.0 / 32.0).abs() < tolerance {
            ScaleType::Scale1To32
        } else if (ratio - 1.0 / 16.0).abs() < tolerance {
            ScaleType::Scale1To16
        } else if (ratio - 3.0 / 32.0).abs() < tolerance {
            ScaleType::Scale3To32
        } else if (ratio - 1.0 / 8.0).abs() < tolerance {
            ScaleType::Scale1To8
        } else if (ratio - 3.0 / 16.0).abs() < tolerance {
            ScaleType::Scale3To16
        } else if (ratio - 1.0 / 4.0).abs() < tolerance {
            ScaleType::Scale1To4
        } else if (ratio - 3.0 / 8.0).abs() < tolerance {
            ScaleType::Scale3To8
        } else if (ratio - 1.0 / 2.0).abs() < tolerance {
            ScaleType::Scale1To2
        } else if (ratio - 3.0 / 4.0).abs() < tolerance {
            ScaleType::Scale3To4
        } else if (ratio - 1.0).abs() < tolerance {
            ScaleType::Scale1To1
        } else if (ratio - 3.0 / 2.0).abs() < tolerance {
            ScaleType::Scale3To2
        } else if (ratio - 2.0).abs() < tolerance {
            ScaleType::Scale2To1
        } else {
            ScaleType::Custom
        }
    }

    #[inline]
    pub fn scale_factor(&self) -> (f64, f64) {
        match self {
            ScaleType::ScaleToFit => (1.0, 1.0),
            ScaleType::Scale1To128 => (1.0, 128.0),
            ScaleType::Scale1To64 => (1.0, 64.0),
            ScaleType::Scale1To32 => (1.0, 32.0),
            ScaleType::Scale1To16 => (1.0, 16.0),
            ScaleType::Scale3To32 => (3.0, 32.0),
            ScaleType::Scale1To8 => (1.0, 8.0),
            ScaleType::Scale3To16 => (3.0, 16.0),
            ScaleType::Scale1To4 => (1.0, 4.0),
            ScaleType::Scale3To8 => (3.0, 8.0),
            ScaleType::Scale1To2 => (1.0, 2.0),
            ScaleType::Scale3To4 => (3.0, 4.0),
            ScaleType::Scale1To1 => (1.0, 1.0),
            ScaleType::Scale3To2 => (3.0, 2.0),
            ScaleType::Scale2To1 => (2.0, 1.0),
            ScaleType::Custom => (1.0, 1.0),
        }
    }

    #[inline]
    pub fn as_ratio(&self) -> String {
        let (num, den) = self.scale_factor();
        match self {
            ScaleType::ScaleToFit => "Scale to fit".to_string(),
            ScaleType::Custom => format!("{}:{}", num, den),
            _ => format!("1:{}", den as u32),
        }
    }
}

impl Default for ScaleType {
    fn default() -> Self {
        ScaleType::ScaleToFit
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LineCapStyle {
    Butt,
    Round,
    Square,
}

impl Default for LineCapStyle {
    fn default() -> Self {
        LineCapStyle::Round
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LineJoinStyle {
    Miter,
    Round,
    Bevel,
}

impl Default for LineJoinStyle {
    fn default() -> Self {
        LineJoinStyle::Round
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FillMode {
    UseColor,
    UseObject,
    Outlined,
    Tiled,
    Solid,
}

impl Default for FillMode {
    fn default() -> Self {
        FillMode::UseObject
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlotStyle {
    pub name: String,
    pub color: Option<u32>,
    pub grayscale: bool,
    pub screening: u8,
    pub end_cap_style: LineCapStyle,
    pub join_style: LineJoinStyle,
    pub lineweight: super::line_type::Lineweight,
    pub linetype_name: String,
    pub fill_mode: FillMode,
    pub style_name: String,
}

impl Default for PlotStyle {
    fn default() -> Self {
        Self {
            name: "Normal".to_string(),
            color: None,
            grayscale: false,
            screening: 100,
            end_cap_style: LineCapStyle::Round,
            join_style: LineJoinStyle::Round,
            lineweight: super::line_type::Lineweight::Default,
            linetype_name: "Continuous".to_string(),
            fill_mode: FillMode::UseObject,
            style_name: "Normal".to_string(),
        }
    }
}

impl PlotStyle {
    #[inline]
    pub fn from_color(color: u32) -> Self {
        Self {
            name: format!("Color {}", color),
            color: Some(color),
            ..Default::default()
        }
    }

    #[inline]
    pub fn is_override(&self) -> bool {
        self.color.is_some() ||
        self.lineweight != super::line_type::Lineweight::ByBlock ||
        !self.linetype_name.is_empty()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlotStyleTable {
    pub name: String,
    pub description: String,
    pub styles: Vec<PlotStyle>,
}

impl Default for PlotStyleTable {
    fn default() -> Self {
        let mut table = Self {
            name: "acad".to_string(),
            description: "AutoCAD 打印样式表".to_string(),
            styles: Vec::new(),
        };
        for color in 0..=255 {
            table.styles.push(PlotStyle::from_color(color));
        }
        table
    }
}

impl PlotStyleTable {
    #[inline]
    pub fn by_color(&self, color: u32) -> Option<&PlotStyle> {
        self.styles.get(color as usize)
    }

    #[inline]
    pub fn by_color_mut(&mut self, color: u32) -> Option<&mut PlotStyle> {
        self.styles.get_mut(color as usize)
    }

    #[inline]
    pub fn add_style(&mut self, style: PlotStyle) {
        self.styles.push(style);
    }

    #[inline]
    pub fn remove_style(&mut self, index: usize) -> bool {
        if index < self.styles.len() {
            self.styles.remove(index);
            true
        } else {
            false
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlotWindowArea {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

impl Default for PlotWindowArea {
    fn default() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            width: 210.0,
            height: 297.0,
        }
    }
}

impl PlotWindowArea {
    #[inline]
    pub fn from_points(p1: super::geometry::Point, p2: super::geometry::Point) -> Self {
        let min_x = p1.x.min(p2.x);
        let min_y = p1.y.min(p2.y);
        let max_x = p1.x.max(p2.x);
        let max_y = p1.y.max(p2.y);
        Self {
            x: min_x,
            y: min_y,
            width: max_x - min_x,
            height: max_y - min_y,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlotSettings {
    pub plot_device: String,
    pub paper_size: PaperSize,
    pub canonical_media_name: String,
    pub plot_origin: (f64, f64),
    pub plot_window_area: PlotWindowArea,
    pub paper_units: PaperUnit,
    pub plot_rotation: PlotRotation,
    pub plot_type: PlotType,
    pub current_style_sheet: String,
    pub shade_plot_mode: ShadePlotMode,
    pub scale_type: ScaleType,
    pub scale_factor: f64,
    pub custom_scale: (f64, f64),
    pub print_lineweights: bool,
    pub scale_lineweights_with_plot: bool,
    pub plot_with_plot_styles: bool,
    pub plot_with_orig_lineweights: bool,
    pub plot_object_lineweights: bool,
}

impl Default for PlotSettings {
    fn default() -> Self {
        Self {
            plot_device: "None".to_string(),
            paper_size: PaperSize::A4,
            canonical_media_name: "A4 (210.00 x 297.00 mm)".to_string(),
            plot_origin: (5.0, 5.0),
            plot_window_area: PlotWindowArea::default(),
            paper_units: PaperUnit::Millimeters,
            plot_rotation: PlotRotation::Degrees0,
            plot_type: PlotType::Layout,
            current_style_sheet: "acad.stb".to_string(),
            shade_plot_mode: ShadePlotMode::AsDisplayed,
            scale_type: ScaleType::ScaleToFit,
            scale_factor: 1.0,
            custom_scale: (1.0, 1.0),
            print_lineweights: false,
            scale_lineweights_with_plot: false,
            plot_with_plot_styles: true,
            plot_with_orig_lineweights: false,
            plot_object_lineweights: true,
        }
    }
}

impl PlotSettings {
    #[inline]
    pub fn set_scale(&mut self, scale_type: ScaleType, custom: Option<(f64, f64)>) {
        self.scale_type = scale_type;
        if let Some(custom_scale) = custom {
            self.custom_scale = custom_scale;
        }
        let (num, den) = scale_type.scale_factor();
        self.scale_factor = if den > 0.0 { num / den } else { 1.0 };
    }

    #[inline]
    pub fn effective_scale(&self) -> f64 {
        match self.scale_type {
            ScaleType::ScaleToFit => self.scale_factor,
            ScaleType::Custom => {
                if self.custom_scale.1 != 0.0 {
                    self.custom_scale.0 / self.custom_scale.1
                } else {
                    1.0
                }
            }
            _ => {
                let (num, den) = self.scale_type.scale_factor();
                if den != 0.0 { num / den } else { 1.0 }
            }
        }
    }

    #[inline]
    pub fn paper_to_model_scale(&self) -> f64 {
        self.effective_scale()
    }

    #[inline]
    pub fn model_to_paper_length(&self, model_length: f64) -> f64 {
        model_length / self.effective_scale()
    }

    #[inline]
    pub fn paper_to_model_length(&self, paper_length: f64) -> f64 {
        paper_length * self.effective_scale()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Layout {
    pub name: String,
    pub paper_size: PaperSize,
    pub margins: (f64, f64, f64, f64),
    pub printable_area: (f64, f64),
    pub orientation: PaperOrientation,
    pub plot_style: Option<String>,
    pub canonical_media_name: String,
    pub tab_order: u32,
    pub is_current: bool,
    pub is_plottable: bool,
}

impl Default for Layout {
    fn default() -> Self {
        Self::new("Layout1")
    }
}

impl Layout {
    #[inline]
    pub fn new(name: &str) -> Self {
        let paper_size = PaperSize::A4;
        let margins = (5.0, 5.0, 5.0, 5.0);
        let printable_width = paper_size.width_mm() - margins.0 - margins.1;
        let printable_height = paper_size.height_mm() - margins.2 - margins.3;

        Self {
            name: name.to_string(),
            paper_size,
            margins,
            printable_area: (printable_width, printable_height),
            orientation: PaperOrientation::Portrait,
            plot_style: None,
            canonical_media_name: paper_size.name(),
            tab_order: 1,
            is_current: false,
            is_plottable: true,
        }
    }

    #[inline]
    pub fn set_paper_size(&mut self, size: PaperSize) {
        self.paper_size = size;
        self.update_printable_area();
    }

    #[inline]
    pub fn set_margins(&mut self, left: f64, right: f64, top: f64, bottom: f64) {
        self.margins = (left, right, top, bottom);
        self.update_printable_area();
    }

    #[inline]
    pub fn set_orientation(&mut self, orientation: PaperOrientation) {
        self.orientation = orientation;
    }

    fn update_printable_area(&mut self) {
        let width = self.paper_size.width_mm() - self.margins.0 - self.margins.1;
        let height = self.paper_size.height_mm() - self.margins.2 - self.margins.3;
        self.printable_area = (width.max(0.0), height.max(0.0));
    }

    #[inline]
    pub fn paper_width(&self) -> f64 {
        match self.orientation {
            PaperOrientation::Portrait => self.paper_size.width_mm(),
            PaperOrientation::Landscape => self.paper_size.height_mm(),
        }
    }

    #[inline]
    pub fn paper_height(&self) -> f64 {
        match self.orientation {
            PaperOrientation::Portrait => self.paper_size.height_mm(),
            PaperOrientation::Landscape => self.paper_size.width_mm(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Viewport {
    pub center: super::geometry::Point,
    pub width: f64,
    pub height: f64,
    pub view_center: super::geometry::Point,
    pub view_height: f64,
    pub view_scale: f64,
    pub snap_base: super::geometry::Point,
    pub snap_spacing: (f64, f64),
    pub grid_spacing: (f64, f64),
    pub is_grid_display: bool,
    pub is_grid_snap: bool,
    pub is_locked: bool,
    pub display_locked: bool,
    pub id: u32,
}

impl Default for Viewport {
    fn default() -> Self {
        Self::new()
    }
}

impl Viewport {
    #[inline]
    pub fn new() -> Self {
        Self {
            center: super::geometry::Point::origin(),
            width: 100.0,
            height: 100.0,
            view_center: super::geometry::Point::origin(),
            view_height: 100.0,
            view_scale: 1.0,
            snap_base: super::geometry::Point::origin(),
            snap_spacing: (1.0, 1.0),
            grid_spacing: (10.0, 10.0),
            is_grid_display: false,
            is_grid_snap: false,
            is_locked: false,
            display_locked: false,
            id: 0,
        }
    }

    #[inline]
    pub fn zoom_to_extents(&mut self, extents: &(super::geometry::Point, super::geometry::Point)) {
        let width = (extents.1.x - extents.0.x).abs();
        let height = (extents.1.y - extents.0.y).abs();
        let center_x = (extents.0.x + extents.1.x) / 2.0;
        let center_y = (extents.0.y + extents.1.y) / 2.0;

        self.view_center = super::geometry::Point::new(center_x, center_y, 0.0);
        self.view_height = height * 1.1;

        if width / height > self.width / self.height {
            self.view_height = width * self.height / self.width / 0.9;
        }
    }

    #[inline]
    pub fn zoom(&mut self, factor: f64) {
        self.view_height /= factor;
        self.view_scale /= factor;
    }

    #[inline]
    pub fn pan(&mut self, delta_x: f64, delta_y: f64) {
        self.view_center.x += delta_x / self.view_scale;
        self.view_center.y += delta_y / self.view_scale;
    }

    #[inline]
    pub fn set_scale(&mut self, scale: f64) {
        self.view_scale = scale;
    }

    #[inline]
    pub fn border_points(&self) -> (super::geometry::Point, super::geometry::Point) {
        let half_width = self.width / 2.0;
        let half_height = self.height / 2.0;
        (
            super::geometry::Point::new(
                self.center.x - half_width,
                self.center.y - half_height,
                0.0,
            ),
            super::geometry::Point::new(
                self.center.x + half_width,
                self.center.y + half_height,
                0.0,
            ),
        )
    }

    #[inline]
    pub fn contains(&self, point: &super::geometry::Point) -> bool {
        let (min, max) = self.border_points();
        point.x >= min.x && point.x <= max.x && point.y >= min.y && point.y <= max.y
    }
}

#[derive(Debug, Clone)]
pub struct LayoutManager {
    layouts: Vec<Layout>,
    model_space: Layout,
    current_layout: usize,
    viewports: std::collections::HashMap<usize, Viewport>,
    active_viewport: u32,
    next_viewport_id: u32,
}

impl Default for LayoutManager {
    fn default() -> Self {
        Self::new()
    }
}

impl LayoutManager {
    #[inline]
    pub fn new() -> Self {
        let mut manager = Self {
            layouts: Vec::new(),
            model_space: Layout::new("Model"),
            current_layout: usize::MAX,
            viewports: std::collections::HashMap::new(),
            active_viewport: 0,
            next_viewport_id: 1,
        };
        manager.model_space.is_plottable = false;
        manager.layouts.push(Layout::new("Layout1"));
        manager.layouts.push(Layout::new("Layout2"));
        manager.set_current(0);
        manager
    }

    #[inline]
    pub fn add_layout(&mut self, name: &str) -> usize {
        let layout = Layout::new(name);
        let index = self.layouts.len();
        self.layouts.push(layout);
        self.set_current(index);
        index
    }

    #[inline]
    pub fn remove_layout(&mut self, index: usize) -> bool {
        if index >= self.layouts.len() || index == self.current_layout {
            return false;
        }
        self.layouts.remove(index);
        if self.current_layout > index {
            self.current_layout -= 1;
        }
        true
    }

    #[inline]
    pub fn set_current(&mut self, index: usize) -> bool {
        if index < self.layouts.len() {
            self.current_layout = index;
            self.layouts[index].is_current = true;
            true
        } else {
            false
        }
    }

    #[inline]
    pub fn current(&self) -> &Layout {
        if self.current_layout < self.layouts.len() {
            &self.layouts[self.current_layout]
        } else {
            &self.model_space
        }
    }

    #[inline]
    pub fn current_mut(&mut self) -> &mut Layout {
        if self.current_layout < self.layouts.len() {
            &mut self.layouts[self.current_layout]
        } else {
            &mut self.model_space
        }
    }

    #[inline]
    pub fn model_space(&self) -> &Layout {
        &self.model_space
    }

    #[inline]
    pub fn model_space_mut(&mut self) -> &mut Layout {
        &mut self.model_space
    }

    #[inline]
    pub fn get_layout(&self, index: usize) -> Option<&Layout> {
        self.layouts.get(index)
    }

    #[inline]
    pub fn get_layout_mut(&mut self, index: usize) -> Option<&mut Layout> {
        self.layouts.get_mut(index)
    }

    #[inline]
    pub fn layout_count(&self) -> usize {
        self.layouts.len()
    }

    #[inline]
    pub fn layouts(&self) -> &[Layout] {
        &self.layouts
    }

    #[inline]
    pub fn add_viewport(&mut self, mut viewport: Viewport) -> u32 {
        let id = self.next_viewport_id;
        self.next_viewport_id += 1;
        viewport.id = id;
        self.viewports.insert(id as usize, viewport);
        id
    }

    #[inline]
    pub fn remove_viewport(&mut self, id: u32) -> bool {
        self.viewports.remove(&(id as usize)).is_some()
    }

    #[inline]
    pub fn get_viewport(&self, id: u32) -> Option<&Viewport> {
        self.viewports.get(&(id as usize))
    }

    #[inline]
    pub fn get_viewport_mut(&mut self, id: u32) -> Option<&mut Viewport> {
        self.viewports.get_mut(&(id as usize))
    }

    #[inline]
    pub fn viewports(&self) -> &std::collections::HashMap<usize, Viewport> {
        &self.viewports
    }

    #[inline]
    pub fn set_active_viewport(&mut self, id: u32) {
        if self.viewports.contains_key(&(id as usize)) {
            self.active_viewport = id;
        }
    }

    #[inline]
    pub fn active_viewport(&self) -> u32 {
        self.active_viewport
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometry::Point;

    #[test]
    fn test_paper_size_dimensions() {
        assert!((PaperSize::A4.width_mm() - 210.0).abs() < 1e-10);
        assert!((PaperSize::A4.height_mm() - 297.0).abs() < 1e-10);
        assert!((PaperSize::A3.width_mm() - 297.0).abs() < 1e-10);
        assert!((PaperSize::A3.height_mm() - 420.0).abs() < 1e-10);
    }

    #[test]
    fn test_plot_rotation() {
        assert_eq!(PlotRotation::from_degrees(0.0), PlotRotation::Degrees0);
        assert_eq!(PlotRotation::from_degrees(90.0), PlotRotation::Degrees90);
        assert_eq!(PlotRotation::from_degrees(180.0), PlotRotation::Degrees180);
        assert_eq!(PlotRotation::from_degrees(270.0), PlotRotation::Degrees270);
    }

    #[test]
    fn test_scale_type() {
        assert_eq!(ScaleType::Scale1To128.scale_factor(), (1.0, 128.0));
        assert_eq!(ScaleType::ScaleToFit.scale_factor(), (1.0, 1.0));
    }

    #[test]
    fn test_plot_settings_scale() {
        let mut settings = PlotSettings::default();
        settings.set_scale(ScaleType::Scale1To64, None);
        assert!((settings.effective_scale() - 0.015625).abs() < 1e-10);
    }

    #[test]
    fn test_layout_printable_area() {
        let mut layout = Layout::new("Test");
        layout.set_margins(10.0, 10.0, 10.0, 10.0);
        let width = layout.paper_size.width_mm() - 20.0;
        let height = layout.paper_size.height_mm() - 20.0;
        assert!((layout.printable_area.0 - width).abs() < 1e-10);
        assert!((layout.printable_area.1 - height).abs() < 1e-10);
    }

    #[test]
    fn test_viewport_zoom() {
        let mut viewport = Viewport::new();
        viewport.view_height = 100.0;
        viewport.zoom(2.0);
        assert!((viewport.view_height - 50.0).abs() < 1e-10);
    }

    #[test]
    fn test_layout_manager() {
        let manager = LayoutManager::new();
        assert_eq!(manager.layout_count(), 2);
        assert!(manager.current().name.contains("Layout"));
    }

    #[test]
    fn test_plot_style_table() {
        let table = PlotStyleTable::default();
        assert_eq!(table.styles.len(), 256);
        assert!(table.by_color(5).is_some());
    }

    #[test]
    fn test_viewport_contains() {
        let viewport = Viewport {
            center: Point::new(100.0, 100.0, 0.0),
            width: 100.0,
            height: 100.0,
            ..Default::default()
        };
        assert!(viewport.contains(&Point::new(100.0, 100.0, 0.0)));
        assert!(viewport.contains(&Point::new(50.0, 50.0, 0.0)));
        assert!(!viewport.contains(&Point::new(200.0, 200.0, 0.0)));
    }

    #[test]
    fn test_viewport_border_points() {
        let viewport = Viewport {
            center: Point::new(100.0, 100.0, 0.0),
            width: 50.0,
            height: 50.0,
            ..Default::default()
        };
        let (min, max) = viewport.border_points();
        assert!((min.x - 75.0).abs() < 1e-10);
        assert!((min.y - 75.0).abs() < 1e-10);
        assert!((max.x - 125.0).abs() < 1e-10);
        assert!((max.y - 125.0).abs() < 1e-10);
    }
}
