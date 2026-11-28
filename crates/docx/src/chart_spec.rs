//! Charts Doc4Me makes ([`ChartSpec`]) as chart parts (ECMA-376 Part 1 §21.2) and back.
//!
//! [`chart_xml`] writes a `c:chartSpace` with the data as literals in the part itself
//! (`c:strLit`, `c:numLit`): no embedded workbook (`c:externalData`) and no external references.
//! [`chart_items`] draws a chart through the same code that draws chart parts read from files.
//! [`spec_of`] turns a chart part back into the model only when writing that model again gives
//! the same markup, so a chart with anything the model leaves out stays as it was read.

use doc4me_doc::chart::{ChartSeries, ChartSpec, ChartType, LegendPos, MAX_POINTS, MAX_SERIES};
use doc4me_doc::graphic::GraphicItem;
use doc4me_doc::props::Rgb;

use doc4me_chart_format::{self as kf, Part};

use crate::xml::{self, El, MAX_DEPTH, Node, W};

const NS_C: &str = "http://schemas.openxmlformats.org/drawingml/2006/chart";
const NS_A: &str = "http://schemas.openxmlformats.org/drawingml/2006/main";
const NS_R: &str = "http://schemas.openxmlformats.org/officeDocument/2006/relationships";
/// Content type of a chart part.
pub(crate) const CHART_CT: &str = "application/vnd.openxmlformats-officedocument.drawingml.chart+xml";
/// `a:graphicData` URI of a chart.
pub(crate) const CHART_URI: &str = "http://schemas.openxmlformats.org/drawingml/2006/chart";
/// Axis ids (any pair of distinct numbers will do; Word keeps them).
const CAT_AX: &str = "500000001";
const VAL_AX: &str = "500000002";

/// The chart part for `spec`: a complete XML document. `spec` should be sanitized
/// ([`ChartSpec::sanitize`]); a value that isn't finite is written as a gap either way.
pub fn chart_xml(spec: &ChartSpec) -> String {
    let spec = &bounded(spec);
    let mut w = W::new();
    w.open("c:chartSpace", &[("xmlns:c", NS_C), ("xmlns:a", NS_A), ("xmlns:r", NS_R)]);
    let f = &spec.format;
    w.empty("c:roundedCorners", &[("val", if f.rounded_corners { "1" } else { "0" })]);
    w.open("c:chart", &[]);
    if let Some(t) = &spec.title {
        w.open("c:title", &[]);
        w.open("c:tx", &[]);
        w.open("c:rich", &[]);
        w.empty("a:bodyPr", &[]);
        w.empty("a:lstStyle", &[]);
        w.open("a:p", &[]);
        // The chart pane's text look (`doc4me_chart_format`) on the title's run.
        let text = part_text(spec, Part::Title);
        if let Some(look) = &text {
            let (attrs, kids) = kf::run_props(look);
            w.raw(&format!("<a:pPr><a:defRPr{attrs}>{kids}</a:defRPr></a:pPr>"));
        }
        w.open("a:r", &[]);
        if let Some(look) = &text {
            let (attrs, kids) = kf::run_props(look);
            w.raw(&format!("<a:rPr lang=\"en-US\"{attrs}>{kids}</a:rPr>"));
        }
        w.leaf("a:t", &[], t);
        w.close("a:r");
        w.close("a:p");
        w.close("c:rich");
        w.close("c:tx");
        w.empty("c:overlay", &[("val", "0")]);
        w.raw(&kf::sp_pr(&f.title));
        w.close("c:title");
    }
    w.empty("c:autoTitleDeleted", &[("val", if spec.title.is_some() { "0" } else { "1" })]);
    w.open("c:plotArea", &[]);
    w.empty("c:layout", &[]);
    plot(&mut w, spec);
    w.raw(&kf::sp_pr(&f.plot_area));
    w.close("c:plotArea");
    if let Some(pos) = spec.legend {
        w.open("c:legend", &[]);
        let p = match pos {
            LegendPos::Right => "r",
            LegendPos::Left => "l",
            LegendPos::Top => "t",
            LegendPos::Bottom => "b",
        };
        w.empty("c:legendPos", &[("val", p)]);
        w.empty("c:overlay", &[("val", "0")]);
        w.raw(&kf::sp_pr(&f.legend));
        w.raw(&part_text(spec, Part::Legend).map(|t| kf::tx_pr(&t)).unwrap_or_default());
        w.close("c:legend");
    }
    w.empty("c:plotVisOnly", &[("val", "1")]);
    w.empty("c:dispBlanksAs", &[("val", "gap")]);
    w.close("c:chart");
    if f.chart_area.shape_is_auto() {
        // A white chart area without a border, in the document's own background colour.
        w.open("c:spPr", &[]);
        w.open("a:solidFill", &[]);
        w.empty("a:schemeClr", &[("val", "bg1")]);
        w.close("a:solidFill");
        w.open("a:ln", &[]);
        w.empty("a:noFill", &[]);
        w.close("a:ln");
        w.close("c:spPr");
    } else {
        w.raw(&kf::sp_pr(&f.chart_area));
    }
    w.raw(&kf::tx_pr(&f.chart_area.text));
    w.close("c:chartSpace");
    w.s
}

