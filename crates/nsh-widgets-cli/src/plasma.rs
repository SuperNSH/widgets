//! `nsh-widgets plasmoid`: a KDE Plasma widget for one kind, drawn by a generic QML renderer from what
//! the app published. Plasma has no widget extension an app can ship, so the plasmoid reads the app's
//! store through `nsh-widgets show --format ir` and presses buttons through `nsh-widgets apply`.

use std::path::PathBuf;

use anyhow::{Context as _, bail};
use serde_json::json;

use crate::Args;

const MAIN: &str = include_str!("../plasma/main.qml");
const NODE: &str = include_str!("../plasma/NshNode.qml");

/// The plasmoid's main.qml for Plasma `version` (5 or 6).
pub fn main_qml(version: u32, kind: &str, command: &str, source: &str) -> anyhow::Result<String> {
    let (imports, root, data, full, compact, expanded, background, preferred) = match version {
        5 => (
            "import org.kde.plasma.plasmoid 2.0\nimport org.kde.plasma.core 2.0 as PlasmaCore\nimport org.kde.kirigami 2.20 as Kirigami",
            "Item",
            "PlasmaCore.DataSource",
            "Plasmoid.fullRepresentation",
            "Plasmoid.compactRepresentation",
            "plasmoid.expanded",
            "Plasmoid.backgroundHints: PlasmaCore.Types.NoBackground",
            "Plasmoid.preferredRepresentation: Plasmoid.fullRepresentation",
        ),
        6 => (
            "import org.kde.plasma.plasmoid\nimport org.kde.plasma.core as PlasmaCore\nimport org.kde.plasma.plasma5support as P5Support\nimport org.kde.kirigami as Kirigami",
            "PlasmoidItem",
            "P5Support.DataSource",
            "fullRepresentation",
            "compactRepresentation",
            "main.expanded",
            "Plasmoid.backgroundHints: PlasmaCore.Types.NoBackground",
            "preferredRepresentation: fullRepresentation",
        ),
        v => bail!("--plasma {v}: use 5 or 6"),
    };
    let quote = |s: &str| s.replace('\\', "\\\\").replace('"', "\\\"");
    Ok(MAIN
        .replace("@IMPORTS@", imports)
        .replace("@ROOT@", root)
        .replace("@DATASOURCE@", data)
        .replace("@FULL@", full)
        .replace("@COMPACT@", compact)
        .replace("@EXPANDED@", expanded)
        .replace("@BACKGROUND@", background)
        .replace("@PREFERRED@", preferred)
        .replace("@KIND@", kind)
        .replace("@CMD@", &quote(command))
        .replace("@SOURCE@", &quote(source)))
}

pub fn metadata(version: u32, id: &str, name: &str, description: &str) -> serde_json::Value {
    let mut m = json!({
        "KPlugin": {
            "Id": id,
            "Name": name,
            "Description": description,
            "Icon": "preferences-desktop-plasma",
            "Category": "Miscellaneous",
            "License": "MIT",
            "Version": env!("CARGO_PKG_VERSION"),
        },
        "KPackageStructure": "Plasma/Applet",
    });
    if version == 5 {
        m["X-Plasma-API"] = json!("declarativeappletscript");
        m["X-Plasma-MainScript"] = json!("ui/main.qml");
        m["X-Plasma-API-Minimum-Version"] = json!("5.0");
    } else {
        m["X-Plasma-API-Minimum-Version"] = json!("6.0");
    }
    m
}

/// A shell word: plain, or single-quoted.
fn shell_word(s: &str) -> String {
    if !s.is_empty() && s.chars().all(|c| c.is_ascii_alphanumeric() || "/._-+:@".contains(c)) {
        s.to_owned()
    } else {
        format!("'{}'", s.replace('\'', "'\\''"))
    }
}

pub fn write(args: &Args) -> anyhow::Result<()> {
    let kind = args
        .kind
        .as_deref()
        .context("plasmoid needs a widget kind: nsh-widgets plasmoid KIND --group NAME")?;
    if !crate::config::valid_kind(kind) {
        bail!("{kind:?} is not a widget kind");
    }
    let source = match (&args.group, &args.source.dir) {
        (Some(g), _) => format!("--group {}", shell_word(g)),
        (None, Some(d)) => format!("--dir {}", shell_word(&d.to_string_lossy())),
        _ => bail!("plasmoid needs --group NAME (as the app's Widgets.configure(NAME)) or --dir ROOT"),
    };
    let command = match &args.exe {
        Some(c) => c.clone(),
        None => std::env::current_exe()
            .context("the nsh-widgets path")?
            .to_string_lossy()
            .into_owned(),
    };
    let base = args.group.clone().unwrap_or_else(|| "local".into());
    let id = format!(
        "dev.supernsh.widgets.{}.{}",
        base.replace(|c: char| !c.is_ascii_alphanumeric() && c != '.', "-"),
        kind.to_lowercase()
    );
    let config = crate::config::Config::read(&args.app.join("widgets.json")).ok();
    let w = config.as_ref().and_then(|c| c.widgets.iter().find(|w| w.kind == kind));
    let name = w.map(|w| w.name.clone()).unwrap_or_else(|| kind.to_owned());
    let description = w.map(|w| w.description.clone()).unwrap_or_default();
    let out = args
        .out
        .clone()
        .unwrap_or_else(|| PathBuf::from(".nsh-widgets/plasma").join(&id));
    let ui = out.join("contents/ui");
    std::fs::create_dir_all(&ui)?;
    std::fs::write(
        out.join("metadata.json"),
        serde_json::to_string_pretty(&metadata(args.plasma, &id, &name, &description))? + "\n",
    )?;
    std::fs::write(
        ui.join("main.qml"),
        main_qml(args.plasma, kind, &shell_word(&command), &source)?,
    )?;
    std::fs::write(ui.join("NshNode.qml"), NODE)?;
    let tool = if args.plasma == 5 {
        "kpackagetool5"
    } else {
        "kpackagetool6"
    };
    println!("wrote {} (Plasma {})", out.display(), args.plasma);
    println!(
        "install: {tool} -t Plasma/Applet -i {}   (or -u to upgrade)",
        shell_word(&out.to_string_lossy())
    );
    println!("then add \"{name}\" from the desktop's or panel's widget list");
    Ok(())
}
