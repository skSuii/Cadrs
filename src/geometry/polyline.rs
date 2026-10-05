//! 多段线图元模块：定义由顶点序列组成的 [`Polyline`]。
//!
//! 多段线按顶点顺序依次连接成折线，`is_closed` 决定是否额外连接末顶点与首顶点形成闭合轮廓。
//! 长度与坐标均为模型空间单位；填充、凸度与线宽等文档属性由上层实体负责，本模块只描述形状。
//! 注意：`crate::geometry` 对外导出的 `Polyline` 来自 `extended_geometry` 模块，本文件中的同名
//! 类型当前未被模块树引用。

use crate::geometry::{Point, Line};
use crate::math::Vector2;
use std::fmt;
use serde::{Serialize, Deserialize};

/// 由顶点序列构成的多段线。
///
/// 顶点数可以少于 2（此时没有有效线段），是否闭合由 `is_closed` 控制；所有方法都不校验
/// 顶点是否重合或共线。修改类方法（[`Polyline::push`]、[`Polyline::close`] 等）会就地修改自身。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Polyline {
    /// 顶点序列，按连接顺序排列，模型空间单位。
    pub vertices: Vec<Point>,
    /// 是否闭合：`true` 时末顶点与首顶点之间也存在一条线段。
    pub is_closed: bool,
}

impl Polyline {
    /// 创建一个不含顶点、开放的多段线。
    #[inline]
    pub fn new() -> Self {
        Self {
            vertices: Vec::new(),
            is_closed: false,
        }
    }

