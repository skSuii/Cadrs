//! SVG 导入/导出。
//! 导出: LINE/CIRCLE/ARC/POLYLINE；导入: line/rect/circle/ellipse/polyline/polygon/path(M/L/H/V/A/Z)。
//!
//! 两条链路共用 `render::tessellation` 的细分结果：导出把实体拆成折线与填充后写成 SVG 元素，
//! 导入把 SVG 图元转回 实体 并放入一个新的 文档。SVG 的 y 轴向下、长度单位为无单位用户单位，
//! 而本 SDK 的世界坐标 y 轴向上，因此导入与导出两侧都会做 y 取反，只有 `circle`/`ellipse`
//! 的半径等标量不受翻转影响。
//!
//! `SVGImporter` 把 `import` 接入 `io::Importer` 注册体系，供按扩展名分发的统一入口调用。

use std::collections::HashMap;

use crate::data_structure::{make_circle, make_line, make_polyline, Document, EntityGeometry};
use crate::geometry::Point;
use crate::io::{Error as ImportError, Importer};
use crate::render::tessellation::{document_bbox, entity_fills, entity_polylines};

// ---------------- 导出 ----------------

/// 把 文档 导出为 SVG 文本，世界坐标按 y 轴翻转后映射到页面坐标。
///
/// 画布尺寸取 文档 中所有实体的外接矩形（`document_bbox`）四周各加 10 个世界单位的边距；
/// 先绘制填充与实心 Hatch，再绘制黑色 1px 描边，标注与文字分解为折线笔画输出。
/// - `doc`：只读，导出过程不修改 文档。
///
/// 返回完整的 `<svg>` 文本；当 文档 没有任何实体或无法求出外接矩形时返回错误
/// （两种情况的错误信息都是 `Canvas is empty, nothing to export`）。
///
/// # 示例
/// ```ignore
/// let svg = export(&doc).unwrap();
/// assert!(svg.contains("<line"));
/// ```
pub fn export(doc: &Document) -> Result<String, String> {
    if doc.entity_count() == 0 {
        return Err("Canvas is empty, nothing to export".to_string());
    }
    let Some((min, max)) = document_bbox(doc) else {
        return Err("Canvas is empty, nothing to export".to_string());
    };
    let m = 10.0;
    let w = (max.x - min.x) + 2.0 * m;
    let h = (max.y - min.y) + 2.0 * m;
    let sx = |x: f64| x - min.x + m;
    let sy = |y: f64| max.y - y + m; // SVG y 向下，翻转

    let mut svg = format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{w:.3}\" height=\"{h:.3}\" viewBox=\"0 0 {w:.3} {h:.3}\">\n"
    );
    svg += &format!(
        "  <rect x=\"0\" y=\"0\" width=\"{w:.3}\" height=\"{h:.3}\" fill=\"white\"/>\n"
    );
    // 填充（Solid / 实心 Hatch）
    for entity in doc.entities().values() {
        for (pts, color) in entity_fills(entity) {
            if pts.len() < 3 {
                continue;
            }
            let pts_str: Vec<String> = pts
                .iter()
                .map(|p| format!("{:.3},{:.3}", sx(p.x), sy(p.y)))
                .collect();
            svg += &format!(
                "  <polygon points=\"{}\" fill=\"#{:02X}{:02X}{:02X}\" stroke=\"none\"/>\n",
                pts_str.join(" "),
                color.0,
                color.1,
                color.2
            );
        }
    }
    for entity in doc.entities().values() {
        match entity.geometry() {
            EntityGeometry::Line(l) => {
                svg += &format!(
                    "  <line x1=\"{:.3}\" y1=\"{:.3}\" x2=\"{:.3}\" y2=\"{:.3}\" stroke=\"black\" stroke-width=\"1\"/>\n",
                    sx(l.start.x), sy(l.start.y), sx(l.end.x), sy(l.end.y)
                );
            }
            EntityGeometry::Circle(c) => {
                svg += &format!(
                    "  <circle cx=\"{:.3}\" cy=\"{:.3}\" r=\"{:.3}\" fill=\"none\" stroke=\"black\" stroke-width=\"1\"/>\n",
                    sx(c.center.x), sy(c.center.y), c.radius
                );
            }
            EntityGeometry::Arc(a) => {
                let span = a.angle_span();
                let sweep = if a.is_counter_clockwise { 0 } else { 1 }; // y 翻转后方向取反
                let large = if span > std::f64::consts::PI { 1 } else { 0 };
                let sp = a.start_point();
                let ep = a.end_point();
                svg += &format!(
                    "  <path d=\"M {:.3} {:.3} A {:.3} {:.3} 0 {large} {sweep} {:.3} {:.3}\" fill=\"none\" stroke=\"black\" stroke-width=\"1\"/>\n",
                    sx(sp.x), sy(sp.y), a.radius, a.radius, sx(ep.x), sy(ep.y)
                );
            }
            EntityGeometry::Polyline(p) => {
                if p.vertices.len() < 2 {
                    continue;
                }
                let pts: Vec<String> = p
                    .vertices
                    .iter()
                    .map(|v| format!("{:.3},{:.3}", sx(v.x), sy(v.y)))
                    .collect();
                let tag = if p.is_closed { "polygon" } else { "polyline" };
                svg += &format!(
                    "  <{tag} points=\"{}\" fill=\"none\" stroke=\"black\" stroke-width=\"1\"/>\n",
                    pts.join(" ")
                );
            }
            EntityGeometry::Dimension { .. } | EntityGeometry::Text { .. } => {
                // 标注 / 文字：分解为折线笔画
                for (pts, closed) in entity_polylines(entity) {
                    if pts.len() < 2 {
                        continue;
                    }
                    let pts_str: Vec<String> = pts
                        .iter()
                        .map(|p| format!("{:.3},{:.3}", sx(p.x), sy(p.y)))
                        .collect();
                    let tag = if closed { "polygon" } else { "polyline" };
                    svg += &format!(
                        "  <{tag} points=\"{}\" fill=\"none\" stroke=\"black\" stroke-width=\"1\"/>\n",
                        pts_str.join(" ")
                    );
                }
            }
            _ => {}
        }
    }
    svg += "</svg>\n";
    Ok(svg)
}

