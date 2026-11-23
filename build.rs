//! Compiles swift/Bridge.swift into a static library on Apple targets: WidgetCenter reloads, the App
//! Group container and the Darwin notification the widget posts after a button press.
use std::{env, path::PathBuf, process::Command};

fn main() {
    println!("cargo:rerun-if-changed=swift/Bridge.swift");
    println!("cargo:rerun-if-env-changed=IPHONEOS_DEPLOYMENT_TARGET");
    println!("cargo:rerun-if-env-changed=MACOSX_DEPLOYMENT_TARGET");
    let target = env::var("TARGET").unwrap_or_default();
    // Match the deployment target the host is built with, so the linker does not warn.
    let ios = env::var("IPHONEOS_DEPLOYMENT_TARGET").unwrap_or_else(|_| "15.0".into());
    let mac = env::var("MACOSX_DEPLOYMENT_TARGET").unwrap_or_else(|_| "11.0".into());
    let (sdk, swift_target, platform) = match target.as_str() {
        "aarch64-apple-ios" => ("iphoneos", format!("arm64-apple-ios{ios}"), "iphoneos"),
        "aarch64-apple-ios-sim" => (
            "iphonesimulator",
            format!("arm64-apple-ios{ios}-simulator"),
            "iphonesimulator",
        ),
        "x86_64-apple-ios" => (
            "iphonesimulator",
            format!("x86_64-apple-ios{ios}-simulator"),
            "iphonesimulator",
        ),
        "aarch64-apple-darwin" => ("macosx", format!("arm64-apple-macos{mac}"), "macosx"),
        "x86_64-apple-darwin" => ("macosx", format!("x86_64-apple-macos{mac}"), "macosx"),
        _ => return,
    };
    let out = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR"));
    let sdk_path = run("xcrun", &["--sdk", sdk, "--show-sdk-path"]);
    let lib = out.join("libnshwidgetsbridge.a");
    let status = Command::new("xcrun")
        .args([
            "--sdk",
            sdk,
            "swiftc",
            "-emit-library",
            "-static",
            "-parse-as-library",
            "-O",
        ])
        .args([
            "-module-name",
            "NshWidgetsBridge",
            "-target",
            &swift_target,
            "-sdk",
            &sdk_path,
            "-o",
        ])
        .arg(&lib)
        .arg("swift/Bridge.swift")
        .status()
        .expect("run swiftc");
    assert!(status.success(), "swiftc failed on swift/Bridge.swift");
    println!("cargo:rustc-link-search=native={}", out.display());
    println!("cargo:rustc-link-lib=static=nshwidgetsbridge");
    // The Swift runtime: the SDK's .tbd stubs plus the toolchain's compatibility libraries.
    println!("cargo:rustc-link-search=native={sdk_path}/usr/lib/swift");
    let swiftc = PathBuf::from(run("xcrun", &["--find", "swiftc"]));
    if let Some(toolchain) = swiftc.parent().and_then(|bin| bin.parent()) {
        println!(
            "cargo:rustc-link-search=native={}",
            toolchain.join("lib/swift").join(platform).display()
        );
    }
    println!("cargo:rustc-link-lib=framework=WidgetKit");
    println!("cargo:rustc-link-lib=framework=Foundation");
}

fn run(program: &str, args: &[&str]) -> String {
    let out = Command::new(program)
        .args(args)
        .output()
        .unwrap_or_else(|e| panic!("{program}: {e}"));
    assert!(out.status.success(), "{program} {args:?} failed");
    String::from_utf8(out.stdout).expect("utf-8").trim().to_owned()
}
