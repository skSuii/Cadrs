//! 最小 DXF (ASCII R12) 导入/导出。
//! 导出：LINE/CIRCLE/ARC/POLYLINE/POINT/ELLIPSE(分解)/SPLINE(分解)/SOLID(填充) + LAYER 表。
//! 导入：LINE/CIRCLE/ARC/LWPOLYLINE/POLYLINE/POINT/ELLIPSE/SOLID + 图层（code 8）。
//!
//! 本模块直接读写 DXF 组码对（code/value），只覆盖 R12 的常用实体，不依赖外部 CAD 库。
//! [`export`] 把 Document 渲染为整段文本，[`import`] 反过来解析为新的 Document；
//! 实体所属图层通过组码 8 的名称映射到文档图层，名称缺失时归入默认图层 "0"，
//! 角度在 DXF 中以度为单位、在 SDK 中以弧度为单位，转换在读写两侧分别完成。

use std::collections::HashMap;

use crate::data_structure::{
    make_arc, make_circle, make_ellipse, make_line, make_point, make_polyline, make_solid,
    Document, Entity, EntityGeometry, Layer, ObjectId,
};
use crate::geometry::{Ellipse, Point};
use crate::io::{Error as ImportError, Importer};
use crate::render::tessellation::entity_polylines;

fn num(v: f64) -> String {
    if v == 0.0 {
        "0.0".to_string()
    } else {
        format!("{v:.6}")
    }
}

fn pair(out: &mut String, code: i32, value: &str) {
    out.push_str(&code.to_string());
    out.push('\n');
    out.push_str(value);
    out.push('\n');
}

fn norm_deg(deg: f64) -> f64 {
    let d = deg % 360.0;
    if d < 0.0 {
        d + 360.0
    } else {
        d
    }
}

/// DXF ACI 颜色 → RGB 近似
/// - `aci`：AutoCAD 颜色索引；仅 1~6 有明确映射，其余取值（含 7 与越界值）一律返回白色。
/// 返回 `(红, 绿, 蓝)`，各分量取值 0~255；用于导入 SOLID 填充色。
pub fn aci_to_rgb(aci: i64) -> (u8, u8, u8) {
    match aci {
        1 => (255, 0, 0),
        2 => (255, 255, 0),
        3 => (0, 255, 0),
        4 => (0, 255, 255),
        5 => (0, 0, 255),
        6 => (255, 0, 255),
        _ => (255, 255, 255),
    }
}

/// DXF ACI 颜色索引近似映射
/// - `color`：`(红, 绿, 蓝)` 分量，取值 0~255。
/// 返回 1~7 的 ACI 索引，取曼哈顿距离最近的调色板项；明显偏白或远离调色板的颜色会落到 7（白）。
pub fn rgb_to_aci(color: (u8, u8, u8)) -> i64 {
    let palette: [(i64, (u8, u8, u8)); 7] = [
        (1, (255, 0, 0)),
        (2, (255, 255, 0)),
        (3, (0, 255, 0)),
        (4, (0, 255, 255)),
        (5, (0, 0, 255)),
        (6, (255, 0, 255)),
        (7, (255, 255, 255)),
    ];
    let mut best = 7;
    let mut best_d = i64::MAX;
    for (idx, (r, g, b)) in palette {
        let d = (r as i64 - color.0 as i64).abs()
            + (g as i64 - color.1 as i64).abs()
            + (b as i64 - color.2 as i64).abs();
        if d < best_d {
            best_d = d;
            best = idx;
        }
    }
    best
}

/// 实体所在图层名（导出用；找不到返回 "0"）
fn layer_name(doc: &Document, layer_id: &ObjectId) -> String {
    doc.get_layer(layer_id)
        .map(|l| l.name().to_string())
        .filter(|n| !n.is_empty())
        .unwrap_or_else(|| "0".to_string())
}

