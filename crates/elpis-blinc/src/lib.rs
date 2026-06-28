//! # elpis-blinc
//!
//! The Blinc UI backend for the Elpis sandbox. It connects the renderer-
//! agnostic [`elpis_host`] sandbox to the [Blinc](https://github.com/project-blinc/Blinc)
//! GPU UI framework.
//!
//! Two layers:
//!
//! * [`lower`] — a pure, always-compiled transform from the [`elpis_protocol`]
//!   widget tree to a [`lower::BlincDom`] (a normalized, blinc-flavored element
//!   description). This is where every Blinc widget family, style attribute,
//!   2D canvas op, 3D scene, and animation is mapped, and it is fully unit
//!   tested without a GPU.
//!
//! * `blinc_backend` — compiled only with the `blinc-backend` feature. It walks
//!   a `BlincDom` and constructs the live `blinc_layout` element tree, wires UI
//!   events back into the host, and provides a windowed run loop
//!   (`run_windowed`) that drives an [`elpis_host::Sandbox`] against a real
//!   Blinc window.
//!
//! Without the feature, [`DefaultBackend`] is the host's headless backend, so a
//! Miniapp can be booted, rendered, diffed, and event-driven entirely on the
//! CPU (used by the test-suite and by `elpis-app` in its headless mode).

pub mod lower;

pub use lower::{lower, BlincContent, BlincDom, BlincStyle};

// Re-export the host surface so downstream crates depend on just `elpis-blinc`.
pub use elpis_host::{
    HeadlessBackend, OutboundMessage, Sandbox, SandboxConfig, Services, SurfaceInfo, UiBackend,
    UiEvent,
};
pub use elpis_protocol::{self as protocol, Node, Patch};

#[cfg(not(feature = "blinc-backend"))]
mod headless_default {
    //! Without the Blinc feature, the default backend is the headless one.
    pub use elpis_host::HeadlessBackend as DefaultBackend;
}
#[cfg(not(feature = "blinc-backend"))]
pub use headless_default::DefaultBackend;

#[cfg(feature = "blinc-backend")]
mod blinc_backend;
#[cfg(feature = "blinc-backend")]
pub use blinc_backend::{build, frame_closure, BlincBackend, BlincShared};
#[cfg(feature = "desktop")]
pub use blinc_backend::run_windowed;
#[cfg(feature = "blinc-backend")]
pub use blinc_backend::BlincBackend as DefaultBackend;

// Re-export Blinc's per-frame context + element trait so the platform demo
// crates (elpis-web, elpis-android) can name the `frame_closure` return type
// and call `WebApp::run` / `AndroidApp::run` without a direct Blinc dependency
// on those types.
#[cfg(feature = "blinc-backend")]
pub use blinc_app::windowed::WindowedContext;
#[cfg(feature = "blinc-backend")]
pub use blinc_app::ElementBuilder;
