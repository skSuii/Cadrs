//! 基础渲染抽象：定义绘制样式 `RenderStyle` 与线型 `LinePattern`、统一的 `Renderer` 绘制接口，
//! 以及内存帧缓冲 `RenderBuffer`、待渲染实体队列 `RenderQueue` 和实体绘制辅助 `EntityRenderer`。
//! 各具体后端（software_renderer、svg_renderer、gpu_renderer）按 `Renderer` 约定实现绘制，
//! 上层通过 `RenderQueue` 收集待绘实体与样式，再由后端消费。

use crate::data_structure::{Entity, EntityGeometry};
use crate::geometry::{Point, Line, Circle, Arc, Ellipse, Polyline};
use crate::render::viewport::Viewport;
use crate::math::Vector2;

/// 单个图元的绘制样式。
/// 颜色统一使用 8 位 RGB 分量；线宽与坐标同单位（像素后端中即像素数），不随缩放自动变化。
#[derive(Debug, Clone, PartialEq)]
pub struct RenderStyle {
    /// 线条颜色，`(红, 绿, 蓝)`，每个分量取值范围 0~255。
    pub color: (u8, u8, u8),
    /// 线宽，软后端按像素处理。
    pub line_width: f64,
    /// 线型（实线、虚线、点线等）。
    pub line_pattern: LinePattern,
    /// 填充色；`None` 表示只描边不填充。
    pub fill: Option<(u8, u8, u8)>,
    /// 是否启用抗锯齿；SVG 后端据此输出 `shape-rendering` 属性，软件后端暂不生效。
    pub anti_aliasing: bool,
}

/// 线型模式，决定虚线段与空白段的交替方式。
#[derive(Debug, Clone, PartialEq)]
pub enum LinePattern {
    /// 实线，无断点。
    Solid,
    /// 虚线，按固定间隔断开。
    Dashed,
    /// 点线，由密集的短划组成。
    Dotted,
    /// 点划线，长短划交替。
    DashDot,
    /// 自定义线型，元素依次表示实线段与空白段的长度。
    Custom(Vec<f64>),
}

impl Default for RenderStyle {
    fn default() -> Self {
        Self {
            color: (0, 0, 0),
            line_width: 1.0,
            line_pattern: LinePattern::Solid,
            fill: None,
            anti_aliasing: true,
        }
    }
}

/// 统一的绘制接口，屏蔽软件、SVG、GPU 等后端的差异。
/// 典型调用顺序为 `initialize` → `clear` → 若干 `draw_*` → `present`；
/// 所有方法都直接修改后端内部状态，图元按调用先后顺序叠加。
pub trait Renderer {
    /// 初始化后端并重置内部绘制状态；失败时返回可读的错误描述。
    fn initialize(&mut self) -> Result<(), String>;
    /// 用单色覆盖整个画面，通常作为一帧绘制的第一步。
    /// - `color`：背景色，`(红, 绿, 蓝)`，每个分量 0~255。
    fn clear(&mut self, color: (u8, u8, u8));
    /// 绘制一条线段。
    /// - `x1`、`y1`、`x2`、`y2`：起点与终点坐标，单位由调用方约定（通常为屏幕像素）。
    /// - `style`：绘制样式，只读，不会被修改。
    fn draw_line(&mut self, x1: f64, y1: f64, x2: f64, y2: f64, style: &RenderStyle);
    /// 绘制整圆。
    /// - `cx`、`cy`：圆心坐标。
    /// - `r`：半径，需为正数。
    fn draw_circle(&mut self, cx: f64, cy: f64, r: f64, style: &RenderStyle);
    /// 绘制一段圆弧。
    /// - `start_angle`、`end_angle`：起始与终止角度，角度制（度），实现内部按需换算为弧度。
    /// - `r`：半径；角度差超过 180 度时按大弧绘制。
    fn draw_arc(&mut self, cx: f64, cy: f64, r: f64, start_angle: f64, end_angle: f64, style: &RenderStyle);
    /// 绘制折线，按数组顺序依次连接相邻顶点，首尾不自动闭合。
    /// - `points`：顶点坐标序列，少于 2 个点时不产生可见线段。
    fn draw_polyline(&mut self, points: &[(f64, f64)], style: &RenderStyle);
    /// 绘制点标记，通常用于表示节点或控制点。
    /// - `size`：标记直径，实现按 `size / 2` 作为绘制半径。
    fn draw_point(&mut self, x: f64, y: f64, size: f64, style: &RenderStyle);
    /// 绘制单行文字，锚点为 `(x, y)`。
    /// - `size`：字高。
    /// - `rotation`：旋转角，角度制（度），正值表示逆时针。
    fn draw_text(&mut self, x: f64, y: f64, text: &str, size: f64, rotation: f64, style: &RenderStyle);
    /// 绘制带完整字体属性的单行文字，用于标注与文字实体的高质量输出。
    /// - `font_name`：字体族名，如 `"Arial"`。
    /// - `width_factor`：字宽因子，用于横向拉伸字形。
    /// - `bold`、`italic`、`underline`：是否加粗、倾斜、加下划线。
    /// - `alignment`：水平对齐，`0` 左对齐、`1` 居中、`2` 右对齐，其余值按左对齐处理。
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
    );
    /// 提交当前帧并显示到目标表面；调用后图元状态是否保留由后端决定。
    fn present(&mut self);
    /// 刷新缓冲并释放本帧占用的临时资源，不改变已绘制内容。
    fn flush(&mut self);
}

