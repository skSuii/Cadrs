//! EPS (Encapsulated PostScript) 导出。PostScript 坐标系 y 向上，与世界坐标一致。

use crate::data_structure::Document;
use crate::render::tessellation::{document_bbox, entity_fills, entity_polylines};

pub fn export(doc: &Document) -> Result<String, String> {
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
    let minx = min.x - m;
    let miny = min.y - m;
    let maxx = max.x + m;
    let maxy = max.y + m;

    let mut ps = String::new();
    ps += "%!PS-Adobe-3.0 EPSF-3.0\n";
    ps += &format!(
        "%%BoundingBox: {} {} {} {}\n",
        minx.floor() as i64,
        miny.floor() as i64,
        maxx.ceil() as i64,
        maxy.ceil() as i64
    );
    ps += "%%Pages: 0\n";
    ps += "%%EndComments\n";
    ps += "1 setlinewidth\n";
    // 填充（先填色，再画线）
    for (pts, color) in &fills {
        ps += &format!(
            "{:.3} {:.3} {:.3} setrgbcolor\n",
            color.0 as f64 / 255.0,
            color.1 as f64 / 255.0,
            color.2 as f64 / 255.0
        );
        ps += &format!("newpath\n{:.3} {:.3} moveto\n", pts[0].0, pts[0].1);
        for p in pts.iter().skip(1) {
            ps += &format!("{:.3} {:.3} lineto\n", p.0, p.1);
        }
        ps += "closepath\nfill\n";
    }
    ps += "0 0 0 setrgbcolor\n";
    for (pts, closed) in &polys {
        let n = pts.len();
        ps += &format!("{:.3} {:.3} moveto\n", pts[0].0, pts[0].1);
        for p in pts.iter().take(n).skip(1) {
            ps += &format!("{:.3} {:.3} lineto\n", p.0, p.1);
        }
        if *closed {
            ps += &format!("{:.3} {:.3} lineto\n", pts[0].0, pts[0].1);
        }
        ps += "stroke\n";
    }
    ps += "%%EOF\n";
    Ok(ps)
}
