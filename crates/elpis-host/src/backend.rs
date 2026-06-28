//! The UI backend seam.
//!
//! `elpis-host` is renderer-agnostic: it owns the VM and the retained widget
//! tree, but delegates the actual drawing to a [`UiBackend`]. The real
//! implementation lives in `elpis-blinc` (it maps the widget tree to Blinc's
//! builder API and patches it in place); a [`HeadlessBackend`] here records
//! activity so the host can be driven and tested without a GPU.

use serde_json::{json, Value};

use elpis_protocol::{Node, Patch};

use crate::event::UiEvent;

/// Information about the render surface, returned to the guest from
/// `askHost("ui.surfaceInfo", [])`.
#[derive(Debug, Clone, Copy)]
pub struct SurfaceInfo {
    pub width: f32,
    pub height: f32,
    pub scale_factor: f32,
}

impl Default for SurfaceInfo {
    fn default() -> Self {
        SurfaceInfo { width: 800.0, height: 600.0, scale_factor: 1.0 }
    }
}

impl SurfaceInfo {
    pub fn to_json(&self) -> Value {
        json!({
            "width": self.width,
            "height": self.height,
            "scaleFactor": self.scale_factor,
        })
    }
}

/// A pluggable UI renderer driven by the host.
///
/// The host hands the backend a full [`Node`] tree on the first frame
/// ([`UiBackend::mount`]) and the minimal [`Patch`] script plus the resulting
/// full tree on subsequent frames ([`UiBackend::patch`]). A backend may consume
/// either; the full tree is always provided so a simple backend can rebuild and
/// a sophisticated one can reconcile.
pub trait UiBackend {
    /// Install the initial widget tree.
    fn mount(&mut self, tree: &Node);

    /// Apply an incremental update. `tree` is the new full tree (already
    /// reconciled by the host); `patches` is the minimal script that turns the
    /// previously-mounted tree into it.
    fn patch(&mut self, tree: &Node, patches: &[Patch]);

    /// Current surface geometry.
    fn surface_info(&self) -> SurfaceInfo {
        SurfaceInfo::default()
    }

    /// Apply a theme blob (`askHost("theme.set", [theme])`).
    fn set_theme(&mut self, _theme: &Value) {}

    /// Collect UI events produced since the last drain. The host pumps these
    /// back into the guest.
    fn drain_events(&mut self) -> Vec<UiEvent> {
        Vec::new()
    }

    /// Handle a backend-directed imperative command that isn't part of the
    /// declarative tree (e.g. `router.push`, `media.play`, focus, clipboard).
    /// Returns the JSON reply value for the guest. Default: acknowledge.
    fn command(&mut self, _channel: &str, _args: &[Value]) -> Value {
        json!({ "ok": true })
    }
}

/// A no-GPU backend that records the latest tree and counts patches. Used by
/// the host's own tests, by headless tooling, and as the default backend of the
/// `elpis-app` binary when built without the `blinc-backend` feature.
#[derive(Debug, Default)]
pub struct HeadlessBackend {
    pub surface: SurfaceInfo,
    pub tree: Option<Node>,
    pub mounts: u32,
    pub patch_batches: u32,
    pub total_patches: u64,
    pub theme: Option<Value>,
    pub pending_events: Vec<UiEvent>,
    pub commands: Vec<(String, Vec<Value>)>,
}

impl HeadlessBackend {
    pub fn new() -> HeadlessBackend {
        HeadlessBackend::default()
    }

    pub fn with_surface(surface: SurfaceInfo) -> HeadlessBackend {
        HeadlessBackend { surface, ..Default::default() }
    }

    /// Queue an event to be delivered to the guest on the next pump.
    pub fn push_event(&mut self, ev: UiEvent) {
        self.pending_events.push(ev);
    }
}

impl UiBackend for HeadlessBackend {
    fn mount(&mut self, tree: &Node) {
        self.tree = Some(tree.clone());
        self.mounts += 1;
    }

    fn patch(&mut self, tree: &Node, patches: &[Patch]) {
        self.tree = Some(tree.clone());
        self.patch_batches += 1;
        self.total_patches += patches.len() as u64;
    }

    fn surface_info(&self) -> SurfaceInfo {
        self.surface
    }

    fn set_theme(&mut self, theme: &Value) {
        self.theme = Some(theme.clone());
    }

    fn drain_events(&mut self) -> Vec<UiEvent> {
        std::mem::take(&mut self.pending_events)
    }

    fn command(&mut self, channel: &str, args: &[Value]) -> Value {
        self.commands.push((channel.to_string(), args.to_vec()));
        json!({ "ok": true })
    }
}
