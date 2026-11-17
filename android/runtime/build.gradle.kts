// The nsh Android runtime as a library.
//
// It uses the prebuilt nsh.aar from the nsh-embed-android release package.
// The AAR includes the runtime's Java classes and native libraries.
// This module simply depends on the AAR without declaring its own namespace
// to avoid conflicts with the AAR's embedded namespace.

plugins {
    id("com.android.library") version "8.5.2" apply true
}

android {
    namespace = "com.longbridge.runtime"
    compileSdk = 35

    defaultConfig {
        minSdk = 26
    }

    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_11
        targetCompatibility = JavaVersion.VERSION_11
    }

    packaging {
        // Exclude the AAR's native libraries since we provide our own
        // via :app's jniLibs/<abi>/libnsh.so. This prevents the warning:
        // "2 files found for path 'lib/arm64-v8a/libnsh.so'"
        jniLibs {
            excludes += "**/libnsh.so"
        }
    }
}

dependencies {
    implementation(files("libs/nsh.aar"))
}
