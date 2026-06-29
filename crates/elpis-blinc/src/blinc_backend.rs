//! The live Blinc backend (compiled only with `--features blinc-backend`).
//!
//! This is the thin interpreter promised by [`crate::lower`]: it walks a
//! [`BlincDom`] and constructs the corresponding `blinc_layout` element tree,
//! wiring events back into the host. It is **platform-agnostic** — the actual
//! window/canvas/activity run loops live in the per-platform demo crates
//! (`elpis-app` for desktop, `elpis-web` for wasm, `elpis-android` for Android),
//! each of which supplies its own `blinc_app` platform feature and calls
//! [`frame_closure`]. A desktop convenience [`run_windowed`] is provided here
//! behind the `desktop` feature.
//!
//! Because Blinc rebuilds its declarative tree from the build closure every
//! frame, the bridge keeps the latest lowered [`BlincDom`] in a shared cell:
//! the host renders the guest's tree into the backend (which lowers + stores
//! it), and the Blinc build closure reads that cell to construct widgets and
//! pushes UI events into a shared queue the host drains on the next frame.
#![cfg(feature = "blinc-backend")]

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use serde_json::Value;

use blinc_app::windowed::WindowedContext;
use blinc_core::{
    Brush as BBrush, ClipShape, Color as BColor, CornerRadius as BCorner, DrawContext, Gradient,
    GradientSpace, GradientSpread, GradientStop as BStop, LineCap as BLineCap, LineJoin as BLineJoin,
    Path as BPath, Point as BPoint, Rect as BRect, Stroke as BStroke, TextStyle as BTextStyle,
    Transform as BTransform,
};
use blinc_app::TextAlign as LTextAlign;
use blinc_layout::canvas::{canvas, Canvas, CanvasBounds};
use blinc_layout::div::{div, Div};
use blinc_layout::image::{image, ObjectFit};
use blinc_layout::prelude::*;
use blinc_layout::svg::svg;
use blinc_layout::text::{text as btext, Text};

use elpis_protocol::canvas::{
    CanvasSpec, DrawOp, LineCap, LineJoin, PathSeg, Point as PPoint, Rect as PRect, Stroke as PStroke,
};
use elpis_protocol::node::{FontWeight, NodeKind, ProgressSpec, SpinnerSpec, TextAlign, TextSpec};
use elpis_protocol::scene3d::Scene3DSpec;
use elpis_protocol::style::ImageFit;

use elpis_host::{Sandbox, SurfaceInfo, UiBackend, UiEvent};
use elpis_protocol::style::{Align, Brush, Color, Display, FlexDirection, Justify};
use elpis_protocol::{Node, Patch};

use crate::lower::{lower, BlincContent, BlincDom, BlincStyle};

/// A boxed Blinc element, the dynamic element type the interpreter yields.
type Boxed = Box<dyn ElementBuilder>;

/// Shared state between the host-side backend and the Blinc build closure.
#[derive(Clone, Default)]
pub struct BlincShared {
    /// The latest lowered tree to draw.
    pub dom: Rc<RefCell<Option<BlincDom>>>,
    /// UI events queued by widget callbacks, drained by the host.
    pub events: Rc<RefCell<Vec<UiEvent>>>,
    /// Imperative backend commands the guest issued (router/media/etc.).
    pub commands: Rc<RefCell<Vec<(String, Vec<Value>)>>>,
    /// Whether the current tree wants per-frame ticks (an animated canvas /
    /// scene, or a node carrying animations). When set, the run loop drives the
    /// guest's `onTick` and keeps requesting frames so animation is continuous.
    pub animated: Rc<Cell<bool>>,
}

/// The [`UiBackend`] implementation that feeds Blinc.
pub struct BlincBackend {
    shared: BlincShared,
    surface: SurfaceInfo,
}

impl BlincBackend {
    /// Create a backend and the shared handle the run loop reads.
    pub fn new(surface: SurfaceInfo) -> (BlincBackend, BlincShared) {
        let shared = BlincShared::default();
        (BlincBackend { shared: shared.clone(), surface }, shared)
    }

    fn store(&mut self, tree: &Node) {
        self.shared.animated.set(tree_wants_animation(tree));
        *self.shared.dom.borrow_mut() = Some(lower(tree));
    }
}

/// Whether a node tree wants per-frame animation: an animated canvas/scene, or
/// any node carrying declarative animations.
fn tree_wants_animation(node: &Node) -> bool {
    let here = !node.animations.is_empty()
        || matches!(&node.kind, NodeKind::Canvas(c) if c.animated)
        || matches!(&node.kind, NodeKind::Scene3D(s) if s.animated);
    here || node.children.iter().any(tree_wants_animation)
}

impl UiBackend for BlincBackend {
    fn mount(&mut self, tree: &Node) {
        self.store(tree);
    }
    fn patch(&mut self, tree: &Node, _patches: &[Patch]) {
        // Blinc reconciles its own tree from the freshly-built closure output,
        // so we store the new full tree; the host's patch list still bounded
        // the VM<->host work and is available for transports that need it.
        self.store(tree);
    }
    fn surface_info(&self) -> SurfaceInfo {
        self.surface
    }
    fn drain_events(&mut self) -> Vec<UiEvent> {
        std::mem::take(&mut self.shared.events.borrow_mut())
    }
    fn command(&mut self, channel: &str, args: &[Value]) -> Value {
        self.shared.commands.borrow_mut().push((channel.to_string(), args.to_vec()));
        serde_json::json!({ "ok": true })
    }
}

// ---------------------------------------------------------------------------
// Color / style mapping onto Blinc.
// ---------------------------------------------------------------------------

