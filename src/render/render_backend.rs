//! 渲染后端抽象：定义后端需实现的 `RenderBackend` 接口、能力描述 `BackendCapabilities`、
//! 后端类型枚举 `BackendType` 与统一驱动入口 `RenderingContext`。
//! 除 CPU 软件后端与 SVG 文本后端外，这里还给出 Direct2D/Direct3D11/Metal/OpenGL/Vulkan/WebGL
//! 的骨架实现，它们的 `is_available` 目前均为 `false`；绘制数据来自 `renderer` 模块，后端只负责落地输出。

use crate::geometry::{Point, Line, Circle, Arc, Ellipse, Polyline};
use crate::render::{RenderStyle, RenderBuffer, Renderer};
use std::error::Error;
use std::fmt;

/// 渲染后端接口：统一暴露初始化、尺寸设置、清屏与呈现能力，屏蔽具体图形 API 差异。
/// 调用方通常不直接持有后端，而是通过 `RenderingContext` 驱动；
/// `get_framebuffer` 只有 CPU 后端返回像素缓冲，硬件与矢量后端返回 `None`。
pub trait RenderBackend {
    /// 后端名称，用于日志与后端选择。
    fn name(&self) -> &str;
    /// 当前环境是否可用；返回 `false` 的后端即使创建成功也无法产出画面，应改选其他后端。
    fn is_available(&self) -> bool;
    /// 初始化后端资源；失败时返回可读的错误描述。
    fn initialize(&mut self) -> Result<(), String>;
    /// 设置输出尺寸（单位像素）；软件后端会重建帧缓冲，已有绘制内容不保留。
    fn set_size(&mut self, width: usize, height: usize);
    /// 用单色覆盖输出目标，作为一帧绘制的背景。
    /// - `color`：背景色，`(红, 绿, 蓝)`，每个分量 0~255。
    fn clear(&mut self, color: (u8, u8, u8));
    /// 提交当前帧到输出目标（屏幕或文件），SVG 后端会在此处补上闭合标签。
    fn present(&mut self);
    /// 取可写的内存帧缓冲；无 CPU 帧缓冲的后端返回 `None`。
    fn get_framebuffer(&mut self) -> Option<&mut RenderBuffer>;
    /// 返回后端能力描述，调用方据此决定是否启用抗锯齿、着色器等特性。
    fn get_capabilities(&self) -> BackendCapabilities;
}

/// 渲染后端能力描述，帮助调用方在运行时选择绘制策略；`Default` 给出保守的软件级配置。
#[derive(Debug, Clone)]
pub struct BackendCapabilities {
    /// 单张纹理的最大边长，单位像素。
    pub max_texture_size: usize,
    /// 是否支持可编程着色器。
    pub supports_shaders: bool,
    /// 是否支持抗锯齿。
    pub supports_anti_aliasing: bool,
    /// 是否使用硬件加速。
    pub supports_hardware_acceleration: bool,
    /// 单帧可提交的最大顶点数，超出时应分批绘制。
    pub max_vertex_count: usize,
    /// 是否支持几何着色器。
    pub supports_geometry_shader: bool,
    /// 是否支持计算着色器。
    pub supports_compute_shader: bool,
}

