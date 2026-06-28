//! The live Blinc backend (compiled only with `--features blinc-backend`).
//!
//! This is the thin interpreter promised by [`crate::lower`]: it walks a
//! [`BlincDom`] and constructs the corresponding `blinc_layout` element tree,
//! wiring events back into the host. It also provides [`run_windowed`], which
//! opens a Blinc window and drives an [`elpis_host::Sandbox`] frame by frame.
//!
//! Because Blinc rebuilds its declarative tree from the build closure every
//! frame, the bridge keeps the latest lowered [`BlincDom`] in a shared cell:
//! the host renders the guest's tree into the backend (which lowers + stores
//! it), and the Blinc build closure reads that cell to construct widgets and
//! pushes UI events into a shared queue the host drains on the next pump.
//!
//! The `blinc_*` crate APIs are pinned to the 0.5 line (see the workspace
//! `Cargo.toml`); this module targets that builder surface.
#![cfg(feature = "blinc-backend")]

use std::cell::RefCell;
use std::rc::Rc;

use serde_json::Value;

use blinc_app::windowed::{WindowedApp, WindowedContext};
use blinc_core::Color as BColor;
use blinc_layout::div::div;
use blinc_layout::prelude::*;
use blinc_layout::text::text as btext;
use blinc_platform::WindowConfig;

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
        offset: blinc_core::Point { x: s.offset[0], y: s.offset[1] },
        blur: s.blur,
        spread: s.spread,
        color: bcolor(s.color),
        ..Default::default()
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
        BlincContent::Button(b) => {
            let label = btext(&b.label).size(15.0).color(BColor::WHITE);
            let mut d = apply_style(div(), &dom.style).child(label);
            d = wire_events(d, dom, shared);
            Box::new(d)
        }
        // Containers and every other family build a styled div with children;
        // the specialized widget builders (input, slider, dropdown, canvas,
        // scene) are layered on in `build_children`/dedicated arms.
        _ => {
            let mut d = apply_style(div(), &dom.style);
            for child in &dom.children {
                d = d.child_boxed(build(child, shared));
            }
            d = wire_events(d, dom, shared);
            Box::new(d)
        }
    }
}

/// Attach event handlers: each binding pushes a [`UiEvent`] into the shared
/// queue so the host delivers it to the guest's `onEvent`.
fn wire_events(mut d: Div, dom: &BlincDom, shared: &BlincShared) -> Div {
    if let Some(handler) = dom.events.get("click") {
        let handler = handler.clone();
        let events = shared.events.clone();
        d = d.on_click(move || {
            events.borrow_mut().push(UiEvent::click(handler.clone()));
        });
    }
    d
}

// ---------------------------------------------------------------------------
// Windowed run loop.
// ---------------------------------------------------------------------------

/// Open a Blinc window and drive `sandbox` (already booted with a
/// [`BlincBackend`] whose `shared` handle is passed here) frame by frame.
pub fn run_windowed(
    title: &str,
    width: u32,
    height: u32,
    sandbox: Sandbox,
    shared: BlincShared,
) -> Result<(), String> {
    let config = WindowConfig {
        title: title.to_string(),
        width,
        height,
        resizable: true,
        ..Default::default()
    };

    let sandbox = Rc::new(RefCell::new(sandbox));

    WindowedApp::run(config, move |ctx: &mut WindowedContext| {
        // 1. Deliver any queued UI events to the guest (which re-renders into
        //    the backend, updating the shared dom).
        let pending: Vec<UiEvent> = std::mem::take(&mut shared.events.borrow_mut());
        if !pending.is_empty() {
            let mut sb = sandbox.borrow_mut();
            for ev in pending {
                let _ = sb.dispatch_event(&ev);
            }
        }
        // 2. Build the current tree.
        let root = shared.dom.borrow();
        match root.as_ref() {
            Some(dom) => build(dom, &shared),
            None => Box::new(div().w(ctx.width).h(ctx.height).bg(BColor::rgba(0.05, 0.05, 0.07, 1.0))),
        }
    })
    .map_err(|e| format!("blinc run failed: {e:?}"))
}