fn bcolor(c: Color) -> BColor {
    BColor::rgba(c.r, c.g, c.b, c.a)
}

/// Apply the resolved [`BlincStyle`] to a `Div` via the Tailwind-like builder.
fn apply_style(mut d: Div, s: &BlincStyle) -> Div {
    // Layout mode + direction.
    match s.display {
        Display::Flex => {
            d = match s.direction {
                FlexDirection::Row => d.flex_row(),
                FlexDirection::Column => d.flex_col(),
                FlexDirection::RowReverse => d.flex_row_reverse(),
                FlexDirection::ColumnReverse => d.flex_col_reverse(),
            };
        }
        Display::Block => d = d.block(),
        Display::Grid => d = d.grid(),
        Display::Stack => {} // stacks are built via the stack() container
        Display::Hidden => d = d.hidden(),
    }
    if s.wrap {
        d = d.flex_wrap();
    }
    if let Some(a) = s.align_items {
        d = match a {
            Align::Start => d.items_start(),
            Align::Center => d.items_center(),
            Align::End => d.items_end(),
            Align::Stretch => d.items_stretch(),
            Align::Baseline => d.items_baseline(),
        };
    }
    if let Some(j) = s.justify {
        d = match j {
            Justify::Start => d.justify_start(),
            Justify::Center => d.justify_center(),
            Justify::End => d.justify_end(),
            Justify::Between => d.justify_between(),
            Justify::Around => d.justify_around(),
            Justify::Evenly => d.justify_evenly(),
        };
    }
    if let Some(g) = s.gap {
        d = d.gap(g);
    }
    if let Some(p) = s.padding {
        d.set_padding_x((p.left + p.right) / 2.0);
        d.set_padding_y((p.top + p.bottom) / 2.0);
    }
    if let Some(w) = s.width.and_then(length_px) {
        d.set_w(w);
    }
    if let Some(h) = s.height.and_then(length_px) {
        d.set_h(h);
    }
    if let Some(g) = s.flex_grow {
        d = d.flex_grow_value(g);
    }
    // Paint.
    if let Some(b) = &s.background {
        d.set_bg(bg_brush(b));
    }
    if let Some(r) = s.radius {
        d.set_rounded(r.tl.max(r.tr).max(r.br).max(r.bl));
    }
    if let Some((w, c)) = s.border {
        d.set_border(w, bcolor(c));
    }
    if let Some(o) = s.opacity {
        d.set_opacity(o);
    }
    for sh in &s.shadows {
        d.set_shadow(blinc_core_shadow(sh));
    }
    if s.overflow_y == Some(elpis_protocol::style::Overflow::Scroll) {
        d = d.overflow_y_scroll();
    }
    for class in &s.classes {
        d = d.class(class);
    }
    d
}

fn length_px(l: elpis_protocol::style::Length) -> Option<f32> {
    use elpis_protocol::style::Length::*;
    match l {
        Px(v) => Some(v),
        _ => None, // %/fr/vw/auto resolved by Blinc's layout engine, not a fixed px
    }
}

fn blinc_core_shadow(s: &elpis_protocol::style::Shadow) -> blinc_core::Shadow {
    blinc_core::Shadow {
        offset_x: s.offset[0],
        offset_y: s.offset[1],
        blur: s.blur,
        spread: s.spread,
        color: bcolor(s.color),
    }
}

// ---------------------------------------------------------------------------
// The interpreter: BlincDom -> boxed Blinc element.
// ---------------------------------------------------------------------------

