//! Android entry point for the Elpis demo.
//!
//! `android_main` is the NativeActivity entry the `android-activity` glue calls.
//! It boots the bundled **showcase** Miniapp inside an Elpis sandbox with a
//! Blinc [`BlincBackend`] and hands Blinc's `AndroidApp` the shared frame
//! closure. Same sandbox + bridge + lowering as the desktop and web builds;
//! only the platform run loop differs.
//!
//! The library is compiled to a `.so` by `cargo-ndk` and packaged into an APK
//! by the Gradle project in `apps/elpis-android/android/` (see the workflow
//! `.github/workflows/android.yml`). Off Android the crate is an empty lib.
#![cfg(target_os = "android")]

use android_activity::AndroidApp;

use elpis_blinc::{BlincBackend, Sandbox, SandboxConfig, SurfaceInfo};

/// The demo Miniapp, bundled into the `.so`.
const MINIAPP: &str = include_str!("../../../miniapps/showcase/app.js");

#[no_mangle]
fn android_main(app: AndroidApp) {
    android_logger::init_once(
        android_logger::Config::default().with_max_level(log::LevelFilter::Info),
    );
    log::info!("elpis-android: booting showcase miniapp");

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
