//! Android entry point for the Elpis demo.
//!
//! `android_main` is the NativeActivity entry the `android-activity` glue calls.
//! It boots a bundled Miniapp gallery inside an Elpis sandbox with a Blinc
//! [`BlincBackend`] and hands Blinc's `AndroidApp` the shared frame closure.
//! Same sandbox + bridge + lowering as the desktop and web builds; only the
//! platform run loop differs.
//!
//! By default this bundles the **liquid glass** gallery (`miniapps/
//! glass-gallery`, driven by `sdk/glass-ui-kit.js`); building with `--features
//! material_demo` bundles the **Material Design 3** gallery instead
//! (`miniapps/material-gallery`, driven by `sdk/material-ui-kit.js`). The
//! workflow (`.github/workflows/android.yml`) builds both into separate APKs
//! (`elpis-demo.apk` / `elpis-demo-material.apk`) so both are installable and
//! testable side by side (see that workflow for how the two builds get
//! distinct `applicationId`/labels via a Gradle project property).
//!
//! The library is compiled to a `.so` by `cargo-ndk` and packaged into an APK
//! by the Gradle project in `apps/elpis-android/android/`. Off Android the
//! crate is an empty lib.
#![cfg(target_os = "android")]

use android_activity::AndroidApp;

use elpis_blinc::{BlincBackend, Sandbox, SandboxConfig, SurfaceInfo};

/// The demo Miniapp, bundled into the `.so`. Module import is denied in the
/// sandbox, so the UI kit it depends on is shared the same way the host
/// binary's `--lib` does it — by prepending the kit source ahead of the
/// Miniapp (the host then prepends the UI prelude, so the guest sees prelude +
/// kit + app). `concat!` over `include_str!` joins them at compile time.
#[cfg(not(feature = "material_demo"))]
const MINIAPP: &str = concat!(
    include_str!("../../../sdk/glass-ui-kit.js"),
    "\n// ---- miniapp ----\n",
    include_str!("../../../miniapps/glass-gallery/app.js"),
);
#[cfg(feature = "material_demo")]
const MINIAPP: &str = concat!(
    include_str!("../../../sdk/material-ui-kit.js"),
    "\n// ---- miniapp ----\n",
    include_str!("../../../miniapps/material-gallery/app.js"),
);

#[cfg(not(feature = "material_demo"))]
const DEMO_NAME: &str = "liquid glass gallery";
#[cfg(feature = "material_demo")]
const DEMO_NAME: &str = "Material Design 3 gallery";

#[no_mangle]
fn android_main(app: AndroidApp) {
    android_logger::init_once(
        android_logger::Config::default().with_max_level(log::LevelFilter::Info),
    );
    log::info!("elpis-android: booting {DEMO_NAME} miniapp");

    if let Err(e) = run(app) {
        log::error!("elpis-android: {e}");
    }
}

fn run(app: AndroidApp) -> Result<(), String> {
    // Surface size is reconfigured by Blinc from the real window; this is just
    // the initial value the guest sees from `surfaceInfo()`.
    let surface = SurfaceInfo { width: 1080.0, height: 2160.0, scale_factor: 3.0 };
    let (backend, shared) = BlincBackend::new(surface);
    let config = SandboxConfig { surface: Some(surface), ..SandboxConfig::new("elpis-android") };

    let mut sandbox = Sandbox::from_js(config, MINIAPP, Box::new(backend))?;
    sandbox.boot()?;

    blinc_app::AndroidApp::run(app, elpis_blinc::frame_closure(sandbox, shared))
        .map_err(|e| format!("AndroidApp::run failed: {e:?}"))
}