/// Build a Blinc element for a lowered node, wiring its events into `shared`.
pub fn build(dom: &BlincDom, shared: &BlincShared) -> Boxed {
    match &dom.content {
        BlincContent::Text(t) => Box::new(text_el(t, text_color(dom))),
        BlincContent::RichText(rt) => {
            let mut row = div().flex_row().flex_wrap().items_baseline();
            for run in &rt.runs {
                let mut e = btext(&run.text).size(run.size.unwrap_or(14.0));
                if is_bold(run.weight) {
                    e = e.bold();
                }
                if run.italic {
                    e = e.italic();
                }
                if run.underline {
                    e = e.underline();
                }
                e = e.color(run.color.map(bcolor).unwrap_or_else(|| text_color(dom)));
                row = row.child(e);
            }
            Box::new(row)
        }
        BlincContent::Markdown(m) => Box::new(render_markdown(&m.source, dom)),
        BlincContent::Image(i) => {
            let mut e = image(i.src.clone()).fit(bfit(i.fit));
            if let Some(w) = dom.style.width.and_then(length_px) {
                e = e.w(w);
            }
            if let Some(h) = dom.style.height.and_then(length_px) {
                e = e.h(h);
            }
            if let Some(r) = dom.style.radius {
                e = e.rounded(r.tl.max(r.tr).max(r.br).max(r.bl));
            }
            Box::new(e)
        }
        BlincContent::Svg(s) => {
            let src = s.source.clone().or_else(|| s.src.clone()).unwrap_or_default();
            let mut e = svg(src);
            if let Some(c) = s.color {
                e = e.color(bcolor(c));
            }
            if let (Some(w), Some(h)) =
                (dom.style.width.and_then(length_px), dom.style.height.and_then(length_px))
            {
                e = e.size(w, h);
            }
            Box::new(e)
        }
        BlincContent::Icon(i) => {
            let col = i.color.map(bcolor).unwrap_or_else(|| text_color(dom));
            Box::new(
                div()
                    .w(i.size)
                    .h(i.size)
                    .items_center()
                    .justify_center()
                    .child(btext(short_icon(&i.name)).size(i.size * 0.7).color(col)),
            )
        }
        BlincContent::Button(b) => {
            let label = btext(&b.label).size(15.0).color(BColor::WHITE);
            let d = chip(&dom.style, BColor::rgba(0.22, 0.34, 0.55, 1.0)).child(label);
            Box::new(wire_events(d, dom, shared))
        }
        BlincContent::Input(i) => {
            let (shown, color) = if i.value.is_empty() {
                (i.placeholder.clone().unwrap_or_default(), BColor::rgba(0.55, 0.55, 0.62, 1.0))
            } else {
                (i.value.clone(), BColor::WHITE)
            };
            let d = field_box(&dom.style).child(btext(&shown).size(15.0).color(color));
            Box::new(wire_events(d, dom, shared))
        }
        BlincContent::Toggle { spec, .. } => {
            let on = spec.checked;
            let track_bg =
                if on { BColor::rgba(0.3, 0.7, 1.0, 1.0) } else { BColor::rgba(0.3, 0.3, 0.4, 1.0) };
            let knob = div().w(18.0).h(18.0).rounded(9.0).bg(BColor::WHITE);
            let mut track = div().w(40.0).h(22.0).rounded(11.0).bg(track_bg).items_center();
            track = if on { track.justify_end() } else { track.justify_start() };
            let mut row = div().flex_row().items_center().gap(10.0).child(track.child(knob));
            if let Some(label) = &spec.label {
                row = row.child(btext(label).size(15.0).color(text_color(dom)));
            }
            Box::new(wire_events(row, dom, shared))
        }
        BlincContent::Radio(r) => {
            let mut col = div().flex_col().gap(8.0);
            for opt in &r.options {
                let selected = r.selected.as_deref() == Some(opt.value.as_str());
                let mut outer = div().w(18.0).h(18.0).rounded(9.0).items_center().justify_center();
                outer.set_border(
                    2.0,
                    if selected { BColor::rgba(0.3, 0.7, 1.0, 1.0) } else { BColor::rgba(0.4, 0.4, 0.5, 1.0) },
                );
                let outer = if selected {
                    outer.child(div().w(8.0).h(8.0).rounded(4.0).bg(BColor::rgba(0.3, 0.7, 1.0, 1.0)))
                } else {
                    outer
                };
                col = col.child(
                    div()
                        .flex_row()
                        .items_center()
                        .gap(8.0)
                        .child(outer)
                        .child(btext(&opt.label).size(15.0).color(text_color(dom))),
                );
            }
            Box::new(wire_events(col, dom, shared))
        }
        BlincContent::Slider(s) => {
            let frac = if s.max > s.min { (s.value - s.min) / (s.max - s.min) } else { 0.0 };
            let fill = div().w(200.0 * frac.clamp(0.0, 1.0)).h(6.0).rounded(3.0).bg(BColor::rgba(0.3, 0.7, 1.0, 1.0));
            let track = div()
                .w(200.0)
                .h(16.0)
                .items_center()
                .child(div().w(200.0).h(6.0).rounded(3.0).bg(BColor::rgba(0.3, 0.3, 0.4, 1.0)).child(fill));
            Box::new(wire_events(div().flex_col().gap(4.0).child(track), dom, shared))
        }
        BlincContent::Dropdown(dd) => {
            let label = dd
                .selected
                .as_ref()
                .and_then(|sel| dd.options.iter().find(|o| &o.value == sel))
                .map(|o| o.label.clone())
                .or_else(|| dd.placeholder.clone())
                .unwrap_or_else(|| "Select…".to_string());
            let field = field_box(&dom.style)
                .flex_row()
                .items_center()
                .justify_between()
                .gap(10.0)
                .child(btext(&label).size(15.0).color(BColor::WHITE))
                .child(btext("▾").size(14.0).color(BColor::rgba(0.6, 0.6, 0.7, 1.0)));
            Box::new(wire_events(field, dom, shared))
        }
        BlincContent::Tabs(t) => {
            let mut row = div().flex_row().gap(6.0);
            for (i, tab) in t.tabs.iter().enumerate() {
                let sel = i as u32 == t.selected;
                let mut chip_ = div().items_center().justify_center();
                chip_.set_padding_x(14.0);
                chip_.set_padding_y(6.0);
                chip_.set_rounded(8.0);
                chip_.set_bg(if sel {
                    BColor::rgba(0.3, 0.7, 1.0, 1.0)
                } else {
                    BColor::rgba(0.18, 0.19, 0.25, 1.0)
                });
                row = row.child(chip_.child(btext(&tab.label).size(14.0).color(BColor::WHITE)));
            }
            Box::new(wire_events(row, dom, shared))
        }
        BlincContent::Carousel(c) => {
            let mut col = div().flex_col().items_center().gap(8.0);
            if !dom.children.is_empty() {
                let idx = (c.index as usize).min(dom.children.len() - 1);
                col = col.child_box(build(&dom.children[idx], shared));
                if c.indicators {
                    let mut dots = div().flex_row().gap(6.0);
                    for i in 0..dom.children.len() {
                        let on = i == idx;
                        dots = dots.child(div().w(8.0).h(8.0).rounded(4.0).bg(if on {
                            BColor::WHITE
                        } else {
                            BColor::rgba(0.4, 0.4, 0.5, 1.0)
                        }));
                    }
                    col = col.child(dots);
                }
            }
            Box::new(wire_events(col, dom, shared))
        }
        BlincContent::Progress(p) => Box::new(build_progress(p)),
        BlincContent::Spinner(s) => Box::new(build_spinner(s)),
        BlincContent::Canvas(c) => Box::new(build_canvas(c, &dom.style)),
        BlincContent::Scene3D(s) => Box::new(build_scene3d(s, &dom.style)),
        BlincContent::Media { spec, media_kind } => {
            let glyph = if *media_kind == "audio" { "♪" } else { "▶" };
            let mut card =
                apply_style(div(), &dom.style).flex_col().items_center().justify_center().gap(8.0);
            card.set_bg(BColor::rgba(0.08, 0.09, 0.13, 1.0));
            card.set_rounded(10.0);
            Box::new(
                card.child(btext(glyph).size(36.0).color(BColor::WHITE))
                    .child(btext(&spec.src).size(12.0).color(BColor::rgba(0.6, 0.6, 0.7, 1.0))),
            )
        }
        BlincContent::Overlay(o) => {
            let mut layer = apply_style(div(), &dom.style);
            if o.backdrop {
                layer.set_bg(BColor::rgba(0.0, 0.0, 0.0, 0.5));
            }
            for child in &dom.children {
                layer = layer.child_box(build(child, shared));
            }
            Box::new(wire_events(layer, dom, shared))
        }
        // Containers (div/row/column/stack/grid/scroll/spacer) + Component.
        _ => {
            let mut d = apply_style(div(), &dom.style);
            for child in &dom.children {
                d = d.child_box(build(child, shared));
            }
            Box::new(wire_events(d, dom, shared))
        }
    }
}

