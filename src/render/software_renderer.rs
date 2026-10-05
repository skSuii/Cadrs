//! 具体渲染后端实现：CPU 软件光栅后端（结果写入内存帧缓冲）与 SVG 文本后端，
//! 另含 WebGL、Direct2D、Metal、OpenGL 的占位实现（仅在其目标平台编译，`is_available` 均为 false）。
//! 接口与能力描述来自 `render_backend` 模块，绘制数据来自 `renderer` 模块。

use crate::geometry::{Point, Line, Circle, Arc, Ellipse, Polyline};
use crate::render::{RenderBackend, RenderBuffer, RenderStyle};
use super::render_backend::BackendCapabilities;

/// CPU 软件光栅后端：把绘制结果写入内存 `RenderBuffer`，全平台可用。
/// 注意 `render_backend` 模块中另有一份同名类型，二者彼此独立，混用会造成类型不匹配。
pub struct SoftwareRenderer {
    width: usize,
    height: usize,
    buffer: RenderBuffer,
}

impl SoftwareRenderer {
    /// 创建默认后端：输出 800×600 像素，并分配同尺寸帧缓冲（初始为黑色）。
    pub fn new() -> Self {
        Self {
            width: 800,
            height: 600,
            buffer: RenderBuffer::new(800, 600),
        }
    }

    /// 创建指定像素尺寸的后端及其帧缓冲。
    /// - `width`、`height`：像素列数与行数，不做上限校验。
    pub fn with_size(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            buffer: RenderBuffer::new(width, height),
        }
    }
}

impl RenderBackend for SoftwareRenderer {
    fn name(&self) -> &str {
        "Software"
    }

    fn is_available(&self) -> bool {
        true
    }

    fn initialize(&mut self) -> Result<(), String> {
        self.buffer = RenderBuffer::new(self.width, self.height);
        Ok(())
    }

    fn set_size(&mut self, width: usize, height: usize) {
        self.width = width;
        self.height = height;
        self.buffer = RenderBuffer::new(width, height);
    }

    fn clear(&mut self, color: (u8, u8, u8)) {
        self.buffer.clear(color);
    }

    fn present(&mut self) {
    }

    fn get_framebuffer(&mut self) -> Option<&mut RenderBuffer> {
        Some(&mut self.buffer)
    }

    fn get_capabilities(&self) -> BackendCapabilities {
        BackendCapabilities {
            max_texture_size: 4096,
            supports_shaders: false,
            supports_anti_aliasing: false,
            supports_hardware_acceleration: false,
            max_vertex_count: 1000000,
            supports_geometry_shader: false,
            supports_compute_shader: false,
        }
    }
}

impl Default for SoftwareRenderer {
    fn default() -> Self {
        Self::new()
    }
}

/// SVG 文本后端：把清屏与图元累积为 SVG 字符串，`present` 时补上 `</svg>` 结束标签。
/// 不提供内存帧缓冲，结果通过 `get_svg` 读取；与 `render_backend` 模块的同名类型同样相互独立。
pub struct SVGRenderer {
    width: usize,
    height: usize,
    svg_content: String,
}

impl SVGRenderer {
    /// 创建默认后端：画布 800×600 像素，内容为空串，需先调用 `initialize` 才写入 `<svg>` 根标签。
    pub fn new() -> Self {
        Self {
            width: 800,
            height: 600,
            svg_content: String::new(),
        }
    }

    /// 创建后端并直接写入与此尺寸对应的 `<svg>` 根标签。
    /// - `width`、`height`：画布尺寸（像素），仅写入标签属性，不做校验。
    pub fn with_size(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            svg_content: format!(r#"<svg xmlns="http://www.w3.org/2000/svg" width="{}" height="{}">"#, width, height),
        }
    }

    /// 在内容末尾追加 `</svg>` 结束标签；重复调用会追加多次并生成非法 SVG，通常只在收尾时调用一次。
    pub fn finalize(&mut self) {
        self.svg_content.push_str("</svg>");
    }

    /// 返回当前累积的 SVG 文本切片，含根元素与已绘制图元；未 `finalize` 时缺少结束标签。
    pub fn get_svg(&self) -> &str {
        &self.svg_content
    }
}

impl RenderBackend for SVGRenderer {
    fn name(&self) -> &str {
        "SVG"
    }

    fn is_available(&self) -> bool {
        true
    }