// ---------------- 导入 ----------------

type Attrs = HashMap<String, String>;

fn attr_f(a: &Attrs, key: &str) -> Option<f64> {
    a.get(key).and_then(|v| v.trim().parse::<f64>().ok())
}

/// 解析标签属性字符串: `x1="0" y1='5' width=10`
fn parse_attrs(input: &str) -> Attrs {
    let mut map = HashMap::new();
    let chars: Vec<char> = input.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        while i < chars.len() && chars[i].is_whitespace() {
            i += 1;
        }
        let key_start = i;
        while i < chars.len() && chars[i] != '=' && !chars[i].is_whitespace() {
            i += 1;
        }
        if i >= chars.len() || chars[i] != '=' {
            break;
        }
        let key: String = chars[key_start..i].iter().collect();
        i += 1; // '='
        while i < chars.len() && chars[i].is_whitespace() {
            i += 1;
        }
        if i < chars.len() && (chars[i] == '"' || chars[i] == '\'') {
            let quote = chars[i];
            i += 1;
            let vstart = i;
            while i < chars.len() && chars[i] != quote {
                i += 1;
            }
            map.insert(key, chars[vstart..i].iter().collect());
            i += 1;
        } else {
            let vstart = i;
            while i < chars.len() && !chars[i].is_whitespace() {
                i += 1;
            }
            if vstart < i {
                map.insert(key, chars[vstart..i].iter().collect());
            }
        }
    }
    map
}

/// 解析数字列表（处理 "1.5-3e2,4" 这类紧凑写法）
fn parse_floats(s: &str) -> Vec<f64> {
    let mut nums = Vec::new();
    let mut cur = String::new();
    let mut have_digit = false;
    for c in s.chars() {
        match c {
            '0'..='9' => {
                cur.push(c);
                have_digit = true;
            }
            '.' => {
                if cur.contains('.') {
                    if have_digit {
                        nums.push(cur.parse().unwrap_or(0.0));
                    }
                    cur.clear();
                    have_digit = false;
                }
                cur.push(c);
            }
            'e' | 'E' => {
                if have_digit {
                    cur.push(c);
                }
            }
            '-' | '+' => {
                if have_digit && !cur.ends_with('e') && !cur.ends_with('E') {
                    nums.push(cur.parse().unwrap_or(0.0));
                    cur.clear();
                    have_digit = false;
                }
                cur.push(c);
            }
            _ => {
                if have_digit {
                    nums.push(cur.parse().unwrap_or(0.0));
                    cur.clear();
                    have_digit = false;
                }
            }
        }
    }
    if have_digit {
        nums.push(cur.parse().unwrap_or(0.0));
    }
    nums
}

struct PathBuilder {
    doc: Document,
    cur: (f64, f64),
    subpath: Vec<Point>, // 世界坐标 (y 已翻转)
    started: bool,
}

impl PathBuilder {
    fn world(x: f64, y: f64) -> Point {
        // SVG y 向下 → 世界 y 向上
        Point::new2d(x, -y)
    }

