//! 基于 wgpu 的 GPU 渲染后端：`WGPURenderer` 管理 wgpu 实例、设备、队列、窗口表面与渲染管线，
//! 把线段与折线图元提交到表面呈现，顶点由内置 WGSL 着色器直接输出颜色。
//! 整个模块在启用 `gpu` feature 时才编译；未成功附加表面时，绘制只会堆积在待提交顶点中。

#[cfg(feature = "gpu")]
use wgpu::{util::DeviceExt, Device, Queue, RenderPipeline};

#[cfg(feature = "gpu")]
const VERTEX_SHADER_SOURCE: &str = r#"
struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) color: vec4<f32>,
}

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) color: vec4<f32>,
}

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    var output: VertexOutput;
    output.position = vec4<f32>(input.position, 0.0, 1.0);
    output.color = input.color;
    return output;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    return input.color;
}
"#;

/// wgpu 渲染器：持有实例、设备、队列、表面、表面配置与渲染管线，并缓存本轮待提交的顶点。
/// 使用流程为 `new` → `attach_surface`（异步）→ `resize` → 若干 `draw_*` → `present`；
/// 未附加表面时 `present` 不产生任何输出，顶点会一直留在 `pending_vertices` 中。
#[cfg(feature = "gpu")]
#[derive(Debug)]
pub struct WGPURenderer<'window> {
    instance: wgpu::Instance,
    device: Option<Device>,
    queue: Option<Queue>,
    surface: Option<wgpu::Surface<'window>>,
    config: Option<wgpu::SurfaceConfiguration>,
    render_pipeline: Option<RenderPipeline>,
    current_width: u32,
    current_height: u32,
    pending_vertices: Vec<RenderVertex>,
}

#[cfg(feature = "gpu")]
#[derive(Debug, Clone, Copy)]
struct RenderVertex {
    position: [f32; 2],
    color: [f32; 4],
}