/// What `spec` draws as, `w` × `h` points, in the colours of `theme` (series take the accents in
/// turn). Empty when the chart has no series.
pub fn chart_items(spec: &ChartSpec, theme: &[Rgb], w: f32, h: f32) -> Vec<GraphicItem> {
    let spec = &bounded(spec);
    match xml::parse(chart_xml(spec).as_bytes()) {
        Ok(space) => crate::read::chart::chart_items(&space, theme, w, h),
        Err(_) => Vec::new(),
    }
}

/// Keep a chart received from an untrusted model within the same budgets as the document model.
fn bounded(spec: &ChartSpec) -> ChartSpec {
    ChartSpec {
        kind: spec.kind,
        categories: spec.categories.iter().take(MAX_POINTS).cloned().collect(),
        series: spec.series.iter().take(MAX_SERIES)
            .map(|s| ChartSeries { name: s.name.clone(), values: s.values.iter().take(MAX_POINTS).copied().collect() })
            .collect(),
        title: spec.title.clone(),
        legend: spec.legend,
        data_labels: spec.data_labels,
        format: spec.format.clone(),
    }.sanitized()
}

/// The chart type element and the series in it, then the axes.
fn plot(w: &mut W, spec: &ChartSpec) {
    let k = spec.kind;
    let el = match k {
        ChartType::Column | ChartType::ColumnStacked | ChartType::Bar | ChartType::BarStacked => "c:barChart",
        ChartType::Line => "c:lineChart",
        ChartType::Area => "c:areaChart",
        ChartType::Pie => "c:pieChart",
        ChartType::Doughnut => "c:doughnutChart",
        ChartType::Scatter => "c:scatterChart",
    };
    let stacked = matches!(k, ChartType::ColumnStacked | ChartType::BarStacked);
    let horizontal = matches!(k, ChartType::Bar | ChartType::BarStacked);
    w.open(el, &[]);
    match k {
        ChartType::Column | ChartType::ColumnStacked | ChartType::Bar | ChartType::BarStacked => {
            w.empty("c:barDir", &[("val", if horizontal { "bar" } else { "col" })]);
            w.empty("c:grouping", &[("val", if stacked { "stacked" } else { "clustered" })]);
        }
        ChartType::Line | ChartType::Area => w.empty("c:grouping", &[("val", "standard")]),
        ChartType::Scatter => w.empty("c:scatterStyle", &[("val", "lineMarker")]),
        ChartType::Pie | ChartType::Doughnut => {}
    }
    w.empty("c:varyColors", &[("val", if k.is_round() { "1" } else { "0" })]);
    // Pie and doughnut charts show the first series; the others are kept for other types.
    for (i, s) in spec.series.iter().enumerate() {
        series(w, spec, i, s);
    }
    if spec.data_labels {
        w.open("c:dLbls", &[]);
        for (name, on) in [
            ("c:showLegendKey", false),
            ("c:showVal", true),
            ("c:showCatName", false),
            ("c:showSerName", false),
            ("c:showPercent", false),
            ("c:showBubbleSize", false),
        ] {
            w.empty(name, &[("val", if on { "1" } else { "0" })]);
        }
        w.close("c:dLbls");
    }
    match k {
        ChartType::Column | ChartType::ColumnStacked | ChartType::Bar | ChartType::BarStacked => {
            w.empty("c:gapWidth", &[("val", "150")]);
            if stacked {
                w.empty("c:overlap", &[("val", "100")]);
            }
        }
        ChartType::Line => w.empty("c:marker", &[("val", "1")]),
        ChartType::Pie => w.empty("c:firstSliceAng", &[("val", "0")]),
        ChartType::Doughnut => {
            w.empty("c:firstSliceAng", &[("val", "0")]);
            w.empty("c:holeSize", &[("val", "50")]);
        }
        ChartType::Area | ChartType::Scatter => {}
    }
    if !k.is_round() {
        w.empty("c:axId", &[("val", CAT_AX)]);
        w.empty("c:axId", &[("val", VAL_AX)]);
    }
    w.close(el);
    match k {
        ChartType::Pie | ChartType::Doughnut => {}
        ChartType::Scatter => {
            value_axis(w, spec, Part::CategoryAxis, (CAT_AX, VAL_AX, "b"), false, "midCat");
            value_axis(w, spec, Part::ValueAxis, (VAL_AX, CAT_AX, "l"), true, "midCat");
        }
        _ => {
            category_axis(w, spec, if horizontal { "l" } else { "b" });
            value_axis(w, spec, Part::ValueAxis, (VAL_AX, CAT_AX, if horizontal { "b" } else { "l" }), true, "between");
        }
    }
}

