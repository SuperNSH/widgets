//! `nsh-widgets gnome`: a GNOME Shell extension for one kind. GNOME has no desktop widgets, so the kind
//! sits in the top bar (its `accessory` tree) and opens as a popup (its `root` tree).

use std::path::PathBuf;

use anyhow::{Context as _, bail};
use serde_json::{Value, json};

use crate::Args;

const EXTENSION: &str = include_str!("../gnome/extension.js");
/// GNOME 45 moved extensions to ES modules, which this one is.
const SHELL_VERSIONS: &[&str] = &["45", "46", "47", "48"];

/// The extension's id: letters, digits, dots, dashes and underscores before the `@`, as GNOME asks.
pub fn uuid(source: &str, kind: &str) -> String {
    let safe = source.replace(|c: char| !c.is_ascii_alphanumeric() && c != '.', "-");
    format!("{safe}.{}@widgets.supernsh.dev", kind.to_lowercase())
}

pub fn metadata(uuid: &str, name: &str, description: &str) -> Value {
    json!({
        "uuid": uuid,
        "name": name,
        "description": if description.is_empty() { name } else { description },
        "shell-version": SHELL_VERSIONS,
        "url": "https://github.com/SuperNSH/widgets",
    })
}

/// What the extension runs: a program and arguments, never a shell line.
pub fn config(kind: &str, command: &str, source: &[String]) -> Value {
    json!({ "kind": kind, "command": command, "source": source })
}

pub fn write(args: &Args) -> anyhow::Result<()> {
    let kind = args
        .kind
        .as_deref()
        .context("gnome needs a widget kind: nsh-widgets gnome KIND --group NAME")?;
    if !crate::config::valid_kind(kind) {
        bail!("{kind:?} is not a widget kind");
    }
    let (name, source) = match (&args.group, &args.source.dir) {
        (Some(g), _) => (g.clone(), vec!["--group".into(), g.clone()]),
        (None, Some(d)) => ("local".into(), vec!["--dir".into(), d.to_string_lossy().into_owned()]),
        _ => bail!("gnome needs --group NAME (as the app's Widgets.configure(NAME)) or --dir ROOT"),
    };
    let command = match &args.exe {
        Some(c) => c.clone(),
        None => std::env::current_exe()
            .context("the nsh-widgets path")?
            .to_string_lossy()
            .into_owned(),
    };
    let id = uuid(&name, kind);
    let widgets = crate::config::Config::read(&args.app.join("widgets.json")).ok();
    let w = widgets.as_ref().and_then(|c| c.widgets.iter().find(|w| w.kind == kind));
    let title = w.map(|w| w.name.clone()).unwrap_or_else(|| kind.to_owned());
    let about = w.map(|w| w.description.clone()).unwrap_or_default();
    let out = args
        .out
        .clone()
        .unwrap_or_else(|| PathBuf::from(".nsh-widgets/gnome").join(&id));
    std::fs::create_dir_all(&out)?;
    let pretty = |v: &Value| serde_json::to_string_pretty(v).map(|s| s + "\n");
    std::fs::write(out.join("metadata.json"), pretty(&metadata(&id, &title, &about))?)?;
    std::fs::write(out.join("config.json"), pretty(&config(kind, &command, &source))?)?;
    std::fs::write(out.join("extension.js"), EXTENSION)?;
    println!("wrote {}", out.display());
    println!(
        "install: cp -r {} ~/.local/share/gnome-shell/extensions/{id}",
        crate::plasma::shell_word(&out.to_string_lossy())
    );
    println!("then log out and in, and run: gnome-extensions enable {id}");
    Ok(())
}
