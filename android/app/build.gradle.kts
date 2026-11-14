// The packaged nsh example app.
//
// It does the minimum a Rust-root nsh app needs:
//   * launch `com.longbridge.nsh.AppActivity` (the runtime's standalone
//     activity) and mark it a Rust app via the `com.longbridge.nsh.RUST`
//     metadata, so the runtime calls `Native.runRust` instead of loading a
//     JS bundle;
//   * ship the per-ABI `libnsh.so` cargo builds, in `src/main/jniLibs/<abi>/`.
//
// The `libnsh.so` is built by cargo + the NDK and copied into `jniLibs` by
// the repo root `Makefile` (`make android-build`), not by Gradle.

plugins {
    id("com.android.application") version "8.5.2" apply true
}

android {
    namespace = "com.example.nshexample"
    compileSdk = 35

    defaultConfig {
        applicationId = "com.example.nshexample"
        minSdk = 26
        targetSdk = 35
        versionCode = 1
        versionName = "0.1.0"
    }

    buildTypes {
        release {
            isMinifyEnabled = false
        }
    }

    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_11
        targetCompatibility = JavaVersion.VERSION_11
    }
}

dependencies {
    // The runtime: `AppActivity`, `NshView`, `Native` and the rest. Its
    // `Native` does `System.loadLibrary("nsh")`, which resolves to the
    // `:app` module's `libnsh.so` (built by cargo) in the assembled APK.
    implementation(project(":runtime"))
}
