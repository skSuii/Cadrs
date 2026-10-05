//! 视口模块：维护世界坐标与屏幕像素坐标之间的映射关系。
//! `ViewRect` 描述世界坐标下的轴对齐可视矩形，`Viewport` 在其之上记录屏幕尺寸与缩放倍率，
//! 渲染后端、捕捉与拾取等上层逻辑都应通过本模块换算坐标，避免各自实现一套变换。

use crate::geometry::Point;
use crate::math::Vector2;

/// 世界坐标系下的轴对齐矩形，用于表示视图范围或实体包围盒。
/// 四边均为闭边界，落在边界上的点也算作在矩形内；坐标单位为世界单位。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ViewRect {
    /// x 方向的较小边界。
    pub min_x: f64,
    /// y 方向的较小边界。
    pub min_y: f64,
    /// x 方向的较大边界。
    pub max_x: f64,
    /// y 方向的较大边界。
    pub max_y: f64,
}

impl ViewRect {
    /// 用四个边界值构造矩形，参数按原样保存，不做排序或合法性校验。
    /// - `min_x`、`min_y`、`max_x`、`max_y`：世界坐标下的边界值。
    pub fn new(min_x: f64, min_y: f64, max_x: f64, max_y: f64) -> Self {
        Self { min_x, min_y, max_x, max_y }
    }

    /// x 方向的跨度 `max_x - min_x`；边界反向时为负值，调用方需自行保证边界有序。
    pub fn width(&self) -> f64 {
        self.max_x - self.min_x
    }

    /// y 方向的跨度 `max_y - min_y`；边界反向时为负值。
    pub fn height(&self) -> f64 {
        self.max_y - self.min_y
    }

    /// 矩形中心的世界坐标 `(x, y)`。
    pub fn center(&self) -> (f64, f64) {
        ((self.min_x + self.max_x) / 2.0, (self.min_y + self.max_y) / 2.0)
    }

    /// 判断世界坐标点是否在矩形内，四条边界上的点视为包含（闭区间）。
    /// # 示例
    /// ```
    /// use cadrs::render::ViewRect;
    /// let rect = ViewRect::new(0.0, 0.0, 100.0, 100.0);
    /// assert!(rect.contains(50.0, 50.0));
    /// assert!(!rect.contains(150.0, 150.0));
    /// ```
    pub fn contains(&self, x: f64, y: f64) -> bool {
        x >= self.min_x && x <= self.max_x && y >= self.min_y && y <= self.max_y
    }

    /// 以中心为基准按比例缩放，返回新矩形，`self` 保持不变。
    /// - `factor`：宽高缩放因子，大于 1 表示扩大可视范围，小于 1 表示收缩。
    pub fn expand(&self, factor: f64) -> Self {
        let cx = (self.min_x + self.max_x) / 2.0;
        let cy = (self.min_y + self.max_y) / 2.0;
        let hw = self.width() / 2.0 * factor;
        let hh = self.height() / 2.0 * factor;
        
        Self::new(cx - hw, cy - hh, cx + hw, cy + hh)
    }
}

/// 视口：把世界坐标映射到屏幕像素坐标。
/// `view_rect` 是当前可见的世界区域，映射时会把它拉伸铺满 `size` 指定的屏幕尺寸，
/// 并且屏幕 y 轴向下、世界 y 轴向上，因此 y 方向在换算中翻转。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Viewport {
    /// 视图平移量（世界单位），由 `pan` 累加，默认 `(0.0, 0.0)`。
    pub origin: (f64, f64),
    /// 屏幕像素尺寸 `(宽, 高)`，默认 `(800.0, 600.0)`。
    pub size: (f64, f64),
    /// 累计缩放倍率，`zoom_at` 会把新因子乘到该值上，默认 `1.0`。
    pub zoom: f64,
    /// 视口旋转角（弧度），仅作为状态保存，当前的坐标换算尚未使用该字段。
    pub rotation: f64,
    /// 当前可见的世界坐标矩形，决定 `world_to_screen` 的映射范围。
    pub view_rect: ViewRect,
}

impl Viewport {
    /// 创建默认视口：屏幕 800×600 像素、缩放 1.0、旋转 0，可视区域为 `(0, 0)-(800, 600)`。
    pub fn new() -> Self {
        Self {
            origin: (0.0, 0.0),
            size: (800.0, 600.0),
            zoom: 1.0,
            rotation: 0.0,
            view_rect: ViewRect::new(0.0, 0.0, 800.0, 600.0),
        }
    }

    /// 创建指定屏幕像素尺寸的视口，可视区域初始为 `(0, 0)-(width, height)`，缩放为 1.0。
    /// - `width`、`height`：屏幕像素尺寸，不做正数校验。
    pub fn with_size(width: f64, height: f64) -> Self {
        let view_rect = ViewRect::new(0.0, 0.0, width, height);
        Self {
            origin: (0.0, 0.0),
            size: (width, height),
            zoom: 1.0,
            rotation: 0.0,
            view_rect,
        }
    }

    /// 直接设置可视区域：以 `(x, y)` 为左下角、按给定宽高生成 `view_rect`。
    /// 不修改 `origin`、`zoom`、`size`，因此不会改变屏幕上的缩放比例。
    pub fn set_view(&mut self, x: f64, y: f64, width: f64, height: f64) {
        self.view_rect = ViewRect::new(x, y, x + width, y + height);
    }

