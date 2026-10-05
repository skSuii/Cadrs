use crate::geometry::{Point, Line, Circle, Arc, Ellipse, Polyline};
use crate::render::{RenderStyle, RenderBuffer, Renderer};
use std::error::Error;
use std::fmt;

pub trait RenderBackend {
    fn name(&self) -> &str;
    fn is_available(&self) -> bool;
    fn initialize(&mut self) -> Result<(), String>;
    fn set_size(&mut self, width: usize, height: usize);
    fn clear(&mut self, color: (u8, u8, u8));
    fn present(&mut self);
    fn get_framebuffer(&mut self) -> Option<&mut RenderBuffer>;
    fn get_capabilities(&self) -> BackendCapabilities;
}

#[derive(Debug, Clone)]
pub struct BackendCapabilities {
    pub max_texture_size: usize,
    pub supports_shaders: bool,
    pub supports_anti_aliasing: bool,
    pub supports_hardware_acceleration: bool,
    pub max_vertex_count: usize,
    pub supports_geometry_shader: bool,
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

#[derive(Debug, Clone, PartialEq)]
pub enum BackendType {
    Software,
    SVG,
    #[cfg(target_arch = "wasm32")]
    WebGL,
    #[cfg(target_os = "windows")]
    Direct2D,
    #[cfg(target_os = "windows")]
    Direct3D11,
    #[cfg(target_os = "macos")]
    Metal,
    #[cfg(any(target_os = "linux", target_os = "freebsd"))]
    OpenGL,
    #[cfg(any(target_os = "linux", target_os = "freebsd"))]
    Vulkan,
}

impl BackendType {
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

    pub fn default() -> Self {
        BackendType::Software
    }

    pub fn default_gpu() -> Self {
        let gpu_backends = Self::gpu_backends();
        if !gpu_backends.is_empty() {
            gpu_backends[0].clone()
        } else {
            BackendType::Software
        }
    }

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

pub struct RenderingContext {
    backend: Box<dyn RenderBackend>,
    width: usize,
    height: usize,
}

impl RenderingContext {
    pub fn new(backend_type: BackendType) -> Result<Self, String> {
        let mut backend = backend_type.create_backend();
        backend.initialize()?;
        
        Ok(Self {
            backend,
            width: 800,
            height: 600,
        })
    }

    pub fn with_size(backend_type: BackendType, width: usize, height: usize) -> Result<Self, String> {
        let mut context = Self::new(backend_type)?;
        context.set_size(width, height);
        Ok(context)
    }

    pub fn set_size(&mut self, width: usize, height: usize) {
        self.width = width;
        self.height = height;
        self.backend.set_size(width, height);
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.height
    }

    pub fn clear(&mut self, color: (u8, u8, u8)) {
        self.backend.clear(color);
    }

    pub fn present(&mut self) {
        self.backend.present();
    }

    pub fn get_framebuffer(&mut self) -> Option<&mut RenderBuffer> {
        self.backend.get_framebuffer()
    }

    pub fn get_capabilities(&self) -> BackendCapabilities {
        self.backend.get_capabilities()
    }
}

pub struct GPURenderer {
    width: usize,
    height: usize,
    capabilities: BackendCapabilities,
    vertex_buffer: Vec<Vertex>,
    index_buffer: Vec<u32>,
    texture_cache: std::collections::HashMap<String, TextureHandle>,
    shader_programs: std::collections::HashMap<String, ShaderProgram>,
}

#[derive(Debug, Clone)]
pub struct Vertex {
    pub position: [f32; 3],
    pub color: [f32; 4],
    pub texcoord: [f32; 2],
    pub normal: [f32; 3],
}

#[derive(Debug, Clone)]
pub struct TextureHandle {
    width: usize,
    height: usize,
    format: TextureFormat,
}

#[derive(Debug, Clone)]
pub enum TextureFormat {
    RGBA8,
    RGBA16F,
    RGBA32F,
    R8,
    R16F,
    R32F,
}

#[derive(Debug, Clone)]
pub struct ShaderProgram {
    vertex_shader: String,
    fragment_shader: String,
    uniforms: Vec<Uniform>,
}

#[derive(Debug, Clone)]
pub struct Uniform {
    name: String,
    uniform_type: UniformType,
    value: UniformValue,
}

#[derive(Debug, Clone)]
pub enum UniformType {
    Float,
    Vec2,
    Vec3,
    Vec4,
    Mat4,
    Int,
    Sampler2D,
}

#[derive(Debug, Clone)]
pub enum UniformValue {
    Float(f32),
    Vec2([f32; 2]),
    Vec3([f32; 3]),
    Vec4([f32; 4]),
    Mat4([[f32; 4]; 4]),
    Int(i32),
    Texture(TextureHandle),
}

impl GPURenderer {
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

pub struct SoftwareRenderer {
    width: usize,
    height: usize,
    buffer: RenderBuffer,
}

impl SoftwareRenderer {
    pub fn new() -> Self {
        Self {
            width: 800,
            height: 600,
            buffer: RenderBuffer::new(800, 600),
        }
    }

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

pub struct SVGRenderer {
    width: usize,
    height: usize,
    svg_content: String,
}

impl SVGRenderer {
    pub fn new() -> Self {
        Self {
            width: 800,
            height: 600,
            svg_content: String::new(),
        }
    }

    pub fn with_size(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            svg_content: format!(r#"<svg xmlns="http://www.w3.org/2000/svg" width="{}" height="{}">"#, width, height),
        }
    }

    pub fn finalize(&mut self) {
        self.svg_content.push_str("</svg>");
    }

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
