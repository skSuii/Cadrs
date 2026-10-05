//! 位图导出：软件光栅化（线条距离场抗锯齿 + 2x 超采样，填充偶奇规则）。
//! 支持 PNG / BMP / JPEG / WebP。
//!
//! 先把 文档 经 `render::tessellation` 拆成线段与填充多边形，再在固定分辨率上限内自建像素缓冲：
//! 线条按到线段的距离场取覆盖度（最大混合，只影响亮度不改变色相），填充按偶奇规则填色，
//! 最后 2x2 下采样合成并交给 `image` crate 编码。输出不透明，背景固定为白色，因此不支持透明通道。
//!
//! 像素尺寸由 文档 外接矩形加 10 个世界单位边距后的宽高比决定，长边约 1600 像素（内部
//! 超采样缓冲为其 2 倍），最大单边被限制在 4000 像素。

use std::io::Cursor;

use crate::data_structure::Document;
use crate::render::tessellation::{document_bbox, entity_fills, entity_polylines};

/// 位图输出格式：决定编码器与文件扩展名，不携带任何图像参数（质量、色深使用 `image` 默认值）。
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum RasterFormat {
    /// PNG：无损，适合线条图与需要后续再编辑的场景。
    Png,
    /// BMP：未压缩位图，兼容性最好但体积最大。
    Bmp,
    /// JPEG：有损压缩，线条边缘可能出现振铃；背景为白色，没有透明区域。
    Jpeg,
    /// WebP：无损或有损由编码器默认设置决定，体积通常小于 PNG。
    WebP,
}

impl RasterFormat {
    /// 该格式惯用的文件扩展名（不含点号），用于拼接输出文件名。
    ///
    /// JPEG 返回 `jpg` 而不是 `jpeg`，`Bmp` 返回小写 `bmp`。
    pub fn extension(self) -> &'static str {
        match self {
            RasterFormat::Png => "png",
            RasterFormat::Bmp => "bmp",
            RasterFormat::Jpeg => "jpg",
            RasterFormat::WebP => "webp",
        }
    }

    fn image_format(self) -> image::ImageFormat {
        match self {
            RasterFormat::Png => image::ImageFormat::Png,
            RasterFormat::Bmp => image::ImageFormat::Bmp,
            RasterFormat::Jpeg => image::ImageFormat::Jpeg,
            RasterFormat::WebP => image::ImageFormat::WebP,
        }
    }
}

