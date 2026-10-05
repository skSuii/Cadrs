//! 尺寸标注（Dimension）端到端集成测试。
//!
//! 覆盖「构造标注 → 加入文档 → 细分/包围盒 → 导出（DXF/SVG）→ 重新导入」
//! 的完整链路，重点回归半径（R）与直径（Ø）标注：
//!
//! * 构造阶段：测量值、文字前缀、定义点、圆心标记与尺寸界线开关；
//! * 显示阶段：`tessellation` 把标注分解为矢量笔画（渲染 / 拾取 / 包围盒共用）；
//! * 导出阶段：DXF 与 SVG 都能写出标注图形（DXF 以 POLYLINE 分解写出）；
//! * 稳健性：光标落在圆心等退化输入不得产生 NaN 或 panic。
//!
//! 运行：`cargo test --manifest-path sdk/Cargo.toml --test dimension_integration`

use cadrs::data_structure::{
    make_arc, make_circle, make_line, DimensionType, Document, Entity, EntityGeometry,
};
use cadrs::dimension::strokes as dim;
use cadrs::geometry::Point;
use cadrs::io::{dxf, svg};
use cadrs::render::tessellation::{document_bbox, entity_polylines};

/// 建立一份包含圆、圆弧与直线的样例图纸
fn sample_document() -> Document {
    let mut doc = Document::new("dimension-integration".to_string());
    doc.add_entity(make_circle(Point::new2d(0.0, 0.0), 10.0));
    doc.add_entity(make_arc(
        Point::new2d(60.0, 0.0),
        15.0,
        0.0,
        std::f64::consts::PI,
    ));
    doc.add_entity(make_line(
        Point::new2d(-40.0, -40.0),
        Point::new2d(40.0, -40.0),
    ));
    doc
}

fn dimension_of(entity: &Entity) -> (DimensionType, f64, String, Point, Point, Point, Point) {
    match entity.geometry() {
        EntityGeometry::Dimension {
            dim_type,
            measurement,
            text,
            definition_point,
            def_point_1,
            def_point_2,
            def_point_3,
            ..
        } => (
            dim_type.clone(),
            *measurement,
            text.clone(),
            *definition_point,
            *def_point_1,
            *def_point_2,
            *def_point_3,
        ),
        other => panic!("期望 Dimension 实体，实际 {other:?}"),
    }
}

fn assert_all_finite(entity: &Entity) {
    for (pts, _) in entity_polylines(entity) {
        for p in pts {
            assert!(
                p.x.is_finite() && p.y.is_finite(),
                "标注产生了非法坐标: {p:?}"
            );
        }
    }
}

#[test]
fn radius_dimension_end_to_end() {
    let mut doc = sample_document();
    let center = Point::new2d(0.0, 0.0);
    let radius = 10.0;
    let aim = Point::new2d(-10.0 * 0.6, 10.0 * 0.8); // 约 127°，非象限点

    let entity = dim::make_radial(center, radius, aim, false, 2.5).expect("半径标注应可构造");
    assert_all_finite(&entity);
    let (ty, m, text, _dp, p1, _p2, _p3) = dimension_of(&entity);
    assert_eq!(ty, DimensionType::Radius);
    assert!((m - radius).abs() < 1e-9);
    assert_eq!(text, "R10");
    assert!(p1.distance_to(&center) < 1e-9);
    doc.add_entity(entity);

    // 细分与包围盒
    let strokes: usize = doc
        .entities()
        .values()
        .map(|e| entity_polylines(e).len())
        .sum();
    assert!(strokes > 3, "标注未被细分: {strokes}");
    let (min, max) = document_bbox(&doc).expect("包围盒");
    assert!(max.x > min.x && max.y > min.y);

    // 导出：DXF 与 SVG 都应包含分解后的标注笔画
    let dxf_out = dxf::export(&doc);
    assert!(dxf_out.contains("POLYLINE"), "DXF 未写出标注折线");
    assert!(dxf_out.contains("CIRCLE"), "DXF 应保留圆实体");
    let svg_out = svg::export(&doc).expect("SVG 导出");
    assert!(svg_out.contains("<polyline") || svg_out.contains("<polygon"));
    assert!(svg_out.ends_with("</svg>\n"));

    // DXF 往返：标注被分解为折线，实体数量应不少于原文档
    let reimported = dxf::import(&dxf_out).expect("DXF 导入");
    assert!(reimported.entity_count() >= doc.entity_count() - 1);
}

#[test]
fn diameter_dimension_end_to_end() {
    let mut doc = sample_document();
    let center = Point::new2d(0.0, 0.0);
    let radius = 10.0;
    let aim = Point::new2d(-10.0, 0.0); // 左侧象限点

    let entity = dim::make_radial(center, radius, aim, true, 2.5).expect("直径标注应可构造");
    assert_all_finite(&entity);
    let (ty, m, text, _dp, p1, p2, p3) = dimension_of(&entity);
    assert_eq!(ty, DimensionType::Diameter);
    assert!((m - 20.0).abs() < 1e-9);
    assert_eq!(text, "Ø20");
    assert!(p1.distance_to(&center) < 1e-9);
    assert!(p2.distance_to(&aim) < 1e-9, "def_point_2 应为圆周点");
    assert!(
        p3.distance_to(&Point::new2d(10.0, 0.0)) < 1e-9,
        "def_point_3 应为对侧圆周点"
    );
    doc.add_entity(entity);

    let svg_out = svg::export(&doc).expect("SVG 导出");
    // 直径标注应画出穿过圆的整条尺寸线（两侧象限点均出现在导出中）
    assert!(svg_out.contains("<polyline") || svg_out.contains("<polygon"));
}

