// The nsh Android runtime as a library AAR.
//
// It compiles the nsh runtime's own Java sources (the `com.longbridge.nsh`
// package: `AppActivity`, `NshView`, `Native`, …) straight from the nsh
// source tree, so it stays in step with the runtime crate. It ships no
// native library — the `:app` module provides `libnsh.so`.
//
// See `nsh/docs/dev/embedding.md` for how the runtime and hosts fit together.

plugins {
    id("com.android.library") version "8.5.2" apply true
}

android {
    namespace = "com.longbridge.nsh"
    compileSdk = 35

    defaultConfig {
        minSdk = 26
    }

    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_11
        targetCompatibility = JavaVersion.VERSION_11
    }

    // The runtime's Java lives in the nsh source tree; compile it in place
    // rather than copying it, so updates to the runtime propagate here. The
    // nsh checkout sits beside this Gradle project (nsh-example/nsh), one
    // level up from the Gradle root (this android/ directory).
    sourceSets {
        getByName("main") {
            java.srcDir("${project.rootDir}/../nsh/crates/nsh-platform-android/java")
        }
    }
}
