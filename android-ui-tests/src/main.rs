//! Headless driver for the nsh Android app's UI tests.
//!
//! The app (`com.example.nshexample`) is built with the `nsh-platform-android`
//! `dev` feature, which compiles the test host (`DevActivity`) plus the `nsh`
//! test agent. This crate starts a host-side `DevServer`, points the app's
//! `DevActivity` at it over `adb reverse`, and drives it with [`TestStep`]s —
//! including [`TestStep::Screenshot`], which asks the host for its real,
//! frame-synced `PixelCopy` (the only way to get a clean, un-torn picture of
//! the composited app).
//!
//! Usage:
//!   android-ui-tests <output-dir> [--device <serial>]
//!
//! It takes a screenshot, types into the todo input, submits, waits for the new
//! todo, and takes a second screenshot — writing the PNGs to `<output-dir>`.

use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Duration;

use anyhow::{bail, Context, Result};
use base64::Engine as _;
use nsh_dev::protocol::TestStep;
use nsh_dev::DevServer;

/// The app's package + `DevActivity` (compiled by the runtime's `dev` feature).
const HOST: &str = "com.example.nshexample";
const DEV_ACTIVITY: &str = "com.longbridge.nsh.DevActivity";

/// The runtime's `nsh_engine::BYTECODE_VERSION`. Irrelevant for a Rust root
/// (no bundle is ever sent), but passed so the host does not log a version
/// mismatch warning. Keep in sync with the checkout's `nsh-js`.
const BYTECODE_VERSION: &str = "quickjs-ng-0.16.2+0fdea21/bc28/le";

/// How long to wait for the host's `DevActivity` to connect to the dev server.
const CONNECT: Duration = Duration::from_secs(90);

/// Adds context to a `step` outcome. `step` returns `Result<_, String>` (a bare
/// message), which `anyhow`'s `.context` cannot wrap, so we convert it here.
fn ctx<T>(result: Result<T, String>, msg: &str) -> Result<T, anyhow::Error> {
    result.map_err(|e| anyhow::anyhow!("{msg}: {e}"))
}

fn main() -> Result<()> {
    let mut args = std::env::args().skip(1);
    let out_dir = args
        .next()
        .context("usage: android-ui-tests <output-dir> [--device <serial>]")?;
    let out_dir = PathBuf::from(out_dir);

    let mut device: Option<String> = None;
    while let Some(flag) = args.next() {
        match flag.as_str() {
            "--device" => {
                device = Some(args.next().context("--device needs a value")?);
            }
            other => bail!("unknown argument: {other}"),
        }
    }

    // 1. Locate adb and a device.
    let adb = adb_path()?;
    let serial = device
        .or_else(find_device)
        .context("no device: pass --device <serial> or connect one")?;

    // 2. Start the host-side dev server on a free localhost port.
    let server = DevServer::start(
        "127.0.0.1:0".parse::<SocketAddr>().unwrap(),
        BYTECODE_VERSION,
        None,
        |_: nsh_dev::ClientEvent| {},
    )
    .context("starting the dev server")?;
    let port = server.local_addr().port();
    println!("dev server listening on ws://127.0.0.1:{port}/ws");

    // 3. Announce the test session and publish the app's Rust root.
    server.test_session();
    server.publish_root();

    // 4. Forward a host port to the device, then launch the host activity.
    reverse(&adb, &serial, port)?;
    launch(&adb, &serial, port)?;

    // 5. Wait for the host's tester to connect.
    wait_for_tester(&server)?;

    // 6. Drive the test, capturing screenshots along the way.
    std::fs::create_dir_all(&out_dir).context("creating the output dir")?;
    let pngs = run_test(&server, &out_dir)?;

    // 7. Clean up (the DevServer is dropped at the end of `main`).
    reverse_remove(&adb, &serial, port);
    let _ = run(&adb, &["-s", &serial, "am", "force-stop", HOST]);

    println!("captured {} screenshot(s):", pngs.len());
    for path in &pngs {
        println!("  {}", path.display());
    }
    Ok(())
}

/// Runs the scripted sequence and returns the paths of the PNGs it wrote.
fn run_test(server: &DevServer, out_dir: &Path) -> Result<Vec<PathBuf>> {
    let mut pngs: Vec<PathBuf> = Vec::new();

    // First render, then wait until the app is quiet.
    let ready = ctx(
        server.step(TestStep::Ready { timeout_ms: 5000 }, Duration::from_secs(30)),
        "Ready step",
    )?;
    println!(
        "ready: {} views, {} pending",
        ready
            .get("views")
            .map(|v| v.as_array().map(Vec::len).unwrap_or(0))
            .unwrap_or(0),
        ready
            .get("pending")
            .map(|v| v.as_array().map(Vec::len).unwrap_or(0))
            .unwrap_or(0),
    );

    // Screenshot 1: the empty list.
    pngs.push(shot(server, out_dir, "01_initial.png")?);

    // Type into the (empty) todo input, matched by its placeholder.
    ctx(
        server.step(
            TestStep::Type {
                selector: "text:/Add a new todo/".into(),
                text: "Buy milk".into(),
            },
            Duration::from_secs(15),
        ),
        "Type step",
    )?;

    // Submit the input (return key), matched by the text it now holds.
    ctx(
        server.step(
            TestStep::Submit {
                selector: "text:/Buy milk/".into(),
            },
            Duration::from_secs(15),
        ),
        "Submit step",
    )?;

    // Wait for the new todo to appear.
    ctx(
        server.step(
            TestStep::WaitFor {
                selector: "text:Buy milk".into(),
                timeout_ms: 5000,
                gone: false,
            },
            Duration::from_secs(15),
        ),
        "WaitFor step",
    )?;

    // Screenshot 2: the list with one todo.
    pngs.push(shot(server, out_dir, "02_added.png")?);

    Ok(pngs)
}

