//! `nsh-widgets window`: a desktop window for one kind, for X11 desktops and for Wayland compositors with
//! layer-shell (wlroots, KWin, COSMIC). It is a GJS script, so it needs no compiler on the user's machine.

use anyhow::bail;
use serde_json::{Value, json};

use crate::{Args, host};

/// `--anchor` and `--margin`, checked.
fn placement(args: &Args) -> anyhow::Result<(String, [i32; 2])> {
    let anchor = args.anchor.as_deref().unwrap_or("top-right");
    if !matches!(anchor, "top-left" | "top-right" | "bottom-left" | "bottom-right") {
        bail!("--anchor {anchor}: use top-left, top-right, bottom-left or bottom-right");
    }
    let margin = args.margin.as_deref().unwrap_or("24,24");
    let parts: Vec<i32> = margin.split(',').filter_map(|p| p.trim().parse().ok()).collect();
    match parts[..] {
        [x, y] if margin.matches(',').count() == 1 => Ok((anchor.to_owned(), [x, y])),
        _ => bail!("--margin {margin}: use X,Y in pixels"),
    }
}

pub fn write(args: &Args) -> anyhow::Result<()> {
    let h = host::Host::read(args, "window")?;
    let (anchor, margin) = placement(args)?;
    let mut config: Value = h.config();
    config["anchor"] = json!(anchor);
    config["margin"] = json!(margin);
    config["width"] = json!(340);
    let js = host::with_renderer(include_str!("../window/window.js"), host::GTK);
    let out = h.write(
        args,
        "window",
        &[("config.json", host::pretty(&config)), ("window.js", js)],
    )?;
    let script = std::path::absolute(out.join("window.js"))?;
    let run = format!("gjs {}", crate::plasma::shell_word(&script.to_string_lossy()));
    println!("run: {run}");
    println!(
        "needs gjs and GTK 3, and gtk-layer-shell on Wayland; start it with the session (autostart, or exec in the compositor's config)"
    );
    Ok(())
}