/// 把 文档 光栅化为位图字节流。最大边约 1600px。
///
/// 输出像素尺寸由 文档 外接矩形加 10 个世界单位边距后的宽高比决定：长边缩放到 1600 像素，
/// 短边等比取整并至少为 1，单边上限 4000 像素。背景为不透明白色，填充按偶奇规则先上色，
/// 线条再以黑色（1 输出像素宽，距离场抗锯齿）叠加在其上。
/// - `doc`：只读，导出过程不修改 文档。
/// - `fmt`：目标编码格式，决定编码器本身，不改变像素尺寸。
///
/// 返回可直接写盘的编码后字节流（PNG/BMP/WebP 无损，JPEG 有损）；
/// 当 文档 无实体、无外接矩形、细分后没有任何线段或填充，或编码失败时返回错误。
///
/// # 示例
/// ```ignore
/// let png = export(&doc, RasterFormat::Png).unwrap();
/// assert_eq!(RasterFormat::Png.extension(), "png");
/// ```
pub fn export(doc: &Document, fmt: RasterFormat) -> Result<Vec<u8>, String> {
    if doc.entity_count() == 0 {
        return Err("Canvas is empty, nothing to export".to_string());
    }
    let Some((min, max)) = document_bbox(doc) else {
        return Err("Canvas is empty, nothing to export".to_string());
    };

    // 收集所有线段与填充（世界坐标）
    let mut segments: Vec<((f64, f64), (f64, f64))> = Vec::new();
    let mut fills: Vec<(Vec<(f64, f64)>, (u8, u8, u8))> = Vec::new();
    for entity in doc.entities().values() {
        for (pts, color) in entity_fills(entity) {
            if pts.len() >= 3 {
                fills.push((pts.iter().map(|p| (p.x, p.y)).collect(), color));
            }
        }
        for (pts, closed) in entity_polylines(entity) {
            let n = pts.len();
            if n < 2 {
                continue;
            }
            for i in 0..n - 1 {
                segments.push(((pts[i].x, pts[i].y), (pts[i + 1].x, pts[i + 1].y)));
            }
            if closed && n > 2 {
                segments.push(((pts[n - 1].x, pts[n - 1].y), (pts[0].x, pts[0].y)));
            }
        }
    }
    if segments.is_empty() && fills.is_empty() {
        return Err("Nothing to rasterize".to_string());
    }

    let margin = 10.0;
    let minx = min.x - margin;
    let maxy = max.y + margin;
    let w = (max.x - min.x) + 2.0 * margin;
    let h = (max.y - min.y) + 2.0 * margin;

    let scale = 1600.0 / w.max(h).max(1e-6);
    let pw = ((w * scale).round() as u32).clamp(1, 4000);
    let ph = ((h * scale).round() as u32).clamp(1, 4000);

    // 2x 超采样
    let (sw, sh) = (pw * 2, ph * 2);
    let ss = scale * 2.0; // 世界 → 超采样像素
    let to_ss = |x: f64, y: f64| ((x - minx) * ss, (maxy - y) * ss);

    // 距离场覆盖（max 混合）：线宽 1 输出像素 = 2 超采样像素，半宽 1.0
    let hw = 1.0f64;
    let mut alpha = vec![0f32; (sw as usize) * (sh as usize)];

    for ((x1, y1), (x2, y2)) in &segments {
        let (sx1, sy1) = to_ss(*x1, *y1);
        let (sx2, sy2) = to_ss(*x2, *y2);
        let reach = hw + 1.5;
        let bx1 = (sx1.min(sx2) - reach).floor().max(0.0) as i64;
        let bx2 = (sx1.max(sx2) + reach).ceil().min(sw as f64 - 1.0) as i64;
        let by1 = (sy1.min(sy2) - reach).floor().max(0.0) as i64;
        let by2 = (sy1.max(sy2) + reach).ceil().min(sh as f64 - 1.0) as i64;
        if bx2 < bx1 || by2 < by1 {
            continue;
        }
        let dx = sx2 - sx1;
        let dy = sy2 - sy1;
        let len_sq = dx * dx + dy * dy;
        for py in by1..=by2 {
            let fy = py as f64 + 0.5;
            for px in bx1..=bx2 {
                let fx = px as f64 + 0.5;
                let d = if len_sq <= f64::EPSILON {
                    ((fx - sx1) * (fx - sx1) + (fy - sy1) * (fy - sy1)).sqrt()
                } else {
                    let t = (((fx - sx1) * dx + (fy - sy1) * dy) / len_sq).clamp(0.0, 1.0);
                    let cx = sx1 + t * dx;
                    let cy = sy1 + t * dy;
                    ((fx - cx) * (fx - cx) + (fy - cy) * (fy - cy)).sqrt()
                };
                let cov = (hw + 0.5 - d).clamp(0.0, 1.0) as f32;
                if cov > 0.0 {
                    let idx = (py * sw as i64 + px) as usize;
                    if cov > alpha[idx] {
                        alpha[idx] = cov;
                    }
                }
            }
        }
    }

    // 填充：偶奇规则扫描线（超采样分辨率），2x2 子采样覆盖率混合到颜色缓冲
    let pwu = pw as usize;
    let phu = ph as usize;
    let mut fill_rgb = vec![[255u8, 255u8, 255u8]; pwu * phu];
    // 每输出像素命中的子采样数（≤4），逐填充复用后清零
    let mut cnt = vec![0u8; pwu * phu];
    for (poly, color) in &fills {
        let col = [color.0, color.1, color.2];
        let sp: Vec<(f64, f64)> = poly.iter().map(|(x, y)| to_ss(*x, *y)).collect();
        let n = sp.len();
        let (mut by1, mut by2) = (i64::MAX, i64::MIN);
        for &(_, y) in &sp {
            by1 = by1.min(y.floor() as i64);
            by2 = by2.max(y.ceil() as i64);
        }
        by1 = by1.max(0);
        by2 = by2.min(sh as i64 - 1);
        if by2 < by1 {
            continue;
        }
        // 标记子采样命中
        let mut ob = (usize::MAX, 0usize, usize::MAX, 0usize); // 输出级包围盒
        for py in by1..=by2 {
            let fy = py as f64 + 0.5;
            let mut xs: Vec<f64> = Vec::new();
            for i in 0..n {
                let (x1, y1) = sp[i];
                let (x2, y2) = sp[(i + 1) % n];
                if (y1 <= fy && y2 > fy) || (y2 <= fy && y1 > fy) {
                    let t = (fy - y1) / (y2 - y1);
                    xs.push(x1 + t * (x2 - x1));
                }
            }
            if xs.len() < 2 {
                continue;
            }
            xs.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
            let oy = (py / 2) as usize;
            let mut k = 0;
            while k + 1 < xs.len() {
                // 采样中心 px+0.5 ∈ [xa, xb]
                let (xa, xb) = (xs[k], xs[k + 1]);
                let px1 = ((xa - 0.5).ceil() as i64).max(0);
                let px2 = ((xb - 0.5).floor() as i64).min(sw as i64 - 1);
                for px in px1..=px2 {
                    let ox = (px / 2) as usize;
                    cnt[oy * pwu + ox] += 1;
                    ob.0 = ob.0.min(ox);
                    ob.1 = ob.1.max(ox);
                    ob.2 = ob.2.min(oy);
                    ob.3 = ob.3.max(oy);
                }
                k += 2;
            }
        }
        // 混合颜色并清零计数（仅包围盒范围）
        if ob.0 <= ob.1 && ob.2 <= ob.3 {
            for oy in ob.2..=ob.3 {
                for ox in ob.0..=ob.1 {
                    let idx = oy * pwu + ox;
                    let c = cnt[idx] as f32 / 4.0;
                    if c > 0.0 {
                        for ch in 0..3 {
                            let v =
                                fill_rgb[idx][ch] as f32 * (1.0 - c) + col[ch] as f32 * c;
                            fill_rgb[idx][ch] = v.round().min(255.0) as u8;
                        }
                    }
                    cnt[idx] = 0;
                }
            }
        }
    }

    // 2x2 下采样合成：线条黑色覆盖在填充色之上
    let mut pixels = vec![0u8; pwu * phu * 3];
    for y in 0..phu {
        for x in 0..pwu {
            let a00 = alpha[(y * 2) * sw as usize + x * 2];
            let a01 = alpha[(y * 2) * sw as usize + x * 2 + 1];
            let a10 = alpha[(y * 2 + 1) * sw as usize + x * 2];
            let a11 = alpha[(y * 2 + 1) * sw as usize + x * 2 + 1];
            let a = (a00 + a01 + a10 + a11) / 4.0;
            let idx = y * pwu + x;
            for ch in 0..3 {
                let v = fill_rgb[idx][ch] as f32 * (1.0 - a);
                pixels[idx * 3 + ch] = v.round().min(255.0) as u8;
            }
        }
    }

    let rgb = image::RgbImage::from_raw(pw, ph, pixels)
        .ok_or_else(|| "Raster buffer size error".to_string())?;
    let mut buf = Cursor::new(Vec::new());
    image::DynamicImage::ImageRgb8(rgb)
        .write_to(&mut buf, fmt.image_format())
        .map_err(|e| format!("Image encoding failed: {e}"))?;
    Ok(buf.into_inner())
}