fn text_color(dom: &BlincDom) -> BColor {
    dom.style.foreground.map(bcolor).unwrap_or(BColor::rgba(0.85, 0.87, 0.95, 1.0))
}

/// A pill-shaped chip honoring the node's style, with a fallback background.
fn chip(style: &BlincStyle, fallback_bg: BColor) -> Div {
    let mut d = apply_style(div(), style).items_center().justify_center();
    if style.background.is_none() {
        d.set_bg(fallback_bg);
    }
    if style.radius.is_none() {
        d.set_rounded(8.0);
    }
    d.set_padding_x(14.0);
    d.set_padding_y(8.0);
    d
}

/// A bordered input/field box.
fn field_box(style: &BlincStyle) -> Div {
    let mut d = apply_style(div(), style).items_center();
    d.set_bg(BColor::rgba(0.16, 0.17, 0.22, 1.0));
    d.set_rounded(8.0);
    d.set_border(1.0, BColor::rgba(0.3, 0.32, 0.4, 1.0));
    d.set_padding_x(12.0);
    d.set_padding_y(10.0);
    d
}

/// Build a fully-styled Blinc text element from a [`TextSpec`].
fn text_el(t: &TextSpec, color: BColor) -> Text {
    let mut e = btext(&t.text).size(t.size).color(color).align(balign(t.align));
    if is_bold(t.weight) {
        e = e.bold();
    }
    if t.italic {
        e = e.italic();
    }
    if t.underline {
        e = e.underline();
    }
    if t.strikethrough {
        e = e.strikethrough();
    }
    if let Some(f) = &t.font {
        e = e.font(f.clone());
    }
    if let Some(lh) = t.line_height {
        e = e.line_height(lh);
    }
    if let Some(ls) = t.letter_spacing {
        e = e.letter_spacing(ls);
    }
    e
}

/// Whether a protocol weight should render bold (the Blinc text element exposes
/// a `bold()` toggle; its numeric `FontWeight` enum is a distinct type from the
/// `blinc_core` one used by the canvas `TextStyle`).
fn is_bold(w: FontWeight) -> bool {
    matches!(w, FontWeight::Semibold | FontWeight::Bold | FontWeight::Black)
}

fn balign(a: TextAlign) -> LTextAlign {
    match a {
        TextAlign::Start | TextAlign::Justify => LTextAlign::Left,
        TextAlign::Center => LTextAlign::Center,
        TextAlign::End => LTextAlign::Right,
    }
}

fn bfit(f: ImageFit) -> ObjectFit {
    match f {
        ImageFit::Cover => ObjectFit::Cover,
        ImageFit::Contain => ObjectFit::Contain,
        ImageFit::Fill => ObjectFit::Fill,
        ImageFit::ScaleDown => ObjectFit::ScaleDown,
        ImageFit::None => ObjectFit::Contain,
    }
}

/// A short glyph for an icon name (first character) until icon-set lookup is
/// wired through `blinc_tabler_icons`.
fn short_icon(name: &str) -> String {
    name.chars().next().map(|c| c.to_uppercase().to_string()).unwrap_or_else(|| "•".to_string())
}

/// Minimal markdown: a column where `#`-prefixed lines become headings, `-`
/// lines become bullets, and the rest is body text.
fn render_markdown(source: &str, dom: &BlincDom) -> Div {
    let mut col = div().flex_col().gap(6.0);
    let fg = text_color(dom);
    for line in source.lines() {
        let trimmed = line.trim_start();
        let el = if let Some(rest) = trimmed.strip_prefix("# ") {
            btext(rest).size(24.0).bold().color(fg)
        } else if let Some(rest) = trimmed.strip_prefix("## ") {
            btext(rest).size(19.0).bold().color(fg)
        } else if let Some(rest) = trimmed.strip_prefix("- ") {
            btext(&format!("•  {rest}")).size(14.0).color(fg)
        } else if trimmed.is_empty() {
            continue;
        } else {
            btext(strip_inline_md(trimmed)).size(14.0).color(fg)
        };
        col = col.child(el);
    }
    col
}

/// Drop the most common inline markdown markers (`**`, `*`, `` ` ``) so the
/// body text reads cleanly even though spans aren't individually styled.
fn strip_inline_md(s: &str) -> String {
    s.replace("**", "").replace('`', "")
}