/// 写一个 SOLID（四点；DXF 渲染顺序 1-2-4-3，故 12/13 交换写入）
fn write_solid(out: &mut String, layer: &str, pts: &[Point; 4], color: (u8, u8, u8)) {
    pair(out, 0, "SOLID");
    pair(out, 8, layer);
    for (code_x, code_y, p) in [(10, 20, pts[0]), (11, 21, pts[1]), (12, 22, pts[3]), (13, 23, pts[2])] {
        pair(out, code_x, &num(p.x));
        pair(out, code_y, &num(p.y));
        pair(out, code_x + 20, "0.0");
    }
    pair(out, 62, &rgb_to_aci(color).to_string());
}

/// 多边形扇形三角化后写为多个 SOLID
fn write_solid_fan(out: &mut String, layer: &str, pts: &[Point], color: (u8, u8, u8)) {
    for i in 1..pts.len().saturating_sub(1) {
        let tri = [pts[0], pts[i], pts[i + 1], pts[i + 1]];
        write_solid(out, layer, &tri, color);
    }
}

/// 导出为 ASCII DXF (R12)
/// 输出完整文本（HEADER/TABLES/ENTITIES 段 + EOF），图层表按文档图层写出颜色与可见性：
/// 图层不可见时颜色组码取负值（DXF 约定）。
/// - `doc`：源文档，仅读取，不会被修改。
/// 返回可直接写盘的 DXF 文本；直线、圆、圆弧、多段线、点按原实体写出，
/// 标注、文字、椭圆、样条与 NURBS 会被分解为折线笔画，实心填充 Hatch 与 Solid 则输出为 SOLID。
/// # 示例
/// ```
/// use cadrs::data_structure::{make_circle, make_line, Document};
/// use cadrs::geometry::Point;
/// use cadrs::io::dxf::export;
///
/// let mut doc = Document::new("test".to_string());
/// doc.add_entity(make_line(Point::new2d(0.0, 0.0), Point::new2d(10.0, 0.0)));
/// doc.add_entity(make_circle(Point::new2d(5.0, 5.0), 2.0));
///
/// let text = export(&doc);
/// assert!(text.contains("LINE"));
/// assert!(text.contains("CIRCLE"));
/// ```
pub fn export(doc: &Document) -> String {
    let mut s = String::new();
    pair(&mut s, 0, "SECTION");
    pair(&mut s, 2, "HEADER");
    pair(&mut s, 9, "$ACADVER");
    pair(&mut s, 1, "AC1009");
    pair(&mut s, 0, "ENDSEC");

    // TABLES / LAYER 表（负颜色 = 图层关闭）
    pair(&mut s, 0, "SECTION");
    pair(&mut s, 2, "TABLES");
    pair(&mut s, 0, "TABLE");
    pair(&mut s, 2, "LAYER");
    pair(&mut s, 70, &doc.layer_count().to_string());
    for layer in doc.layers().values() {
        pair(&mut s, 0, "LAYER");
        pair(&mut s, 2, layer.name());
        pair(&mut s, 70, "0");
        let color = layer.color();
        let aci = rgb_to_aci((color.red, color.green, color.blue));
        let aci = if layer.is_visible() { aci } else { -aci };
        pair(&mut s, 62, &aci.to_string());
        pair(&mut s, 6, "CONTINUOUS");
    }
    pair(&mut s, 0, "ENDTAB");
    pair(&mut s, 0, "ENDSEC");

    pair(&mut s, 0, "SECTION");
    pair(&mut s, 2, "ENTITIES");
    for entity in doc.entities().values() {
        let lname = layer_name(doc, &entity.layer_id);
        let lname = lname.as_str();
        match entity.geometry() {
            EntityGeometry::Line(l) => {
                pair(&mut s, 0, "LINE");
                pair(&mut s, 8, lname);
                pair(&mut s, 10, &num(l.start.x));
                pair(&mut s, 20, &num(l.start.y));
                pair(&mut s, 30, "0.0");
                pair(&mut s, 11, &num(l.end.x));
                pair(&mut s, 21, &num(l.end.y));
                pair(&mut s, 31, "0.0");
            }
            EntityGeometry::Circle(c) => {
                pair(&mut s, 0, "CIRCLE");
                pair(&mut s, 8, lname);
                pair(&mut s, 10, &num(c.center.x));
                pair(&mut s, 20, &num(c.center.y));
                pair(&mut s, 30, "0.0");
                pair(&mut s, 40, &num(c.radius));
            }
            EntityGeometry::Arc(a) => {
                // DXF 圆弧总是逆时针：顺时针弧交换起止角
                let (start_deg, end_deg) = if a.is_counter_clockwise {
                    (a.start_angle.to_degrees(), a.end_angle.to_degrees())
                } else {
                    (a.end_angle.to_degrees(), a.start_angle.to_degrees())
                };
                pair(&mut s, 0, "ARC");
                pair(&mut s, 8, lname);
                pair(&mut s, 10, &num(a.center.x));
                pair(&mut s, 20, &num(a.center.y));
                pair(&mut s, 30, "0.0");
                pair(&mut s, 40, &num(a.radius));
                pair(&mut s, 50, &num(norm_deg(start_deg)));
                pair(&mut s, 51, &num(norm_deg(end_deg)));
            }
            EntityGeometry::Point(p) => {
                pair(&mut s, 0, "POINT");
                pair(&mut s, 8, lname);
                pair(&mut s, 10, &num(p.x));
                pair(&mut s, 20, &num(p.y));
                pair(&mut s, 30, "0.0");
            }
            EntityGeometry::Polyline(p) => {
                if p.vertices.len() < 2 {
                    continue;
                }
                pair(&mut s, 0, "POLYLINE");
                pair(&mut s, 8, lname);
                pair(&mut s, 66, "1");
                pair(&mut s, 70, if p.is_closed { "1" } else { "0" });
                for v in &p.vertices {
                    pair(&mut s, 0, "VERTEX");
                    pair(&mut s, 8, lname);
                    pair(&mut s, 10, &num(v.x));
                    pair(&mut s, 20, &num(v.y));
                    pair(&mut s, 30, "0.0");
                }
                pair(&mut s, 0, "SEQEND");
                pair(&mut s, 8, lname);
            }
            EntityGeometry::Dimension { .. }
            | EntityGeometry::Text { .. }
            | EntityGeometry::Ellipse(_)
            | EntityGeometry::BSpline(_)
            | EntityGeometry::NURBS(_) => {
                // 标注 / 文字 / 椭圆 / 样条：分解为折线笔画，以 POLYLINE 写出（R12 兼容）
                for (pts, closed) in entity_polylines(entity) {
                    if pts.len() < 2 {
                        continue;
                    }
                    pair(&mut s, 0, "POLYLINE");
                    pair(&mut s, 8, lname);
                    pair(&mut s, 66, "1");
                    pair(&mut s, 70, if closed { "1" } else { "0" });
                    for p in &pts {
                        pair(&mut s, 0, "VERTEX");
                        pair(&mut s, 8, lname);
                        pair(&mut s, 10, &num(p.x));
                        pair(&mut s, 20, &num(p.y));
                        pair(&mut s, 30, "0.0");
                    }
                    pair(&mut s, 0, "SEQEND");
                    pair(&mut s, 8, lname);
                }
            }
            EntityGeometry::Solid { points, color } => {
                write_solid(&mut s, lname, points, *color);
            }
            EntityGeometry::Hatch {
                solid_fill,
                fill_color,
                boundary_paths,
                ..
            } if *solid_fill => {
                // 实心填充：边界扇形三角化为 SOLID
                for bp in boundary_paths {
                    if !bp.is_polyline {
                        continue;
                    }
                    let pts: Vec<Point> = bp.edges.iter().map(|e| e.start_point).collect();
                    if pts.len() >= 3 {
                        write_solid_fan(&mut s, lname, &pts, *fill_color);
                    }
                }
            }
            _ => {}
        }
    }
    pair(&mut s, 0, "ENDSEC");
    pair(&mut s, 0, "EOF");
    s
}