/// One `c:ser` (ECMA-376 element order: idx, order, tx, spPr, marker, cat/xVal, val/yVal, smooth).
fn series(w: &mut W, spec: &ChartSpec, i: usize, s: &ChartSeries) {
    let k = spec.kind;
    let i = i.to_string();
    w.open("c:ser", &[]);
    w.empty("c:idx", &[("val", &i)]);
    w.empty("c:order", &[("val", &i)]);
    if !s.name.is_empty() {
        w.open("c:tx", &[]);
        w.leaf("c:v", &[], &s.name);
        w.close("c:tx");
    }
    let own = spec.format.get(Part::Series(i.parse().unwrap_or(0)));
    if !own.shape_is_auto() {
        w.raw(&kf::sp_pr(&own));
    }
    match k {
        // Markers only: the series has no line.
        ChartType::Scatter if own.shape_is_auto() => {
            w.open("c:spPr", &[]);
            w.open("a:ln", &[("w", "19050")]);
            w.empty("a:noFill", &[]);
            w.close("a:ln");
            w.close("c:spPr");
        }
        ChartType::Column | ChartType::ColumnStacked | ChartType::Bar | ChartType::BarStacked => {
            w.empty("c:invertIfNegative", &[("val", "0")]);
        }
        ChartType::Line => {
            w.open("c:marker", &[]);
            w.empty("c:symbol", &[("val", "none")]);
            w.close("c:marker");
        }
        _ => {}
    }
    if k == ChartType::Scatter {
        let xs: Option<Vec<Option<f64>>> = spec
            .categories
            .iter()
            .map(|c| if c.is_empty() { Some(None) } else { c.parse::<f64>().ok().filter(|v| v.is_finite()).map(Some) })
            .collect();
        w.open("c:xVal", &[]);
        match xs {
            Some(xs) => num_lit(w, &xs),
            None => str_lit(w, &spec.categories),
        }
        w.close("c:xVal");
        w.open("c:yVal", &[]);
        num_lit(w, &s.values);
        w.close("c:yVal");
    } else {
        w.open("c:cat", &[]);
        str_lit(w, &spec.categories);
        w.close("c:cat");
        w.open("c:val", &[]);
        num_lit(w, &s.values);
        w.close("c:val");
    }
    if matches!(k, ChartType::Line | ChartType::Scatter) {
        w.empty("c:smooth", &[("val", "0")]);
    }
    w.close("c:ser");
}

