//! The host-API bridge: services every `askHost(api, [..args])` the guest emits.
//!
//! This is the host side of the Elpis<->Blinc bridge. The guest's UI code never
//! touches Blinc directly; it speaks the [`elpis_protocol`] vocabulary over
//! `askHost`, and [`Services::dispatch`] turns each call into an action on the
//! retained widget tree, the [`UiBackend`], or a sandboxed host facility
//! (storage, clock, randomness, custom host messaging).

use std::collections::HashMap;
// `web_time` re-exports `std::time` on native targets and provides a
// browser-backed implementation on `wasm32-unknown-unknown`, where the real
// `std::time::SystemTime::now()` panics. This keeps the clock working in the
// web demo without changing behavior on desktop/Android.
use web_time::{SystemTime, UNIX_EPOCH};

use serde_json::{json, Value};

use elpis_protocol::hostcall::reply;
use elpis_protocol::{diff, Node, Patch};

use crate::backend::{SurfaceInfo, UiBackend};

/// A message the guest pushed out to the embedder via `host.send`.
#[derive(Debug, Clone)]
pub struct OutboundMessage {
    pub channel: String,
    pub message: Value,
}

/// The serviceable state behind the bridge: everything `dispatch` may touch.
/// Kept separate from the VM driver so the pump loop can borrow the driver
/// (just a machine id string) and these services disjointly.
pub struct Services {
    pub backend: Box<dyn UiBackend>,
    /// The retained widget tree (the last tree the guest rendered).
    pub retained: Option<Node>,
    /// Sandboxed key/value store (per-instance, in memory).
    pub storage: HashMap<String, String>,
    /// Current theme blob.
    pub theme: Option<Value>,
    /// Router history stack (each entry an arbitrary route value).
    pub router: Vec<Value>,
    /// Messages the guest pushed out via `host.send`, for the embedder to read.
    pub outbox: Vec<OutboundMessage>,
    /// Optional surface override (else taken from the backend).
    pub surface_override: Option<SurfaceInfo>,
    /// Monotonic clock origin.
    start: SystemTime,
    /// PRNG state.
    rng: u64,
    /// Total frames rendered (diagnostics).
    pub frames: u64,
}