struct Pair {
    code: i32,
    value: String,
}

fn parse_pairs(src: &str) -> Vec<Pair> {
    let mut pairs = Vec::new();
    let mut lines = src.lines();
    while let (Some(code), Some(value)) = (lines.next(), lines.next()) {
        let code: i32 = code.trim().parse().unwrap_or(-999);
        pairs.push(Pair {
            code,
            value: value.trim().to_string(),
        });
    }
    pairs
}

/// 读取从 start 开始、到下一个 code==0 之前的字段（后出现的值覆盖前面的）
fn read_fields(pairs: &[Pair], start: usize) -> (HashMap<i32, f64>, HashMap<i32, String>, usize) {
    let mut map = HashMap::new();
    let mut strs = HashMap::new();
    let mut i = start;
    while i < pairs.len() && pairs[i].code != 0 {
        if let Ok(v) = pairs[i].value.parse::<f64>() {
            map.insert(pairs[i].code, v);
        } else {
            strs.insert(pairs[i].code, pairs[i].value.clone());
        }
        i += 1;
    }
    (map, strs, i)
}

fn f(map: &HashMap<i32, f64>, code: i32) -> f64 {
    map.get(&code).copied().unwrap_or(0.0)
}

fn layer_field(strs: &HashMap<i32, String>) -> String {
    strs.get(&8).cloned().unwrap_or_else(|| "0".to_string())
}