/// Takes one screenshot and writes it as a PNG in `out_dir`.
fn shot(server: &DevServer, out_dir: &Path, name: &str) -> Result<PathBuf> {
    let outcome = ctx(
        server.step(TestStep::Screenshot, Duration::from_secs(15)),
        "Screenshot step",
    )?;
    let encoded = outcome
        .as_str()
        .context("the host did not send a picture (is the app on screen?)")?;
    let png = base64::engine::general_purpose::STANDARD
        .decode(encoded)
        .map_err(|e| anyhow::anyhow!("the host sent a broken picture: {e}"))?;
    let path = out_dir.join(name);
    std::fs::write(&path, &png).with_context(|| format!("writing {}", path.display()))?;
    Ok(path)
}

/// Waits until the host's `DevActivity` reports it is connected.
fn wait_for_tester(server: &DevServer) -> Result<()> {
    let start = std::time::Instant::now();
    while start.elapsed() < CONNECT {
        if server.has_tester() {
            println!("host connected after {:?}", start.elapsed());
            return Ok(());
        }
        std::thread::sleep(Duration::from_millis(100));
    }
    bail!("the host did not connect within {CONNECT:?}");
}

/// Forwards `port` on the host to `port` on the device.
fn reverse(adb: &Path, serial: &str, port: u16) -> Result<()> {
    run(
        adb,
        &["-s", serial, "reverse", &format!("tcp:{port}"), &format!("tcp:{port}")],
    )
    .context("adb reverse")?;
    println!("forwarded tcp:{port} -> device");
    Ok(())
}

fn reverse_remove(adb: &Path, serial: &str, port: u16) {
    let _ = run(adb, &["-s", serial, "reverse", "--remove", &format!("tcp:{port}")]);
}

/// Launches the host `DevActivity`, pointing it at the dev server.
fn launch(adb: &Path, serial: &str, port: u16) -> Result<()> {
    let url = format!("ws://127.0.0.1:{port}/ws");
    run(
        adb,
        &[
            "-s",
            serial,
            "am",
            "start",
            "-S", // force stop for a clean host
            "-n",
            &format!("{HOST}/{DEV_ACTIVITY}"),
            "--es",
            "url",
            &url,
            "--es",
            "dataDir",
            "nsh-ui-test",
        ],
    )
    .context("adb am start DevActivity")?;
    println!("launched {HOST}/{DEV_ACTIVITY} -> {url}");
    Ok(())
}

/// Returns the first connected, authorized device serial.
fn find_device() -> Option<String> {
    let output = run(&adb_path().ok()?, &["devices"]).ok()?;
    for line in output.lines() {
        let line = line.trim();
        if line.starts_with("Device") || line.contains('\t') || line.contains(' ') {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 {
                let (name, state) = (parts[0], parts[1]);
                if state == "device" {
                    return Some(name.to_string());
                }
            }
        }
    }
    None
}

/// Runs a command, returning its combined output (or an error if it fails).
fn run(adb: &Path, args: &[&str]) -> Result<String> {
    let output = Command::new(adb).args(args).output().with_context(|| {
        format!(
            "running {} {}",
            adb.display(),
            args.join(" ")
        )
    })?;
    if !output.status.success() {
        bail!(
            "{} failed ({}): {}",
            args.join(" "),
            output.status,
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

/// Locates `adb`, preferring the NDK/Sdk `platform-tools`.
fn adb_path() -> Result<PathBuf> {
    if let Ok(home) = std::env::var("ANDROID_HOME") {
        let candidate = Path::new(&home).join("platform-tools").join("adb");
        if candidate.exists() {
            return Ok(candidate);
        }
    }
    if let Ok(ndk) = std::env::var("ANDROID_NDK") {
        let candidate = Path::new(&ndk).join("toolchains").join("llvm").join("prebuilt").join("linux-x86_64").join("bin").join("adb");
        if candidate.exists() {
            return Ok(candidate);
        }
    }
    let on_path = PathBuf::from("adb");
    if on_path.exists() {
        return Ok(on_path);
    }
    bail!(
        "adb not found: set ANDROID_HOME (or ANDROID_NDK), or put adb on PATH"
    )
}