    /// 平移视图：`origin` 与 `view_rect` 同步偏移 `(dx, dy)`，缩放倍率与可视范围大小不变。
    /// - `dx`、`dy`：世界单位下的位移量，正值表示视图向正方向移动。
    pub fn pan(&mut self, dx: f64, dy: f64) {
        self.origin.0 += dx;
        self.origin.1 += dy;
        self.view_rect.min_x += dx;
        self.view_rect.max_x += dx;
        self.view_rect.min_y += dy;
        self.view_rect.max_y += dy;
    }

    /// 按因子缩放视图：可视范围宽高变为原来的 `1 / factor`，并围绕当前可视区域中心收缩或扩张。
    /// 缩放结果通过直接改写 `view_rect` 体现，`zoom` 字段同步累乘。
    /// - `factor`：缩放因子，大于 1 表示放大；必须为正，为 0 时结果无意义。
    /// - `center_x`、`center_y`：为指定缩放中心预留，当前实现未使用该中心，始终以可视区域中心缩放。
    pub fn zoom_at(&mut self, factor: f64, center_x: f64, center_y: f64) {
        let before_width = self.view_rect.width();
        let before_height = self.view_rect.height();
        
        self.zoom *= factor;
        
        let after_width = before_width / factor;
        let after_height = before_height / factor;
        
        let cx = (self.view_rect.min_x + self.view_rect.max_x) / 2.0;
        let cy = (self.view_rect.min_y + self.view_rect.max_y) / 2.0;
        
        self.view_rect.min_x = cx - after_width / 2.0;
        self.view_rect.max_x = cx + after_width / 2.0;
        self.view_rect.min_y = cy - after_height / 2.0;
        self.view_rect.max_y = cy + after_height / 2.0;
    }

    /// 缩放到给定世界矩形：以 `rect` 为中心建立可视区域，四周额外留出 10% 余量以免图形贴边。
    /// 仅改写 `view_rect`，不更新 `origin` 与 `zoom`。
    /// - `rect`：目标世界矩形，通常来自 `document_bbox` 等包围盒计算。
    pub fn zoom_extents(&mut self, rect: ViewRect) {
        let padding = 1.1;
        let width = rect.width() * padding;
        let height = rect.height() * padding;
        
        let cx = (rect.min_x + rect.max_x) / 2.0;
        let cy = (rect.min_y + rect.max_y) / 2.0;
        
        self.view_rect.min_x = cx - width / 2.0;
        self.view_rect.max_x = cx + width / 2.0;
        self.view_rect.min_y = cy - height / 2.0;
        self.view_rect.max_y = cy + height / 2.0;
    }

    /// 世界坐标转屏幕像素坐标，y 方向翻转（世界 y 增大对应屏幕 y 减小）。
    /// 可视区域内的点映射到 `0..=size` 的像素范围，区域外的点会得到越界值，不做裁剪。
    /// - `x`、`y`：世界坐标，单位与 `view_rect` 一致。
    /// 返回 `(screen_x, screen_y)`，单位是屏幕像素。
    /// # 示例
    /// ```
    /// use cadrs::render::Viewport;
    /// let mut viewport = Viewport::with_size(800.0, 600.0);
    /// viewport.set_view(0.0, 0.0, 100.0, 100.0);
    /// let (sx, sy) = viewport.world_to_screen(50.0, 50.0); // (400.0, 300.0)
    /// ```
    pub fn world_to_screen(&self, x: f64, y: f64) -> (f64, f64) {
        let vw = self.view_rect.width();
        let vh = self.view_rect.height();
        let sw = self.size.0;
        let sh = self.size.1;
        
        let screen_x = (x - self.view_rect.min_x) / vw * sw;
        let screen_y = sh - (y - self.view_rect.min_y) / vh * sh;
        
        (screen_x, screen_y)
    }

    /// 屏幕像素坐标转世界坐标，是 `world_to_screen` 的逆变换，同样翻转 y 方向。
    /// - `x`、`y`：以左上角为原点、y 轴向下、单位为像素的屏幕坐标。
    /// 返回世界坐标 `(world_x, world_y)`。
    pub fn screen_to_world(&self, x: f64, y: f64) -> (f64, f64) {
        let vw = self.view_rect.width();
        let vh = self.view_rect.height();
        let sw = self.size.0;
        let sh = self.size.1;
        
        let world_x = self.view_rect.min_x + x / sw * vw;
        let world_y = self.view_rect.min_y + (sh - y) / sh * vh;
        
        (world_x, world_y)
    }
}

impl Default for Viewport {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_view_rect() {
        let rect = ViewRect::new(0.0, 0.0, 100.0, 100.0);
        assert_eq!(rect.width(), 100.0);
        assert_eq!(rect.height(), 100.0);
        assert!(rect.contains(50.0, 50.0));
        assert!(!rect.contains(150.0, 150.0));
    }

    #[test]
    fn test_viewport_world_to_screen() {
        let mut viewport = Viewport::with_size(800.0, 600.0);
        viewport.set_view(0.0, 0.0, 100.0, 100.0);
        
        let (sx, sy) = viewport.world_to_screen(50.0, 50.0);
        assert!((sx - 400.0).abs() < 1e-10);
        assert!((sy - 300.0).abs() < 1e-10);
    }

    #[test]
    fn test_viewport_pan() {
        let mut viewport = Viewport::new();
        viewport.pan(100.0, 50.0);
        
        assert_eq!(viewport.origin.0, 100.0);
        assert_eq!(viewport.origin.1, 50.0);
    }
}
