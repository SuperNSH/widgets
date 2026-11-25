//! `nsh-widgets`: builds an NSH app's widgets from `widgets.json`.
//!
//! - `init`: writes a starter `widgets.json`.
//! - `build`: builds the WidgetKit extension (iOS simulator, iOS devices, macOS) and the app's
//!   Android widget library, then writes their entries into `nsh.json`.
//! - `check`: checks `widgets.json` and that `nsh.json` carries what `build` writes.
//! - `preview`: draws the widgets as an HTML page, light and dark, in every size.
//! - `plasmoid`, `gnome`, `cinnamon`, `window`: write a KDE Plasma applet, a GNOME Shell extension, a Cinnamon
//!   desklet, or an X11 and layer-shell window for one kind.
//! - `show`: prints one widget as text or Waybar JSON, for terminals and Linux status bars.
//! - `doctor`: checks the tools `build` runs.

use std::{
    path::{Path, PathBuf},
    process::{Command, ExitCode},
};

use anyhow::{Context as _, bail};
use serde_json::Value;

mod android;
mod apple;
mod cinnamon;
mod config;
mod gnome;
mod host;
mod nshjson;
mod plasma;
mod render;
mod signing;
mod source;
#[cfg(test)]
mod tests;
mod window;

use config::{App, Config};

const USAGE: &str = "nsh-widgets: home-screen widgets for NSH apps

usage:
  nsh-widgets init   [--app DIR] [--group GROUP]
  nsh-widgets build  [--app DIR] [--only ios|macos|android]...
  nsh-widgets check  [--app DIR]
  nsh-widgets preview [--app DIR] [SOURCE] [--out FILE]
  nsh-widgets show KIND [SOURCE] [--format text|waybar|ir] [--follow]
  nsh-widgets apply KIND ACTION [SOURCE]
  nsh-widgets plasmoid KIND --group NAME [--plasma 5|6] [--out DIR] [--command PATH]
  nsh-widgets gnome KIND --group NAME [--out DIR] [--command PATH]
  nsh-widgets cinnamon KIND --group NAME [--out DIR] [--command PATH]
  nsh-widgets window KIND --group NAME [--anchor CORNER] [--margin X,Y] [--out DIR] [--command PATH]
  nsh-widgets doctor [--app DIR]

DIR holds nsh.json and widgets.json (default: the current directory).
SOURCE is where the app wrote its widgets:
  --simulator          the booted iOS simulator's App Group container (needs --app)
  --dir ROOT           a store root (holding nsh/widgets), such as NSH_WIDGETS_DIR
  --group NAME         a Linux or Windows app's store, as Widgets.configure(NAME) placed it
  --snapshot F [--data F]  one snapshot.json and data.json
With none, show reads NSH_WIDGETS_DIR.";

struct Args {
    command: String,
    app: PathBuf,
    only: Vec<String>,
    group: Option<String>,
    source: source::Source,
    out: Option<PathBuf>,
    format: String,
    follow: bool,
    kind: Option<String>,
    /// A second positional argument: apply's action.
    extra: Option<String>,
    /// plasmoid: the Plasma version (5 or 6) and the command its QML runs.
    plasma: u32,
    exe: Option<String>,
    /// window: the screen corner and the distance from it.
    anchor: Option<String>,
    margin: Option<String>,
}

fn parse() -> anyhow::Result<Args> {
    let mut it = std::env::args().skip(1);
    let command = it.next().unwrap_or_default();
    let mut args = Args {
        command,
        app: PathBuf::from("."),
        only: Vec::new(),
        group: None,
        source: source::Source::default(),
        out: None,
        format: "text".into(),
        follow: false,
        kind: None,
        extra: None,
        plasma: 6,
        exe: None,
        anchor: None,
        margin: None,
    };
    while let Some(a) = it.next() {
        match a.as_str() {
            "--app" => args.app = it.next().context("--app needs a directory")?.into(),
            "--only" => args.only.push(it.next().context("--only needs ios, macos or android")?),
            "--group" => args.group = Some(it.next().context("--group needs an App Group")?),
            "--simulator" => args.source.simulator = true,
            "--dir" => args.source.dir = Some(it.next().context("--dir needs a directory")?.into()),
            "--snapshot" => args.source.snapshot = Some(it.next().context("--snapshot needs a file")?.into()),
            "--data" => args.source.data = Some(it.next().context("--data needs a file")?.into()),
            "--out" => args.out = Some(it.next().context("--out needs a file")?.into()),
            "--format" => args.format = it.next().context("--format needs text or waybar")?,
            "--follow" => args.follow = true,
            "--plasma" => {
                args.plasma = it
                    .next()
                    .context("--plasma needs 5 or 6")?
                    .parse()
                    .context("--plasma needs 5 or 6")?
            }
            "--anchor" => args.anchor = Some(it.next().context("--anchor needs a corner")?),
            "--margin" => args.margin = Some(it.next().context("--margin needs X,Y")?),
            "--command" => args.exe = Some(it.next().context("--command needs a path")?),
            other if !other.starts_with("--") && args.kind.is_none() => args.kind = Some(other.to_owned()),
            other if !other.starts_with("--") && args.extra.is_none() => args.extra = Some(other.to_owned()),
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
        "doctor" => doctor(&args.app),
        "preview" => source::preview(&args),
        "show" => source::show(&args),
        "apply" => source::apply(&args, args.extra.as_deref().unwrap_or("")),
        "plasmoid" => plasma::write(&args),
        "gnome" => gnome::write(&args),
        "cinnamon" => cinnamon::write(&args),
        "window" => window::write(&args),
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
        let n = config.widgets.len();
        println!("ok: {n} widget {}", if n == 1 { "kind" } else { "kinds" });
        Ok(())
    } else {
        bail!("{}", problems.join("\n"))
    }
}

fn doctor(app: &Path) -> anyhow::Result<()> {
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
    println!("note packaging widget extensions needs NSH 0.7.1 or later; the 0.6.0 and 0.6.1 releases cannot");
    if cfg!(target_os = "macos") {
        signing::report(app);
    }
    if failed { bail!("some checks failed") } else { Ok(()) }
}
