//! # elpis-host
//!
//! The **Elpis** sandbox: a host that instantiates an [`elpian_vm`] instance,
//! loads a Miniapp written in JavaScript, runs it, and bridges the Miniapp's
//! UI calls to a pluggable [`UiBackend`] (the real one is `elpis-blinc`, which
//! drives the Blinc UI framework).
//!
//! ```text
//!            ┌──────────── Elpis sandbox ─────────────┐
//!   JS  ───▶ │  elpian-vm  ──askHost(ui.render,…)──▶  │
//! Miniapp    │     ▲                 │   Services      │ ──▶ UiBackend ─▶ Blinc
//!            │     └── onEvent ◀─────┘  (bridge.rs)    │ ◀── UiEvent
//!            └────────────────────────────────────────┘
//! ```
//!
//! The Miniapp drives rendering itself: its top-level code calls `render(tree)`
//! (a prelude wrapper over `askHost("ui.render", [tree])`); the host diffs the
//! tree against the retained one and patches the backend. UI events flow back
//! by the host invoking the guest's `onEvent` / `onTick` / `onMessage`.
//!
//! The guest runs fully sandboxed: capabilities (network, filesystem, module
//! import) are denied by default and the VM's governor caps CPU/heap/storage.

mod backend;
mod bridge;
mod event;

pub use backend::{HeadlessBackend, SurfaceInfo, UiBackend};
pub use bridge::{OutboundMessage, Services};
pub use event::UiEvent;

pub use elpis_protocol::{self as protocol, Node, Patch};

use elpian_vm::api;
use elpian_vm::api::{Capability, CapabilitySet};
use elpis_protocol::HostCall;

/// The JS prelude prepended to every Miniapp.
pub const PRELUDE: &str = include_str!("prelude.js");

/// Guest entry points the host invokes (all optional).
const ENTRY_EVENT: &str = "onEvent";
const ENTRY_TICK: &str = "onTick";
const ENTRY_MESSAGE: &str = "onMessage";

/// Configuration for an Elpis sandbox instance.
#[derive(Debug, Clone)]
pub struct SandboxConfig {
    /// Unique instance id (the VM `machineId`).
    pub id: String,
    /// Grant outbound network (`net.*`). Denied by default.
    pub allow_network: bool,
    /// Grant the fabricated filesystem (`fs.*`). Denied by default.
    pub allow_filesystem: bool,
    /// Grant runtime module import (`vm.import`). Denied by default.
    pub allow_module_import: bool,
    /// Prepend the UI prelude to the Miniapp source.
    pub include_prelude: bool,
    /// Optional surface geometry reported to the guest (else the backend's).
    pub surface: Option<SurfaceInfo>,
    /// Max event/render settle iterations per pump (loop guard).
    pub max_settle_iterations: u32,
}

impl Default for SandboxConfig {
    fn default() -> Self {
        SandboxConfig {
            id: "elpis-instance".to_string(),
            allow_network: false,
            allow_filesystem: false,
            allow_module_import: false,
            include_prelude: true,
            surface: None,
            max_settle_iterations: 64,
        }
    }
}

impl SandboxConfig {
    pub fn new(id: impl Into<String>) -> SandboxConfig {
        SandboxConfig { id: id.into(), ..Default::default() }
    }

    fn capabilities(&self) -> CapabilitySet {
        // Start fully denied, then grant exactly what a UI Miniapp needs. The
        // UI bridge, storage, host messaging, theme, router and animation
        // families all map to `Capability::Other`; logging/clock/randomness
        // each have their own family.
        let mut caps = CapabilitySet::deny_all();
        caps.grant(Capability::Logging);
        caps.grant(Capability::Clock);
        caps.grant(Capability::Randomness);
        caps.grant(Capability::Other);
        if self.allow_network {
            caps.grant(Capability::Network);
        }
        if self.allow_filesystem {
            caps.grant(Capability::Storage);
        }
        if self.allow_module_import {
            caps.grant(Capability::ModuleImport);
        }
        caps
    }
}

/// How a pump turn starts.
enum Start {
    /// Run the program's top level (boot).
    Main,
    /// Invoke a named guest function with a JSON input.
    Func { name: String, input: String },
}

/// A running Elpis sandbox: one elpian-vm + the UI bridge services.
pub struct Sandbox {
    machine_id: String,
    services: Services,
    config: SandboxConfig,
    cb: i64,
    alive: bool,
}

impl Sandbox {
    /// Create a sandbox from Miniapp JavaScript source and a UI backend.
    ///
    /// Returns `Err` if the source is outside the VM's supported JS subset.
    pub fn from_js(
        config: SandboxConfig,
        source: &str,
        backend: Box<dyn UiBackend>,
    ) -> Result<Sandbox, String> {
        api::init_vm_system();

        let full_source = if config.include_prelude {
            format!("{PRELUDE}\n// ---- Miniapp ----\n{source}")
        } else {
            source.to_string()
        };

        if !api::create_vm_from_js(config.id.clone(), full_source) {
            return Err("Miniapp failed to compile (outside the supported JS subset)".to_string());
        }

        api::set_capabilities(&config.id, config.capabilities());

        let mut services = Services::new(backend);
        services.surface_override = config.surface;

        Ok(Sandbox { machine_id: config.id.clone(), services, config, cb: 0, alive: true })
    }