fn build_progress(p: &ProgressSpec) -> Div {
    let frac = p.value.unwrap_or(0.3).clamp(0.0, 1.0);
    let circular = p.shape.as_deref() == Some("circular");
    if circular {
        // A ring drawn on a small canvas.
        let c = canvas(move |ctx: &mut dyn DrawContext, b: CanvasBounds| {
            let cx = b.width * 0.5;
            let cy = b.height * 0.5;
            let r = cx.min(cy) - 4.0;
            ctx.stroke_circle(
                BPoint::new(cx, cy),
                r,
                &BStroke::new(4.0),
                BBrush::Solid(BColor::rgba(0.25, 0.25, 0.32, 1.0)),
            );
            let mut path = BPath::new();
            let n = 48;
            for i in 0..=n {
                let t = -std::f32::consts::FRAC_PI_2
                    + std::f32::consts::TAU * frac * (i as f32 / n as f32);
                let (x, y) = (cx + r * t.cos(), cy + r * t.sin());
                path = if i == 0 { path.move_to(x, y) } else { path.line_to(x, y) };
            }
            ctx.stroke_path(
                &path,
                &BStroke::new(4.0).with_cap(BLineCap::Round),
                BBrush::Solid(BColor::rgba(0.4, 0.8, 1.0, 1.0)),
            );
        })
        .w(44.0)
        .h(44.0);
        div().child(c)
    } else {
        let fill = div().w(200.0 * frac).h(8.0).rounded(4.0).bg(BColor::rgba(0.4, 0.8, 1.0, 1.0));
        div().w(200.0).h(8.0).rounded(4.0).bg(BColor::rgba(0.25, 0.25, 0.32, 1.0)).child(fill)
    }
}

fn build_spinner(s: &SpinnerSpec) -> Canvas {
    let size = s.size;
    let col = s.color.map(bcolor).unwrap_or(BColor::rgba(0.4, 0.8, 1.0, 1.0));
    canvas(move |ctx: &mut dyn DrawContext, b: CanvasBounds| {
        let cx = b.width * 0.5;
        let cy = b.height * 0.5;
        let r = cx.min(cy) - 3.0;
        ctx.stroke_circle(
            BPoint::new(cx, cy),
            r,
            &BStroke::new(3.0),
            BBrush::Solid(BColor::rgba(0.3, 0.3, 0.4, 1.0)),
        );
        // A bright three-quarter arc as the "active" sweep.
        let mut path = BPath::new();
        let n = 36;
        for i in 0..=n {
            let t = std::f32::consts::TAU * 0.75 * (i as f32 / n as f32);
            let (x, y) = (cx + r * t.cos(), cy + r * t.sin());
            path = if i == 0 { path.move_to(x, y) } else { path.line_to(x, y) };
        }
        ctx.stroke_path(&path, &BStroke::new(3.0).with_cap(BLineCap::Round), BBrush::Solid(col));
    })
    .w(size)
    .h(size)
}

fn build_scene3d(spec: &Scene3DSpec, style: &BlincStyle) -> Canvas {
    let spec = spec.clone();
    let mut c = canvas(move |ctx: &mut dyn DrawContext, bounds: CanvasBounds| {
        crate::scene::render(ctx, bounds.width, bounds.height, &spec);
    });
    if let Some(w) = style.width.and_then(length_px) {
        c = c.w(w);
    }
    if let Some(h) = style.height.and_then(length_px) {
        c = c.h(h);
    }
    c
}

// ---------------------------------------------------------------------------
// 2D canvas: replay the protocol DrawOp list into a Blinc DrawContext.
// ---------------------------------------------------------------------------

fn build_canvas(spec: &CanvasSpec, style: &BlincStyle) -> Canvas {
    let ops = spec.ops.clone();
    let mut c = canvas(move |ctx: &mut dyn DrawContext, bounds: CanvasBounds| {
        replay(ctx, bounds, &ops);
    });
    if let Some(w) = style.width.and_then(length_px) {
        c = c.w(w);
    }
    if let Some(h) = style.height.and_then(length_px) {
        c = c.h(h);
    }
    c
}

fn bstops(stops: &[elpis_protocol::style::GradientStop]) -> Vec<BStop> {
    stops.iter().map(|s| BStop::new(s.offset, bcolor(s.color))).collect()
}

/// A brush for an element **background**, using the bounding-box (0..1) space so
/// the gradient scales with the element regardless of size.
fn bg_brush(b: &Brush) -> BBrush {
    match b {
        Brush::Solid { color } => BBrush::Solid(bcolor(*color)),
        Brush::LinearGradient { angle, stops } => {
            let rad = angle.to_radians();
            let (dx, dy) = (rad.cos() * 0.5, rad.sin() * 0.5);
            BBrush::Gradient(Gradient::Linear {
                start: BPoint::new(0.5 - dx, 0.5 - dy),
                end: BPoint::new(0.5 + dx, 0.5 + dy),
                stops: bstops(stops),
                space: GradientSpace::ObjectBoundingBox,
                spread: GradientSpread::Pad,
            })
        }
        Brush::RadialGradient { center, radius, stops } => BBrush::Gradient(Gradient::Radial {
            center: BPoint::new(center[0], center[1]),
            radius: *radius,
            focal: None,
            stops: bstops(stops),
            space: GradientSpace::ObjectBoundingBox,
            spread: GradientSpread::Pad,
        }),
        Brush::ConicGradient { center, start_angle, stops } => BBrush::Gradient(Gradient::Conic {
            center: BPoint::new(center[0], center[1]),
            start_angle: start_angle.to_radians(),
            stops: bstops(stops),
            space: GradientSpace::ObjectBoundingBox,
        }),
        Brush::Image { .. } => BBrush::Solid(BColor::WHITE),
    }
}

