//! Runs the Rust application directly on the machine's native backend.

#[cfg(target_os = "ios")]
use nsh_platform_ios as platform;

#[cfg(target_os = "android")]
use nsh_platform_android as platform;

#[cfg(target_arch = "wasm32")]
use nsh_platform_web as platform;

#[cfg(any(target_os = "android", target_os = "ios"))]
fn main() -> impl std::process::Termination {
    env_logger::init();
    platform::run_app(
        platform::AppSource::Rust(platform::Root::of::<nsh_app_nsh_example::App>()),
        platform::AppOptions {
            #[cfg(not(target_os = "ios"))]
            title: "nsh-example".to_owned(),
            #[cfg(any(target_os = "linux", target_os = "windows"))]
            app_id: "com.example.nshexample".to_owned(),

            #[cfg(not(target_os = "ios"))]
            size: (480, 640),
            ..Default::default()
        },
    )
}

#[cfg(not(any(target_os = "ios", target_os = "android", target_arch = "wasm32")))]
compile_error!(
    "Run this binary on macOS, Windows or Linux; use the TSX entry with nsh run for iOS and Android."
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
