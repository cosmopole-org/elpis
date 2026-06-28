plugins {
    id("com.android.application")
}

android {
    namespace = "org.cosmopole.elpis"
    // compileSdk 35 (Android 15) gives the headers + automatic 16 KB-page
    // jniLib alignment that recent devices (Android 16 / Pixel 10) enforce.
    compileSdk = 35

    defaultConfig {
        applicationId = "org.cosmopole.elpis"
        minSdk = 24
        targetSdk = 35
        versionCode = 1
        versionName = "1.0"

        ndk {
            // The CI builds the Rust .so for arm64-v8a; add more ABIs here if
            // you also run `cargo ndk -t <abi>` for them.
            abiFilters += listOf("arm64-v8a")
        }
    }

    buildTypes {
        release {
            isMinifyEnabled = false
        }
        debug {
            isMinifyEnabled = false
        }
    }

    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_17
        targetCompatibility = JavaVersion.VERSION_17
    }

    // Keep native libs uncompressed + 16 KB-aligned in the APK.
    packaging {
        jniLibs {
            useLegacyPackaging = false
        }
    }

    // `cargo ndk -o app/src/main/jniLibs` drops libelpis_android.so here.
    sourceSets {
        getByName("main") {
            jniLibs.srcDirs("src/main/jniLibs")
        }
    }
}

dependencies {
    implementation("androidx.core:core-ktx:1.13.1")
}