/// A brush for a **canvas** fill/stroke, in user space spanning the shape's
/// bounding box `(x, y, w, h)`.
fn canvas_brush(b: &Brush, x: f32, y: f32, w: f32, h: f32) -> BBrush {
    match b {
        Brush::Solid { color } => BBrush::Solid(bcolor(*color)),
        Brush::LinearGradient { angle, stops } => {
            let rad = angle.to_radians();
            let (cx, cy) = (x + w * 0.5, y + h * 0.5);
            let (dx, dy) = (rad.cos() * w * 0.5, rad.sin() * h * 0.5);
            BBrush::Gradient(Gradient::Linear {
                start: BPoint::new(cx - dx, cy - dy),
                end: BPoint::new(cx + dx, cy + dy),
                stops: bstops(stops),
                space: GradientSpace::UserSpace,
                spread: GradientSpread::Pad,
            })
        }
        Brush::RadialGradient { center, radius, stops } => BBrush::Gradient(Gradient::Radial {
            center: BPoint::new(x + center[0] * w, y + center[1] * h),
            radius: radius * w.max(h),
            focal: None,
            stops: bstops(stops),
            space: GradientSpace::UserSpace,
            spread: GradientSpread::Pad,
        }),
        Brush::ConicGradient { center, start_angle, stops } => BBrush::Gradient(Gradient::Conic {
            center: BPoint::new(x + center[0] * w, y + center[1] * h),
            start_angle: start_angle.to_radians(),
            stops: bstops(stops),
            space: GradientSpace::UserSpace,
        }),
        Brush::Image { .. } => BBrush::Solid(BColor::WHITE),
    }
}

fn rect_brush(b: &Brush, r: &PRect) -> BBrush {
    canvas_brush(b, r.x, r.y, r.w, r.h)
}

fn points_bbox(pts: &[PPoint]) -> (f32, f32, f32, f32) {
    let (mut minx, mut miny, mut maxx, mut maxy) = (f32::MAX, f32::MAX, f32::MIN, f32::MIN);
    for p in pts {
        minx = minx.min(p.x);
        miny = miny.min(p.y);
        maxx = maxx.max(p.x);
        maxy = maxy.max(p.y);
    }
    if pts.is_empty() {
        (0.0, 0.0, 0.0, 0.0)
    } else {
        (minx, miny, maxx - minx, maxy - miny)
    }
}

fn seg_bbox(segs: &[PathSeg]) -> (f32, f32, f32, f32) {
    let mut pts = Vec::new();
    for s in segs {
        match *s {
            PathSeg::MoveTo { x, y }
            | PathSeg::LineTo { x, y }
            | PathSeg::QuadTo { x, y, .. }
            | PathSeg::CubicTo { x, y, .. }
            | PathSeg::ArcTo { x, y, .. } => pts.push(PPoint { x, y }),
            PathSeg::Close => {}
        }
    }
    points_bbox(&pts)
}

fn brect(r: &PRect) -> BRect {
    BRect::new(r.x, r.y, r.w, r.h)
}

fn bpoint(p: &PPoint) -> BPoint {
    BPoint::new(p.x, p.y)
}

fn bstroke(s: &PStroke) -> BStroke {
    let cap = match s.cap {
        LineCap::Butt => BLineCap::Butt,
        LineCap::Round => BLineCap::Round,
        LineCap::Square => BLineCap::Square,
    };
    let join = match s.join {
        LineJoin::Miter => BLineJoin::Miter,
        LineJoin::Round => BLineJoin::Round,
        LineJoin::Bevel => BLineJoin::Bevel,
    };
    let mut st = BStroke::new(s.width).with_cap(cap).with_join(join);
    if !s.dash.is_empty() {
        st = st.with_dash(s.dash.clone(), 0.0);
    }
    st
}

fn bpath(segments: &[PathSeg]) -> BPath {
    let mut p = BPath::new();
    for seg in segments {
        p = match *seg {
            PathSeg::MoveTo { x, y } => p.move_to(x, y),
            PathSeg::LineTo { x, y } => p.line_to(x, y),
            PathSeg::QuadTo { cx, cy, x, y } => p.quad_to(cx, cy, x, y),
            PathSeg::CubicTo { c1x, c1y, c2x, c2y, x, y } => p.cubic_to(c1x, c1y, c2x, c2y, x, y),
            // Approximate elliptical arcs with a line to the endpoint for now.
            PathSeg::ArcTo { x, y, .. } => p.line_to(x, y),
            PathSeg::Close => p.close(),
        };
    }
    p
}