    /// 创建预留指定顶点容量的开放多段线，顶点数为 0。
    ///
    /// - `capacity`：预分配的顶点容量，仅影响内存分配，不改变长度语义。
    #[inline]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            vertices: Vec::with_capacity(capacity),
            is_closed: false,
        }
    }

    /// 由已有顶点切片复制出开放多段线。
    ///
    /// - `points`：顶点序列，按给定顺序复制，不共享所有权。
    #[inline]
    pub fn from_points(points: &[Point]) -> Self {
        Self {
            vertices: points.to_vec(),
            is_closed: false,
        }
    }

    /// 在末尾追加一个顶点，就地修改自身，不返回新对象。
    ///
    /// - `point`：新增顶点的坐标。
    #[inline]
    pub fn push(&mut self, point: Point) {
        self.vertices.push(point);
    }

    /// 返回顶点数量，闭合多段线的首尾重复顶点不会被合并计数。
    #[inline]
    pub fn vertex_count(&self) -> usize {
        self.vertices.len()
    }

    /// 判断是否没有任何顶点。
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.vertices.is_empty()
    }

    /// 将多段线标记为闭合，就地修改自身。
    ///
    /// 仅当当前为开放状态且顶点数不少于 3 时才生效，否则静默不做任何改变（不报错）。
    #[inline]
    pub fn close(&mut self) {
        if !self.is_closed && self.vertices.len() >= 3 {
            self.is_closed = true;
        }
    }

    /// 将多段线标记为开放，就地修改自身；顶点序列保持不变。
    #[inline]
    pub fn open(&mut self) {
        self.is_closed = false;
    }

    /// 返回第 `index` 条线段。
    ///
    /// - `index`：线段序号，即起点在 `vertices` 中的下标。
    ///
    /// 开放多段线的末条线段起点为倒数第二个顶点；闭合多段线的线段数等于顶点数，最后一条由
    /// 末顶点连回首顶点。顶点不足 2 个、或开放状态下 `index` 已是最后一个顶点时返回 `None`；
    /// `index` 越界会触发索引越界 panic。
    #[inline]
    pub fn segment(&self, index: usize) -> Option<Line> {
        if self.vertices.len() < 2 {
            return None;
        }
        
        let next_index = if self.is_closed {
            (index + 1) % self.vertices.len()
        } else if index + 1 < self.vertices.len() {
            index + 1
        } else {
            return None;
        };
        
        Some(Line::new(self.vertices[index], self.vertices[next_index]))
    }

    /// 返回折线总长。
    ///
    /// 顶点少于 2 个时为 `0.0`；闭合且顶点数不少于 3 时会额外计入末顶点到首顶点的闭合段长度。
    #[inline]
    pub fn total_length(&self) -> f64 {
        if self.vertices.len() < 2 {
            return 0.0;
        }

        let mut length = 0.0;
        for i in 0..self.vertices.len() - 1 {
            length += self.vertices[i].distance_to(&self.vertices[i + 1]);
        }
        
        if self.is_closed && self.vertices.len() >= 3 {
            length += self.vertices.last().unwrap().distance_to(&self.vertices[0]);
        }
        
        length
    }

    /// 返回轴对齐包围盒，包含全部顶点，端点为最小值与最大值处的顶点。
    ///
    /// 返回 `(min, max)`，其中 `min` 三个坐标分量均为最小值、`max` 均为最大值；包围盒按顶点
    /// 计算，不含曲线凸出部分。无顶点时返回 `None`。
    #[inline]
    pub fn bounding_box(&self) -> Option<(Point, Point)> {
        if self.vertices.is_empty() {
            return None;
        }

        let mut min_x = self.vertices[0].x;
        let mut max_x = self.vertices[0].x;
        let mut min_y = self.vertices[0].y;
        let mut max_y = self.vertices[0].y;
        let mut min_z = self.vertices[0].z;
        let mut max_z = self.vertices[0].z;

        for vertex in &self.vertices {
            min_x = min_x.min(vertex.x);
            max_x = max_x.max(vertex.x);
            min_y = min_y.min(vertex.y);
            max_y = max_y.max(vertex.y);
            min_z = min_z.min(vertex.z);
            max_z = max_z.max(vertex.z);
        }

        Some((
            Point::new(min_x, min_y, min_z),
            Point::new(max_x, max_y, max_z),
        ))
    }

    /// 返回顶点坐标的算术平均值（顶点形心）。
    ///
    /// 各顶点等权，不按边长加权，因此结果通常不同于按面积计算的形心；无顶点时返回 `None`。
    #[inline]
    pub fn centroid(&self) -> Option<Point> {
        if self.vertices.is_empty() {
            return None;
        }

        let mut cx = 0.0;
        let mut cy = 0.0;
        let mut cz = 0.0;

        for vertex in &self.vertices {
            cx += vertex.x;
            cy += vertex.y;
            cz += vertex.z;
        }

        let n = self.vertices.len() as f64;
        Some(Point::new(cx / n, cy / n, cz / n))
    }
}

impl Default for Polyline {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for Polyline {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Polyline(vertices: {}, is_closed: {})",
            self.vertex_count(),
            self.is_closed
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_polyline_creation() {
        let polyline = Polyline::new();
        assert!(polyline.is_empty());
        assert_eq!(polyline.vertex_count(), 0);
    }

    #[test]
    fn test_polyline_push() {
        let mut polyline = Polyline::new();
        polyline.push(Point::new(0.0, 0.0, 0.0));
        polyline.push(Point::new(1.0, 0.0, 0.0));
        polyline.push(Point::new(1.0, 1.0, 0.0));
        
        assert_eq!(polyline.vertex_count(), 3);
    }

    #[test]
    fn test_polyline_length() {
        let mut polyline = Polyline::new();
        polyline.push(Point::new(0.0, 0.0, 0.0));
        polyline.push(Point::new(3.0, 0.0, 0.0));
        polyline.push(Point::new(3.0, 4.0, 0.0));
        
        assert!((polyline.total_length() - 7.0).abs() < 1e-10);
    }

    #[test]
    fn test_polyline_close() {
        let mut polyline = Polyline::new();
        polyline.push(Point::new(0.0, 0.0, 0.0));
        polyline.push(Point::new(1.0, 0.0, 0.0));
        polyline.push(Point::new(1.0, 1.0, 0.0));
        polyline.close();
        
        assert!(polyline.is_closed);
    }
}