impl Services {
    pub fn new(backend: Box<dyn UiBackend>) -> Services {
        let seed = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos() as u64)
            .unwrap_or(0x9E3779B97F4A7C15)
            | 1;
        Services {
            backend,
            retained: None,
            storage: HashMap::new(),
            theme: None,
            router: Vec::new(),
            outbox: Vec::new(),
            surface_override: None,
            start: SystemTime::now(),
            rng: seed,
            frames: 0,
        }
    }

    fn surface(&self) -> SurfaceInfo {
        self.surface_override.unwrap_or_else(|| self.backend.surface_info())
    }

    fn next_rng(&mut self) -> u64 {
        // xorshift64*
        let mut x = self.rng;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.rng = x;
        x.wrapping_mul(0x2545F4914F6CDD1D)
    }

    /// Service one host call. `api` is the `askHost` name; `args` is the
    /// argument array. Returns the plain-JSON reply the VM is resumed with.
    pub fn dispatch(&mut self, api: &str, args: &[Value]) -> String {
        let arg0 = args.first().cloned().unwrap_or(Value::Null);
        match api {
            // ---- Diagnostics ------------------------------------------
            "log" => {
                let line = args
                    .iter()
                    .map(render_log_arg)
                    .collect::<Vec<_>>()
                    .join(" ");
                eprintln!("[miniapp] {line}");
                reply::null()
            }

            // ---- The render path --------------------------------------
            "ui.render" => self.render(arg0),
            "ui.patch" => self.apply_guest_patches(arg0),
            "ui.surfaceInfo" => reply::value(self.surface().to_json()),

            // ---- Theming ----------------------------------------------
            "theme.set" => {
                self.backend.set_theme(&arg0);
                self.theme = Some(arg0);
                reply::ok()
            }
            "theme.get" => reply::value(self.theme.clone().unwrap_or(Value::Null)),

            // ---- Router -----------------------------------------------
            "router.push" => {
                self.router.push(arg0.clone());
                self.backend.command("router.push", args);
                reply::value(self.router.last().cloned().unwrap_or(Value::Null))
            }
            "router.replace" => {
                if let Some(top) = self.router.last_mut() {
                    *top = arg0.clone();
                } else {
                    self.router.push(arg0.clone());
                }
                self.backend.command("router.replace", args);
                reply::value(self.router.last().cloned().unwrap_or(Value::Null))
            }
            "router.pop" => {
                let popped = self.router.pop().unwrap_or(Value::Null);
                self.backend.command("router.pop", args);
                reply::value(popped)
            }
            "router.current" => reply::value(self.router.last().cloned().unwrap_or(Value::Null)),

            // ---- Sandboxed key/value storage --------------------------
            "storage.get" => {
                let key = arg0.as_str().unwrap_or_default();
                match self.storage.get(key) {
                    Some(v) => reply::value(parse_or_string(v)),
                    None => reply::null(),
                }
            }
            "storage.set" => {
                let key = arg0.as_str().unwrap_or_default().to_string();
                let val = args.get(1).cloned().unwrap_or(Value::Null);
                self.storage.insert(key, val.to_string());
                reply::ok()
            }
            "storage.remove" => {
                let key = arg0.as_str().unwrap_or_default();
                self.storage.remove(key);
                reply::ok()
            }
            "storage.keys" => {
                let keys: Vec<&String> = self.storage.keys().collect();
                reply::value(json!(keys))
            }
            "storage.clear" => {
                self.storage.clear();
                reply::ok()
            }

            // ---- Clock ------------------------------------------------
            "time.now" => {
                let ms = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .map(|d| d.as_millis() as u64)
                    .unwrap_or(0);
                reply::value(json!({ "ms": ms }))
            }
            "time.monotonic" => {
                let ms = self.start.elapsed().map(|d| d.as_millis() as u64).unwrap_or(0);
                reply::value(json!({ "ms": ms }))
            }

            // ---- Randomness -------------------------------------------
            "random.next" => {
                let v = (self.next_rng() >> 11) as f64 / (1u64 << 53) as f64;
                reply::value(json!(v))
            }
            "random.bytes" => {
                let n = arg0.as_u64().unwrap_or(16).min(4096) as usize;
                let mut out = Vec::with_capacity(n);
                while out.len() < n {
                    let r = self.next_rng();
                    out.extend_from_slice(&r.to_le_bytes());
                }
                out.truncate(n);
                reply::value(json!(out))
            }

            // ---- Custom host messaging --------------------------------
            "host.send" => {
                let channel = arg0.as_str().unwrap_or_default().to_string();
                let message = args.get(1).cloned().unwrap_or(Value::Null);
                self.outbox.push(OutboundMessage { channel, message });
                reply::ok()
            }
            "host.request" => {
                let channel = arg0.as_str().unwrap_or_default();
                let rest = if args.len() > 1 { &args[1..] } else { &[] };
                reply::value(self.backend.command(channel, rest))
            }

            // ---- Imperative families forwarded to the backend ---------
            // Animation control, canvas/scene imperative ops, media playback,
            // focus/clipboard, etc. The declarative state still lives in the
            // tree; these are the side-channel imperatives.
            _ if is_backend_channel(api) => reply::value(self.backend.command(api, args)),

            // Unknown api: acknowledge with null so the guest stays deterministic.
            _ => reply::null(),
        }
    }

    /// The full render path: diff the new tree against the retained one and
    /// hand the backend either a mount (first frame) or a minimal patch script.
    fn render(&mut self, tree_json: Value) -> String {
        let new_tree: Node = match serde_json::from_value(tree_json) {
            Ok(t) => t,
            Err(e) => return reply::err(&format!("invalid ui tree: {e}")),
        };
        self.frames += 1;
        match self.retained.take() {
            None => {
                self.backend.mount(&new_tree);
                self.retained = Some(new_tree);
            }
            Some(old) => {
                let patches = diff(&old, &new_tree);
                if !patches.is_empty() {
                    self.backend.patch(&new_tree, &patches);
                }
                self.retained = Some(new_tree);
            }
        }
        reply::ok()
    }

    /// Apply a guest-authored patch script directly (advanced API for guests
    /// that maintain their own retained tree and want to skip the host diff).
    fn apply_guest_patches(&mut self, patches_json: Value) -> String {
        let patches: Vec<Patch> = match serde_json::from_value(patches_json) {
            Ok(p) => p,
            Err(e) => return reply::err(&format!("invalid patches: {e}")),
        };
        match self.retained.as_mut() {
            Some(tree) => {
                if let Err(e) = elpis_protocol::apply(tree, &patches) {
                    return reply::err(&format!("patch failed: {e}"));
                }
                let snapshot = tree.clone();
                self.backend.patch(&snapshot, &patches);
                reply::ok()
            }
            None => reply::err("no tree mounted yet"),
        }
    }
}

/// Whether an api name belongs to an imperative family the backend services.
fn is_backend_channel(api: &str) -> bool {
    const PREFIXES: [&str; 7] =
        ["anim.", "canvas.", "scene3d.", "media.", "focus.", "clipboard.", "window."];
    PREFIXES.iter().any(|p| api.starts_with(p))
}

fn render_log_arg(v: &Value) -> String {
    match v {
        Value::String(s) => s.clone(),
        other => other.to_string(),
    }
}

fn parse_or_string(s: &str) -> Value {
    serde_json::from_str(s).unwrap_or_else(|_| Value::String(s.to_string()))
}