fn replay(ctx: &mut dyn DrawContext, bounds: CanvasBounds, ops: &[DrawOp]) {
    // Per-`Save` nesting: counts of [transforms, clips, opacities] pushed since
    // the matching Save, popped on Restore. The base frame (index 0) catches
    // pushes made without a Save and is flushed at the end so nothing leaks.
    let mut frames: Vec<[u32; 3]> = vec![[0, 0, 0]];
    let bump = |frames: &mut Vec<[u32; 3]>, idx: usize| {
        if let Some(f) = frames.last_mut() {
            f[idx] += 1;
        }
    };

    for op in ops {
        match op {
            DrawOp::Clear { color } => ctx.fill_rect(
                BRect::new(0.0, 0.0, bounds.width, bounds.height),
                BCorner::ZERO,
                BBrush::Solid(bcolor(*color)),
            ),
            DrawOp::FillRect { rect, brush } => {
                ctx.fill_rect(brect(rect), BCorner::ZERO, rect_brush(brush, rect));
            }
            DrawOp::StrokeRect { rect, stroke } => {
                ctx.stroke_path(&BPath::rect(brect(rect)), &bstroke(stroke), rect_brush(&stroke.brush, rect));
            }
            DrawOp::FillRoundRect { rect, radius, brush } => {
                ctx.fill_rect(
                    brect(rect),
                    BCorner::new(*radius, *radius, *radius, *radius),
                    rect_brush(brush, rect),
                );
            }
            DrawOp::StrokeRoundRect { rect, stroke, .. } => {
                ctx.stroke_path(&BPath::rect(brect(rect)), &bstroke(stroke), rect_brush(&stroke.brush, rect));
            }
            DrawOp::FillCircle { center, radius, brush } => {
                let b = canvas_brush(brush, center.x - radius, center.y - radius, radius * 2.0, radius * 2.0);
                ctx.fill_circle(bpoint(center), *radius, b);
            }
            DrawOp::StrokeCircle { center, radius, stroke } => {
                let b = canvas_brush(&stroke.brush, center.x - radius, center.y - radius, radius * 2.0, radius * 2.0);
                ctx.stroke_circle(bpoint(center), *radius, &bstroke(stroke), b);
            }
            DrawOp::FillEllipse { center, rx, ry, brush } => {
                let b = canvas_brush(brush, center.x - rx, center.y - ry, rx * 2.0, ry * 2.0);
                // No native ellipse primitive: scale a unit circle path.
                let mut p = BPath::new();
                let n = 48;
                for i in 0..=n {
                    let t = std::f32::consts::TAU * (i as f32 / n as f32);
                    let (x, y) = (center.x + rx * t.cos(), center.y + ry * t.sin());
                    p = if i == 0 { p.move_to(x, y) } else { p.line_to(x, y) };
                }
                ctx.fill_path(&p.close(), b);
            }
            DrawOp::Line { from, to, stroke } => {
                let (x, y, w, h) = points_bbox(&[*from, *to]);
                ctx.stroke_path(&BPath::line(bpoint(from), bpoint(to)), &bstroke(stroke), canvas_brush(&stroke.brush, x, y, w, h));
            }
            DrawOp::Polyline { points, stroke } => {
                if let Some(first) = points.first() {
                    let mut p = BPath::new().move_to(first.x, first.y);
                    for pt in &points[1..] {
                        p = p.line_to(pt.x, pt.y);
                    }
                    let (x, y, w, h) = points_bbox(points);
                    ctx.stroke_path(&p, &bstroke(stroke), canvas_brush(&stroke.brush, x, y, w, h));
                }
            }
            DrawOp::Polygon { points, brush } => {
                if let Some(first) = points.first() {
                    let mut p = BPath::new().move_to(first.x, first.y);
                    for pt in &points[1..] {
                        p = p.line_to(pt.x, pt.y);
                    }
                    let (x, y, w, h) = points_bbox(points);
                    ctx.fill_path(&p.close(), canvas_brush(brush, x, y, w, h));
                }
            }
            DrawOp::FillPath { segments, brush } => {
                let (x, y, w, h) = seg_bbox(segments);
                ctx.fill_path(&bpath(segments), canvas_brush(brush, x, y, w, h));
            }
            DrawOp::StrokePath { segments, stroke } => {
                let (x, y, w, h) = seg_bbox(segments);
                ctx.stroke_path(&bpath(segments), &bstroke(stroke), canvas_brush(&stroke.brush, x, y, w, h));
            }
            DrawOp::Arc { center, radius, start_angle, end_angle, brush, stroke } => {
                let n = 48;
                let (bx, by, bw, bh) = (center.x - radius, center.y - radius, radius * 2.0, radius * 2.0);
                let arc_pts = |path: BPath, with_center: bool| {
                    let mut p = path;
                    if with_center {
                        p = p.move_to(center.x, center.y);
                    }
                    for i in 0..=n {
                        let t = start_angle + (end_angle - start_angle) * (i as f32 / n as f32);
                        let (x, y) = (center.x + radius * t.cos(), center.y + radius * t.sin());
                        p = if i == 0 && !with_center { p.move_to(x, y) } else { p.line_to(x, y) };
                    }
                    p
                };
                if let Some(b) = brush {
                    ctx.fill_path(&arc_pts(BPath::new(), true).close(), canvas_brush(b, bx, by, bw, bh));
                }
                if let Some(s) = stroke {
                    ctx.stroke_path(&arc_pts(BPath::new(), false), &bstroke(s), canvas_brush(&s.brush, bx, by, bw, bh));
                }
            }
            DrawOp::Text { text, at, size, color, .. } => {
                ctx.draw_text(text, bpoint(at), &BTextStyle::new(*size).with_color(bcolor(*color)));
            }
            DrawOp::Image { .. } => { /* needs an asset loader; tracked separately */ }
            DrawOp::Save => frames.push([0, 0, 0]),
            DrawOp::Restore => {
                if frames.len() > 1 {
                    let f = frames.pop().unwrap();
                    for _ in 0..f[0] {
                        ctx.pop_transform();
                    }
                    for _ in 0..f[1] {
                        ctx.pop_clip();
                    }
                    for _ in 0..f[2] {
                        ctx.pop_opacity();
                    }
                }
            }
            DrawOp::Translate { x, y } => {
                ctx.push_transform(BTransform::translate(*x, *y));
                bump(&mut frames, 0);
            }
            DrawOp::Scale { x, y } => {
                ctx.push_transform(BTransform::scale(*x, *y));
                bump(&mut frames, 0);
            }
            DrawOp::Rotate { degrees } => {
                ctx.push_transform(BTransform::rotate(degrees.to_radians()));
                bump(&mut frames, 0);
            }
            DrawOp::ClipRect { rect } => {
                ctx.push_clip(ClipShape::rect(brect(rect)));
                bump(&mut frames, 1);
            }
            DrawOp::ClipPath { segments } => {
                ctx.push_clip(ClipShape::Path(bpath(segments)));
                bump(&mut frames, 1);
            }
            DrawOp::GlobalAlpha { alpha } => {
                ctx.push_opacity(*alpha);
                bump(&mut frames, 2);
            }
        }
    }

    // Flush anything still pushed (including the base frame) so canvas state
    // never leaks between frames.
    while let Some(f) = frames.pop() {
        for _ in 0..f[0] {
            ctx.pop_transform();
        }
        for _ in 0..f[1] {
            ctx.pop_clip();
        }
        for _ in 0..f[2] {
            ctx.pop_opacity();
        }
    }
}