fn str_lit(w: &mut W, items: &[String]) {
    w.open("c:strLit", &[]);
    w.empty("c:ptCount", &[("val", &items.len().to_string())]);
    for (i, s) in items.iter().enumerate().filter(|(_, s)| !s.is_empty()) {
        w.open("c:pt", &[("idx", &i.to_string())]);
        w.leaf("c:v", &[], s);
        w.close("c:pt");
    }
    w.close("c:strLit");
}

fn num_lit(w: &mut W, values: &[Option<f64>]) {
    w.open("c:numLit", &[]);
    w.leaf("c:formatCode", &[], "General");
    w.empty("c:ptCount", &[("val", &values.len().to_string())]);
    for (i, v) in values.iter().enumerate() {
        if let Some(v) = v.filter(|v| v.is_finite()) {
            w.open("c:pt", &[("idx", &i.to_string())]);
            w.leaf("c:v", &[], &num(v));
            w.close("c:pt");
        }
    }
    w.close("c:numLit");
}

/// A value as the shortest text that reads back as the same number.
fn num(v: f64) -> String {
    if v == 0.0 { "0".to_string() } else { format!("{v}") }
}

/// The text look the chart pane gave `part` itself. The chart area's (the chart space's
/// `c:txPr`) reaches every part by itself in the file format, so it isn't copied here.
fn part_text(spec: &ChartSpec, part: Part) -> Option<kf::Text> {
    let own = spec.format.get(part).text;
    (!own.is_auto()).then_some(own)
}

/// An axis' `c:spPr` and `c:txPr` from the chart pane, after `c:tickLblPos`.
fn axis_look(w: &mut W, spec: &ChartSpec, part: Part) {
    w.raw(&kf::sp_pr(&spec.format.get(part)));
    w.raw(&spec.format.get(part).text.is_auto().then(String::new).unwrap_or_else(|| kf::tx_pr(&spec.format.get(part).text)));
}

fn category_axis(w: &mut W, spec: &ChartSpec, pos: &str) {
    w.open("c:catAx", &[]);
    w.empty("c:axId", &[("val", CAT_AX)]);
    w.open("c:scaling", &[]);
    w.empty("c:orientation", &[("val", "minMax")]);
    w.close("c:scaling");
    w.empty("c:delete", &[("val", "0")]);
    w.empty("c:axPos", &[("val", pos)]);
    w.empty("c:majorTickMark", &[("val", "none")]);
    w.empty("c:minorTickMark", &[("val", "none")]);
    w.empty("c:tickLblPos", &[("val", "nextTo")]);
    axis_look(w, spec, Part::CategoryAxis);
    w.empty("c:crossAx", &[("val", VAL_AX)]);
    w.empty("c:crosses", &[("val", "autoZero")]);
    w.empty("c:auto", &[("val", "1")]);
    w.empty("c:lblAlgn", &[("val", "ctr")]);
    w.empty("c:lblOffset", &[("val", "100")]);
    w.empty("c:noMultiLvlLbl", &[("val", "0")]);
    w.close("c:catAx");
}

/// A value axis: with gridlines and no axis line (`grid`), else with its line and no gridlines.
fn value_axis(w: &mut W, spec: &ChartSpec, part: Part, (id, cross, pos): (&str, &str, &str), grid: bool, between: &str) {
    w.open("c:valAx", &[]);
    w.empty("c:axId", &[("val", id)]);
    w.open("c:scaling", &[]);
    w.empty("c:orientation", &[("val", "minMax")]);
    w.close("c:scaling");
    w.empty("c:delete", &[("val", "0")]);
    w.empty("c:axPos", &[("val", pos)]);
    if grid {
        let g = &spec.format.gridlines;
        if g.shape_is_auto() {
            w.empty("c:majorGridlines", &[]);
        } else {
            w.open("c:majorGridlines", &[]);
            w.raw(&kf::sp_pr(g));
            w.close("c:majorGridlines");
        }
    }
    w.empty("c:majorTickMark", &[("val", "none")]);
    w.empty("c:minorTickMark", &[("val", "none")]);
    w.empty("c:tickLblPos", &[("val", "nextTo")]);
    let own = spec.format.get(part);
    if grid && own.shape_is_auto() {
        w.open("c:spPr", &[]);
        w.open("a:ln", &[]);
        w.empty("a:noFill", &[]);
        w.close("a:ln");
        w.close("c:spPr");
        if !own.text.is_auto() {
            w.raw(&kf::tx_pr(&own.text));
        }
    } else {
        axis_look(w, spec, part);
    }
    w.empty("c:crossAx", &[("val", cross)]);
    w.empty("c:crosses", &[("val", "autoZero")]);
    w.empty("c:crossBetween", &[("val", between)]);
    w.close("c:valAx");
}