/// 内存帧缓冲，按行优先保存 32 位像素，每像素编码为 `0x00RRGGBB`。
/// 坐标原点在左上角，x 向右、y 向下，尺寸单位为像素。
pub struct RenderBuffer {
    width: usize,
    height: usize,
    pixels: Vec<u32>,
}

impl RenderBuffer {
    /// 创建指定像素尺寸的帧缓冲，所有像素初始化为 0（黑色）。
    /// - `width`、`height`：像素列数与行数，内部按 `width * height` 分配内存。
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            pixels: vec![0; width * height],
        }
    }

    /// 用给定颜色覆盖全部像素，尺寸与坐标原点不变。
    /// - `color`：填充色，`(红, 绿, 蓝)`，每个分量 0~255。
    /// # 示例
    /// ```
    /// use cadrs::render::RenderBuffer;
    /// let mut buffer = RenderBuffer::new(2, 2);
    /// buffer.clear((255, 255, 255));
    /// assert_eq!(buffer.pixels()[0], 0x00FFFFFF);
    /// ```
    pub fn clear(&mut self, color: (u8, u8, u8)) {
        let color_value = ((color.0 as u32) << 16) | ((color.1 as u32) << 8) | (color.2 as u32);
        for pixel in &mut self.pixels {
            *pixel = color_value;
        }
    }

    /// 用 Bresenham 算法绘制线段，坐标取整为像素。
    /// 只写入缓冲范围内的像素，越界部分被跳过，因此线段在边界处会被裁剪；端点包含在绘制范围内。
    /// - `x0`、`y0`、`x1`、`y1`：起点与终点的像素坐标，可为负值。
    pub fn draw_line(&mut self, x0: i32, y0: i32, x1: i32, y1: i32, color: (u8, u8, u8)) {
        let dx = (x1 - x0).abs();
        let dy = (y1 - y0).abs();
        let sx = if x0 < x1 { 1 } else { -1 };
        let sy = if y0 < y1 { 1 } else { -1 };
        let mut err = if dx > dy { dx } else { dy } as i32 / 2;
        
        let mut x = x0;
        let mut y = y0;
        
        while x >= 0 && x < self.width as i32 && y >= 0 && y < self.height as i32 {
            self.set_pixel(x as usize, y as usize, color);
            
            if x == x1 && y == y1 {
                break;
            }
            
            let e2 = 2 * err;
            if e2 > -dx {
                err -= dx;
                x += sx;
            }
            if e2 < dy {
                err += dy;
                y += sy;
            }
        }
    }

    /// 写入单个像素；越界坐标会被静默忽略，不报错也不扩容。
    /// - `x`、`y`：像素坐标，原点在左上角。
    /// # 示例
    /// ```
    /// use cadrs::render::RenderBuffer;
    /// let mut buffer = RenderBuffer::new(2, 2);
    /// buffer.set_pixel(1, 0, (255, 0, 0));
    /// assert_eq!(buffer.pixels()[1], 0x00FF0000);
    /// ```
    pub fn set_pixel(&mut self, x: usize, y: usize, color: (u8, u8, u8)) {
        if x < self.width && y < self.height {
            let idx = y * self.width + x;
            self.pixels[idx] = ((color.0 as u32) << 16) | ((color.1 as u32) << 8) | (color.2 as u32);
        }
    }

    /// 缓冲宽度（像素列数）。
    pub fn width(&self) -> usize {
        self.width
    }

    /// 缓冲高度（像素行数）。
    pub fn height(&self) -> usize {
        self.height
    }

    /// 按行优先顺序返回像素只读切片，长度为 `width * height`，元素为 `0x00RRGGBB`。
    pub fn pixels(&self) -> &[u32] {
        &self.pixels
    }
}

