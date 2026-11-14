// Root of the Android project that bundles the nsh example app.
//
//   :runtime  compiles the nsh Android runtime's Java sources (the
//             `com.longbridge.nsh` package: `AppActivity`, `NshView`,
//             `Native`, …) into an AAR. It ships no native library — the
//             `:app` module provides `libnsh.so`, built by cargo.
//   :app      the packaged app: a manifest that launches `AppActivity`
//             as a Rust app, plus the per-ABI `libnsh.so` cargo builds.
//
// The native library is built outside Gradle with cargo + the NDK and
// dropped into `:app/src/main/jniLibs/<abi>/` by the Makefile (see the
// repo root `Makefile`), so Gradle only assembles the APK.

plugins {
    id("com.android.application") version "8.5.2" apply false
    id("com.android.library") version "8.5.2" apply false
}
