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

use std::cell::RefCell;
use std::rc::Rc;

use serde_json::Value;

use blinc_app::windowed::WindowedContext;
use blinc_core::{
    Color as BColor, CornerRadius as BCorner, DrawContext, LineCap as BLineCap,
    LineJoin as BLineJoin, Path as BPath, Point as BPoint, Rect as BRect, Stroke as BStroke,
    TextStyle as BTextStyle, Transform as BTransform,
};
use blinc_layout::canvas::{canvas, Canvas, CanvasBounds};
use blinc_layout::div::{div, Div};
use blinc_layout::prelude::*;
use blinc_layout::text::text as btext;

use elpis_protocol::canvas::{CanvasSpec, DrawOp, LineCap, LineJoin, PathSeg, Point as PPoint, Rect as PRect, Stroke as PStroke};

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
        *self.shared.dom.borrow_mut() = Some(lower(tree));
    }
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

/// Resolve a brush to a representative Blinc color (gradients fall back to their
/// first stop until the gradient builder path is wired).
fn brush_color(b: &Brush) -> BColor {
    match b {
        Brush::Solid { color } => bcolor(*color),
        Brush::LinearGradient { stops, .. }
        | Brush::RadialGradient { stops, .. }
        | Brush::ConicGradient { stops, .. } => {
            stops.first().map(|s| bcolor(s.color)).unwrap_or(BColor::WHITE)
        }
        Brush::Image { .. } => BColor::WHITE,
    }
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
        d.set_bg(brush_color(b));
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
        BlincContent::Text(t) => {
            let mut e = btext(&t.text).size(t.size);
            if let Some(c) = dom.style.foreground {
                e = e.color(bcolor(c));
            }
            Box::new(e)
        }
        BlincContent::Markdown(m) => {
            // Minimal markdown: render the raw source as body text. (Full
            // markdown goes through blinc_layout's markdown element later.)
            Box::new(btext(&m.source).size(14.0).color(text_color(dom)))
        }
        BlincContent::Button(b) => {
            let label = btext(&b.label).size(15.0).color(BColor::WHITE);
            let mut d = chip(&dom.style, BColor::rgba(0.22, 0.34, 0.55, 1.0)).child(label);
            d = wire_events(d, dom, shared);
            Box::new(d)
        }
        BlincContent::Input(i) => {
            let shown = if i.value.is_empty() {
                i.placeholder.clone().unwrap_or_default()
            } else {
                i.value.clone()
            };
            let color = if i.value.is_empty() {
                BColor::rgba(0.55, 0.55, 0.62, 1.0)
            } else {
                BColor::WHITE
            };
            let mut d = field_box(&dom.style).child(btext(&shown).size(15.0).color(color));
            d = wire_events(d, dom, shared);
            Box::new(d)
        }
        BlincContent::Toggle { spec, toggle_kind } => {
            let on = spec.checked;
            let knob = chip_fixed(
                40.0,
                22.0,
                if on { BColor::rgba(0.3, 0.7, 1.0, 1.0) } else { BColor::rgba(0.3, 0.3, 0.4, 1.0) },
            );
            let mut row = div().flex_row().items_center().gap(10.0).child(knob);
            if let Some(label) = &spec.label {
                row = row.child(btext(label).size(15.0).color(text_color(dom)));
            }
            let _ = toggle_kind;
            Box::new(wire_events(row, dom, shared))
        }
        BlincContent::Slider(s) => {
            let frac = if s.max > s.min { (s.value - s.min) / (s.max - s.min) } else { 0.0 };
            let track = div()
                .w(200.0)
                .h(6.0)
                .rounded(3.0)
                .bg(BColor::rgba(0.3, 0.3, 0.4, 1.0))
                .child(div().w(200.0 * frac.clamp(0.0, 1.0)).h(6.0).rounded(3.0).bg(BColor::rgba(0.3, 0.7, 1.0, 1.0)));
            Box::new(wire_events(div().flex_col().gap(4.0).child(track), dom, shared))
        }
        BlincContent::Progress(p) => {
            let frac = p.value.unwrap_or(0.3).clamp(0.0, 1.0);
            let bar = div()
                .w(200.0)
                .h(8.0)
                .rounded(4.0)
                .bg(BColor::rgba(0.25, 0.25, 0.32, 1.0))
                .child(div().w(200.0 * frac).h(8.0).rounded(4.0).bg(BColor::rgba(0.4, 0.8, 1.0, 1.0)));
            Box::new(bar)
        }
        BlincContent::Dropdown(d) => {
            let label = d
                .selected
                .as_ref()
                .and_then(|sel| d.options.iter().find(|o| &o.value == sel))
                .map(|o| o.label.clone())
                .or_else(|| d.placeholder.clone())
                .unwrap_or_else(|| "Select…".to_string());
            let mut e = field_box(&dom.style).child(btext(&label).size(15.0).color(BColor::WHITE));
            e = wire_events(e, dom, shared);
            Box::new(e)
        }
        BlincContent::Canvas(c) => Box::new(build_canvas(c, &dom.style)),
        BlincContent::Scene3D(s) => {
            // A real 3D viewport drives DrawContext's camera/mesh API; here we
            // show a labeled placeholder card so the scene tab isn't blank.
            let label = format!("3D scene · {} entities · {} lights", s.entities.len(), s.lights.len());
            let card = apply_style(div(), &dom.style)
                .items_center()
                .justify_center()
                .bg(BColor::rgba(0.08, 0.10, 0.16, 1.0))
                .child(btext(&label).size(16.0).color(BColor::rgba(0.6, 0.8, 1.0, 1.0)));
            Box::new(card)
        }
        // Containers and the remaining families build a styled div with children.
        _ => {
            let mut d = apply_style(div(), &dom.style);
            for child in &dom.children {
                d = d.child_box(build(child, shared));
            }
            d = wire_events(d, dom, shared);
            Box::new(d)
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

fn chip_fixed(w: f32, h: f32, bg: BColor) -> Div {
    div().w(w).h(h).rounded(h / 2.0).bg(bg)
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

fn bbrush(b: &Brush) -> blinc_core::Brush {
    blinc_core::Brush::Solid(brush_color(b))
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
    for op in ops {
        match op {
            DrawOp::Clear { color } => {
                ctx.fill_rect(
                    BRect::new(0.0, 0.0, bounds.width, bounds.height),
                    BCorner::ZERO,
                    blinc_core::Brush::Solid(bcolor(*color)),
                );
            }
            DrawOp::FillRect { rect, brush } => {
                ctx.fill_rect(brect(rect), BCorner::ZERO, bbrush(brush));
            }
            DrawOp::StrokeRect { rect, stroke } => {
                ctx.stroke_path(&BPath::rect(brect(rect)), &bstroke(stroke), bbrush(&stroke.brush));
            }
            DrawOp::FillRoundRect { rect, radius, brush } => {
                ctx.fill_rect(brect(rect), BCorner::new(*radius, *radius, *radius, *radius), bbrush(brush));
            }
            DrawOp::StrokeRoundRect { rect, stroke, .. } => {
                ctx.stroke_path(&BPath::rect(brect(rect)), &bstroke(stroke), bbrush(&stroke.brush));
            }
            DrawOp::FillCircle { center, radius, brush } => {
                ctx.fill_circle(bpoint(center), *radius, bbrush(brush));
            }
            DrawOp::StrokeCircle { center, radius, stroke } => {
                ctx.stroke_circle(bpoint(center), *radius, &bstroke(stroke), bbrush(&stroke.brush));
            }
            DrawOp::FillEllipse { center, rx, brush, .. } => {
                ctx.fill_circle(bpoint(center), *rx, bbrush(brush));
            }
            DrawOp::Line { from, to, stroke } => {
                ctx.stroke_path(&BPath::line(bpoint(from), bpoint(to)), &bstroke(stroke), bbrush(&stroke.brush));
            }
            DrawOp::Polyline { points, stroke } => {
                if let Some(first) = points.first() {
                    let mut p = BPath::new().move_to(first.x, first.y);
                    for pt in &points[1..] {
                        p = p.line_to(pt.x, pt.y);
                    }
                    ctx.stroke_path(&p, &bstroke(stroke), bbrush(&stroke.brush));
                }
            }
            DrawOp::Polygon { points, brush } => {
                if let Some(first) = points.first() {
                    let mut p = BPath::new().move_to(first.x, first.y);
                    for pt in &points[1..] {
                        p = p.line_to(pt.x, pt.y);
                    }
                    ctx.fill_path(&p.close(), bbrush(brush));
                }
            }
            DrawOp::FillPath { segments, brush } => {
                ctx.fill_path(&bpath(segments), bbrush(brush));
            }
            DrawOp::StrokePath { segments, stroke } => {
                ctx.stroke_path(&bpath(segments), &bstroke(stroke), bbrush(&stroke.brush));
            }
            DrawOp::Text { text, at, size, color, .. } => {
                ctx.draw_text(text, bpoint(at), &BTextStyle::new(*size).with_color(bcolor(*color)));
            }
            DrawOp::Save => ctx.push_transform(BTransform::identity()),
            DrawOp::Restore => ctx.pop_transform(),
            DrawOp::Translate { x, y } => ctx.push_transform(BTransform::translate(*x, *y)),
            DrawOp::Scale { x, y } => ctx.push_transform(BTransform::scale(*x, *y)),
            DrawOp::Rotate { degrees } => {
                ctx.push_transform(BTransform::rotate(degrees.to_radians()))
            }
            // Arc, image, and clip ops are mapped as the bridge matures.
            _ => {}
        }
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
        let pending: Vec<UiEvent> = std::mem::take(&mut shared.events.borrow_mut());
        if !pending.is_empty() {
            let mut sb = sandbox.borrow_mut();
            for ev in pending {
                let _ = sb.dispatch_event(&ev);
            }
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
