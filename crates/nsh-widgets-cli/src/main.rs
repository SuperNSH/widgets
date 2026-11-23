//! `nsh-widgets`: builds an NSH app's widgets from `widgets.json`.
//!
//! - `init`: writes a starter `widgets.json`.
//! - `build`: builds the WidgetKit extension (iOS simulator, iOS devices, macOS) and the app's
//!   Android widget library, then writes their entries into `nsh.json`.
//! - `check`: checks `widgets.json` and that `nsh.json` carries what `build` writes.
//! - `doctor`: checks the tools `build` runs.

use std::{
    path::{Path, PathBuf},
    process::{Command, ExitCode},
};

use anyhow::{Context as _, bail};
use serde_json::Value;

mod android;
mod apple;
mod config;
mod nshjson;
#[cfg(test)]
mod tests;

use config::{App, Config};

const USAGE: &str = "nsh-widgets: home-screen widgets for NSH apps

usage:
  nsh-widgets init   [--app DIR] [--group GROUP]
  nsh-widgets build  [--app DIR] [--only ios|macos|android]...
  nsh-widgets check  [--app DIR]
  nsh-widgets doctor

DIR holds nsh.json and widgets.json (default: the current directory).";

struct Args {
    command: String,
    app: PathBuf,
    only: Vec<String>,
    group: Option<String>,
}

