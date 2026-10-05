//! SVG 文本渲染实现：`SVGRendererImpl` 按 `Renderer` 接口把每次绘制调用拼接为一个 SVG 元素，
//! 最终由 `get_svg_content` 生成带 `viewBox` 的完整文档，供导出与网页预览复用。
//! 数值原样写入 SVG 用户坐标（原点在左上、y 轴向下），本模块不做世界坐标到屏幕坐标的变换。

use crate::geometry::{Point, Line, Circle, Arc, Ellipse, Polyline};
use crate::render::{RenderStyle, Renderer};
use std::f64::consts::PI;

/// 基于字符串拼接的 SVG 渲染器：把逐个图元累积为 SVG 元素列表。
/// 调用方提供的数值直接写入 SVG 用户坐标；输出只能通过 `get_svg_content` 取回，
/// `present` 与 `flush` 为空实现，不影响已累积的内容。
pub struct SVGRendererImpl {
    width: usize,
    height: usize,
    elements: Vec<String>,
    current_style: RenderStyle,
}

impl SVGRendererImpl {
    /// 创建指定画布尺寸的渲染器：图元列表为空，当前样式为 `RenderStyle::default()`。
    /// - `width`、`height`：画布尺寸（像素），同时写入 SVG 的 `width`、`height` 与 `viewBox`。
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            elements: Vec::new(),
            current_style: RenderStyle::default(),
        }
    }

    /// 生成完整 SVG 文本：根标签带 `width`、`height` 与 `viewBox="0 0 宽 高"`，后面按绘制顺序拼接所有元素。
    /// 返回新构造的字符串，可反复调用；元素列表与当前样式不会被修改。
    /// # 示例
    /// ```
    /// use cadrs::render::svg_renderer::SVGRendererImpl;
    /// let renderer = SVGRendererImpl::new(800, 600);
    /// assert!(renderer.get_svg_content().contains("<svg"));
    /// ```
    pub fn get_svg_content(&self) -> String {
        let mut svg = format!(
            r#"<svg xmlns="http://www.w3.org/2000/svg" width="{}" height="{}" viewBox="0 0 {} {}">"#,
            self.width, self.height, self.width, self.height
        );
        
        for element in &self.elements {
            svg.push_str(element);
        }
        
        svg.push_str("</svg>");
        svg
    }

    fn color_to_rgb(&self, color: (u8, u8, u8)) -> String {
        format!("rgb({},{},{})", color.0, color.1, color.2)
    }

    fn style_to_stroke(&self, style: &RenderStyle) -> String {
        let mut stroke = format!(r#"stroke="{}""#, self.color_to_rgb(style.color));
        
        match style.line_pattern {
            super::renderer::LinePattern::Solid => {},
            super::renderer::LinePattern::Dashed => {
                stroke.push_str(r#" stroke-dasharray="5,5""#);
            }
            super::renderer::LinePattern::Dotted => {
                stroke.push_str(r#" stroke-dasharray="2,2""#);
            }
            super::renderer::LinePattern::DashDot => {
                stroke.push_str(r#" stroke-dasharray="5,2,2""#);
            }
            super::renderer::LinePattern::Custom(ref pattern) => {
                let pattern_str = pattern.iter()
                    .map(|f| f.to_string())
                    .collect::<Vec<_>>()
                    .join(",");
                stroke.push_str(&format!(r#" stroke-dasharray="{}""#, pattern_str));
            }
        }
        
        stroke
    }

    fn format_line_style(&self, style: &RenderStyle) -> String {
        let mut attrs = format!(r#"stroke-width="{}" fill="none""#, style.line_width);
        attrs.push_str(&format!(r#" stroke="{}""#, self.color_to_rgb(style.color)));
        
        match style.line_pattern {
            super::renderer::LinePattern::Solid => {},
            super::renderer::LinePattern::Dashed => {
                attrs.push_str(r#" stroke-dasharray="5,5""#);
            }
            super::renderer::LinePattern::Dotted => {
                attrs.push_str(r#" stroke-dasharray="2,2""#);
            }
            _ => {},
        }
        
        if style.anti_aliasing {
            attrs.push_str(r#" shape-rendering="geometricPrecision""#);
        }
        
        attrs
    }
}

impl Renderer for SVGRendererImpl {
    fn initialize(&mut self) -> Result<(), String> {
        self.elements.clear();
        Ok(())
    }

    fn clear(&mut self, color: (u8, u8, u8)) {
        self.elements.push(format!(
            r#"<rect width="100%" height="100%" fill="{}"/>"#,
            self.color_to_rgb(color)
        ));
    }

    fn draw_line(&mut self, x1: f64, y1: f64, x2: f64, y2: f64, style: &RenderStyle) {
        let attrs = self.format_line_style(style);
        self.elements.push(format!(
            r#"<line x1="{:.2}" y1="{:.2}" x2="{:.2}" y2="{:.2}" {}"/>"#,
            x1, y1, x2, y2, attrs
        ));
    }

    fn draw_circle(&mut self, cx: f64, cy: f64, r: f64, style: &RenderStyle) {
        let fill = match style.fill {
            Some(color) => self.color_to_rgb(color),
            None => "none".to_string(),
        };
        
        self.elements.push(format!(
            r#"<circle cx="{:.2}" cy="{:.2}" r="{:.2}" fill="{}" stroke="{}" stroke-width="{:.2}"/>"#,
            cx, cy, r, fill, self.color_to_rgb(style.color), style.line_width
        ));
    }

    fn draw_arc(&mut self, cx: f64, cy: f64, r: f64, start_angle: f64, end_angle: f64, style: &RenderStyle) {
        let start_rad = start_angle * PI / 180.0;
        let end_rad = end_angle * PI / 180.0;
        
        let x1 = cx + r * start_rad.cos();
        let y1 = cy + r * start_rad.sin();
        let x2 = cx + r * end_rad.cos();
        let y2 = cy + r * end_rad.sin();
        
        let large_arc = if (end_angle - start_angle).abs() > 180.0 { 1 } else { 0 };
        
        let sweep = if start_angle < end_angle { 1 } else { 0 };
        
        let attrs = self.format_line_style(style);
        self.elements.push(format!(
            r#"<path d="M {:.2},{:.2} A {:.2},{:.2} 0 {} {} {:.2},{:.2}" {}"/>"#,
            x1, y1, r, r, large_arc, sweep, x2, y2, attrs
        ));
    }

    fn draw_polyline(&mut self, points: &[(f64, f64)], style: &RenderStyle) {
        if points.is_empty() {
            return;
        }
        
        let points_str: Vec<String> = points.iter()
            .map(|(x, y)| format!("{:.2},{:.2}", x, y))
            .collect();
        
        let attrs = self.format_line_style(style);
        self.elements.push(format!(
            r#"<polyline points="{}" {}"/>"#,
            points_str.join(" "), attrs
        ));
    }

    fn draw_point(&mut self, x: f64, y: f64, size: f64, style: &RenderStyle) {
        self.elements.push(format!(
            r#"<circle cx="{:.2}" cy="{:.2}" r="{:.2}" fill="{}"/>"#,
            x, y, size / 2.0, self.color_to_rgb(style.color)
        ));
    }

    fn draw_text(&mut self, x: f64, y: f64, text: &str, size: f64, rotation: f64, style: &RenderStyle) {
        let rotation_rad = rotation * PI / 180.0;
        let transform = if rotation.abs() > 0.001 {
            format!(r#" transform="rotate({:.2} {:.2},{:.2})""#, -rotation, x, y)
        } else {
            String::new()
        };
        
        self.elements.push(format!(
            r#"<text x="{:.2}" y="{:.2}" font-size="{:.2}" fill="{}"{}>{}</text>"#,
            x, y, size, self.color_to_rgb(style.color), transform, text
        ));
    }
    
    fn draw_text_enhanced(
        &mut self,
        x: f64,
        y: f64,
        text: &str,
        font_name: &str,
        size: f64,
        rotation: f64,
        width_factor: f64,
        style: &RenderStyle,
        bold: bool,
        italic: bool,
        underline: bool,
        alignment: i32,
    ) {
        let rotation_rad = rotation * PI / 180.0;
        let transform = if rotation.abs() > 0.001 {
            format!(r#" transform="rotate({:.2} {:.2},{:.2})""#, -rotation, x, y)
        } else {
            String::new()
        };
        
        let font_style = if italic { "italic" } else { "normal" };
        let font_weight = if bold { "bold" } else { "normal" };
        let text_decoration = if underline { "underline" } else { "none" };
        
        let text_anchor = match alignment {
            1 => "middle",
            2 => "end",
            _ => "start",
        };
        
        self.elements.push(format!(
            r#"<text x="{:.2}" y="{:.2}" font-family="{}" font-size="{:.2}" font-style="{}" font-weight="{}" text-decoration="{}" text-anchor="{}" fill="{}"{}>{}</text>"#,
            x, y, font_name, size, font_style, font_weight, text_decoration, text_anchor, self.color_to_rgb(style.color), transform, text
        ));
    }

    fn present(&mut self) {
    }

    fn flush(&mut self) {
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_svg_renderer_creation() {
        let renderer = SVGRendererImpl::new(800, 600);
        assert!(renderer.get_svg_content().contains("<svg"));
    }

    #[test]
    fn test_svg_renderer_draw_line() {
        let mut renderer = SVGRendererImpl::new(800, 600);
        renderer.initialize().unwrap();
        renderer.clear((255, 255, 255));
        renderer.draw_line(0.0, 0.0, 100.0, 100.0, &RenderStyle::default());
        
        let content = renderer.get_svg_content();
        assert!(content.contains("<line"));
        assert!(content.contains("100.00,100.00"));
    }

    #[test]
    fn test_svg_renderer_draw_circle() {
        let mut renderer = SVGRendererImpl::new(800, 600);
        renderer.initialize().unwrap();
        renderer.draw_circle(100.0, 100.0, 50.0, &RenderStyle::default());
        
        let content = renderer.get_svg_content();
        assert!(content.contains("<circle"));
        assert!(content.contains("100.00"));
        assert!(content.contains("50.00"));
    }
}