/// 待渲染队列：持有视口以及按加入顺序排列的「实体 + 样式」二元组。
/// 队列本身不做坐标变换，仅在 `entities` 中按插入顺序保存数据，供渲染循环批量消费。
pub struct RenderQueue {
    viewport: Viewport,
    entities: Vec<(Entity, RenderStyle)>,
}

impl RenderQueue {
    /// 用给定视口创建空队列，初始不含任何实体。
    /// - `viewport`：后续绘制使用的视口，所有权移入队列。
    pub fn new(viewport: Viewport) -> Self {
        Self {
            viewport,
            entities: Vec::new(),
        }
    }

    /// 把实体及其绘制样式追加到队尾，实体所有权移入队列，后加入者后绘制。
    /// - `entity`：待渲染实体；同一实体可多次加入以叠加不同样式。
    pub fn add_entity(&mut self, entity: Entity, style: RenderStyle) {
        self.entities.push((entity, style));
    }

    /// 移除队列中的全部实体，保留视口，队列可继续复用。
    pub fn clear(&mut self) {
        self.entities.clear();
    }

    /// 按实体所属图层 id 的字符串顺序就地排序，使同图层实体相邻，从而决定绘制先后。
    /// 排序会改变 `entities` 的返回顺序，但不增删任何条目。
    pub fn sort_by_layer(&mut self) {
        self.entities.sort_by_key(|(e, _): &(Entity, RenderStyle)| {
            e.layer_id().to_string()
        });
    }

    /// 按当前顺序返回队列表项的只读切片，元素为 `(实体, 样式)`。
    pub fn entities(&self) -> &[(Entity, RenderStyle)] {
        &self.entities
    }
}

/// 实体绘制辅助类型，把实体几何分类后转成坐标并输出，本身不保存状态。
/// 当前实现仅把结果打印到标准输出，作为接入真实 `Renderer` 前的占位与调试手段。
pub struct EntityRenderer;

impl EntityRenderer {
    /// 处理直线实体：把两端点由世界坐标换算为屏幕坐标后输出。
    /// - `viewport`：提供 world_to_screen 变换。
    /// - `style`：为接口一致性保留，当前未参与输出。
    pub fn render_line(line: &Line, viewport: &Viewport, style: &RenderStyle) {
        let start = line.start;
        let end = line.end;
        
        let (sx1, sy1) = viewport.world_to_screen(start.x, start.y);
        let (sx2, sy2) = viewport.world_to_screen(end.x, end.y);
        
        println!("Line: ({:.1}, {:.1}) -> ({:.1}, {:.1})", sx1, sy1, sx2, sy2);
    }

