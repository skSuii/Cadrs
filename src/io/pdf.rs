//! 最小 PDF 1.4 导出：单页矢量线条。PDF 坐标系 y 向上，与世界坐标一致。
//!
//! 与 EPS 导出共用同一套细分结果（`render::tessellation` 的折线笔画与填充多边形），差别在于
//! 本模块直接生成字节流：4 个间接对象（Catalog、Pages、Page、内容流）加 xref 表与 trailer，
//! MediaBox 即页面尺寸。y 轴方向与世界坐标一致，无需翻转，只按页面左下角做平移。
//!
//! 输出不压缩、不嵌入字体，因此体积小但只适合线条图；调用方负责落盘。

use crate::data_structure::Document;
use crate::render::tessellation::{document_bbox, entity_fills, entity_polylines};

/// 把 文档 导出为单页 PDF 1.4 字节流，世界坐标平移到页面左下角，y 轴不翻转。
///
/// 页面尺寸取 文档 外接矩形四周各外扩 10 个世界单位（单位按 1 单位 = 1 点），内容流中填充
/// 使用各自的 RGB 填充色（`rg` + `f`），线条统一黑色（`RG` + `S`）、线宽 `1`。
/// - `doc`：只读，导出过程不修改 文档。
///
/// 返回带 xref 与 trailer 的完整文件内容，可直接写盘或作为 HTTP 响应体；
/// 当 文档 无实体、无外接矩形或细分后没有任何折线/填充时返回错误。
///
/// # 示例
/// ```ignore
/// let bytes = export(&doc).unwrap();
/// assert!(bytes.starts_with(b"%PDF-1.4"));
/// ```
pub fn export(doc: &Document) -> Result<Vec<u8>, String> {
    if doc.entity_count() == 0 {
        return Err("Canvas is empty, nothing to export".to_string());
    }
    let Some((min, max)) = document_bbox(doc) else {
        return Err("Canvas is empty, nothing to export".to_string());
    };

    let mut polys: Vec<(Vec<(f64, f64)>, bool)> = Vec::new();
    let mut fills: Vec<(Vec<(f64, f64)>, (u8, u8, u8))> = Vec::new();
    for entity in doc.entities().values() {
        for (pts, color) in entity_fills(entity) {
            if pts.len() >= 3 {
                fills.push((pts.iter().map(|p| (p.x, p.y)).collect(), color));
            }
        }
        for (pts, closed) in entity_polylines(entity) {
            if pts.len() >= 2 {
                polys.push((pts.iter().map(|p| (p.x, p.y)).collect(), closed));
            }
        }
    }
    if polys.is_empty() && fills.is_empty() {
        return Err("Nothing to draw".to_string());
    }

    let m = 10.0;
    let w = (max.x - min.x) + 2.0 * m;
    let h = (max.y - min.y) + 2.0 * m;
    // 平移到页面原点（页面左下角）
    let tx = |x: f64| x - min.x + m;
    let ty = |y: f64| y - min.y + m;

    let mut c = String::new();
    c += "1 w\n";
    // 填充（rg 填充色 + f 填充路径）
    for (pts, color) in &fills {
        c += &format!(
            "{:.3} {:.3} {:.3} rg\n",
            color.0 as f64 / 255.0,
            color.1 as f64 / 255.0,
            color.2 as f64 / 255.0
        );
        c += &format!("{:.2} {:.2} m\n", tx(pts[0].0), ty(pts[0].1));
        for p in pts.iter().skip(1) {
            c += &format!("{:.2} {:.2} l\n", tx(p.0), ty(p.1));
        }
        c += "h f\n";
    }
    c += "0 0 0 RG\n";
    for (pts, closed) in &polys {
        let n = pts.len();
        c += &format!("{:.2} {:.2} m\n", tx(pts[0].0), ty(pts[0].1));
        for p in pts.iter().take(n).skip(1) {
            c += &format!("{:.2} {:.2} l\n", tx(p.0), ty(p.1));
        }
        if *closed {
            c += &format!("{:.2} {:.2} l\n", tx(pts[0].0), ty(pts[0].1));
        }
        c += "S\n";
    }

    let obj3 = format!(
        "<< /Type /Page /Parent 2 0 R /MediaBox [0 0 {w:.2} {h:.2}] /Contents 4 0 R /Resources << >> >>"
    );
    let obj4 = format!("<< /Length {} >>\nstream\n{c}endstream", c.len());

    let bodies = [
        "<< /Type /Catalog /Pages 2 0 R >>".to_string(),
        "<< /Type /Pages /Kids [3 0 R] /Count 1 >>".to_string(),
        obj3,
        obj4,
    ];

    let mut out: Vec<u8> = b"%PDF-1.4\n".to_vec();
    let mut offsets = [0usize; 4];
    for (i, body) in bodies.iter().enumerate() {
        offsets[i] = out.len();
        out.extend_from_slice(format!("{} 0 obj\n{}\nendobj\n", i + 1, body).as_bytes());
    }
    let xref_off = out.len();
    out.extend_from_slice(b"xref\n0 5\n");
    out.extend_from_slice(b"0000000000 65535 f \n");
    for off in offsets {
        out.extend_from_slice(format!("{off:010} 00000 n \n").as_bytes());
    }
    out.extend_from_slice(
        format!("trailer\n<< /Size 5 /Root 1 0 R >>\nstartxref\n{xref_off}\n%%EOF").as_bytes(),
    );
    Ok(out)
}