/// Mark the tree dirty **and wake the platform run loop** so it actually
/// renders a frame.
///
/// `request_full_rebuild()` only sets flags (NEEDS_REBUILD / NEEDS_RELAYOUT /
/// NEEDS_REDRAW). On desktop the winit loop sits in `ControlFlow::Wait`, and on
/// mobile/web the loop parks between frames — a flag alone does **not** wake
/// them, so the queued work (our `onEvent`) only runs on the next OS event
/// (resize / app restore). `AnimationScheduler::request_redraw()` fires the
/// scheduler's wake callback, which — per Blinc's own docs — "is the only thing
/// that gets the main thread out of `ControlFlow::Wait`"; it's the same path
/// Blinc's video / background-thread repaints use. The next frame then consumes
/// `take_needs_rebuild()`, re-runs the build closure, drains the event, and
/// paints the new tree.
fn schedule_rebuild() {
    blinc_layout::widgets::request_full_rebuild();
    // `try_get_scheduler` avoids a panic if called before the platform has
    // installed the global scheduler.
    if let Some(scheduler) = blinc_animation::try_get_scheduler() {
        scheduler.request_redraw();
    }
}

/// Attach event handlers: each binding pushes a [`UiEvent`] into the shared
/// queue so the host delivers it to the guest's `onEvent`.
fn wire_events(mut d: Div, dom: &BlincDom, shared: &BlincShared) -> Div {
    if let Some(handler) = dom.events.get("click") {
        let handler = handler.clone();
        let events = shared.events.clone();
        d = d.on_click(move |_| {
            events.borrow_mut().push(UiEvent::click(handler.clone()));
            schedule_rebuild();
        });
    }
    d
}

// ---------------------------------------------------------------------------
// The shared frame closure (used by every platform run loop).
// ---------------------------------------------------------------------------

/// Build the per-frame UI closure that every Blinc run loop drives.
///
/// On each frame it (1) delivers queued UI events to the guest — which
/// re-renders into the backend, updating the shared `BlincDom` — and (2)
/// constructs the current Blinc element tree. The returned closure has the
/// exact `FnMut(&mut WindowedContext) -> impl ElementBuilder` shape that
/// `WindowedApp::run`, `WebApp::run`, and `AndroidApp::run` all expect.
pub fn frame_closure(
    sandbox: Sandbox,
    shared: BlincShared,
) -> impl FnMut(&mut WindowedContext) -> Div + 'static {
    let sandbox = Rc::new(RefCell::new(sandbox));
    move |ctx: &mut WindowedContext| {
        // 1. Deliver queued UI events to the guest (updates the retained dom),
        //    then — if the resulting tree is animated — drive one tick.
        {
            let pending: Vec<UiEvent> = std::mem::take(&mut shared.events.borrow_mut());
            let mut sb = sandbox.borrow_mut();
            for ev in pending {
                let _ = sb.dispatch_event(&ev);
            }
            if shared.animated.get() {
                let _ = sb.tick(16.0);
            }
        }

        // 2. Read the animation state AFTER dispatch/tick (the dom may have just
        //    switched to/from an animated tree). Animated content re-arms a
        //    rebuild + wakes the next frame each time, so animation runs
        //    continuously at vsync; idle content drops back to on-demand (events
        //    wake a frame through `schedule_rebuild`), so a static UI costs
        //    nothing.
        if shared.animated.get() {
            schedule_rebuild();
        }

        let root = shared.dom.borrow();
        let inner: Boxed = match root.as_ref() {
            Some(dom) => build(dom, &shared),
            None => Box::new(div()),
        };
        div()
            .w(ctx.width)
            .h(ctx.height)
            .bg(BColor::rgba(0.05, 0.05, 0.07, 1.0))
            .child_box(inner)
    }
}

// ---------------------------------------------------------------------------
// Desktop convenience run loop.
// ---------------------------------------------------------------------------

/// Open a Blinc desktop window and drive `sandbox` (already booted with a
/// [`BlincBackend`] whose `shared` handle is passed here) frame by frame.
#[cfg(feature = "desktop")]
pub fn run_windowed(
    title: &str,
    width: u32,
    height: u32,
    sandbox: Sandbox,
    shared: BlincShared,
) -> Result<(), String> {
    use blinc_app::windowed::WindowedApp;
    use blinc_app::WindowConfig;

    let config = WindowConfig {
        title: title.to_string(),
        width,
        height,
        resizable: true,
        ..Default::default()
    };
    WindowedApp::run(config, frame_closure(sandbox, shared)).map_err(|e| format!("blinc run failed: {e:?}"))
}