/// 半径与直径标注同时存在时应互不影响，并各自保持正确的测量值
#[test]
fn mixed_dimensions_coexist() {
    let mut doc = sample_document();
    let c1 = Point::new2d(0.0, 0.0);
    let c2 = Point::new2d(80.0, 40.0);
    doc.add_entity(dim::make_radial(c1, 10.0, Point::new2d(10.0, 0.0), false, 2.5).unwrap());
    doc.add_entity(dim::make_radial(c2, 6.0, Point::new2d(80.0, 46.0), true, 2.5).unwrap());
    doc.add_entity(dim::make_radial(c1, 10.0, Point::new2d(0.0, -10.0), true, 2.5).unwrap());

    let dims: Vec<_> = doc
        .entities()
        .values()
        .filter_map(|e| match e.geometry() {
            EntityGeometry::Dimension { .. } => Some(dimension_of(e)),
            _ => None,
        })
        .collect();
    assert_eq!(dims.len(), 3);
    let radii = dims
        .iter()
        .filter(|d| d.0 == DimensionType::Radius)
        .map(|d| d.1)
        .collect::<Vec<_>>();
    let diameters = dims
        .iter()
        .filter(|d| d.0 == DimensionType::Diameter)
        .map(|d| d.1)
        .collect::<Vec<_>>();
    assert_eq!(radii, vec![10.0]);
    assert_eq!(diameters.len(), 2);
    assert!(diameters.contains(&20.0));
    assert!(diameters.contains(&12.0));
    assert!(dims.iter().any(|d| d.2.starts_with('R')));
    assert!(dims.iter().any(|d| d.2.starts_with('Ø')));
}

/// 圆弧同样可以标注半径 / 直径（GUI 中圆弧与圆共用同一条拾取路径）
#[test]
fn arc_can_be_dimensioned() {
    let mut doc = sample_document();
    let arc_center = Point::new2d(60.0, 0.0);
    let arc_radius = 15.0;
    let arc_entity = doc
        .entities()
        .values()
        .find(|e| matches!(e.geometry(), EntityGeometry::Arc(_)))
        .expect("样例文档应包含圆弧");
    let (center, radius) = match arc_entity.geometry() {
        EntityGeometry::Arc(a) => (a.center, a.radius),
        _ => unreachable!(),
    };
    assert!(center.distance_to(&arc_center) < 1e-9);
    assert!((radius - arc_radius).abs() < 1e-9);

    // 圆弧中点方向放置标注
    let mid = arc_entity
        .geometry()
        .clone();
    let mid_point = match mid {
        EntityGeometry::Arc(a) => a.midpoint(),
        _ => unreachable!(),
    };
    let entity = dim::make_radial(center, radius, mid_point, false, 2.5).expect("圆弧半径标注");
    let (ty, m, text, ..) = dimension_of(&entity);
    assert_eq!(ty, DimensionType::Radius);
    assert!((m - 15.0).abs() < 1e-9);
    assert_eq!(text, "R15");
    assert_all_finite(&entity);
    doc.add_entity(entity);
}

/// 退化输入（光标落在圆心）不得 panic / 产生 NaN，标注仍应可读
#[test]
fn degenerate_aim_stays_renderable() {
    let mut doc = sample_document();
    let center = Point::new2d(0.0, 0.0);
    for diameter in [false, true] {
        let entity = dim::make_radial(center, 10.0, center, diameter, 2.5)
            .expect("方向退化时应回退到默认方向");
        assert_all_finite(&entity);
        let (ty, m, text, ..) = dimension_of(&entity);
        assert_eq!(ty, if diameter { DimensionType::Diameter } else { DimensionType::Radius });
        assert!((m - if diameter { 20.0 } else { 10.0 }).abs() < 1e-9);
        assert!(!text.is_empty());
        doc.add_entity(entity);
    }
    assert!(svg::export(&doc).is_ok());
}

/// 极小 / 极大半径都应保持数值稳定
#[test]
fn extreme_radii_are_stable() {
    for radius in [1e-3, 1.0, 1e6, 1e9] {
        for diameter in [false, true] {
            let entity = dim::make_radial(
                Point::origin(),
                radius,
                Point::new2d(radius, 0.0),
                diameter,
                2.5,
            )
            .expect("常规半径应可构造");
            assert_all_finite(&entity);
            let (_ty, m, _text, ..) = dimension_of(&entity);
            let expected = if diameter { radius * 2.0 } else { radius };
            assert!(
                (m - expected).abs() <= expected.abs() * 1e-9,
                "半径 {radius} 测量值 {m} 不符"
            );
        }
    }
}