    /// 处理圆实体：圆心做坐标变换，半径由变换后包围矩形的宽度推算，因此会随缩放变化。
    /// - `circle`：世界坐标下的圆，半径以世界单位给出。
    /// - `style`：为接口一致性保留，当前未参与输出。
    pub fn render_circle(circle: &Circle, viewport: &Viewport, style: &RenderStyle) {
        let (cx, cy) = viewport.world_to_screen(circle.center.x, circle.center.y);
        
        let (min_x, min_y) = viewport.world_to_screen(
            circle.center.x - circle.radius,
            circle.center.y - circle.radius,
        );
        let (max_x, _) = viewport.world_to_screen(
            circle.center.x + circle.radius,
            circle.center.y + circle.radius,
        );
        
        let radius = ((max_x - min_x).abs() / 2.0).abs();
        
        println!("Circle: center=({:.1}, {:.1}), radius={:.1}", cx, cy, radius);
    }

    /// 处理圆弧实体：圆心、起点与终点分别换算到屏幕坐标后输出。
    /// 起止点由 `Arc::start_angle`、`Arc::end_angle` 求得，角度为弧度。
    /// - `style`：为接口一致性保留，当前未参与输出。
    pub fn render_arc(arc: &Arc, viewport: &Viewport, style: &RenderStyle) {
        let (cx, cy) = viewport.world_to_screen(arc.center.x, arc.center.y);
        
        let start = arc.point_at_angle(arc.start_angle);
        let end = arc.point_at_angle(arc.end_angle);
        let (sx1, sy1) = viewport.world_to_screen(start.x, start.y);
        let (sx2, sy2) = viewport.world_to_screen(end.x, end.y);
        
        println!("Arc: center=({:.1}, {:.1}), start=({:.1}, {:.1}), end=({:.1}, {:.1})", 
                 cx, cy, sx1, sy1, sx2, sy2);
    }

    /// 处理折线实体：按顶点顺序逐个换算屏幕坐标并拼接为路径输出，不自动闭合。
    /// - `style`：为接口一致性保留，当前未参与输出。
    pub fn render_polyline(polyline: &Polyline, viewport: &Viewport, style: &RenderStyle) {
        let mut path = String::new();
        for (i, vertex) in polyline.vertices.iter().enumerate() {
            let (sx, sy) = viewport.world_to_screen(vertex.x, vertex.y);
            if i > 0 {
                path.push_str(" -> ");
            }
            path.push_str(&format!("({:.1}, {:.1})", sx, sy));
        }
        println!("Polyline: {}", path);
    }

    /// 按实体几何类型分派到对应的绘制方法。
    /// 目前仅支持直线、圆、圆弧与折线；其他几何类型不产生图形输出。
    pub fn render(entity: &Entity, viewport: &Viewport, style: &RenderStyle) {
        match entity.geometry() {
            crate::data_structure::EntityGeometry::Line(line) => {
                Self::render_line(line, viewport, style);
            }
            crate::data_structure::EntityGeometry::Circle(circle) => {
                Self::render_circle(circle, viewport, style);
            }
            crate::data_structure::EntityGeometry::Arc(arc) => {
                Self::render_arc(arc, viewport, style);
            }
            crate::data_structure::EntityGeometry::Polyline(polyline) => {
                Self::render_polyline(polyline, viewport, style);
            }
            _ => {
                println!("Entity type not fully supported for rendering yet");
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_buffer() {
        let buffer = RenderBuffer::new(800, 600);
        assert_eq!(buffer.width(), 800);
        assert_eq!(buffer.height(), 600);
    }

    #[test]
    fn test_render_style() {
        let style = RenderStyle::default();
        assert_eq!(style.color, (0, 0, 0));
        assert_eq!(style.line_width, 1.0);
    }

    #[test]
    fn test_render_queue() {
        let viewport = Viewport::new();
        let queue = RenderQueue::new(viewport);
        assert!(queue.entities().is_empty());
    }
}