    /// Boot the Miniapp: run its top-level code (which performs the first
    /// render), then settle any UI events it produced.
    pub fn boot(&mut self) -> Result<(), String> {
        if !self.alive {
            return Err("sandbox terminated".to_string());
        }
        self.pump(Start::Main);
        self.settle();
        self.check_trap()
    }

    /// Deliver a UI event to the guest's `onEvent`, then settle.
    pub fn dispatch_event(&mut self, ev: &UiEvent) -> Result<(), String> {
        let input = serde_json::to_string(ev).unwrap_or_else(|_| "{}".to_string());
        self.pump(Start::Func { name: ENTRY_EVENT.to_string(), input });
        self.settle();
        self.check_trap()
    }

    /// Drive one animation frame: invoke the guest's `onTick` with `{dt, time}`
    /// (milliseconds), then settle.
    pub fn tick(&mut self, dt_ms: f64) -> Result<(), String> {
        let time = self.services.frames as f64;
        let input = serde_json::json!({ "dt": dt_ms, "time": time }).to_string();
        self.pump(Start::Func { name: ENTRY_TICK.to_string(), input });
        self.settle();
        self.check_trap()
    }

    /// Deliver a host->guest message to the guest's `onMessage`.
    pub fn deliver_message(&mut self, channel: &str, message: serde_json::Value) -> Result<(), String> {
        let input = serde_json::json!({ "channel": channel, "message": message }).to_string();
        self.pump(Start::Func { name: ENTRY_MESSAGE.to_string(), input });
        self.settle();
        self.check_trap()
    }

    /// Pump backend-produced events into the guest until none remain (or the
    /// settle guard trips), so a click that triggers a re-render that produces
    /// further events converges within one public call.
    fn settle(&mut self) {
        for _ in 0..self.config.max_settle_iterations {
            let events = self.services.backend.drain_events();
            if events.is_empty() {
                break;
            }
            for ev in events {
                let input = serde_json::to_string(&ev).unwrap_or_else(|_| "{}".to_string());
                self.pump(Start::Func { name: ENTRY_EVENT.to_string(), input });
            }
        }
    }

    /// The pump loop: step the VM, service each host call via the bridge, and
    /// resume — until the VM finishes this turn.
    fn pump(&mut self, start: Start) {
        if !self.alive {
            return;
        }
        let mid = self.machine_id.clone();
        let mut result = match start {
            Start::Main => api::execute_vm(mid.clone()),
            Start::Func { name, input } => {
                self.cb += 1;
                api::execute_vm_func_with_input(mid.clone(), name, input, self.cb)
            }
        };
        loop {
            if !result.has_host_call {
                return;
            }
            let reply = match HostCall::parse(&result.host_call_data) {
                Ok(hc) => self.services.dispatch(&hc.api_name, &hc.args()),
                Err(_) => elpis_protocol::hostcall::reply::null(),
            };
            result = api::continue_execution(mid.clone(), reply);
        }
    }

    fn check_trap(&self) -> Result<(), String> {
        match api::trap_reason(&self.machine_id) {
            Some(reason) => Err(format!("Miniapp trapped: {reason}")),
            None => Ok(()),
        }
    }

    // ---- Accessors --------------------------------------------------------

    pub fn id(&self) -> &str {
        &self.machine_id
    }

    /// The retained widget tree (last rendered).
    pub fn tree(&self) -> Option<&Node> {
        self.services.retained.as_ref()
    }

    /// The backend, for inspection (e.g. downcasting in tests).
    pub fn backend(&self) -> &dyn UiBackend {
        self.services.backend.as_ref()
    }

    pub fn backend_mut(&mut self) -> &mut dyn UiBackend {
        self.services.backend.as_mut()
    }

    /// Take any messages the guest pushed out via `host.send`.
    pub fn take_outbox(&mut self) -> Vec<OutboundMessage> {
        std::mem::take(&mut self.services.outbox)
    }

    /// Number of frames the guest has rendered.
    pub fn frames(&self) -> u64 {
        self.services.frames
    }

    /// Live resource usage (CPU steps, heap, storage) from the VM governor.
    pub fn usage(&self) -> Option<elpian_vm::api::ResourceUsage> {
        api::usage(&self.machine_id)
    }

    /// Terminate the instance and free its VM slot.
    pub fn terminate(&mut self) {
        if self.alive {
            api::terminate_vm(&self.machine_id);
            api::destroy_vm(self.machine_id.clone());
            self.alive = false;
        }
    }
}

impl Drop for Sandbox {
    fn drop(&mut self) {
        self.terminate();
    }
}

#[cfg(test)]
mod tests;