/// 按图层名查找/创建图层，返回图层 id
fn resolve_layer(
    doc: &mut Document,
    map: &mut HashMap<String, ObjectId>,
    name: &str,
) -> ObjectId {
    if let Some(id) = map.get(name) {
        return id.clone();
    }
    let layer = Layer::new(name.to_string());
    let id = doc.add_layer(layer);
    map.insert(name.to_string(), id.clone());
    id
}

fn add_to_layer(
    doc: &mut Document,
    map: &mut HashMap<String, ObjectId>,
    name: &str,
    mut entity: Entity,
) {
    let id = resolve_layer(doc, map, name);
    entity.layer_id = id;
    doc.add_entity(entity);
}

fn add_polyline(
    doc: &mut Document,
    map: &mut HashMap<String, ObjectId>,
    name: &str,
    verts: Vec<(f64, f64)>,
    closed: bool,
) {
    if verts.len() < 2 {
        return;
    }
    let pts: Vec<Point> = verts
        .iter()
        .map(|(x, y)| Point::new2d(*x, *y))
        .collect();
    add_to_layer(doc, map, name, make_polyline(&pts, closed));
}

/// 导入 ASCII DXF，返回新文档
/// 只解析 ENTITIES 段内的 LINE/CIRCLE/ARC/LWPOLYLINE/POLYLINE/POINT/ELLIPSE/SOLID，
/// 图层名（组码 8）不存在时复用已有图层或新建图层；DXF 默认图层 "0" 直接映射到 ModelSpace。
/// - `src`：完整的 ASCII DXF 文本，非 UTF-8 内容请在调用前自行解码。
/// 返回新建的 Document；文本为空、不是 ASCII DXF，或未识别出任何实体时返回错误描述。
/// 被跳过的内容：DXF 圆弧总按逆时针解释，部分椭圆以 64 段折线近似，重复的 SOLID 尾点会被去除。
/// # 示例
/// ```
/// use cadrs::io::dxf::import;
///
/// let text = "0\nSECTION\n2\nENTITIES\n0\nLINE\n8\n0\n10\n0.0\n20\n0.0\n11\n1.0\n21\n1.0\n0\nENDSEC\n0\nEOF\n";
/// let doc = import(text).unwrap();
/// assert_eq!(doc.entity_count(), 1);
/// ```
pub fn import(src: &str) -> Result<Document, String> {
    let pairs = parse_pairs(src);
    if pairs.is_empty() {
        return Err("Empty file or not an ASCII DXF".to_string());
    }
    let mut doc = Document::new("imported".to_string());
    // DXF 默认图层 "0" 映射到 ModelSpace
    let mut layer_ids: HashMap<String, ObjectId> = HashMap::new();
    layer_ids.insert("0".to_string(), doc.model_space().clone());
    let n = pairs.len();
    let mut in_entities = false;
    let mut i = 0;

    while i < n {
        let p = &pairs[i];
        if p.code != 0 {
            i += 1;
            continue;
        }
        match p.value.as_str() {
            "SECTION" => {
                if i + 1 < n && pairs[i + 1].code == 2 {
                    in_entities = pairs[i + 1].value == "ENTITIES";
                    i += 2;
                    continue;
                }
            }
            "ENDSEC" => in_entities = false,
            "LINE" if in_entities => {
                let (m, ms, j) = read_fields(&pairs, i + 1);
                let a = Point::new2d(f(&m, 10), f(&m, 20));
                let b = Point::new2d(f(&m, 11), f(&m, 21));
                if a.distance_to(&b) > 1e-12 {
                    add_to_layer(&mut doc, &mut layer_ids, &layer_field(&ms), make_line(a, b));
                }
                i = j;
                continue;
            }
            "CIRCLE" if in_entities => {
                let (m, ms, j) = read_fields(&pairs, i + 1);
                let c = Point::new2d(f(&m, 10), f(&m, 20));
                let r = f(&m, 40);
                if r > 1e-12 {
                    add_to_layer(&mut doc, &mut layer_ids, &layer_field(&ms), make_circle(c, r));
                }
                i = j;
                continue;
            }
            "ARC" if in_entities => {
                let (m, ms, j) = read_fields(&pairs, i + 1);
                let c = Point::new2d(f(&m, 10), f(&m, 20));
                let r = f(&m, 40);
                let s = f(&m, 50).to_radians();
                let e = f(&m, 51).to_radians();
                if r > 1e-12 {
                    add_to_layer(&mut doc, &mut layer_ids, &layer_field(&ms), make_arc(c, r, s, e));
                }
                i = j;
                continue;
            }
            "POINT" if in_entities => {
                let (m, ms, j) = read_fields(&pairs, i + 1);
                let p = Point::new2d(f(&m, 10), f(&m, 20));
                add_to_layer(&mut doc, &mut layer_ids, &layer_field(&ms), make_point(p));
                i = j;
                continue;
            }
            "ELLIPSE" if in_entities => {
                let (m, ms, j) = read_fields(&pairs, i + 1);
                let c = Point::new2d(f(&m, 10), f(&m, 20));
                let (mx, my) = (f(&m, 11), f(&m, 21));
                let ratio = f(&m, 40);
                let t0 = f(&m, 41);
                let t1 = f(&m, 42);
                let a = (mx * mx + my * my).sqrt();
                let rot = my.atan2(mx);
                let b = a * ratio;
                let full = (t0.abs() < 1e-9 && (t1 - std::f64::consts::TAU).abs() < 1e-3)
                    || (t0.abs() < 1e-9 && t1.abs() < 1e-9);
                if a > 1e-12 && b > 1e-12 && ratio > 0.0 && ratio <= 1.0 + 1e-9 {
                    let entity = if full {
                        make_ellipse(c, a, b, rot)
                    } else {
                        // 部分椭圆 → 折线近似（参数为弧度，SDK 参数 0..1 对应整椭圆）
                        let e = Ellipse::new(c, a, b, rot);
                        let steps = 64;
                        let pts: Vec<Point> = (0..=steps)
                            .map(|k| {
                                let t = t0 + (t1 - t0) * k as f64 / steps as f64;
                                e.point_at_parameter(t / std::f64::consts::TAU)
                            })
                            .collect();
                        make_polyline(&pts, false)
                    };
                    add_to_layer(&mut doc, &mut layer_ids, &layer_field(&ms), entity);
                }
                i = j;
                continue;
            }
            "SOLID" if in_entities => {
                let (m, ms, j) = read_fields(&pairs, i + 1);
                // DXF 顶点 1,2,3,4 渲染顺序为 1-2-4-3 → 还原为多边形顺序 1,2,4,3
                let a = Point::new2d(f(&m, 10), f(&m, 20));
                let b = Point::new2d(f(&m, 11), f(&m, 21));
                let c = Point::new2d(f(&m, 13), f(&m, 23));
                let d = Point::new2d(f(&m, 12), f(&m, 22));
                let mut pts = vec![a, b, c, d];
                // 去掉重复尾点（三角形）
                while pts.len() > 3 && pts[pts.len() - 1].distance_to(&pts[pts.len() - 2]) < 1e-12 {
                    pts.pop();
                }
                let color = aci_to_rgb(f(&m, 62) as i64);
                if let Some(entity) = make_solid(&pts, color) {
                    add_to_layer(&mut doc, &mut layer_ids, &layer_field(&ms), entity);
                }
                i = j;
                continue;
            }
            "LWPOLYLINE" if in_entities => {
                let mut verts: Vec<(f64, f64)> = Vec::new();
                let mut closed = false;
                let mut cur_x: Option<f64> = None;
                let mut lname = "0".to_string();
                let mut j = i + 1;
                while j < n && pairs[j].code != 0 {
                    let pj = &pairs[j];
                    match pj.code {
                        8 => lname = pj.value.clone(),
                        70 => {
                            closed = pj.value.parse::<f64>().map(|v| v as i64 & 1 != 0).unwrap_or(false);
                        }
                        10 => cur_x = pj.value.parse::<f64>().ok(),
                        20 => {
                            if let Some(x) = cur_x {
                                if let Ok(y) = pj.value.parse::<f64>() {
                                    verts.push((x, y));
                                }
                            }
                            cur_x = None;
                        }
                        _ => {}
                    }
                    j += 1;
                }
                add_polyline(&mut doc, &mut layer_ids, &lname, verts, closed);
                i = j;
                continue;
            }
            "POLYLINE" if in_entities => {
                let (m, ms, j) = read_fields(&pairs, i + 1);
                let closed = (f(&m, 70) as i64) & 1 != 0;
                let lname = layer_field(&ms);
                let mut verts: Vec<(f64, f64)> = Vec::new();
                let mut k = j;
                while k < n && pairs[k].code == 0 && pairs[k].value == "VERTEX" {
                    let (vf, _vs, nk) = read_fields(&pairs, k + 1);
                    verts.push((f(&vf, 10), f(&vf, 20)));
                    k = nk;
                }
                add_polyline(&mut doc, &mut layer_ids, &lname, verts, closed);
                i = k;
                continue;
            }
            _ => {}
        }
        i += 1;
    }

    if doc.entity_count() == 0 {
        return Err("No recognizable entities found (LINE/CIRCLE/ARC/POLYLINE)".to_string());
    }
    Ok(doc)
}