/// The chart a chart part holds, when the part is exactly what [`chart_xml`] writes for it (so
/// editing and saving it loses nothing). `None` for any other chart.
pub(crate) fn spec_of(space: &El) -> Option<ChartSpec> {
    let chart = space.child("c:chart")?;
    let plot = chart.child("c:plotArea")?;
    let ty = plot.els().find(|e| e.name.ends_with("Chart"))?;
    let val = |e: &El, n: &str| e.child(n).and_then(|c| c.attr("val")).map(str::to_string);
    let bar = |stacked_kind, clustered_kind| if val(ty, "c:grouping").as_deref() == Some("stacked") { stacked_kind } else { clustered_kind };
    let kind = match ty.name.as_str() {
        "c:barChart" if val(ty, "c:barDir").as_deref() == Some("bar") => bar(ChartType::BarStacked, ChartType::Bar),
        "c:barChart" => bar(ChartType::ColumnStacked, ChartType::Column),
        "c:lineChart" => ChartType::Line,
        "c:areaChart" => ChartType::Area,
        "c:pieChart" => ChartType::Pie,
        "c:doughnutChart" => ChartType::Doughnut,
        "c:scatterChart" => ChartType::Scatter,
        _ => return None,
    };
    let scatter = kind == ChartType::Scatter;
    let sers: Vec<&El> = ty.children("c:ser").take(doc4me_doc::chart::MAX_SERIES + 1).collect();
    // More than the model holds can't be the same chart: don't build it to find out.
    let count = |s: &&El| s.find("c:ptCount").and_then(|c| c.attr("val")).and_then(|v| v.parse::<usize>().ok()).unwrap_or(0);
    let longest = sers.iter().map(count).max().unwrap_or(0);
    if sers.len() > doc4me_doc::chart::MAX_SERIES
        || longest > doc4me_doc::chart::MAX_POINTS
        || sers.len().saturating_mul(longest) > doc4me_doc::chart::MAX_CELLS
    {
        return None;
    }
    let categories = sers.first().and_then(|s| s.child(if scatter { "c:xVal" } else { "c:cat" })).map(literal).unwrap_or_default();
    let series = sers
        .iter()
        .map(|s| ChartSeries {
            name: s.child("c:tx").and_then(|t| t.child("c:v")).map(|v| v.text()).unwrap_or_default(),
            values: s
                .child(if scatter { "c:yVal" } else { "c:val" })
                .map(literal)
                .unwrap_or_default()
                .iter()
                .map(|v| v.parse::<f64>().ok().filter(|v| v.is_finite()))
                .collect(),
        })
        .collect();
    let title = chart.child("c:title").and_then(|t| t.find("a:t")).map(|t| t.text());
    let legend = chart.child("c:legend").map(|l| match val(l, "c:legendPos").as_deref() {
        Some("l") => LegendPos::Left,
        Some("t") => LegendPos::Top,
        Some("b") => LegendPos::Bottom,
        _ => LegendPos::Right,
    });
    let data_labels = ty.child("c:dLbls").and_then(|d| d.child("c:showVal")).and_then(|v| v.attr("val")) == Some("1");
    let format = formats_of(space, chart, plot, ty);
    let spec = ChartSpec { kind, categories, series, title, legend, data_labels, format }.sanitized();
    // Editable only when nothing would be lost: writing the model again gives the same part.
    let again = xml::parse(chart_xml(&spec).as_bytes()).ok()?;
    same(space, &again, 0).then_some(spec)
}

