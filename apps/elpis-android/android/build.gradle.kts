// Root Gradle build for the Elpis Android demo. The actual app module is
// `:app`; the Rust `.so` it packages is produced by `cargo ndk` (see
// `.github/workflows/android.yml`).
plugins {
    id("com.android.application") version "8.5.2" apply false
}