/// ASCII DXF (R12) 导入器
/// 无状态的零尺寸类型，所有导入逻辑都在 [`import`] 中；文件读取与解码由 trait 方法完成。
#[derive(Debug, Clone, Copy)]
pub struct DXFImporter;

impl DXFImporter {
    /// 创建 DXF 导入器。
    /// 无字段可初始化，返回的实例可跨线程共享并重复使用。
    pub fn new() -> Self {
        Self
    }
}

impl Default for DXFImporter {
    fn default() -> Self {
        Self::new()
    }
}

impl Importer for DXFImporter {
    fn can_import(&self, extension: &str) -> bool {
        extension.to_lowercase() == "dxf"
    }

    fn import_from_file(&self, filename: &str) -> Result<Document, ImportError> {
        let content = std::fs::read_to_string(filename)
            .map_err(|e| ImportError::Io(e.to_string()))?;
        self.import_from_bytes(content.as_bytes(), "dxf")
    }

    fn import_from_bytes(&self, data: &[u8], extension: &str) -> Result<Document, ImportError> {
        if !self.can_import(extension) {
            return Err(ImportError::UnsupportedFormat(extension.to_string()));
        }
        let content = String::from_utf8_lossy(data);
        import(&content).map_err(ImportError::ParseError)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::io::Importer;

    #[test]
    fn test_dxf_importer_can_import() {
        let importer = DXFImporter::new();
        assert!(importer.can_import("dxf"));
        assert!(importer.can_import("DXF"));
        assert!(!importer.can_import("svg"));
    }

    #[test]
    fn test_dxf_export_import_roundtrip() {
        let mut doc = Document::new("test".to_string());
        doc.add_entity(make_line(Point::new2d(0.0, 0.0), Point::new2d(10.0, 0.0)));
        doc.add_entity(make_circle(Point::new2d(5.0, 5.0), 2.0));

        let text = export(&doc);
        assert!(text.contains("LINE"));
        assert!(text.contains("CIRCLE"));
        assert!(text.contains("AC1009"));

        let imported = import(&text).unwrap();
        assert_eq!(imported.entity_count(), 2);
    }

    #[test]
    fn test_dxf_import_rejects_empty() {
        assert!(import("").is_err());
    }
}