/// The shared tree (`doc4me_chart_format::X`) of `el`, at most `depth` levels and 64 children each.
pub(crate) fn x_of(el: &El, depth: usize) -> kf::X {
    let local = |n: &str| n.rsplit(':').next().unwrap_or(n).to_string();
    kf::X {
        name: local(&el.name),
        attrs: el.attrs.iter().map(|(k, v)| (local(k), v.clone())).collect(),
        kids: if depth == 0 { Vec::new() } else { el.els().take(64).map(|k| x_of(k, depth - 1)).collect() },
    }
}

/// A part's look from its `c:spPr`, `c:txPr` and title text, when fully understood.
pub(crate) fn part_of(el: &El) -> kf::Format {
    let mut x = kf::X { name: "part".into(), ..Default::default() };
    for name in ["c:spPr", "c:txPr", "c:tx"] {
        if let Some(k) = el.child(name) {
            x.kids.push(x_of(k, 8));
        }
    }
    kf::read_part_strict(&x)
}

/// The chart pane's formats of a chart part (`ty` its chart type element).
pub(crate) fn formats_of(space: &El, chart: &El, plot: &El, ty: &El) -> kf::Formats {
    let mut f = kf::Formats {
        chart_area: part_of(space),
        plot_area: part_of(plot),
        rounded_corners: space.child("c:roundedCorners").and_then(|r| r.attr("val")) == Some("1"),
        ..Default::default()
    };
    if let Some(t) = chart.child("c:title") {
        f.title = part_of(t);
    }
    if let Some(l) = chart.child("c:legend") {
        f.legend = part_of(l);
    }
    let horizontal_val = plot
        .child("c:catAx")
        .is_none()
        .then(|| plot.children("c:valAx").find(|a| a.child("c:axPos").and_then(|p| p.attr("val")) == Some("b")))
        .flatten();
    if let Some(a) = plot.child("c:catAx").or(horizontal_val) {
        f.category_axis = part_of(a);
    }
    if let Some(a) = plot.children("c:valAx").find(|a| !horizontal_val.is_some_and(|h| std::ptr::eq(*a, h))) {
        f.value_axis = part_of(a);
        // The value axis without a line under gridlines is the automatic one.
        if a.child("c:majorGridlines").is_some() && f.value_axis.border == kf::Border::None && f.value_axis.fill == kf::Fill::Auto {
            f.value_axis.border = kf::Border::Auto;
        }
        if let Some(g) = a.child("c:majorGridlines") {
            f.gridlines = part_of(g);
        }
    }
    // A scatter chart's automatic series have no line, written 1.5 pt wide (`a:ln w="19050"`).
    let scatter_default = |ser: &El| {
        let ln = ser.child("c:spPr").and_then(|p| p.child("a:ln"));
        ln.is_some_and(|l| l.attr("w") == Some("19050") && l.child("a:noFill").is_some())
    };
    f.series = ty.children("c:ser").take(kf::MAX_SERIES).map(|ser| if scatter_default(ser) { kf::Format::default() } else { part_of(ser) }).collect();
    while f.series.last().is_some_and(kf::Format::is_auto) {
        f.series.pop();
    }
    f
}

/// The points of a `c:strLit` / `c:numLit` (or a cache) as texts by index, `ptCount` long.
fn literal(e: &El) -> Vec<String> {
    let Some(lit) = e.els().find(|c| matches!(c.name.as_str(), "c:strLit" | "c:numLit")) else { return Vec::new() };
    let n = lit
        .child("c:ptCount")
        .and_then(|c| c.attr("val"))
        .and_then(|v| v.parse::<usize>().ok())
        .unwrap_or(0)
        .min(doc4me_doc::chart::MAX_POINTS + 1);
    let mut out = vec![String::new(); n];
    for p in lit.children("c:pt") {
        if let Some(slot) = p.attr("idx").and_then(|i| i.parse::<usize>().ok()).and_then(|i| out.get_mut(i)) {
            *slot = p.child("c:v").map(|v| v.text()).unwrap_or_default();
        }
    }
    out
}