impl Default for BackendCapabilities {
    fn default() -> Self {
        Self {
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

/// 渲染后端类型标识，用于选择与描述具体的渲染实现。
/// 部分变体仅在对应平台编译：WebGL 限 wasm32，Direct2D 与 Direct3D11 限 Windows，
/// Metal 限 macOS，OpenGL 与 Vulkan 限 Linux/FreeBSD，跨平台匹配时必须使用通配分支。
#[derive(Debug, Clone, PartialEq)]
pub enum BackendType {
    /// CPU 软件光栅后端，全平台可用，结果写入内存帧缓冲。
    Software,
    /// SVG 矢量后端，把图元累积为 SVG 文本，便于缩放浏览与导出。
    SVG,
    /// 浏览器 WebGL 后端，仅 wasm32 目标下存在。
    #[cfg(target_arch = "wasm32")]
    WebGL,
    /// Windows Direct2D 后端（骨架实现，当前不可用）。
    #[cfg(target_os = "windows")]
    Direct2D,
    /// Windows Direct3D 11 后端（骨架实现，当前不可用）。
    #[cfg(target_os = "windows")]
    Direct3D11,
    /// macOS Metal 后端（骨架实现，当前不可用）。
    #[cfg(target_os = "macos")]
    Metal,
    /// Linux/FreeBSD OpenGL 后端（骨架实现，当前不可用）。
    #[cfg(any(target_os = "linux", target_os = "freebsd"))]
    OpenGL,
    /// Linux/FreeBSD Vulkan 后端（骨架实现，当前不可用）。
    #[cfg(any(target_os = "linux", target_os = "freebsd"))]
    Vulkan,
}

impl BackendType {
    /// 后端名称，为静态字符串，可与 `from_str` 的输入对应（比较时不区分大小写）。
    pub fn name(&self) -> &'static str {
        match self {
            BackendType::Software => "Software",
            BackendType::SVG => "SVG",
            #[cfg(target_arch = "wasm32")]
            BackendType::WebGL => "WebGL",
            #[cfg(target_os = "windows")]
            BackendType::Direct2D => "Direct2D",
            #[cfg(target_os = "windows")]
            BackendType::Direct3D11 => "Direct3D11",
            #[cfg(target_os = "macos")]
            BackendType::Metal => "Metal",
            #[cfg(any(target_os = "linux", target_os = "freebsd"))]
            BackendType::OpenGL => "OpenGL",
            #[cfg(any(target_os = "linux", target_os = "freebsd"))]
            BackendType::Vulkan => "Vulkan",
        }
    }

    /// 按名称解析后端类型，忽略大小写，并接受常见别名（如 `"d2d"`、`"dx11"`、`"gl"`、`"vk"`）。
    /// - `name`：后端名称，例如 `"software"`、`"svg"`。
    /// 返回匹配的后端；名称未知、或该变体在当前平台未编译时返回 `None`。
    pub fn from_str(name: &str) -> Option<Self> {
        match name.to_lowercase().as_str() {
            "software" => Some(BackendType::Software),
            "svg" => Some(BackendType::SVG),
            #[cfg(target_arch = "wasm32")]
            "webgl" | "web_gpu" => Some(BackendType::WebGL),
            #[cfg(target_os = "windows")]
            "direct2d" | "d2d" => Some(BackendType::Direct2D),
            #[cfg(target_os = "windows")]
            "direct3d" | "d3d11" | "dx11" => Some(BackendType::Direct3D11),
            #[cfg(target_os = "macos")]
            "metal" | "mtl" => Some(BackendType::Metal),
            #[cfg(any(target_os = "linux", target_os = "freebsd"))]
            "opengl" | "gl" | "gl3" => Some(BackendType::OpenGL),
            #[cfg(any(target_os = "linux", target_os = "freebsd"))]
            "vulkan" | "vk" => Some(BackendType::Vulkan),
            _ => None,
        }
    }

    /// 当前平台可用的 GPU 后端列表，按优先级从高到低排列。
    /// 只包含条件编译后存在的后端；未编译任何 GPU 后端时返回空向量。
    pub fn gpu_backends() -> Vec<Self> {
        let mut backends = Vec::new();
        
        #[cfg(target_os = "windows")]
        backends.push(BackendType::Direct3D11);
        
        #[cfg(target_os = "macos")]
        backends.push(BackendType::Metal);
        
        #[cfg(any(target_os = "linux", target_os = "freebsd"))]
        backends.push(BackendType::Vulkan);
        
        #[cfg(target_arch = "wasm32")]
        backends.push(BackendType::WebGL);
        
        backends
    }

    /// 默认后端，固定为跨平台可用的 `Software`。
    pub fn default() -> Self {
        BackendType::Software
    }

    /// 默认 GPU 后端：取 `gpu_backends` 的首项（平台优先级最高的一个）。
    /// 当前平台没有可用 GPU 后端时退回 `Software`，因此调用方无需再判空。
    pub fn default_gpu() -> Self {
        let gpu_backends = Self::gpu_backends();
        if !gpu_backends.is_empty() {
            gpu_backends[0].clone()
        } else {
            BackendType::Software
        }
    }

    /// 依据类型创建对应的后端实例，返回装箱后的 trait 对象。
    /// 实例尚未初始化，需由调用方再调用 `RenderBackend::initialize`（`RenderingContext::new` 已代为完成）。
    pub fn create_backend(&self) -> Box<dyn RenderBackend> {
        match self {
            BackendType::Software => Box::new(SoftwareRenderer::new()),
            BackendType::SVG => Box::new(SVGRenderer::new()),
            #[cfg(target_arch = "wasm32")]
            BackendType::WebGL => Box::new(WebGLRenderer::new()),
            #[cfg(target_os = "windows")]
            BackendType::Direct2D => Box::new(Direct2DRenderer::new()),
            #[cfg(target_os = "windows")]
            BackendType::Direct3D11 => Box::new(D3D11Renderer::new()),
            #[cfg(target_os = "macos")]
            BackendType::Metal => Box::new(MetalRenderer::new()),
            #[cfg(any(target_os = "linux", target_os = "freebsd"))]
            BackendType::OpenGL => Box::new(OpenGLRenderer::new()),
            #[cfg(any(target_os = "linux", target_os = "freebsd"))]
            BackendType::Vulkan => Box::new(VulkanRenderer::new()),
        }
    }
}

/// 渲染上下文：把后端实现与当前输出尺寸打包，作为上层绘制逻辑的统一入口。
/// 持有后端所有权并在创建时完成初始化；默认输出尺寸为 800×600 像素，所有绘制调用都会转发给后端。
pub struct RenderingContext {
    backend: Box<dyn RenderBackend>,
    width: usize,
    height: usize,
}

impl RenderingContext {
    /// 用指定后端创建上下文，内部立即调用后端的 `initialize`，输出尺寸默认 800×600 像素。
    /// - `backend_type`：后端类型，决定实际渲染路径。
    /// 返回初始化完成的上下文；后端初始化失败时返回错误描述。
    pub fn new(backend_type: BackendType) -> Result<Self, String> {
        let mut backend = backend_type.create_backend();
        backend.initialize()?;
        
        Ok(Self {
            backend,
            width: 800,
            height: 600,
        })
    }

    /// 创建上下文并立即设定输出尺寸。
    /// - `width`、`height`：输出尺寸，单位像素。
    /// 返回已初始化且尺寸生效的上下文；后端初始化失败时返回错误描述。
    pub fn with_size(backend_type: BackendType, width: usize, height: usize) -> Result<Self, String> {
        let mut context = Self::new(backend_type)?;
        context.set_size(width, height);
        Ok(context)
    }

    /// 设置输出尺寸并同步通知后端；软件后端会据此重建帧缓冲，已有绘制内容丢失。
    /// - `width`、`height`：新的像素尺寸，不做上限校验。
    pub fn set_size(&mut self, width: usize, height: usize) {
        self.width = width;
        self.height = height;
        self.backend.set_size(width, height);
    }

    /// 当前输出宽度，单位像素。
    pub fn width(&self) -> usize {
        self.width
    }

    /// 当前输出高度，单位像素。
    pub fn height(&self) -> usize {
        self.height
    }

    /// 用单色清空输出目标，直接转发给后端。
    pub fn clear(&mut self, color: (u8, u8, u8)) {
        self.backend.clear(color);
    }

    /// 提交当前帧到输出目标；SVG 后端会在此处补上结束标签。
    pub fn present(&mut self) {
        self.backend.present();
    }

    /// 取可写的内存帧缓冲；非软件后端返回 `None`，调用方需先判空。
    pub fn get_framebuffer(&mut self) -> Option<&mut RenderBuffer> {
        self.backend.get_framebuffer()
    }

    /// 返回底层后端的能力描述，未做任何缓存，每次调用都会重新查询。
    pub fn get_capabilities(&self) -> BackendCapabilities {
        self.backend.get_capabilities()
    }
}

/// 与图形 API 无关的高层 GPU 渲染状态容器：登记顶点/索引缓冲、纹理缓存与着色器程序。
/// 当前只维护资源清单与能力描述，尚未真正调用图形 API 提交绘制。
pub struct GPURenderer {
    width: usize,
    height: usize,
    capabilities: BackendCapabilities,
    vertex_buffer: Vec<Vertex>,
    index_buffer: Vec<u32>,
    texture_cache: std::collections::HashMap<String, TextureHandle>,
    shader_programs: std::collections::HashMap<String, ShaderProgram>,
}

/// 单个顶点，按位置/颜色/纹理坐标/法线的顺序描述，供 GPU 顶点缓冲布局使用。
#[derive(Debug, Clone)]
pub struct Vertex {
    /// 顶点位置 `[x, y, z]`；采用世界坐标还是裁剪空间由着色器约定。
    pub position: [f32; 3],
    /// 顶点颜色 `[r, g, b, a]`，每个分量取值 0.0~1.0。
    pub color: [f32; 4],
    /// 纹理坐标 `[u, v]`，通常取 0.0~1.0，原点在纹理左上角。
    pub texcoord: [f32; 2],
    /// 法线方向 `[x, y, z]`，应为单位向量，用于光照计算。
    pub normal: [f32; 3],
}

/// 纹理句柄：记录纹理尺寸与像素格式的轻量描述，真正的纹理对象由后端另行持有。
#[derive(Debug, Clone)]
pub struct TextureHandle {
    width: usize,
    height: usize,
    format: TextureFormat,
}

/// 纹理像素格式，命名沿用图形 API 惯例：字母表示通道，数字表示每通道位数，`F` 表示浮点。
#[derive(Debug, Clone)]
pub enum TextureFormat {
    /// 8 位无符号 RGBA，最常用的颜色纹理格式。
    RGBA8,
    /// 16 位浮点 RGBA，用于高动态范围数据。
    RGBA16F,
    /// 32 位浮点 RGBA。
    RGBA32F,
    /// 8 位单通道，常用于遮罩或字形位图。
    R8,
    /// 16 位浮点单通道。
    R16F,
    /// 32 位浮点单通道。
    R32F,
}

/// 着色器程序：保存顶点与片元着色器源码，以及需要上传的 uniform 变量列表。
#[derive(Debug, Clone)]
pub struct ShaderProgram {
    vertex_shader: String,
    fragment_shader: String,
    uniforms: Vec<Uniform>,
}

/// 单个 uniform 变量：名称、声明类型与当前取值三者必须保持一致。
#[derive(Debug, Clone)]
pub struct Uniform {
    name: String,
    uniform_type: UniformType,
    value: UniformValue,
}

/// uniform 变量的声明类型，用于上传前校验取值与类型是否匹配。
#[derive(Debug, Clone)]
pub enum UniformType {
    /// 单精度浮点数。
    Float,
    /// 二维向量。
    Vec2,
    /// 三维向量。
    Vec3,
    /// 四维向量。
    Vec4,
    /// 4×4 矩阵，常用于模型、视图与投影变换。
    Mat4,
    /// 32 位有符号整数。
    Int,
    /// 二维纹理采样器。
    Sampler2D,
}

/// uniform 变量的实际取值，需与对应的 `UniformType` 一致。
#[derive(Debug, Clone)]
pub enum UniformValue {
    /// 浮点取值，对应 `UniformType::Float`。
    Float(f32),
    /// 二维向量取值 `[x, y]`。
    Vec2([f32; 2]),
    /// 三维向量取值 `[x, y, z]`。
    Vec3([f32; 3]),
    /// 四维向量取值 `[x, y, z, w]`，也可表示 RGBA 颜色。
    Vec4([f32; 4]),
    /// 4×4 矩阵，按行主序存储。
    Mat4([[f32; 4]; 4]),
    /// 32 位整数取值。
    Int(i32),
    /// 纹理采样取值，持有纹理句柄的副本。
    Texture(TextureHandle),
}

impl GPURenderer {
    /// 创建渲染器：输出尺寸 800×600 像素，能力描述宣称支持着色器、抗锯齿、几何与计算着色器，
    /// 顶点缓冲、索引缓冲、纹理缓存与着色器程序均为空，不触发任何图形 API 调用。
    pub fn new() -> Self {
        Self {
            width: 800,
            height: 600,
            capabilities: BackendCapabilities {
                max_texture_size: 8192,
                supports_shaders: true,
                supports_anti_aliasing: true,
                supports_hardware_acceleration: true,
                max_vertex_count: 10000000,
                supports_geometry_shader: true,
                supports_compute_shader: true,
            },
            vertex_buffer: Vec::new(),
            index_buffer: Vec::new(),
            texture_cache: std::collections::HashMap::new(),
            shader_programs: std::collections::HashMap::new(),
        }
    }

    /// 创建渲染器并指定输出尺寸（单位像素），其余状态与 `new` 相同。
    /// - `width`、`height`：输出尺寸。
    pub fn with_size(width: usize, height: usize) -> Self {
        let mut renderer = Self::new();
        renderer.width = width;
        renderer.height = height;
        renderer
    }

    fn create_default_shaders(&mut self) {
        let vertex_shader = r#"#version 330 core
            layout(location = 0) in vec3 aPos;
            layout(location = 1) in vec4 aColor;
            layout(location = 2) in vec2 aTexCoord;
            layout(location = 3) in vec3 aNormal;
            
            uniform mat4 uModel;
            uniform mat4 uView;
            uniform mat4 uProjection;
            
            out vec4 vColor;
            out vec2 vTexCoord;
            out vec3 vNormal;
            out vec3 vFragPos;
            
            void main() {
                gl_Position = uProjection * uView * uModel * vec4(aPos, 1.0);
                vColor = aColor;
                vTexCoord = aTexCoord;
                vNormal = mat3(transpose(inverse(uModel))) * aNormal;
                vFragPos = vec3(uModel * vec4(aPos, 1.0));
            }
        "#.to_string();

        let fragment_shader = r#"#version 330 core
            out vec4 FragColor;
            
            in vec4 vColor;
            in vec2 vTexCoord;
            in vec3 vNormal;
            in vec3 vFragPos;
            
            uniform sampler2D uTexture;
            uniform vec4 uColor;
            uniform bool uUseTexture;
            uniform bool uUseLighting;
            uniform vec3 uLightPos;
            uniform vec3 uLightColor;
            uniform vec3 uViewPos;
            
            void main() {
                vec4 result = vColor * uColor;
                
                if (uUseTexture) {
                    vec4 texColor = texture(uTexture, vTexCoord);
                    result *= texColor;
                }
                
                if (uUseLighting) {
                    float ambientStrength = 0.1;
                    vec3 ambient = ambientStrength * uLightColor;
                    
                    vec3 norm = normalize(vNormal);
                    vec3 lightDir = normalize(uLightPos - vFragPos);
                    float diff = max(dot(norm, lightDir), 0.0);
                    vec3 diffuse = diff * uLightColor;
                    
                    vec3 viewDir = normalize(uViewPos - vFragPos);
                    vec3 reflectDir = reflect(-lightDir, norm);
                    float spec = 0.0;
                    if (diff > 0.0) {
                        float specularStrength = 0.5;
                        vec3 halfwayDir = normalize(lightDir + viewDir);
                        spec = pow(max(dot(norm, halfwayDir), 0.0), 32.0);
                    }
                    vec3 specular = specularStrength * spec * uLightColor;
                    
                    vec4 lighting = vec4(ambient + diffuse + specular, 1.0);
                    result *= lighting;
                }
                
                FragColor = result;
            }
        "#.to_string();

        self.shader_programs.insert("default".to_string(), ShaderProgram {
            vertex_shader,
            fragment_shader,
            uniforms: Vec::new(),
        });
    }

    fn create_text_shaders(&mut self) {
        let vertex_shader = r#"#version 330 core
            layout(location = 0) in vec3 aPos;
            layout(location = 1) in vec4 aColor;
            layout(location = 2) in vec2 aTexCoord;
            
            uniform mat4 uProjection;
            
            out vec4 vColor;
            out vec2 vTexCoord;
            
            void main() {
                gl_Position = uProjection * vec4(aPos, 1.0);
                vColor = aColor;
                vTexCoord = vTexCoord;
            }
        "#.to_string();

        let fragment_shader = r#"#version 330 core
            in vec4 vColor;
            in vec2 vTexCoord;
            
            out vec4 FragColor;
            
            uniform sampler2D uTexture;
            
            void main() {
                vec4 texColor = texture(uTexture, vTexCoord);
                if (texColor.a < 0.1) discard;
                FragColor = vColor * texColor;
            }
        "#.to_string();

        self.shader_programs.insert("text".to_string(), ShaderProgram {
            vertex_shader,
            fragment_shader,
            uniforms: Vec::new(),
        });
    }

    fn create_anti_aliased_shaders(&mut self) {
        let vertex_shader = r#"#version 330 core
            layout(location = 0) in vec3 aPos;
            layout(location = 1) in vec4 aColor;
            layout(location = 2) in vec2 aTexCoord;
            layout(location = 3) in float aEdgeDistance;
            
            uniform mat4 uModel;
            uniform mat4 uView;
            uniform mat4 uProjection;
            
            out vec4 vColor;
            out vec2 vTexCoord;
            out float vEdgeDistance;
            
            void main() {
                gl_Position = uProjection * uView * uModel * vec4(aPos, 1.0);
                vColor = aColor;
                vTexCoord = vTexCoord;
                vEdgeDistance = aEdgeDistance;
            }
        "#.to_string();

        let fragment_shader = r#"#version 330 core
            in vec4 vColor;
            in vec2 vTexCoord;
            in float vEdgeDistance;
            
            out vec4 FragColor;
            
            uniform float uEdgeWidth;
            
            void main() {
                float edgeFactor = smoothstep(0.0, uEdgeWidth, vEdgeDistance);
                FragColor = vColor * edgeFactor;
            }
        "#.to_string();

        self.shader_programs.insert("anti_aliased".to_string(), ShaderProgram {
            vertex_shader,
            fragment_shader,
            uniforms: Vec::new(),
        });
    }
}

/// Windows Direct2D 后端骨架：仅保存尺寸与渲染目标、设备上下文、交换链的句柄占位。
/// 所有绘制方法均为空实现，`is_available` 恒为 `false`，暂不能产出画面。
#[cfg(target_os = "windows")]
pub struct Direct2DRenderer {
    width: usize,
    height: usize,
    render_target: Option<*mut std::ffi::c_void>,
    device_context: Option<*mut std::ffi::c_void>,
    swap_chain: Option<*mut std::ffi::c_void>,
}

#[cfg(target_os = "windows")]
impl Direct2DRenderer {
    /// 创建后端实例：尺寸 800×600 像素，所有设备句柄为 `None`，不申请真实设备资源。
    pub fn new() -> Self {
        Self {
            width: 800,
            height: 600,
            render_target: None,
            device_context: None,
            swap_chain: None,
        }
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
        Ok(())
    }

    fn set_size(&mut self, width: usize, height: usize) {
        self.width = width;
        self.height = height;
    }

    fn clear(&mut self, _color: (u8, u8, u8)) {}

    fn present(&mut self) {}

    fn get_framebuffer(&mut self) -> Option<&mut RenderBuffer> {
        None
    }

    fn get_capabilities(&self) -> BackendCapabilities {
        BackendCapabilities {
            max_texture_size: 8192,
            supports_shaders: false,
            supports_anti_aliasing: true,
            supports_hardware_acceleration: true,
            max_vertex_count: 1000000,
            supports_geometry_shader: false,
            supports_compute_shader: false,
        }
    }
}

/// Windows Direct3D 11 后端骨架：保存设备、交换链、着色器与各类状态对象的句柄占位。
/// 所有绘制方法均为空实现，`is_available` 恒为 `false`，暂不能产出画面。
#[cfg(target_os = "windows")]
pub struct D3D11Renderer {
    width: usize,
    height: usize,
    device: Option<*mut std::ffi::c_void>,
    device_context: Option<*mut std::ffi::c_void>,
    swap_chain: Option<*mut std::ffi::c_void>,
    render_target_view: Option<*mut std::ffi::c_void>,
    depth_stencil_view: Option<*mut std::ffi::c_void>,
    vertex_shader: Option<*mut std::ffi::c_void>,
    pixel_shader: Option<*mut std::ffi::c_void>,
    input_layout: Option<*mut std::ffi::c_void>,
    vertex_buffer: Option<*mut std::ffi::c_void>,
    index_buffer: Option<*mut std::ffi::c_void>,
    constant_buffer: Option<*mut std::ffi::c_void>,
    blend_state: Option<*mut std::ffi::c_void>,
    rasterizer_state: Option<*mut std::ffi::c_void>,
    depth_stencil_state: Option<*mut std::ffi::c_void>,
}

#[cfg(target_os = "windows")]
impl D3D11Renderer {
    /// 创建后端实例：尺寸 800×600 像素，所有设备与状态句柄为 `None`，不申请真实设备资源。
    pub fn new() -> Self {
        Self {
            width: 800,
            height: 600,
            device: None,
            device_context: None,
            swap_chain: None,
            render_target_view: None,
            depth_stencil_view: None,
            vertex_shader: None,
            pixel_shader: None,
            input_layout: None,
            vertex_buffer: None,
            index_buffer: None,
            constant_buffer: None,
            blend_state: None,
            rasterizer_state: None,
            depth_stencil_state: None,
        }
    }

    fn compile_shader(source: &[u8], target: &str, entry_point: &str) -> Result<Vec<u8>, String> {
        Ok(Vec::new())
    }
}

#[cfg(target_os = "windows")]
impl RenderBackend for D3D11Renderer {
    fn name(&self) -> &str {
        "Direct3D 11"
    }

    fn is_available(&self) -> bool {
        false
    }

    fn initialize(&mut self) -> Result<(), String> {
        Ok(())
    }

    fn set_size(&mut self, width: usize, height: usize) {
        self.width = width;
        self.height = height;
    }

    fn clear(&mut self, _color: (u8, u8, u8)) {}

    fn present(&mut self) {}

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

/// macOS Metal 后端骨架：保存设备、命令队列、渲染管线与缓冲区的句柄占位。
/// 所有绘制方法均为空实现，`is_available` 恒为 `false`，暂不能产出画面。
#[cfg(target_os = "macos")]
pub struct MetalRenderer {
    width: usize,
    height: usize,
    device: Option<*mut std::ffi::c_void>,
    command_queue: Option<*mut std::ffi::c_void>,
    render_pipeline_state: Option<*mut std::ffi::c_void>,
    vertex_buffer: Option<*mut std::ffi::c_void>,
    uniform_buffer: Option<*mut std::ffi::c_void>,
}

#[cfg(target_os = "macos")]
impl MetalRenderer {
    /// 创建后端实例：尺寸 800×600 像素，所有设备句柄为 `None`，不申请真实设备资源。
    pub fn new() -> Self {
        Self {
            width: 800,
            height: 600,
            device: None,
            command_queue: None,
            render_pipeline_state: None,
            vertex_buffer: None,
            uniform_buffer: None,
        }
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
        Ok(())
    }

    fn set_size(&mut self, width: usize, height: usize) {
        self.width = width;
        self.height = height;
    }

    fn clear(&mut self, _color: (u8, u8, u8)) {}

    fn present(&mut self) {}

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

/// Linux/FreeBSD OpenGL 后端骨架：保存 glutin 上下文、窗口表面与顶点数组、着色器等对象编号。
/// `new` 通过零值初始化上下文与表面，仅作占位；所有绘制方法为空实现，`is_available` 恒为 `false`。
#[cfg(any(target_os = "linux", target_os = "freebsd"))]
pub struct OpenGLRenderer {
    width: usize,
    height: usize,
    context: glutin::context::PossiblyCurrentContext,
    surface: glutin::surface::Surface<glutin::surface::Window>,
    vertex_array: Option<gl::types::GLuint>,
    vertex_buffer: Option<gl::types::GLuint>,
    shader_program: Option<gl::types::GLuint>,
    texture: Option<gl::types::GLuint>,
}

#[cfg(any(target_os = "linux", target_os = "freebsd"))]
impl OpenGLRenderer {
    /// 创建后端实例：尺寸 800×600 像素，上下文与表面为零值占位（不可直接使用），
    /// 不创建窗口、不加载 OpenGL 函数。
    pub fn new() -> Self {
        Self {
            width: 800,
            height: 600,
            context: unsafe { std::mem::zeroed() },
            surface: unsafe { std::mem::zeroed() },
            vertex_array: None,
            vertex_buffer: None,
            shader_program: None,
            texture: None,
        }
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
        Ok(())
    }

    fn set_size(&mut self, width: usize, height: usize) {
        self.width = width;
        self.height = height;
    }

    fn clear(&mut self, _color: (u8, u8, u8)) {}

    fn present(&mut self) {}

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

/// Linux/FreeBSD Vulkan 后端骨架：保存实例、物理设备、逻辑设备、队列、交换链与管线的句柄占位。
/// 所有绘制方法均为空实现，`is_available` 恒为 `false`，暂不能产出画面。
#[cfg(any(target_os = "linux", target_os = "freebsd"))]
pub struct VulkanRenderer {
    width: usize,
    height: usize,
    instance: Option<*mut std::ffi::c_void>,
    physical_device: Option<*mut std::ffi::c_void>,
    device: Option<*mut std::ffi::c_void>,
    queue: Option<*mut std::ffi::c_void>,
    swap_chain: Option<*mut std::ffi::c_void>,
    pipeline: Option<*mut std::ffi::c_void>,
}

#[cfg(any(target_os = "linux", target_os = "freebsd"))]
impl VulkanRenderer {
    /// 创建后端实例：尺寸 800×600 像素，所有句柄为 `None`，不创建 Vulkan 实例与设备。
    pub fn new() -> Self {
        Self {
            width: 800,
            height: 600,
            instance: None,
            physical_device: None,
            device: None,
            queue: None,
            swap_chain: None,
            pipeline: None,
        }
    }
}

#[cfg(any(target_os = "linux", target_os = "freebsd"))]
impl RenderBackend for VulkanRenderer {
    fn name(&self) -> &str {
        "Vulkan"
    }

    fn is_available(&self) -> bool {
        false
    }

    fn initialize(&mut self) -> Result<(), String> {
        Ok(())
    }

    fn set_size(&mut self, width: usize, height: usize) {
        self.width = width;
        self.height = height;
    }

    fn clear(&mut self, _color: (u8, u8, u8)) {}

    fn present(&mut self) {}

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

/// wasm32 目标的 WebGL 后端骨架：保存画布、WebGL 上下文、程序与缓冲区。
/// `new` 以零值初始化画布与上下文（不可直接使用），所有绘制方法为空实现，`is_available` 恒为 `false`。
#[cfg(target_arch = "wasm32")]
pub struct WebGLRenderer {
    width: usize,
    height: usize,
    canvas: web_sys::HtmlCanvasElement,
    context: web_sys::WebGlRenderingContext,
    program: Option<web_sys::WebGlProgram>,
    buffers: std::collections::HashMap<String, web_sys::WebGlBuffer>,
}

#[cfg(target_arch = "wasm32")]
impl WebGLRenderer {
    /// 创建后端实例：尺寸 800×600 像素，画布与上下文为零值占位，缓冲区映射为空。
    pub fn new() -> Self {
        Self {
            width: 800,
            height: 600,
            canvas: unsafe { std::mem::zeroed() },
            context: unsafe { std::mem::zeroed() },
            program: None,
            buffers: std::collections::HashMap::new(),
        }
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
        Ok(())
    }

    fn set_size(&mut self, width: usize, height: usize) {
        self.width = width;
        self.height = height;
    }

    fn clear(&mut self, _color: (u8, u8, u8)) {}

    fn present(&mut self) {}

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

/// CPU 软件光栅后端：把绘制结果写入内存中的 `RenderBuffer`，全平台可用且始终 `is_available` 为真。
/// 与 `software_renderer` 模块中的同名类型是两份彼此独立的实现，都通过 `RenderBackend` 接口对外服务。
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

    /// 创建指定像素尺寸的后端，并分配同尺寸帧缓冲。
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

    fn present(&mut self) {}

    fn get_framebuffer(&mut self) -> Option<&mut RenderBuffer> {
        Some(&mut self.buffer)
    }

    fn get_capabilities(&self) -> BackendCapabilities {
        BackendCapabilities::default()
    }
}

/// SVG 文本后端：把清屏与图元累积为 SVG 字符串，`present` 时补上结束标签，便于导出矢量结果。
/// 不提供内存帧缓冲，绘制内容只能通过 `get_svg` 读出。
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

    /// 返回当前累积的 SVG 文本切片，包含根元素与已绘制图元；未 `finalize` 时缺少结束标签。
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
        BackendCapabilities::default()
    }
}

impl Default for SoftwareRenderer {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for SVGRenderer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_backend_type_creation() {
        let backend = BackendType::Software;
        assert_eq!(backend.name(), "Software");
    }

    #[test]
    fn test_rendering_context() {
        let context = RenderingContext::new(BackendType::Software);
        assert!(context.is_ok());
    }

    #[test]
    fn test_gpu_renderer_creation() {
        let renderer = GPURenderer::new();
        assert_eq!(renderer.capabilities.supports_shaders, true);
        assert_eq!(renderer.capabilities.max_vertex_count, 10000000);
    }

    #[test]
    fn test_backend_capabilities() {
        let capabilities = BackendCapabilities::default();
        assert_eq!(capabilities.max_texture_size, 4096);
        assert!(!capabilities.supports_shaders);
    }

    #[test]
    fn test_shader_program_creation() {
        let program = ShaderProgram {
            vertex_shader: "#version 330 core\nvoid main() { gl_Position = vec4(0.0); }".to_string(),
            fragment_shader: "#version 330 core\nvoid main() { FragColor = vec4(1.0); }".to_string(),
            uniforms: Vec::new(),
        };
        assert!(!program.vertex_shader.is_empty());
        assert!(!program.fragment_shader.is_empty());
    }

    #[test]
    fn test_vertex_structure() {
        let vertex = Vertex {
            position: [0.0, 0.0, 0.0],
            color: [1.0, 0.0, 0.0, 1.0],
            texcoord: [0.5, 0.5],
            normal: [0.0, 0.0, 1.0],
        };
        assert_eq!(vertex.color[0], 1.0);
        assert_eq!(vertex.position[2], 0.0);
    }
}
