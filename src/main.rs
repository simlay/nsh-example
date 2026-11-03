//! Runs the Rust application directly on the machine's native backend.

#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]

#[cfg(target_os = "linux")]
use nsh_platform_linux as platform;

#[cfg(target_os = "macos")]
use nsh_platform_macos as platform;

#[cfg(target_os = "ios")]
use nsh_platform_ios as platform;

#[cfg(target_os = "android")]
use nsh_platform_android as platform;

#[cfg(target_os = "windows")]
use nsh_platform_windows as platform;

#[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows", target_os = "ios"))]
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

#[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows", target_os = "ios", target_os = "android")))]
compile_error!(
    "Run this binary on macOS, Windows or Linux; use the TSX entry with nsh run for iOS and Android."
);
