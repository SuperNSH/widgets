//! `nsh-widgets gnome`: a GNOME Shell extension for one kind. GNOME has no desktop widgets, so the kind
//! sits in the top bar (its `accessory` tree) and opens as a popup (its `root` tree).

use serde_json::json;

use crate::{Args, host};

/// The versions it has been run on (docs/status.md). GNOME 45 moved extensions to ES modules, which this
/// one is, and GNOME turns an extension off on a version it does not list.
const SHELL_VERSIONS: &[&str] = &["45", "46", "47", "48", "49", "50", "51"];

pub fn write(args: &Args) -> anyhow::Result<()> {
    let h = host::Host::read(args, "gnome")?;
    let id = h.uuid();
    let metadata = json!({
        "uuid": id,
        "name": h.title,
        "description": h.description,
        "shell-version": SHELL_VERSIONS,
        "url": "https://github.com/SuperNSH/widgets",
    });
    let js = host::with_renderer(include_str!("../gnome/extension.js"), host::ST);
    let out = h.write(
        args,
        "gnome",
        &[
            ("metadata.json", host::pretty(&metadata)),
            ("config.json", host::pretty(&h.config())),
            ("extension.js", js),
        ],
    )?;
    let dir = format!("~/.local/share/gnome-shell/extensions/{id}");
    println!(
        "install: mkdir -p {dir} && cp -r {}/. {dir}",
        crate::plasma::shell_word(&out.to_string_lossy())
    );
    println!("then log out and in, and run: gnome-extensions enable {id}");
    Ok(())
}