    fn flush(&mut self) {
        if self.subpath.len() >= 2 {
            let pts = std::mem::take(&mut self.subpath);
            self.doc.add_entity(make_polyline(&pts, false));
        } else {
            self.subpath.clear();
        }
    }

    fn line_to(&mut self, x: f64, y: f64) {
        if !self.started {
            self.cur = (x, y);
            self.started = true;
            self.subpath.push(Self::world(x, y));
            return;
        }
        if self.subpath.is_empty() {
            self.subpath.push(Self::world(self.cur.0, self.cur.1));
        }
        self.cur = (x, y);
        self.subpath.push(Self::world(x, y));
    }

    fn arc_to(&mut self, rx: f64, large: bool, sweep: bool, x: f64, y: f64) {
        if !self.started {
            self.started = true;
            self.cur = (x, y);
            return;
        }
        let (x1, y1) = self.cur;
        let (x2, y2) = (x, y);
        let r = rx.max(1e-9);
        // 求圆心：位于弦的中垂线上，距中点 h
        let mx = (x1 + x2) / 2.0;
        let my = (y1 + y2) / 2.0;
        let dx = x2 - x1;
        let dy = y2 - y1;
        let chord = (dx * dx + dy * dy).sqrt();
        if chord < 1e-12 {
            return;
        }
        let r_eff = r.max(chord / 2.0);
        let h = (r_eff * r_eff - chord * chord / 4.0).max(0.0).sqrt();
        // 两个候选圆心，取与 large/sweep 标志匹配的：扫描角 > π 对应 large
        let ux = -dy / chord;
        let uy = dx / chord;
        let mut chosen: Option<((f64, f64), f64, f64)> = None; // (center, a1, span_ccw)
        for sign in [1.0, -1.0] {
            let cx = mx + sign * h * ux;
            let cy = my + sign * h * uy;
            let a1 = (y1 - cy).atan2(x1 - cx);
            let a2 = (y2 - cy).atan2(x2 - cx);
            let mut span = (a2 - a1) % (2.0 * std::f64::consts::PI);
            if span < 0.0 {
                span += 2.0 * std::f64::consts::PI;
            }
            // sweep=1 表示沿角度增大方向（SVG 坐标系内）
            let is_large = span > std::f64::consts::PI;
            let is_sweep = true; // 走 CCW 方向即可到达终点
            if is_large == large && is_sweep == sweep {
                chosen = Some(((cx, cy), a1, span));
                break;
            }
            if chosen.is_none() {
                chosen = Some(((cx, cy), a1, span));
            }
        }
        let Some(((cx, cy), a1, span)) = chosen else {
            return;
        };
        if self.subpath.is_empty() {
            self.subpath.push(Self::world(x1, y1));
        }
        let segments = ((span / 0.1).ceil() as usize).clamp(8, 128);
        for i in 1..=segments {
            let a = a1 + span * i as f64 / segments as f64;
            let px = cx + r_eff * a.cos();
            let py = cy + r_eff * a.sin();
            self.subpath.push(Self::world(px, py));
        }
        self.cur = (x2, y2);
    }

    fn parse(&mut self, d: &str) {
        // 拆分命令字母与参数
        let mut cmd = ' ';
        let mut args = String::new();
        let process = |builder: &mut PathBuilder, cmd: char, args: &str| {
            let v = parse_floats(args);
            let mut idx = 0;
            match cmd {
                'M' | 'm' | 'L' | 'l' => {
                    while idx + 1 < v.len() + 1 && idx + 1 < v.len() + 1 {
                        if idx + 2 > v.len() {
                            break;
                        }
                        let (ax, ay) = (v[idx], v[idx + 1]);
                        idx += 2;
                        match cmd {
                            'M' => {
                                builder.flush();
                                builder.line_to(ax, ay);
                            }
                            'm' => {
                                builder.flush();
                                builder.line_to(builder.cur.0 + ax, builder.cur.1 + ay);
                            }
                            'L' => builder.line_to(ax, ay),
                            _ => builder.line_to(builder.cur.0 + ax, builder.cur.1 + ay),
                        }
                        // M 之后的隐式 L
                    }
                }
                'H' | 'h' | 'V' | 'v' => {
                    for &val in &v {
                        match cmd {
                            'H' => builder.line_to(val, builder.cur.1),
                            'h' => builder.line_to(builder.cur.0 + val, builder.cur.1),
                            'V' => builder.line_to(builder.cur.0, val),
                            _ => builder.line_to(builder.cur.0, builder.cur.1 + val),
                        }
                    }
                }
                'A' | 'a' => {
                    while idx + 7 <= v.len() {
                        let (rx, _ry, _rot, large, sweep, ax, ay) =
                            (v[idx], v[idx + 1], v[idx + 2], v[idx + 3], v[idx + 4], v[idx + 5], v[idx + 6]);
                        idx += 7;
                        let large = large != 0.0;
                        let sweep = sweep != 0.0;
                        match cmd {
                            'A' => builder.arc_to(rx, large, sweep, ax, ay),
                            _ => builder.arc_to(
                                rx,
                                large,
                                sweep,
                                builder.cur.0 + ax,
                                builder.cur.1 + ay,
                            ),
                        }
                    }
                }
                'Z' | 'z' => {
                    if let Some(first) = builder.subpath.first().copied() {
                        builder.line_to(first.x, -first.y);
                        let pts = std::mem::take(&mut builder.subpath);
                        builder.doc.add_entity(make_polyline(&pts, true));
                    }
                }
                _ => {}
            }
        };
        for c in d.chars() {
            if c.is_ascii_alphabetic() {
                if cmd != ' ' && !args.is_empty() {
                    process(self, cmd, &args);
                } else if cmd != ' ' {
                    process(self, cmd, "");
                }
                cmd = c;
                args.clear();
            } else {
                args.push(c);
            }
        }
        if cmd != ' ' {
            process(self, cmd, &args);
        }
        self.flush();
    }
}

