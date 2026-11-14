//! Runs the Rust application directly on the machine's native backend.

#[cfg(target_os = "ios")]
use nsh_platform_ios as platform;

#[cfg(target_arch = "wasm32")]
use nsh_platform_web as platform;

// Desktop binaries mount the Rust root directly through the platform's
// `run_app`. Android and iOS do not run this binary: they are hosted by the
// nsh Android/iOS runtime, which calls the `Native.runRust` entry the `main!`
// macro in `src/lib.rs` exports (see the `android/` Gradle project).
#[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
fn main() -> impl std::process::Termination {
    env_logger::init();
    platform::run_app(
        platform::AppSource::Rust(platform::Root::of::<nsh_app_nsh_example::App>()),
        platform::AppOptions {
            title: "nsh-example".to_owned(),
            #[cfg(any(target_os = "linux", target_os = "windows"))]
            app_id: "com.example.nshexample".to_owned(),

            size: (480, 640),
            ..Default::default()
        },
    )
}

#[cfg(any(target_os = "android", target_os = "ios", target_arch = "wasm32"))]
compile_error!(
    "Run this binary on macOS, Windows or Linux; the nsh runtime hosts Android and iOS."
);

/*
#[cfg(target_arch = "wasm32")]
nsh_platform_web::rust_app!(
    vec![nsh_base::registry(), set_app_nsh_app_nsh_example()],
    app_root()
);


/// The UI set of the crate nsh-app-nsh-example: the entry `#[native_module("…")]` makes in it.
#[cfg(target_arch = "wasm32")]
fn set_app_nsh_app_nsh_example() -> nsh_core::element::Registry {
    #[allow(unused_imports)]
    use nsh_app_nsh_example::*;
    __nsh_native_module() // the crate nsh-app-nsh-example (nsh.json "rust")
}

// Without the entry: nsh's stand-in, which fails the build saying what to add.
#[cfg(target_arch = "wasm32")]
#[allow(unused_imports)]
use nsh_core::element::__private::no_native_module as __nsh_native_module;

/// The app's root (`nsh.json` `"root"`), with its theme, appearance and
/// status bar.
#[cfg(target_arch = "wasm32")]
fn app_root() -> nsh_core::element::Root {
    nsh_core::element::Root::of::<nsh_app_nsh_example::App>()
        .app_id("com.example.nshexample")
}

#[cfg(target_arch = "wasm32")]
fn main() {
}
*/