fn parse() -> anyhow::Result<Args> {
    let mut it = std::env::args().skip(1);
    let command = it.next().unwrap_or_default();
    let mut args = Args {
        command,
        app: PathBuf::from("."),
        only: Vec::new(),
        group: None,
    };
    while let Some(a) = it.next() {
        match a.as_str() {
            "--app" => args.app = it.next().context("--app needs a directory")?.into(),
            "--only" => args.only.push(it.next().context("--only needs ios, macos or android")?),
            "--group" => args.group = Some(it.next().context("--group needs an App Group")?),
            other => bail!("unknown argument {other}\n\n{USAGE}"),
        }
    }
    for o in &args.only {
        if !matches!(o.as_str(), "ios" | "macos" | "android") {
            bail!("--only {o}: use ios, macos or android");
        }
    }
    Ok(args)
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("error: {e:#}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> anyhow::Result<()> {
    let args = parse()?;
    match args.command.as_str() {
        "init" => init(&args),
        "build" => build(&args),
        "check" => check(&args.app),
        "doctor" => doctor(),
        "" | "help" | "--help" | "-h" => {
            println!("{USAGE}");
            Ok(())
        }
        other => bail!("unknown command {other}\n\n{USAGE}"),
    }
}

fn init(args: &Args) -> anyhow::Result<()> {
    let target = args.app.join("widgets.json");
    if target.exists() {
        bail!("{} exists already", target.display());
    }
    let app = App::read(&args.app.join("nsh.json"))?;
    let group = args.group.clone().unwrap_or_else(|| format!("group.{}", app.id));
    let starter = serde_json::json!({
        "appGroup": group,
        "widgets": [{
            "kind": "Counter",
            "name": format!("{} counter", app.name),
            "description": "A number the app sets and the widget's buttons change.",
            "families": ["small", "medium"]
        }]
    });
    std::fs::write(&target, serde_json::to_string_pretty(&starter)? + "\n")?;
    println!("wrote {}: one kind, Counter. Next: nsh-widgets build", target.display());
    Ok(())
}

fn wants(args: &Args, platform: &str) -> bool {
    args.only.is_empty() || args.only.iter().any(|o| o == platform)
}

fn build(args: &Args) -> anyhow::Result<()> {
    let dir = &args.app;
    let nsh_path = dir.join("nsh.json");
    let app = App::read(&nsh_path)?;
    let config = Config::read(&dir.join("widgets.json"))?;
    let mut built = nshjson::Built::default();
    let apple_host = cfg!(target_os = "macos");
    if wants(args, "ios") {
        if apple_host {
            for p in [apple::Platform::Simulator, apple::Platform::Device] {
                let out = apple::build(dir, &app, &config, p)?;
                println!("built {}", out.display());
            }
            built.ios = true;
        } else if !args.only.is_empty() {
            bail!("the iOS widget extension builds only on macOS");
        }
    }
    if wants(args, "macos") {
        if apple_host {
            let out = apple::build(dir, &app, &config, apple::Platform::Mac)?;
            println!("built {}", out.display());
            built.macos = true;
        } else if !args.only.is_empty() {
            bail!("the macOS widget extension builds only on macOS");
        }
    }
    if wants(args, "android") {
        let out = android::build(dir, &config)?;
        println!("built {}", out.display());
        built.android = true;
    }
    let text = std::fs::read_to_string(&nsh_path)?;
    let mut nsh: Value = serde_json::from_str(&text).with_context(|| nsh_path.display().to_string())?;
    let before = nsh.clone();
    nshjson::apply(&mut nsh, &config, built);
    if nsh != before {
        std::fs::write(&nsh_path, serde_json::to_string_pretty(&nsh)? + "\n")?;
        println!("updated {}", nsh_path.display());
    }
    Ok(())
}

fn check(dir: &Path) -> anyhow::Result<()> {
    let config = Config::read(&dir.join("widgets.json"))?;
    let nsh_path = dir.join("nsh.json");
    let nsh: Value = serde_json::from_str(&std::fs::read_to_string(&nsh_path)?)?;
    let mut expected = nsh.clone();
    nshjson::apply(
        &mut expected,
        &config,
        nshjson::Built {
            ios: true,
            macos: true,
            android: true,
        },
    );
    let mut problems = Vec::new();
    for (platform, key) in [("ios", "extensions"), ("macos", "extensions"), ("android", "libraries")] {
        if nsh.pointer(&format!("/{platform}/{key}")) != expected.pointer(&format!("/{platform}/{key}")) {
            problems.push(format!(
                "nsh.json {platform}.{key} is not what nsh-widgets build writes: run it"
            ));
        }
    }
    for font in &config.fonts {
        if !dir.join(font).is_file() {
            problems.push(format!("font {} does not exist", font.display()));
        }
    }
    if problems.is_empty() {
        println!("ok: {} widget kinds", config.widgets.len());
        Ok(())
    } else {
        bail!("{}", problems.join("\n"))
    }
}

fn doctor() -> anyhow::Result<()> {
    let mut failed = false;
    let mut probe = |what: &str, cmd: &mut Command| match cmd.output() {
        Ok(o) if o.status.success() => {
            let first = String::from_utf8_lossy(&o.stdout)
                .lines()
                .next()
                .unwrap_or("")
                .to_owned();
            println!("ok   {what}: {first}");
        }
        _ => {
            println!("FAIL {what}");
            failed = true;
        }
    };
    if cfg!(target_os = "macos") {
        probe("Xcode", Command::new("xcodebuild").arg("-version"));
        probe("swiftc", Command::new("xcrun").args(["swiftc", "--version"]));
        probe(
            "App Intents metadata processor",
            Command::new("xcrun").args(["--find", "appintentsmetadataprocessor"]),
        );
        probe(
            "iOS simulator SDK",
            Command::new("xcrun").args(["--sdk", "iphonesimulator", "--show-sdk-path"]),
        );
        probe(
            "iOS SDK",
            Command::new("xcrun").args(["--sdk", "iphoneos", "--show-sdk-path"]),
        );
        probe(
            "macOS SDK",
            Command::new("xcrun").args(["--sdk", "macosx", "--show-sdk-path"]),
        );
    } else {
        println!("--   Apple extensions build only on macOS; Android needs nothing beyond nsh");
    }
    probe("nsh", Command::new("nsh").arg("--version"));
    if failed { bail!("some checks failed") } else { Ok(()) }
}
