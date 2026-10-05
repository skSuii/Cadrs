//! WMF (Windows Metafile, Placeable) 导出。
//! 16 位坐标空间，POLYLINE 记录绘制所有实体，黑色画笔。

use crate::data_structure::Document;
use crate::render::tessellation::{document_bbox, entity_fills, entity_polylines};

const SPAN: i32 = 30000; // 窗口坐标范围（i16 内）
const MARGIN: i32 = 500;

fn put_u16(buf: &mut Vec<u8>, v: u16) {
    buf.extend_from_slice(&v.to_le_bytes());
}

fn put_i16(buf: &mut Vec<u8>, v: i16) {
    buf.extend_from_slice(&v.to_le_bytes());
}

fn put_u32(buf: &mut Vec<u8>, v: u32) {
    buf.extend_from_slice(&v.to_le_bytes());
}

/// 记录头：Size(4字节, 字数) + Function(2字节)
fn rec_header(buf: &mut Vec<u8>, words: u32, func: u16) {
    put_u32(buf, words);
    put_u16(buf, func);
}

pub fn export(doc: &Document) -> Result<Vec<u8>, String> {
    if doc.entity_count() == 0 {
        return Err("Canvas is empty, nothing to export".to_string());
    }
    let Some((min, max)) = document_bbox(doc) else {
        return Err("Canvas is empty, nothing to export".to_string());
    };

    // 收集折线与填充（闭合折线首尾补点，折线用 POLYLINE；填充用 POLYGON+画刷）
    let mut polys: Vec<Vec<(i16, i16)>> = Vec::new();
    let mut fills: Vec<(Vec<(i16, i16)>, (u8, u8, u8))> = Vec::new();
    let w = (max.x - min.x).max(1e-9);
    let h = (max.y - min.y).max(1e-9);
    let inner = (SPAN - 2 * MARGIN) as f64;
    let map = |x: f64, y: f64| -> (i16, i16) {
        let px = MARGIN as f64 + (x - min.x) / w * inner;
        // WMF y 向下，翻转
        let py = MARGIN as f64 + (max.y - y) / h * inner;
        (px.round() as i16, py.round() as i16)
    };
    for entity in doc.entities().values() {
        for (pts, color) in entity_fills(entity) {
            if pts.len() < 3 {
                continue;
            }
            let scr: Vec<(i16, i16)> = pts.iter().map(|p| map(p.x, p.y)).collect();
            fills.push((scr, color));
        }
        for (pts, closed) in entity_polylines(entity) {
            if pts.len() < 2 {
                continue;
            }
            let mut scr: Vec<(i16, i16)> = pts.iter().map(|p| map(p.x, p.y)).collect();
            if closed {
                scr.push(scr[0]);
            }
            polys.push(scr);
        }
    }
    if polys.is_empty() && fills.is_empty() {
        return Err("Nothing to draw".to_string());
    }

    let mut recs: Vec<u8> = Vec::new();
    let mut max_record: u32 = 3;

    // META_SETWINDOWORG: size=5, 参数 Y 后 X
    rec_header(&mut recs, 5, 0x020B);
    put_i16(&mut recs, 0);
    put_i16(&mut recs, 0);

    // META_SETWINDOWEXT: size=5, 参数 Height 后 Width
    rec_header(&mut recs, 5, 0x020C);
    put_i16(&mut recs, SPAN as i16);
    put_i16(&mut recs, SPAN as i16);

    // META_CREATEPEN: size=8, 样式/宽X/宽Y/BGR 颜色
    rec_header(&mut recs, 8, 0x02FA);
    put_i16(&mut recs, 0); // PS_SOLID
    put_i16(&mut recs, 30); // 宽（逻辑单位）
    put_i16(&mut recs, 0);
    put_u32(&mut recs, 0x00000000); // 黑色
    max_record = max_record.max(8);

    // META_CREATEBRUSHINDIRECT: size=7, 样式/COLORREF/阴影（每个填充一把画刷）
    for (_, color) in &fills {
        rec_header(&mut recs, 7, 0x02FC);
        put_i16(&mut recs, 0); // BS_SOLID
        put_u32(
            &mut recs,
            ((color.2 as u32) << 16) | ((color.1 as u32) << 8) | color.0 as u32, // 0x00BBGGRR
        );
        put_i16(&mut recs, 0); // HS_HORIZONTAL（实心时忽略）
        max_record = max_record.max(7);
    }

    // META_SELECTOBJECT: size=4, 对象索引 0（画笔）
    rec_header(&mut recs, 4, 0x012D);
    put_u16(&mut recs, 0);

    // 填充：META_POLYGON（size = 4 + 2n，含 Size 自身 2 字）
    for (i, (pts, _)) in fills.iter().enumerate() {
        // 选中该填充的画刷
        rec_header(&mut recs, 4, 0x012D);
        put_u16(&mut recs, 1 + i as u16);
        let n = pts.len() as u32;
        let words = 4 + 2 * n;
        rec_header(&mut recs, words, 0x0324);
        put_u16(&mut recs, n as u16);
        for &(x, y) in pts {
            put_i16(&mut recs, x);
            put_i16(&mut recs, y);
        }
        max_record = max_record.max(words);
    }

    // 回到画笔绘制线段
    rec_header(&mut recs, 4, 0x012D);
    put_u16(&mut recs, 0);

    // META_POLYLINE: size = 3 + 1 + 2n
    for pts in &polys {
        let n = pts.len() as u32;
        let words = 4 + 2 * n;
        rec_header(&mut recs, words, 0x0325);
        put_u16(&mut recs, n as u16);
        for &(x, y) in pts {
            put_i16(&mut recs, x);
            put_i16(&mut recs, y);
        }
        max_record = max_record.max(words);
    }

    // META_DELETEOBJECT
    rec_header(&mut recs, 4, 0x01F0);
    put_u16(&mut recs, 0);

    // META_EOF
    rec_header(&mut recs, 3, 0x0000);
    put_u16(&mut recs, 0);

    // WMF 头（18 字节）
    let total_words = ((18 + recs.len()) / 2) as u32;
    let mut meta: Vec<u8> = Vec::new();
    put_u16(&mut meta, 1); // 类型: 内存图元文件
    put_u16(&mut meta, 9); // 头大小（字）
    put_u16(&mut meta, 0x0300); // 版本
    put_u32(&mut meta, total_words);
    put_u16(&mut meta, 1 + fills.len() as u16); // 对象数（画笔 + 画刷）
    put_u32(&mut meta, max_record);
    put_u16(&mut meta, 0);
    meta.extend_from_slice(&recs);

    // Placeable 头（22 字节）
    let mut out: Vec<u8> = Vec::new();
    put_u32(&mut out, 0x9AC6_CDD7); // 魔数
    put_i16(&mut out, 0); // hmf
    put_i16(&mut out, 0); // bbox left
    put_i16(&mut out, 0); // bbox top
    put_i16(&mut out, SPAN as i16); // bbox right
    put_i16(&mut out, SPAN as i16); // bbox bottom
    put_i16(&mut out, 1440); // 每英寸逻辑单位数（twip）
    put_i16(&mut out, 0); // 保留
    // 校验和：前 10 个字的异或
    let mut cs: u16 = 0;
    for i in 0..10 {
        cs ^= u16::from_le_bytes([out[i * 2], out[i * 2 + 1]]);
    }
    put_u16(&mut out, cs);

    out.extend_from_slice(&meta);
    Ok(out)
}