#[cfg(feature = "gpu")]
impl<'window> WGPURenderer<'window> {
    /// 创建渲染器并初始化 wgpu 实例；设备、队列与表面留待 `attach_surface` 建立。
    /// 尺寸暂为 1×1，待提交顶点为空。构造过程不会失败，返回值当前恒为 `Ok`。
    pub fn new() -> Result<Self, String> {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            flags: wgpu::InstanceFlags::default(),
            dx12_shader_compiler: wgpu::Dx12Compiler::default(),
            gles_minor_version: wgpu::Gles3MinorVersion::default(),
        });

        Ok(Self {
            instance,
            device: None,
            queue: None,
            surface: None,
            config: None,
            render_pipeline: None,
            current_width: 1,
            current_height: 1,
            pending_vertices: Vec::new(),
        })
    }

    /// 异步为渲染器附加绘制目标并完成 GPU 初始化：创建表面、请求适配器与设备、
    /// 按表面能力选择 sRGB 格式、配置表面并构建渲染管线。
    /// - `target`：窗口或画布等表面目标，可转换为 wgpu 的 `SurfaceTarget`。
    /// 返回 `Ok(())` 表示设备与管线已就绪；创建表面、找不到适配器或请求设备失败时返回错误描述。
    pub async fn attach_surface(
        &mut self,
        target: impl Into<wgpu::SurfaceTarget<'window>>,
    ) -> Result<(), String> {
        let surface = self
            .instance
            .create_surface(target)
            .map_err(|e| format!("Failed to create surface: {:?}", e))?;

        let adapter = self
            .instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .ok_or("No suitable GPU adapter found")?;

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: Some("CAD GPU Device"),
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::default(),
                    memory_hints: wgpu::MemoryHints::default(),
                },
                None,
            )
            .await
            .map_err(|e| format!("Failed to request device: {:?}", e))?;

        let surface_capabilities = surface.get_capabilities(&adapter);
        let format = surface_capabilities
            .formats
            .iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(surface_capabilities.formats[0]);

        let width = self.current_width.max(1);
        let height = self.current_height.max(1);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,
            width,
            height,
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: surface_capabilities.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        surface.configure(&device, &config);

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("CAD Shader"),
            source: wgpu::ShaderSource::Wgsl(VERTEX_SHADER_SOURCE.into()),
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("CAD Pipeline Layout"),
            bind_group_layouts: &[],
            push_constant_ranges: &[],
        });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("CAD Render Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                compilation_options: Default::default(),
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: std::mem::size_of::<RenderVertex>() as wgpu::BufferAddress,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &[
                        wgpu::VertexAttribute {
                            format: wgpu::VertexFormat::Float32x2,
                            offset: 0,
                            shader_location: 0,
                        },
                        wgpu::VertexAttribute {
                            format: wgpu::VertexFormat::Float32x4,
                            offset: std::mem::size_of::<[f32; 2]>() as wgpu::BufferAddress,
                            shader_location: 1,
                        },
                    ],
                }],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                compilation_options: Default::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: Some(wgpu::BlendState::PREMULTIPLIED_ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                unclipped_depth: false,
                polygon_mode: wgpu::PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
            cache: None,
        });

        self.device = Some(device);
        self.queue = Some(queue);
        self.surface = Some(surface);
        self.config = Some(config);
        self.render_pipeline = Some(render_pipeline);
        Ok(())
    }

    /// 调整输出尺寸并重新配置表面，宽高会被夹到至少 1，避免出现零尺寸表面。
    /// - `width`、`height`：新的像素尺寸。
    /// 尚未附加表面（无设备/表面/配置）时只记录尺寸，不做实际配置。
    pub fn resize(&mut self, width: u32, height: u32) {
        self.current_width = width.max(1);
        self.current_height = height.max(1);
        if let (Some(device), Some(surface), Some(config)) =
            (&self.device, &self.surface, &mut self.config)
        {
            config.width = self.current_width;
            config.height = self.current_height;
            surface.configure(device, config);
        }
    }

    /// 清空待提交顶点列表，开始新的一帧；RGBA 参数为接口兼容而保留，当前不影响清屏颜色
    /// （`present` 始终以白色清屏），也不会立即擦除屏幕上已呈现的画面。
    pub fn clear(&mut self, _r: f32, _g: f32, _b: f32, _a: f32) {
        self.pending_vertices.clear();
    }

    /// 追加一条线段到待提交顶点，顶点着色器不做矩阵变换，坐标即裁剪空间坐标（x、y 取 -1.0~1.0）。
    /// 只有调用 `present` 后才会显示；追加的顶点在下一次 `clear` 前一直保留。
    /// - `color`：RGBA 颜色，各分量取 0.0~1.0。
    pub fn draw_line(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, color: &[f32; 4]) {
        let v1 = RenderVertex {
            position: [x1, y1],
            color: *color,
        };
        let v2 = RenderVertex {
            position: [x2, y2],
            color: *color,
        };
        self.pending_vertices.push(v1);
        self.pending_vertices.push(v2);
    }

    /// 用首尾相接的线段逼近圆，逐段追加到待提交顶点。
    /// - `cx`、`cy`：圆心，`r`：半径，均为裁剪空间单位。
    /// - `segments`：分段数，小于 32 时按 32 处理，分段越多圆周越平滑。
    pub fn draw_circle(&mut self, cx: f32, cy: f32, r: f32, color: &[f32; 4], segments: u32) {
        let segments = segments.max(32);

        for i in 0..segments {
            let theta1 = (i as f32 / segments as f32) * std::f32::consts::TAU;
            let theta2 = ((i + 1) as f32 / segments as f32) * std::f32::consts::TAU;

            let x1 = cx + theta1.cos() * r;
            let y1 = cy + theta1.sin() * r;
            let x2 = cx + theta2.cos() * r;
            let y2 = cy + theta2.sin() * r;

            self.draw_line(x1, y1, x2, y2, color);
        }
    }

    /// 提交并呈现当前帧：把待提交顶点上传为顶点缓冲，以白色清屏后逐顶点绘制，最后交换表面缓冲。
    /// 待提交顶点为空，或设备、队列、表面、配置任一缺失（如尚未 `attach_surface`）时直接返回，不报错。
    /// 呈现后不会清空顶点列表，需自行调用 `clear`，否则下一次呈现会重复提交同一批顶点。
    pub fn present(&mut self) {
        if self.pending_vertices.is_empty() {
            return;
        }

        let device = match self.device.take() {
            Some(d) => d,
            None => return,
        };
        let queue = match self.queue.take() {
            Some(q) => q,
            None => {
                self.device = Some(device);
                return;
            }
        };
        let surface = match self.surface.take() {
            Some(s) => s,
            None => {
                self.device = Some(device);
                self.queue = Some(queue);
                return;
            }
        };
        let config = match self.config.take() {
            Some(c) => c,
            None => {
                self.device = Some(device);
                self.queue = Some(queue);
                self.surface = Some(surface);
                return;
            }
        };

        let frame = match surface.get_current_texture() {
            Ok(frame) => frame,
            Err(_) => {
                self.device = Some(device);
                self.queue = Some(queue);
                self.surface = Some(surface);
                self.config = Some(config);
                return;
            }
        };

        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("CAD Render Encoder"),
        });

        let vertex_data: Vec<f32> = self
            .pending_vertices
            .iter()
            .flat_map(|v| v.position.iter().chain(v.color.iter()))
            .copied()
            .collect();

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("CAD Vertex Buffer"),
            contents: bytemuck::cast_slice(&vertex_data),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("CAD Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: 1.0,
                        g: 1.0,
                        b: 1.0,
                        a: 1.0,
                    }),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        if let Some(pipeline) = &self.render_pipeline {
            render_pass.set_pipeline(pipeline);
            render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
            render_pass.draw(0..(self.pending_vertices.len() as u32), 0..1);
        }

        drop(render_pass);

        queue.submit(std::iter::once(encoder.finish()));

        frame.present();

        self.device = Some(device);
        self.queue = Some(queue);
        self.surface = Some(surface);
        self.config = Some(config);
    }

    /// 立即呈现当前帧，等价于调用 `present`。
    pub fn flush(&mut self) {
        self.present();
    }

    /// 该后端在编译期可用（已启用 `gpu` feature），固定返回 `true`；
    /// 实际能否出图仍取决于运行时的 `attach_surface` 是否成功。
    pub fn is_available() -> bool {
        true
    }
}
