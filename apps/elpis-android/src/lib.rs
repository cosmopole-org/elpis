//! Android entry point for the Elpis demo.
//!
//! `android_main` is the NativeActivity entry the `android-activity` glue calls.
//! It boots the bundled **liquid glass** Miniapp (`miniapps/glass-gallery`,
//! driven by the Glass UI kit `sdk/glass-ui-kit.js`) inside an Elpis sandbox
//! with a Blinc [`BlincBackend`] and hands Blinc's `AndroidApp` the shared frame
//! closure. Same sandbox + bridge + lowering as the desktop and web builds;
//! only the platform run loop differs.
//!
//! The library is compiled to a `.so` by `cargo-ndk` and packaged into an APK
//! by the Gradle project in `apps/elpis-android/android/` (see the workflow
//! `.github/workflows/android.yml`). Off Android the crate is an empty lib.
#![cfg(target_os = "android")]

use android_activity::AndroidApp;

use elpis_blinc::{BlincBackend, Sandbox, SandboxConfig, SurfaceInfo};

/// The demo Miniapp, bundled into the `.so`: the **liquid glass** gallery.
/// Module import is denied in the sandbox, so the Glass UI kit it depends on is
/// shared the same way the host binary's `--lib` does it — by prepending the
/// kit source ahead of the Miniapp (the host then prepends the UI prelude, so
/// the guest sees prelude + kit + app). `concat!` over `include_str!` joins
/// them at compile time.
const MINIAPP: &str = concat!(
    include_str!("../../../sdk/glass-ui-kit.js"),
    "\n// ---- miniapp ----\n",
    include_str!("../../../miniapps/glass-gallery/app.js"),
);

#[no_mangle]
fn android_main(app: AndroidApp) {
    android_logger::init_once(
        android_logger::Config::default().with_max_level(log::LevelFilter::Info),
    );
    log::info!("elpis-android: booting liquid glass gallery miniapp");

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