fn handle_tag(doc: &mut Document, name: &str, attrs: &Attrs) {
    match name {
        "line" => {
            let (Some(x1), Some(y1), Some(x2), Some(y2)) = (
                attr_f(attrs, "x1"),
                attr_f(attrs, "y1"),
                attr_f(attrs, "x2"),
                attr_f(attrs, "y2"),
            ) else {
                return;
            };
            doc.add_entity(make_line(Point::new2d(x1, -y1), Point::new2d(x2, -y2)));
        }
        "rect" => {
            let (Some(x), Some(y), Some(w), Some(h)) = (
                attr_f(attrs, "x"),
                attr_f(attrs, "y"),
                attr_f(attrs, "width"),
                attr_f(attrs, "height"),
            ) else {
                return;
            };
            if w <= 0.0 || h <= 0.0 {
                return;
            }
            let pts = [
                Point::new2d(x, -y),
                Point::new2d(x + w, -y),
                Point::new2d(x + w, -y - h),
                Point::new2d(x, -y - h),
            ];
            doc.add_entity(make_polyline(&pts, true));
        }
        "circle" => {
            let (Some(cx), Some(cy), Some(r)) =
                (attr_f(attrs, "cx"), attr_f(attrs, "cy"), attr_f(attrs, "r"))
            else {
                return;
            };
            if r > 0.0 {
                doc.add_entity(make_circle(Point::new2d(cx, -cy), r));
            }
        }
        "ellipse" => {
            let (Some(cx), Some(cy), Some(rx), Some(ry)) = (
                attr_f(attrs, "cx"),
                attr_f(attrs, "cy"),
                attr_f(attrs, "rx"),
                attr_f(attrs, "ry"),
            ) else {
                return;
            };
            if (rx - ry).abs() < 1e-9 && rx > 0.0 {
                doc.add_entity(make_circle(Point::new2d(cx, -cy), rx));
            } else if rx > 0.0 && ry > 0.0 {
                // 椭圆按参数采样成多段线
                let segments = 72;
                let pts: Vec<Point> = (0..=segments)
                    .map(|i| {
                        let t = 2.0 * std::f64::consts::PI * i as f64 / segments as f64;
                        Point::new2d(cx + rx * t.cos(), -cy - ry * t.sin())
                    })
                    .collect();
                doc.add_entity(make_polyline(&pts, true));
            }
        }
        "polyline" | "polygon" => {
            let Some(points_str) = attrs.get("points") else {
                return;
            };
            let v = parse_floats(points_str);
            let mut pts: Vec<Point> = v
                .chunks(2)
                .filter(|c| c.len() == 2)
                .map(|c| Point::new2d(c[0], -c[1]))
                .collect();
            // 去掉重复的收尾点（polygon 常见）
            if name == "polygon" && pts.len() > 1 && pts.first() == pts.last() {
                pts.pop();
            }
            doc.add_entity(make_polyline(&pts, name == "polygon"));
        }
        "path" => {
            let Some(d) = attrs.get("d") else {
                return;
            };
            let mut builder = PathBuilder {
                doc: Document::new("svg-path".to_string()),
                cur: (0.0, 0.0),
                subpath: Vec::new(),
                started: false,
            };
            builder.parse(d);
            let mut sub_doc = builder.doc;
            for (_, entity) in sub_doc.entities_mut().drain() {
                doc.add_entity(entity);
            }
        }
        _ => {}
    }
}