    fn initialize(&mut self) -> Result<(), String> {
        self.svg_content = format!(r#"<svg xmlns="http://www.w3.org/2000/svg" width="{}" height="{}">"#, self.width, self.height);
        Ok(())
    }

    fn set_size(&mut self, width: usize, height: usize) {
        self.width = width;
        self.height = height;
        self.svg_content = format!(r#"<svg xmlns="http://www.w3.org/2000/svg" width="{}" height="{}">"#, width, height);
    }

    fn clear(&mut self, color: (u8, u8, u8)) {
        self.svg_content.push_str(&format!(
            r#"<rect width="100%" height="100%" fill="rgb({},{},{})"/>"#,
            color.0, color.1, color.2
        ));
    }

    fn present(&mut self) {
        self.finalize();
    }

    fn get_framebuffer(&mut self) -> Option<&mut RenderBuffer> {
        None
    }

    fn get_capabilities(&self) -> BackendCapabilities {
        BackendCapabilities {
            max_texture_size: 4096,
            supports_shaders: false,
            supports_anti_aliasing: false,
            supports_hardware_acceleration: false,
            max_vertex_count: 1000000,
            supports_geometry_shader: false,
            supports_compute_shader: false,
        }
    }
}

impl Default for SVGRenderer {
    fn default() -> Self {
        Self::new()
    }
}

/// wasm32 目标的 WebGL 后端占位实现：不持有任何资源，`initialize` 直接返回错误，`is_available` 恒为 false。
#[cfg(target_arch = "wasm32")]
pub struct WebGLRenderer;

#[cfg(target_arch = "wasm32")]
impl WebGLRenderer {
    /// 创建占位实例，不初始化任何 WebGL 资源。
    pub fn new() -> Self {
        Self
    }
}

#[cfg(target_arch = "wasm32")]
impl RenderBackend for WebGLRenderer {
    fn name(&self) -> &str {
        "WebGL"
    }

    fn is_available(&self) -> bool {
        false
    }

    fn initialize(&mut self) -> Result<(), String> {
        Err("WebGL not available on this platform".to_string())
    }

    fn set_size(&mut self, _width: usize, _height: usize) {
    }

    fn clear(&mut self, _color: (u8, u8, u8)) {
    }

    fn present(&mut self) {
    }

    fn get_framebuffer(&mut self) -> Option<&mut RenderBuffer> {
        None
    }

    fn get_capabilities(&self) -> BackendCapabilities {
        BackendCapabilities {
            max_texture_size: 4096,
            supports_shaders: true,
            supports_anti_aliasing: true,
            supports_hardware_acceleration: true,
            max_vertex_count: 1000000,
            supports_geometry_shader: false,
            supports_compute_shader: false,
        }
    }
}

/// Windows Direct2D 后端占位实现：不持有设备资源，`initialize` 直接返回错误，`is_available` 恒为 false。
#[cfg(target_os = "windows")]
pub struct Direct2DRenderer;

#[cfg(target_os = "windows")]
impl Direct2DRenderer {
    /// 创建占位实例，不申请 Direct2D 设备与渲染目标。
    pub fn new() -> Self {
        Self
    }
}

#[cfg(target_os = "windows")]
impl RenderBackend for Direct2DRenderer {
    fn name(&self) -> &str {
        "Direct2D"
    }

    fn is_available(&self) -> bool {
        false
    }

    fn initialize(&mut self) -> Result<(), String> {
        Err("Direct2D not available on this platform".to_string())
    }

    fn set_size(&mut self, _width: usize, _height: usize) {
    }

    fn clear(&mut self, _color: (u8, u8, u8)) {
    }

    fn present(&mut self) {
    }

    fn get_framebuffer(&mut self) -> Option<&mut RenderBuffer> {
        None
    }

    fn get_capabilities(&self) -> BackendCapabilities {
        BackendCapabilities {
            max_texture_size: 16384,
            supports_shaders: true,
            supports_anti_aliasing: true,
            supports_hardware_acceleration: true,
            max_vertex_count: 10000000,
            supports_geometry_shader: true,
            supports_compute_shader: true,
        }
    }
}

/// macOS Metal 后端占位实现：不持有设备资源，`initialize` 直接返回错误，`is_available` 恒为 false。
#[cfg(target_os = "macos")]
pub struct MetalRenderer;

#[cfg(target_os = "macos")]
impl MetalRenderer {
    /// 创建占位实例，不申请 Metal 设备与命令队列。
    pub fn new() -> Self {
        Self
    }
}

#[cfg(target_os = "macos")]
impl RenderBackend for MetalRenderer {
    fn name(&self) -> &str {
        "Metal"
    }

    fn is_available(&self) -> bool {
        false
    }

    fn initialize(&mut self) -> Result<(), String> {
        Err("Metal not available on this platform".to_string())
    }

    fn set_size(&mut self, _width: usize, _height: usize) {
    }

    fn clear(&mut self, _color: (u8, u8, u8)) {
    }

    fn present(&mut self) {
    }

    fn get_framebuffer(&mut self) -> Option<&mut RenderBuffer> {
        None
    }

    fn get_capabilities(&self) -> BackendCapabilities {
        BackendCapabilities {
            max_texture_size: 16384,
            supports_shaders: true,
            supports_anti_aliasing: true,
            supports_hardware_acceleration: true,
            max_vertex_count: 10000000,
            supports_geometry_shader: true,
            supports_compute_shader: true,
        }
    }
}

/// Linux/FreeBSD OpenGL 后端占位实现：不持有上下文资源，`initialize` 直接返回错误，`is_available` 恒为 false。
#[cfg(any(target_os = "linux", target_os = "freebsd"))]
pub struct OpenGLRenderer;

#[cfg(any(target_os = "linux", target_os = "freebsd"))]
impl OpenGLRenderer {
    /// 创建占位实例，不创建 OpenGL 上下文与窗口表面。
    pub fn new() -> Self {
        Self
    }
}

#[cfg(any(target_os = "linux", target_os = "freebsd"))]
impl RenderBackend for OpenGLRenderer {
    fn name(&self) -> &str {
        "OpenGL"
    }

    fn is_available(&self) -> bool {
        false
    }

    fn initialize(&mut self) -> Result<(), String> {
        Err("OpenGL not available on this platform".to_string())
    }

    fn set_size(&mut self, _width: usize, _height: usize) {
    }

    fn clear(&mut self, _color: (u8, u8, u8)) {
    }

    fn present(&mut self) {
    }

    fn get_framebuffer(&mut self) -> Option<&mut RenderBuffer> {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_software_renderer_creation() {
        let renderer = SoftwareRenderer::new();
        assert_eq!(renderer.name(), "Software");
        assert!(renderer.is_available());
    }

    #[test]
    fn test_software_renderer_with_size() {
        let renderer = SoftwareRenderer::with_size(1024, 768);
        assert!(renderer.is_available());
    }

    #[test]
    fn test_svg_renderer_creation() {
        let renderer = SVGRenderer::new();
        assert_eq!(renderer.name(), "SVG");
        assert!(renderer.is_available());
    }
}