/// Whether two elements are the same markup: names, attributes (in any order), child elements in
/// order and text (whitespace-only text between elements ignored).
fn same(a: &El, b: &El, depth: usize) -> bool {
    if depth > MAX_DEPTH || a.name != b.name || a.attrs.len() != b.attrs.len() {
        return false;
    }
    if !a.attrs.iter().all(|(k, v)| b.attr(k) == Some(v.as_str()) && b.attrs.iter().any(|(bk, _)| bk == k)) {
        return false;
    }
    let text = |e: &El| -> String {
        e.kids
            .iter()
            .filter_map(|n| match n {
                Node::Text(t) if !t.trim().is_empty() => Some(t.as_str()),
                _ => None,
            })
            .collect()
    };
    if text(a) != text(b) {
        return false;
    }
    let (mut x, mut y) = (a.els(), b.els());
    loop {
        match (x.next(), y.next()) {
            (None, None) => return true,
            (Some(p), Some(q)) if same(p, q, depth + 1) => {}
            _ => return false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use doc4me_doc::THEME_COLORS;

    fn parse(spec: &ChartSpec) -> El {
        xml::parse(chart_xml(spec).as_bytes()).unwrap()
    }

    /// Children of `e` by name, in order.
    fn names(e: &El) -> Vec<&str> {
        e.els().map(|c| c.name.as_str()).collect()
    }

    /// The chart pane's formats (owner, 2026-11-19) are written and read back for every type, so
    /// a formatted chart stays editable after a save.
    #[test]
    fn chart_pane_formats_read_back() {
        use doc4me_chart_format::{Dash, Edit, Kind};
        for kind in ChartType::ALL {
            let mut spec = ChartSpec::sample(kind);
            spec.title = Some("Sales".into());
            let edits = [
                (Part::ChartArea, Edit::FillColor([0xF7, 0xF4, 0xEC])),
                (Part::ChartArea, Edit::Shadow(true)),
                (Part::ChartArea, Edit::RoundedCorners(true)),
                (Part::ChartArea, Edit::TextColor([0x1E, 0x3A, 0x6E])),
                (Part::PlotArea, Edit::BorderColor([0, 0, 0])),
                (Part::Title, Edit::FontSize(Some(18.0))),
                (Part::Legend, Edit::Fill(Kind::None)),
                (Part::CategoryAxis, Edit::Dash(Dash::Dotted)),
                (Part::ValueAxis, Edit::TextColor([0xC0, 0, 0])),
                (Part::Gridlines, Edit::BorderColor([0xEE, 0xEE, 0xEE])),
                (Part::Series(0), Edit::FillColor([0xE9, 0xC4, 0x6A])),
                (Part::Series(0), Edit::GlowSize(3.0)),
            ];
            // A pie or doughnut has no axes or gridlines to write them on.
            let axes = |p: Part| matches!(p, Part::CategoryAxis | Part::ValueAxis | Part::Gridlines);
            for (p, e) in edits.into_iter().filter(|(p, _)| !(kind.is_round() && axes(*p))) {
                spec.format.apply(p, &e);
            }
            let spec = spec.sanitized();
            let back = spec_of(&parse(&spec));
            assert_eq!(back.as_ref().map(|b| &b.format), Some(&spec.format), "{kind:?}: {}", chart_xml(&spec));
            assert!(!chart_items(&spec, &THEME_COLORS, 300.0, 200.0).is_empty(), "{kind:?} draws");
        }
    }

    #[test]
    fn every_type_reads_back_as_itself() {
        for kind in ChartType::ALL {
            let mut spec = ChartSpec::sample(kind);
            spec.title = Some("Sales & <costs>".into());
            spec.data_labels = true;
            spec.legend = Some(LegendPos::Right);
            spec.series[0].values[1] = None;
            spec.categories[2] = String::new();
            let spec = spec.sanitized();
            assert_eq!(spec_of(&parse(&spec)).as_ref(), Some(&spec), "{kind:?}");
            assert!(!chart_items(&spec, &THEME_COLORS, 300.0, 200.0).is_empty(), "{kind:?} draws");
        }
        let plain = ChartSpec { legend: None, ..ChartSpec::sample(ChartType::Line) };
        assert_eq!(spec_of(&parse(&plain)), Some(plain));
    }

    #[test]
    fn a_chart_with_more_than_the_model_stays_read_only() {
        let spec = ChartSpec::sample(ChartType::Column);
        let x = chart_xml(&spec).replace("<c:roundedCorners val=\"0\"/>", "<c:roundedCorners val=\"0\"/><c:style val=\"2\"/>");
        assert_eq!(spec_of(&xml::parse(x.as_bytes()).unwrap()), None);
        let x = chart_xml(&spec).replace("<c:v>3.2</c:v>", "<c:v>3.20</c:v>");
        assert_eq!(spec_of(&xml::parse(x.as_bytes()).unwrap()), None);
    }

    #[test]
    fn elements_come_in_schema_order() {
        let mut spec = ChartSpec::sample(ChartType::ColumnStacked);
        spec.title = Some("T".into());
        spec.data_labels = true;
        let space = parse(&spec);
        assert_eq!(names(&space), ["c:roundedCorners", "c:chart", "c:spPr"]);
        let chart = space.child("c:chart").unwrap();
        assert_eq!(names(chart), ["c:title", "c:autoTitleDeleted", "c:plotArea", "c:legend", "c:plotVisOnly", "c:dispBlanksAs"]);
        let plot = chart.child("c:plotArea").unwrap();
        assert_eq!(names(plot), ["c:layout", "c:barChart", "c:catAx", "c:valAx"]);
        let bar = plot.child("c:barChart").unwrap();
        assert_eq!(
            names(bar),
            ["c:barDir", "c:grouping", "c:varyColors", "c:ser", "c:ser", "c:ser", "c:dLbls", "c:gapWidth", "c:overlap", "c:axId", "c:axId"]
        );
        assert_eq!(names(bar.child("c:ser").unwrap()), ["c:idx", "c:order", "c:tx", "c:invertIfNegative", "c:cat", "c:val"]);
        let val_ax = plot.child("c:valAx").unwrap();
        assert_eq!(
            names(val_ax),
            [
                "c:axId",
                "c:scaling",
                "c:delete",
                "c:axPos",
                "c:majorGridlines",
                "c:majorTickMark",
                "c:minorTickMark",
                "c:tickLblPos",
                "c:spPr",
                "c:crossAx",
                "c:crosses",
                "c:crossBetween"
            ]
        );
        let line = parse(&ChartSpec { data_labels: true, ..ChartSpec::sample(ChartType::Line) });
        let lc = line.child("c:chart").unwrap().child("c:plotArea").unwrap().child("c:lineChart").unwrap();
        assert_eq!(names(lc), ["c:grouping", "c:varyColors", "c:ser", "c:ser", "c:ser", "c:dLbls", "c:marker", "c:axId", "c:axId"]);
        assert_eq!(names(lc.child("c:ser").unwrap()), ["c:idx", "c:order", "c:tx", "c:marker", "c:cat", "c:val", "c:smooth"]);
        let sc = parse(&ChartSpec::sample(ChartType::Scatter));
        let sc = sc.child("c:chart").unwrap().child("c:plotArea").unwrap().child("c:scatterChart").unwrap();
        assert_eq!(names(sc.child("c:ser").unwrap()), ["c:idx", "c:order", "c:tx", "c:spPr", "c:xVal", "c:yVal", "c:smooth"]);
        let dn = parse(&ChartSpec::sample(ChartType::Doughnut));
        let dn = dn.child("c:chart").unwrap().child("c:plotArea").unwrap();
        assert_eq!(names(dn), ["c:layout", "c:doughnutChart"]);
        assert_eq!(names(dn.child("c:doughnutChart").unwrap()), ["c:varyColors", "c:ser", "c:firstSliceAng", "c:holeSize"]);
    }
}