/// 解析 SVG 文本并返回新建的 文档（固定名称 `imported-svg`），坐标已翻转为 y 轴向上。
///
/// 逐个扫描标签：跳过注释与 `<?...?>`/`<!...>` 声明，不处理嵌套结构、样式表与变换属性，
/// 因此只识别当前标签自身的几何属性。`rect`/`polyline`/`polygon`/非正圆 `ellipse` 都转换为
/// 折线（椭圆固定采样 72 段，圆整为 实体 圆），`path` 的弧段（A/a）按 0.1 弧度步长离散为折线。
/// - `src`：SVG 文本，函数不会读取磁盘。
///
/// 返回至少含一个实体的 文档；若整个文本没有可识别的图形则返回错误
/// （`No recognizable shapes found in SVG`），而不是返回空 文档。
///
/// # 示例
/// ```ignore
/// let doc = import("<svg><path d=\"M 0 0 L 10 0 L 10 10 Z\"/></svg>").unwrap();
/// assert!(doc.entity_count() >= 1);
/// ```
pub fn import(src: &str) -> Result<Document, String> {
    let mut doc = Document::new("imported-svg".to_string());
    let mut pos = 0usize;
    while let Some(off) = src[pos..].find('<') {
        let tag_start = pos + off;
        if src[tag_start..].starts_with("<!--") {
            if let Some(end) = src[tag_start..].find("-->") {
                pos = tag_start + end + 3;
                continue;
            }
            break;
        }
        let Some(gt) = src[tag_start..].find('>') else {
            break;
        };
        let tag_end = tag_start + gt;
        let inner = &src[tag_start + 1..tag_end];
        let content = inner.trim_end_matches('/');
        if !content.starts_with('?') && !content.starts_with('!') && !content.starts_with('/') {
            let mut parts = content.splitn(2, |c: char| c.is_whitespace());
            let name = parts.next().unwrap_or("").to_lowercase();
            let attrs_str = parts.next().unwrap_or("");
            let attrs = parse_attrs(attrs_str);
            handle_tag(&mut doc, &name, &attrs);
        }
        pos = tag_end + 1;
    }
    if doc.entity_count() == 0 {
        return Err("No recognizable shapes found in SVG".to_string());
    }
    Ok(doc)
}

/// SVG 导入器：把 `import` 包装成 `io::Importer` 实现，供按扩展名分发的入口统一调用。
///
/// 无状态，可自由按值拷贝；只接受扩展名 `svg`（忽略大小写）。
#[derive(Debug, Clone, Copy)]
pub struct SVGImporter;

impl SVGImporter {
    /// 创建一个 SVG 导入器。状态为空，重复调用等价。
    pub fn new() -> Self {
        Self
    }
}

impl Default for SVGImporter {
    fn default() -> Self {
        Self::new()
    }
}

impl Importer for SVGImporter {
    fn can_import(&self, extension: &str) -> bool {
        extension.to_lowercase() == "svg"
    }

    fn import_from_file(&self, filename: &str) -> Result<Document, ImportError> {
        let content = std::fs::read_to_string(filename)
            .map_err(|e| ImportError::Io(e.to_string()))?;
        self.import_from_bytes(content.as_bytes(), "svg")
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
    fn test_svg_importer_can_import() {
        let importer = SVGImporter::new();
        assert!(importer.can_import("svg"));
        assert!(importer.can_import("SVG"));
        assert!(!importer.can_import("dxf"));
    }

    #[test]
    fn test_svg_export_import_roundtrip() {
        let mut doc = Document::new("test".to_string());
        doc.add_entity(make_line(Point::new2d(0.0, 0.0), Point::new2d(10.0, 0.0)));
        doc.add_entity(make_circle(Point::new2d(5.0, 5.0), 2.0));

        let svg = export(&doc).unwrap();
        assert!(svg.contains("<line"));
        assert!(svg.contains("<circle"));

        let imported = import(&svg).unwrap();
        assert!(imported.entity_count() >= 2);
    }

    #[test]
    fn test_svg_import_path() {
        let src = "<svg><path d=\"M 0 0 L 10 0 L 10 10 Z\"/></svg>";
        let doc = import(src).unwrap();
        assert!(doc.entity_count() >= 1);
    }

    #[test]
    fn test_svg_import_rejects_empty() {
        assert!(import("<svg></svg>").is_err());
    }
}
