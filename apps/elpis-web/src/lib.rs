//! Web (wasm32) entry point for the Elpis demo.
//!
//! Boots the bundled **showcase** Miniapp inside an Elpis sandbox whose backend
//! is a Blinc [`BlincBackend`], then hands Blinc's `WebApp` the shared frame
//! closure so the UI renders to a WebGPU `<canvas>` in the browser. This is the
//! exact same sandbox + bridge + lowering used by the desktop and Android
//! builds — only the platform run loop differs.
//!
//! Build with `wasm-pack build --target web --release` and serve the directory
//! (the GitHub Pages workflow does this). The page must provide a
//! `<canvas id="elpis-canvas">` and a browser with WebGPU (Chrome 113+).
#![cfg(target_arch = "wasm32")]

use wasm_bindgen::prelude::*;

use elpis_blinc::{BlincBackend, Sandbox, SandboxConfig, SurfaceInfo};

/// The demo Miniapp, bundled into the wasm artifact.
const MINIAPP: &str = include_str!("../../../miniapps/showcase/app.js");

/// An OFL-licensed font bundled so text shapes in the browser (browsers don't
/// expose system fonts to the WebGPU pipeline).
const FONT: &[u8] = include_bytes!("../assets/FiraCode-Regular.ttf");

/// Canvas element id the page must provide.
const CANVAS_ID: &str = "elpis-canvas";

#[wasm_bindgen(start)]
pub fn start() {
    console_error_panic_hook::set_once();
    web_sys::console::log_1(&"elpis-web: booting showcase miniapp".into());

    wasm_bindgen_futures::spawn_local(async {
        if let Err(e) = run().await {
            web_sys::console::error_1(&format!("elpis-web: {e}").into());
        }
    });
}

async fn run() -> Result<(), String> {
    // 1. Instantiate the Elpis sandbox with a Blinc backend and boot the
    //    Miniapp (top-level JS performs the first render).
    let surface = SurfaceInfo { width: 1024.0, height: 720.0, scale_factor: 1.0 };
    let (backend, shared) = BlincBackend::new(surface);
    let config = SandboxConfig { surface: Some(surface), ..SandboxConfig::new("elpis-web") };

    let mut sandbox = Sandbox::from_js(config, MINIAPP, Box::new(backend))?;
    sandbox.boot()?;

    // 2. Hand Blinc's WebApp the shared per-frame closure. `run_with_setup`
    //    registers the bundled font before the first frame so text shapes.
    use blinc_app::web::WebApp;
    let font = FONT.to_vec();
    WebApp::run_with_setup(
        CANVAS_ID,
        move |app| {
            let faces = app.load_font_data(font.clone());
            web_sys::console::log_1(&format!("elpis-web: registered {faces} font face(s)").into());
        },
        elpis_blinc::frame_closure(sandbox, shared),
    )
    .await
    .map_err(|e| format!("WebApp::run failed: {e:?}"))
}
