//! Web (wasm32) entry point for the Elpis demo.
//!
//! Boots the bundled **Glass UI kit gallery** Miniapp inside an Elpis sandbox
//! whose backend is a Blinc [`BlincBackend`], then hands Blinc's `WebApp` the
//! shared frame closure so the UI renders to a WebGPU `<canvas>` in the
//! browser. This is the exact same sandbox + bridge + lowering used by the
//! desktop and Android builds — only the platform run loop differs.
//!
//! The gallery is built on the Glass UI kit SDK (`sdk/glass-ui-kit.js`); since
//! the sandbox denies runtime module import, the kit source is **prepended** to
//! the Miniapp here (the same composition the host binary does with `--lib`).
//!
//! Build with `wasm-pack build --target web --release` and serve the directory
//! (the GitHub Pages workflow does this). The page must provide a
//! `<canvas id="elpis-canvas">` and a browser with WebGPU (Chrome 113+).
#![cfg(target_arch = "wasm32")]

use wasm_bindgen::prelude::*;

use elpis_blinc::{BlincBackend, Sandbox, SandboxConfig, SurfaceInfo};

/// The Glass UI kit SDK + the gallery Miniapp, bundled into the wasm artifact.
/// The kit is prepended to the app (module import is denied in the sandbox).
const GLASS_KIT: &str = include_str!("../../../sdk/glass-ui-kit.js");
const GALLERY: &str = include_str!("../../../miniapps/glass-gallery/app.js");

/// Fonts bundled so text shapes in the browser (browsers don't expose system
/// fonts to the WebGPU pipeline). DejaVu Sans is the **sans-serif** family the
/// default text path resolves to — without a registered sans-serif face Blinc's
/// generic resolution finds nothing and every default text element renders
/// blank. Fira Code provides a **monospace** family.
const FONT_SANS: &[u8] = include_bytes!("../assets/DejaVuSans.ttf");
const FONT_SANS_BOLD: &[u8] = include_bytes!("../assets/DejaVuSans-Bold.ttf");
const FONT_MONO: &[u8] = include_bytes!("../assets/FiraCode-Regular.ttf");

/// Canvas element id the page must provide.
const CANVAS_ID: &str = "elpis-canvas";

#[wasm_bindgen(start)]
pub fn start() {
    console_error_panic_hook::set_once();
    web_sys::console::log_1(&"elpis-web: booting Glass UI kit gallery".into());

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

    let source = format!("{GLASS_KIT}\n// ---- gallery ----\n{GALLERY}");
    let mut sandbox = Sandbox::from_js(config, &source, Box::new(backend))?;
    sandbox.boot()?;

    // 2. Hand Blinc's WebApp the shared per-frame closure. `run_with_setup`
    //    registers the bundled fonts before the first frame so text shapes. We
    //    register a sans-serif family (regular + bold) AND a monospace family so
    //    both the default text path and any monospace text resolve a face.
    use blinc_app::web::WebApp;
    WebApp::run_with_setup(
        CANVAS_ID,
        move |app| {
            let mut faces = 0;
            faces += app.load_font_data(FONT_SANS.to_vec());
            faces += app.load_font_data(FONT_SANS_BOLD.to_vec());
            faces += app.load_font_data(FONT_MONO.to_vec());
            web_sys::console::log_1(&format!("elpis-web: registered {faces} font face(s)").into());
        },
        elpis_blinc::frame_closure(sandbox, shared),
    )
    .await
    .map_err(|e| format!("WebApp::run failed: {e:?}"))
}
