//! 渲染模块：把文档中的实体经视口变换后绘制到内存帧缓冲、SVG 文本或 GPU 表面。
//! 核心概念：`Viewport` 负责世界坐标与屏幕坐标互转，`RenderStyle` 描述线型与填充，
//! `Renderer`/`RenderBackend` 定义统一的绘制接口与后端能力，`tessellation` 把曲线细分为折线。
//! 具体的后端实现位于各子模块，上层绘制与导出逻辑只依赖这里导出的公共接口。

/// 视口与视图矩形：世界坐标与屏幕坐标的映射、平移、缩放与范围适配。
pub mod viewport;
/// 基础渲染抽象：绘制样式、线型、内存帧缓冲、渲染队列与实体绘制辅助。
pub mod renderer;
/// 软件光栅与 SVG 后端：把绘制结果写入内存像素缓冲或累积为 SVG 文本。
pub mod software_renderer;
/// SVG 文本后端实现：按 `Renderer` 接口把图元拼接为 SVG 元素。
pub mod svg_renderer;
/// 渲染后端抽象：后端类型枚举、能力描述、渲染上下文与各平台 GPU 后端骨架。
pub mod render_backend;
/// 实体细分：曲线离散为折线、提取填充多边形并计算文档包围盒。
pub mod tessellation;

/// 基于 wgpu 的 GPU 渲染后端，仅在启用 `gpu` feature 时参与编译。
#[cfg(feature = "gpu")]
pub mod gpu_renderer;

pub use viewport::{Viewport, ViewRect};
pub use renderer::{Renderer, RenderStyle, LinePattern, RenderBuffer, RenderQueue, EntityRenderer};
pub use software_renderer::{SoftwareRenderer, SVGRenderer};
pub use render_backend::{RenderBackend, BackendType, BackendCapabilities, RenderingContext};

#[cfg(feature = "gpu")]
pub use gpu_renderer::WGPURenderer;
